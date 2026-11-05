// Menu bar, main toolbar, left rail, status bar.
use eframe::egui;
use egui::FontFamily;
use crate::icons::{self, Chev};
use crate::theme::*;
use crate::{BottomTab, App};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

fn icon_button(ui: &mut egui::Ui, _id: &str, size: Vec2, tooltip: &str, paint: impl FnOnce(&egui::Painter, Pos2)) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.painter().rect_filled(rect, 3.0, HOVER);
    }
    paint(ui.painter(), rect.center());
    resp.on_hover_text(tooltip).clicked()
}

// ---------- menu bar ----------

pub fn menu_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        menu(ui, "File", |ui| {
            if item(ui, "New File", "Ctrl+N") {
                app.intent_new_file();
            }
            if item(ui, "New Folder", "") {
                app.intent_new_folder();
            }
            ui.separator();
            if item(ui, "Open Folder…", "") {
                app.open_dialog = Some(crate::OpenDialog {
                    path: String::new(),
                    error: None,
                });
            }
            if !app.recent.is_empty() {
                ui.menu_button("Open Recent", |ui| {
                    let recents = app.recent.clone();
                    for r in recents {
                        if ui.button(egui::RichText::new(r.clone()).size(12.0)).clicked() {
                            app.open_project_dir(&r);
                            ui.close();
                        }
                    }
                    if ui.button("Clear List").clicked() {
                        app.recent.clear();
                        if let Some(dir) = App::settings_dir() {
                            let _ = std::fs::remove_file(dir.join("recents.txt"));
                        }
                        ui.close();
                    }
                });
            }
            ui.separator();
            if item(ui, "Save", "Ctrl+S") {
                app.save_active();
            }
            if item(ui, "Save All", "Ctrl+Shift+S") {
                app.save_all();
            }
            if item(ui, "Refresh Project", "") {
                app.refresh_tree();
            }
            ui.separator();
            item_disabled(ui, "Settings…", "");
            ui.separator();
            if item(ui, "Exit", "Alt+F4") {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        menu(ui, "Edit", |ui| {
            item_disabled(ui, "Undo", "Ctrl+Z");
            item_disabled(ui, "Redo", "Ctrl+Shift+Z");
            ui.separator();
            item_disabled(ui, "Cut", "Ctrl+X");
            item_disabled(ui, "Copy", "Ctrl+C");
            item_disabled(ui, "Paste", "Ctrl+V");
            ui.separator();
            item_disabled(ui, "Find…", "Ctrl+F");
            item_disabled(ui, "Toggle Line Comment", "Ctrl+/");
        });
        menu(ui, "Selection", |ui| {
            item_disabled(ui, "Select All", "Ctrl+A");
            item_disabled(ui, "Extend Selection", "Ctrl+W");
            item_disabled(ui, "Shrink Selection", "Ctrl+Shift+W");
        });
        menu(ui, "View", |ui| {
            if check(ui, "Project Sidebar", app.show_tree) {
                app.show_tree = !app.show_tree;
            }
            if check(ui, "Bottom Panel", app.bottom_open) {
                app.bottom_open = !app.bottom_open;
            }
            ui.separator();
            if item(ui, "Collapse All Folds", "") {
                app.toggle_folds(true);
            }
            if item(ui, "Expand All Folds", "") {
                app.toggle_folds(false);
            }
        });
        menu(ui, "Run", |ui| {
            if item(ui, "Build Project", "F9") {
                app.do_build();
            }
            if item(ui, "Run 'demo-project'", "Shift+F10") {
                app.do_run();
            }
            if item(ui, "Debug 'demo-project'", "Shift+F9") {
                app.do_debug();
            }
            ui.separator();
            if item(ui, "Stop", "Ctrl+F2") {
                app.do_stop();
            }
        });
        menu(ui, "Terminal", |ui| {
            if item(ui, "New Terminal", "Ctrl+`") {
                app.bottom_open = true;
                app.bottom_tab = BottomTab::Terminal;
                app.term_focus = true;
            }
            if item(ui, "Clear Terminal", "") {
                app.term.lines.clear();
            }
            ui.separator();
            if item(ui, "Hide Bottom Panel", "") {
                app.bottom_open = false;
            }
        });
        menu(ui, "Help", |ui| {
            if item(ui, "Keyboard Shortcuts", "") {
                app.shortcuts_open = true;
            }
            if item(ui, "About CodeCraft", "") {
                app.about_open = true;
            }
        });
    });
}

