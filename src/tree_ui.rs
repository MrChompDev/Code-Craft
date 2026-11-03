// Project explorer: tree view with file-type icons and quick actions.
use eframe::egui;
use egui::{Color32, FontFamily, Stroke};
use crate::icons::{self, Chev};
use crate::project::{ft_of, Id};
use crate::theme::*;
use crate::{Pending, App};
use egui::{Align2, FontId, Pos2, Sense, Vec2};

const ROW_H: f32 = 22.0;

pub fn draw(app: &mut App, ui: &mut egui::Ui) {
    // header
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.label(egui::RichText::new("Project").size(12.0).strong().color(TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if quick_btn(ui, "collapse_all", "Collapse All", |p, c| icons::chevrons_in(p, c, TEXT2)) {
                app.expanded.clear();
                app.expanded.insert(app.project.root);
            }
            if quick_btn(ui, "new_folder", "New Folder", |p, c| icons::folder_plus(p, c, TEXT2)) {
                app.intent_new_folder();
            }
            if quick_btn(ui, "new_file", "New File", |p, c| icons::plus(p, c, TEXT2)) {
                app.intent_new_file();
            }
        });
    });
    ui.add_space(2.0);
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            rec(app, ui, app.project.root, 0);
        });
}

fn quick_btn(
    ui: &mut egui::Ui,
    id: &str,
    tooltip: &str,
    paint: impl FnOnce(&egui::Painter, Pos2),
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(22.0, 20.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 3.0, HOVER);
    }
    paint(ui.painter(), rect.center());
    resp.on_hover_text(tooltip).clicked()
}

fn rec(app: &mut App, ui: &mut egui::Ui, id: Id, depth: usize) {
    let is_dir = app.project.nodes[id].is_dir;
    let name = app.project.nodes[id].name.clone();
    let children: Vec<Id> = app.project.nodes[id].children.clone();
    let expanded = app.expanded.contains(&id);
    let selected = app.selected == Some(id);

    // pending rename renders an editable row in place
    let renaming = matches!(&app.pending, Some(Pending::Rename { node, .. }) if *node == id);
    if renaming {
        let name = match &app.pending {
            Some(Pending::Rename { name, .. }) => name.clone(),
            _ => String::new(),
        };
        let mut n = name;
        let entered = entry_row(ui, depth, &mut n);
        if entered {
            if let Some(Pending::Rename { name, .. }) = &mut app.pending {
                *name = n.clone();
            }
            app.commit_pending();
        }
        return;
    }

    let indent = 6.0 + depth as f32 * 14.0;
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width().max(120.0), ROW_H),
        Sense::click(),
    );
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 3.0, SEL_BG);
    } else if resp.hovered() {
        p.rect_filled(rect, 3.0, HOVER);
    }

    let cy = rect.center().y;
    let mut chev_hit = None;
    if is_dir {
        let cc = Pos2::new(rect.left() + indent + 4.0, cy);
        icons::chevron(p, cc, 5.5, if expanded { TEXT } else { TEXT2 }, if expanded { Chev::Down } else { Chev::Right });
        let cr = egui::Rect::from_center_size(cc, Vec2::splat(16.0));
        chev_hit = Some(ui.interact(cr, egui::Id::new("chev").with(id), Sense::click()));
        icons::folder(p, Pos2::new(rect.left() + indent + 22.0, cy), FOLDER, expanded);
    } else {
        icons::badge(p, Pos2::new(rect.left() + indent + 13.0, cy), ft_of(&name), false);
    }
    p.text(
        Pos2::new(rect.left() + indent + (if is_dir { 34.0 } else { 22.0 }), cy),
        Align2::LEFT_CENTER,
        &name,
        FontId::new(12.0, FontFamily::Proportional),
        if selected { TEXT } else { TEXT2 },
    );

    if let Some(ch) = chev_hit {
        if ch.clicked() {
            toggle(app, id);
        }
    }
    if resp.clicked() {
        app.selected = Some(id);
        app.term_focus = false;
        if is_dir && resp.double_clicked() {
            toggle(app, id);
        }
        if !is_dir {
            app.open_file(id);
        }
    }

    // context menu
    resp.context_menu(|ui| {
        ui.set_min_width(160.0);
        if is_dir {
            if ui.button("New File…").clicked() {
                app.expanded.insert(id);
                app.pending = Some(Pending::New { parent: id, is_dir: false, name: String::new() });
                ui.close();
            }
            if ui.button("New Folder…").clicked() {
                app.expanded.insert(id);
                app.pending = Some(Pending::New { parent: id, is_dir: true, name: String::new() });
                ui.close();
            }
            ui.separator();
        }
        if id != app.project.root && ui.button("Rename…").clicked() {
            let n = app.project.nodes[id].name.clone();
            app.pending = Some(Pending::Rename { node: id, name: n });
            ui.close();
        }
        let del = ui.add_enabled(id != app.project.root, egui::Button::new("Delete"));
        if del.clicked() {
            app.delete_node(id);
            ui.close();
        }
    });

    // children
    if is_dir && expanded {
        for &c in &children {
            rec(app, ui, c, depth + 1);
        }
        // inline "new" entry under this dir
        let want_new = matches!(&app.pending, Some(Pending::New { parent, .. }) if *parent == id);
        if want_new {
            let (pd, name) = match &app.pending {
                Some(Pending::New { is_dir, name, .. }) => (*is_dir, name.clone()),
                _ => (false, String::new()),
            };
            let mut n = name;
            let entered = entry_row(ui, depth + 1, &mut n);
            if entered {
                app.pending = Some(Pending::New { parent: id, is_dir: pd, name: n });
                app.commit_pending();
            }
        }
    }
}

fn toggle(app: &mut App, id: Id) {
    if app.expanded.contains(&id) {
        app.expanded.remove(&id);
    } else {
        app.expanded.insert(id);
    }
}

fn entry_row(ui: &mut egui::Ui, depth: usize, name: &mut String) -> bool {
    let indent = 6.0 + (depth + 1) as f32 * 14.0 + 4.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width().max(120.0), ROW_H), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 3.0, Color32::from_rgb(0x33, 0x35, 0x39));

    let box_w = 132.0;
    let brect = egui::Rect::from_min_size(
        Pos2::new(rect.left() + indent, rect.center().y - 9.0),
        Vec2::new(box_w, 18.0),
    );
    p.rect_filled(brect, 3.0, EDITOR_BG);
    p.rect_stroke(
        brect,
        3.0,
        Stroke::new(1.0, ACCENT),
        egui::StrokeKind::Inside,
    );

    let out = ui
        .scope_builder(
            egui::UiBuilder::new().max_rect(brect),
            |ui| {
                egui::TextEdit::singleline(name)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(box_w - 8.0)
                    .frame(false)
                    .show(ui)
            },
        )
        .inner;
    let resp = out.response;
    let entered = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if !resp.has_focus() {
        resp.request_focus();
    }
    entered
}
