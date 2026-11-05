// Bottom panel: Terminal / Build Output / Debug Console.
use eframe::egui;
use egui::FontFamily;
use crate::icons;
use crate::theme::*;
use crate::{BottomTab, LogLevel, TermLine, App};
use egui::{Align2, Color32, FontId, Pos2, Sense, Vec2};

pub fn draw(app: &mut App, ui: &mut egui::Ui) {
    // header
    let (hrect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(hrect, 0.0, PANEL_BG);
    p.line_segment(
        [Pos2::new(hrect.left(), hrect.bottom()), Pos2::new(hrect.right(), hrect.bottom())],
        egui::Stroke::new(1.0_f32, BORDER),
    );

    let names = [
        (BottomTab::Terminal, "Terminal"),
        (BottomTab::Build, "Build Output"),
        (BottomTab::Debug, "Debug Console"),
    ];
    let mut x = hrect.left() + 10.0;
    for (tab, label) in names {
        let active = app.bottom_tab == tab;
        let font = FontId::new(12.0, FontFamily::Proportional);
        let w = p.layout_no_wrap(label.to_string(), font.clone(), TEXT).rect.width() + 16.0;
        let tr = egui::Rect::from_min_size(Pos2::new(x, hrect.top() + 3.0), Vec2::new(w, 26.0));
        let resp = ui.interact(tr, egui::Id::new("btab").with(x as u64), Sense::click());
        if resp.clicked() {
            app.bottom_tab = tab;
            if tab == BottomTab::Terminal {
                app.term_focus = true;
            }
        }
        if active {
            p.line_segment(
                [Pos2::new(tr.left() + 4.0, tr.bottom()), Pos2::new(tr.right() - 4.0, tr.bottom())],
                egui::Stroke::new(2.0_f32, ACCENT),
            );
        }
        p.text(
            Pos2::new(tr.center().x, tr.center().y),
            Align2::CENTER_CENTER,
            label,
            font,
            if active { TEXT } else { TEXT2 },
        );
        x += w;
    }

    // right side: stop / clear / collapse
    let cy = hrect.center().y;
    let stop_r = egui::Rect::from_center_size(Pos2::new(hrect.right() - 78.0, cy), Vec2::splat(22.0));
    let clear_r = egui::Rect::from_center_size(Pos2::new(hrect.right() - 48.0, cy), Vec2::splat(22.0));
    let col_r = egui::Rect::from_center_size(Pos2::new(hrect.right() - 18.0, cy), Vec2::splat(22.0));

    if app.running {
        let r = ui.interact(stop_r, egui::Id::new("stop_btn"), Sense::click());
        if r.hovered() {
            p.rect_filled(stop_r, 3.0, HOVER);
        }
        icons::stop(p, stop_r.center(), if r.hovered() { TEXT } else { LOG_ERR });
        if r.clicked() {
            app.do_stop();
        }
        r.on_hover_text("Stop");
    }

    let r = ui.interact(clear_r, egui::Id::new("clear_btn"), Sense::click());
    if r.hovered() {
        p.rect_filled(clear_r, 3.0, HOVER);
    }
    icons::trash(p, clear_r.center(), if r.hovered() { TEXT } else { TEXT3 });
    if r.clicked() {
        match app.bottom_tab {
            BottomTab::Terminal => app.term.lines.clear(),
            BottomTab::Build => app.build_log.clear(),
            BottomTab::Debug => app.dbg_log.clear(),
        }
    }
    r.on_hover_text("Clear");

    let r = ui.interact(col_r, egui::Id::new("collapse_btn"), Sense::click());
    if r.hovered() {
        p.rect_filled(col_r, 3.0, HOVER);
    }
    icons::chevron(p, col_r.center(), 5.0, TEXT2, icons::Chev::Down);
    if r.clicked() {
        app.bottom_open = false;
    }
    r.on_hover_text("Hide panel");

    ui.add_space(2.0);

    match app.bottom_tab {
        BottomTab::Terminal => terminal_tab(app, ui),
        BottomTab::Build => log_tab(ui, &app.build_log),
        BottomTab::Debug => log_tab(ui, &app.dbg_log),
    }
}

pub fn draw_collapsed(app: &mut App, ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 0.0, PANEL_BG);
    let names = ["Terminal", "Build Output", "Debug Console"];
    let mut x = rect.left() + 12.0;
    for (i, label) in names.iter().enumerate() {
        let tab = [BottomTab::Terminal, BottomTab::Build, BottomTab::Debug][i];
        let font = FontId::new(11.5, FontFamily::Proportional);
        let w = p.layout_no_wrap((*label).to_string(), font.clone(), TEXT2).rect.width() + 12.0;
        let tr = egui::Rect::from_min_size(Pos2::new(x, rect.center().y - 11.0), Vec2::new(w, 22.0));
        let r = ui.interact(tr, egui::Id::new("btab_c").with(i), Sense::click());
        p.text(
            Pos2::new(tr.center().x, tr.center().y),
            Align2::CENTER_CENTER,
            label,
            font,
            if r.hovered() { TEXT } else { TEXT3 },
        );
        if r.clicked() {
            app.bottom_open = true;
            app.bottom_tab = tab;
        }
        x += w + 8.0;
    }
    let up_r = egui::Rect::from_center_size(Pos2::new(rect.right() - 18.0, rect.center().y), Vec2::splat(22.0));
    let r = ui.interact(up_r, egui::Id::new("expand_btn"), Sense::click());
    if r.hovered() {
        p.rect_filled(up_r, 3.0, HOVER);
    }
    icons::chevron(p, up_r.center(), 5.0, TEXT2, icons::Chev::Up);
    if r.clicked() {
        app.bottom_open = true;
    }
    r.on_hover_text("Show panel");
}