fn menu<R>(ui: &mut egui::Ui, label: &str, content: impl FnOnce(&mut egui::Ui) -> R) {
    ui.menu_button(label, |ui| content(ui));
}

fn item(ui: &mut egui::Ui, label: &str, shortcut: &str) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 24.0;
        let b = ui.button(label);
        if !shortcut.is_empty() {
            ui.label(egui::RichText::new(shortcut).size(10.5).color(TEXT3));
        }
        clicked = b.clicked();
        if clicked {
            ui.close();
        }
    });
    clicked
}

fn item_disabled(ui: &mut egui::Ui, label: &str, shortcut: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 24.0;
        ui.add_enabled(false, egui::Button::new(label));
        if !shortcut.is_empty() {
            ui.label(egui::RichText::new(shortcut).size(10.5).color(TEXT3));
        }
    });
}

fn check(ui: &mut egui::Ui, label: &str, on: bool) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        let mark = if on { "✓" } else { " " };
        ui.label(egui::RichText::new(mark).color(TEXT2).size(11.0));
        let b = ui.button(label);
        clicked = b.clicked();
        if clicked {
            ui.close();
        }
    });
    clicked
}

// ---------- toolbar ----------

pub fn toolbar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        // brand + project dropdown
        let (brect, bresp) = ui.allocate_exact_size(Vec2::new(132.0, 28.0), Sense::click());
        if bresp.hovered() {
            ui.painter().rect_filled(brect, 4.0, HOVER);
        }
        let p = ui.painter();
        icons::logo(p, Pos2::new(brect.left() + 16.0, brect.center().y), FOLDER);
        p.text(
            Pos2::new(brect.left() + 30.0, brect.center().y),
            Align2::LEFT_CENTER,
            "CodeCraft",
            FontId::new(13.5, FontFamily::Proportional),
            TEXT,
        );
        icons::chevron(p, Pos2::new(brect.right() - 12.0, brect.center().y), 6.0, TEXT2, Chev::Down);
        bresp.on_hover_text("Project 'demo-project'");

        // git branch widget
        let (grect, gresp) = ui.allocate_exact_size(Vec2::new(74.0, 28.0), Sense::click());
        if gresp.hovered() {
            ui.painter().rect_filled(grect, 4.0, HOVER);
        }
        let p = ui.painter();
        icons::branch(p, Pos2::new(grect.left() + 12.0, grect.center().y), TEXT2);
        p.text(
            Pos2::new(grect.left() + 24.0, grect.center().y),
            Align2::LEFT_CENTER,
            "main",
            FontId::new(12.5, FontFamily::Proportional),
            TEXT,
        );
        gresp.on_hover_text("Branch: main");

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |_| {});

        // center: run configuration + actions
        ui.add_space(ui.available_width() * 0.5 - 180.0);
        egui::ComboBox::from_id_salt("env_combo")
            .selected_text(egui::RichText::new(&app.env).color(TEXT))
            .width(150.0)
            .show_ui(ui, |ui| {
                for e in app.envs.clone() {
                    ui.selectable_value(&mut app.env, e.to_string(), e);
                }
            })
            .response
            .on_hover_text("Target environment");

        // run button (green)
        let (rrect, rresp) = ui.allocate_exact_size(Vec2::new(34.0, 26.0), Sense::click());
        let p = ui.painter();
        if rresp.hovered() {
            p.rect_filled(rrect, 4.0, RUN_GREEN_HI);
        } else {
            p.rect_filled(rrect, 4.0, RUN_GREEN);
        }
        icons::play(p, rrect.center(), 10.0, Color32::WHITE);
        if rresp.clicked() {
            app.do_run();
        }
        rresp.on_hover_text("Run 'demo-project' (Shift+F10)");

        // debug button
        if icon_button(ui, "dbg", Vec2::new(30.0, 26.0), "Debug 'demo-project' (Shift+F9)", |p, c| {
            icons::bug(p, c, TEXT2)
        }) {
            app.do_debug();
        }

        // more menu
        ui.menu_button("⋮", |ui| {
            ui.set_min_width(150.0);
            if ui.button("Run with Options…").clicked() {
                app.do_run();
                ui.close();
            }
            if ui.button("Profile…").clicked() {
                ui.close();
            }
            if ui.button("Clean Project").clicked() {
                app.build_log.push(crate::LogLine::new(crate::LogLevel::Info, "[00:00:00] Cleaned build directory"));
                app.bottom_open = true;
                app.bottom_tab = BottomTab::Build;
                ui.close();
            }
        })
        .response
        .on_hover_text("More actions");

        // right cluster
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icon_button(ui, "bell", Vec2::new(28.0, 26.0), "Notifications", |p, c| {
                icons::bell(p, c, TEXT2);
                p.circle_filled(c + Vec2::new(3.5, -3.5), 2.0, LOG_ERR);
            }) {}
            if icon_button(ui, "person", Vec2::new(28.0, 26.0), "Account: dev", |p, c| icons::person(p, c, TEXT2)) {}
            if icon_button(ui, "gear", Vec2::new(28.0, 26.0), "Settings", |p, c| icons::gear(p, c, TEXT2)) {}
            if icon_button(ui, "search", Vec2::new(28.0, 26.0), "Search Everywhere", |p, c| {
                icons::magnifier(p, c, TEXT2)
            }) {}
        });
    });
}

