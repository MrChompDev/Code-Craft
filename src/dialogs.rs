// Modal dialogs: open-folder, about, keyboard shortcuts.
use eframe::egui;
use crate::icons;
use crate::theme;
use crate::App;

pub fn modals(app: &mut App, ctx: &egui::Context) {
    if app.open_dialog.is_some() {
        let mut commit: Option<String> = None;
        let mut close = false;
        egui::Window::new("Open Folder")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, -40.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(420.0);
                ui.label(egui::RichText::new("Project folder path:").color(theme::TEXT2));
                let mut open_now = false;
                let Some(dialog) = app.open_dialog.as_mut() else { return };
                let field = ui.add_sized(
                    [ui.available_width(), 24.0],
                    egui::TextEdit::singleline(&mut dialog.path),
                );
                ui.horizontal(|ui| {
                    if ui.button("Open").clicked() {
                        open_now = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                    ui.weak("~ expands to your home directory");
                });
                if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    open_now = true;
                }
                if !app.recent.is_empty() {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Recent").small().color(theme::TEXT3));
                    egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        let recents = app.recent.clone();
                        for r in recents {
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new(r.clone()).size(12.0).color(theme::TEXT2),
                                ))
                                .clicked()
                            {
                                commit = Some(r);
                            }
                        }
                    });
                }
                if open_now && commit.is_none() {
                    let p = dialog.path.trim().to_string();
                    if !p.is_empty() {
                        commit = Some(p);
                    }
                }
            });
        if let Some(p) = commit {
            app.open_dialog = None;
            app.open_project_dir(&p);
        } else if close {
            app.open_dialog = None;
        }
    }
    if app.palette.is_some() {
        crate::palette::palette_ui(app, ctx);
    }
    if app.about_open {
        egui::Window::new("About CodeCraft")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(320.0);
                ui.vertical_centered(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(0.0, 34.0), egui::Sense::hover());
                    icons::logo(ui.painter(), rect.center(), theme::FOLDER);
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("CodeCraft").size(17.0).strong().color(theme::TEXT));
                    ui.label(egui::RichText::new("Version 1.0.0").color(theme::TEXT2));
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("A personal IDE inspired by JetBrains IntelliJ IDEA.\nBuilt with Rust + egui.")
                            .color(theme::TEXT2),
                    );
                    ui.add_space(8.0);
                    if ui.button("OK").clicked() {
                        app.about_open = false;
                    }
                });
            });
    }
    if app.shortcuts_open {
        egui::Window::new("Keyboard Shortcuts")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(330.0);
                let rows = [
                    ("Ctrl+N", "New file"),
                    ("Ctrl+B", "Toggle project sidebar"),
                    ("Ctrl+`", "Toggle bottom panel"),
                    ("Shift+F10", "Run"),
                    ("Shift+F9", "Debug"),
                    ("Esc", "Cancel / close"),
                ];
                egui::Grid::new("shortcuts_grid").spacing([16.0, 4.0]).show(ui, |ui| {
                    for (k, v) in rows {
                        ui.label(egui::RichText::new(k).monospace().color(theme::SYN_KW));
                        ui.label(egui::RichText::new(v).color(theme::TEXT2));
                        ui.end_row();
                    }
                });
                ui.add_space(6.0);
                if ui.button("Close").clicked() {
                    app.shortcuts_open = false;
                }
            });
    }
}
