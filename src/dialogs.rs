// Modal dialogs: open-folder, about, keyboard shortcuts.
use crate::icons;
use crate::theme;
use crate::App;
use eframe::egui;

pub fn modals(app: &mut App, ctx: &egui::Context) {
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
                egui::Grid::new("shortcuts_grid")
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
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