// ---------- left rail ----------

pub fn rail(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.add_space(6.0);
        rail_btn(ui, "files", "Project", app.show_tree, |p, c| icons::grid_dots(p, c, TEXT2));
        rail_btn(ui, "search", "Search Everywhere", false, |p, c| icons::magnifier(p, c, TEXT2));
        rail_btn(ui, "branch", "Git: main", false, |p, c| icons::branch(p, c, TEXT2));
        rail_btn(ui, "run", "Run 'demo-project'", false, |p, c| {
            icons::play(p, c, 8.0, PLAY_GREEN)
        });
        rail_btn(ui, "term", "Terminal", app.bottom_open && app.bottom_tab == BottomTab::Terminal, |p, c| {
            icons::terminal_ico(p, c, TEXT2)
        });
    });
}

fn rail_btn(
    ui: &mut egui::Ui,
    _id: &str,
    tooltip: &str,
    active: bool,
    paint: impl FnOnce(&egui::Painter, Pos2),
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(40.0, 34.0), Sense::click());
    if active {
        ui.painter().rect_filled(rect, 0.0, SEL_BG);
        ui.painter().line_segment(
            [Pos2::new(rect.left(), rect.top()), Pos2::new(rect.left(), rect.bottom())],
            Stroke::new(2.0_f32, ACCENT),
        );
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 0.0, HOVER);
    }
    paint(ui.painter(), rect.center());
    resp.on_hover_text(tooltip).clicked()
}

// ---------- status bar ----------

pub fn status_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_centered(|ui| {
        let path = match app.active {
            Some(id) => app.project.path_of(id),
            None => "demo-project".to_string(),
        };
        ui.label(egui::RichText::new(path).size(10.5).color(TEXT2));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.label(egui::RichText::new(&app.env).size(10.5).color(TEXT3));
            ui.label(egui::RichText::new("UTF-8").size(10.5).color(TEXT2));
            ui.label(egui::RichText::new("LF").size(10.5).color(TEXT2));
            ui.label(egui::RichText::new("4 spaces").size(10.5).color(TEXT2));
            let ln = app.active_line();
            ui.label(
                egui::RichText::new(format!("Ln {}, Col 1", ln))
                    .size(10.5)
                    .color(TEXT2),
            );
        });
    });
}

#[allow(dead_code)]
fn unused(_: Rect, _: Stroke) {}