// ---------------- terminal ----------------

fn terminal_tab(app: &mut App, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            let mono = FontId::monospace(12.2);
            let lines = &app.term.lines;
            for line in lines {
                match line {
                    TermLine::Prompt { cmd } => {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label(
                                egui::RichText::new("dev@codecraft")
                                    .font(mono.clone())
                                    .color(LOG_OK),
                            );
                            ui.label(egui::RichText::new(":").font(mono.clone()).color(TEXT2));
                            ui.label(
                                egui::RichText::new("~/demo-project")
                                    .font(mono.clone())
                                    .color(SYN_FN),
                            );
                            ui.label(egui::RichText::new("$ ").font(mono.clone()).color(TEXT2));
                            ui.label(egui::RichText::new(cmd.clone()).font(mono.clone()).color(TEXT));
                        });
                    }
                    TermLine::Out(t) => {
                        ui.label(egui::RichText::new(t.clone()).font(mono.clone()).color(SYN_PLAIN));
                    }
                    TermLine::Err(t) => {
                        ui.label(egui::RichText::new(t.clone()).font(mono.clone()).color(LOG_ERR));
                    }
                    TermLine::Ok(t) => {
                        ui.label(egui::RichText::new(t.clone()).font(mono.clone()).color(LOG_OK));
                    }
                    TermLine::Info(t) => {
                        ui.label(egui::RichText::new(t.clone()).font(mono.clone()).color(TEXT2));
                    }
                }
            }

            // prompt + input
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
                ui.label(egui::RichText::new(user).font(mono.clone()).color(LOG_OK));
                ui.label(egui::RichText::new(":").font(mono.clone()).color(TEXT2));
                let short_cwd = app
                    .cwd
                    .to_string_lossy()
                    .replacen(&std::env::var("HOME").unwrap_or_default(), "~", 1);
                ui.label(egui::RichText::new(short_cwd).font(mono.clone()).color(SYN_FN));
                ui.label(egui::RichText::new("$ ").font(mono.clone()).color(TEXT2));
                let out = egui::TextEdit::singleline(&mut app.term.input)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(ui.available_width())
                    .frame(false)
                    .show(ui);
                let entered = out.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if app.term_focus && !out.response.has_focus() {
                    out.response.request_focus();
                }
                if entered {
                    let cmd = app.term.input.trim().to_string();
                    app.term.input.clear();
                    app.exec_command(&cmd);
                    app.term_focus = true;
                }
                // history navigation
                if out.response.has_focus() {
                    let up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                    let down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
                    if up && !app.term.hist.is_empty() {
                        let idx = app.term.hist_idx.unwrap_or(app.term.hist.len());
                        app.term.hist_idx = Some(idx.saturating_sub(1));
                        if let Some(i) = app.term.hist_idx {
                            app.term.input = app.term.hist[i].clone();
                        }
                    } else if down {
                        if let Some(i) = app.term.hist_idx {
                            if i + 1 < app.term.hist.len() {
                                app.term.hist_idx = Some(i + 1);
                                app.term.input = app.term.hist[i + 1].clone();
                            } else {
                                app.term.hist_idx = None;
                                app.term.input.clear();
                            }
                        }
                    }
                }
            });
        });
}



// ---------------- logs ----------------

fn log_tab(ui: &mut egui::Ui, log: &[crate::LogLine]) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            let mono = FontId::monospace(12.2);
            for line in log {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.label(
                        egui::RichText::new(level_label(line.level))
                            .font(mono.clone())
                            .color(level_color(line.level)),
                    );
                    ui.label(egui::RichText::new(line.text.clone()).font(mono.clone()).color(SYN_PLAIN));
                });
            }
        });
}

fn level_label(l: LogLevel) -> &'static str {
    match l {
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Err => "ERROR",
        LogLevel::Ok => "DONE",
        LogLevel::Debug => "DBG",
        LogLevel::Plain => "OUT",
    }
}

fn level_color(l: LogLevel) -> Color32 {
    match l {
        LogLevel::Info => LOG_INFO,
        LogLevel::Warn => LOG_WARN,
        LogLevel::Err => LOG_ERR,
        LogLevel::Ok => LOG_OK,
        LogLevel::Debug => SYN_TYPE,
        LogLevel::Plain => TEXT2,
    }
}
