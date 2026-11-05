// Quick-open (Ctrl+P) and find-in-project (Ctrl+Shift+F) overlays.
use eframe::egui;
use crate::project::Id;
use crate::theme;
use crate::App;

/// Quick-open / find-in-files overlay state.
pub struct Palette {
    pub mode: PaletteMode,
    pub query: String,
    /// matched file ids for Files mode
    pub files: Vec<Id>,
    pub selected: usize,
    /// incremental text-search state for Search mode
    pub scan: Option<ScanState>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PaletteMode {
    Files,
    Search,
}

/// Incremental "find in project" scan: advances a few files per frame.
pub struct ScanState {
    pub queue: Vec<Id>,
    pub pos: usize,
    pub results: Vec<(Id, usize, String)>,
}



/// Quick-open (Ctrl+P) / find-in-files (Ctrl+Shift+F) overlay.
pub fn palette_ui(app: &mut App, ctx: &egui::Context) {
    let mut close = false;
    let mut activate = false;
    egui::Window::new(match app.palette.as_ref().map(|p| p.mode) {
        Some(PaletteMode::Search) => "Find in Project",
        _ => "Go to File",
    })
    .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
    .collapsible(false)
    .resizable(false)
    .default_width(560.0)
    .show(ctx, |ui| {
        let Some(palette) = app.palette.as_mut() else { return };
        let field = ui.add_sized(
            [ui.available_width(), 26.0],
            egui::TextEdit::singleline(&mut palette.query)
                .hint_text(match palette.mode {
                    PaletteMode::Files => "type a file name…",
                    PaletteMode::Search => "search text across the project…",
                }),
        );
        if !field.has_focus() {
            field.request_focus();
        }
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            activate = true;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }

        match palette.mode {
            PaletteMode::Files => {
                // fuzzy subsequence filter over project files
                let q = palette.query.to_lowercase();
                let files: Vec<Id> = app.project.walk_files();
                let mut scored: Vec<(usize, Id)> = files
                    .into_iter()
                    .filter_map(|id| {
                        let name = app.project.nodes[id].name.to_lowercase();
                        if q.is_empty() {
                            return Some((0, id));
                        }
                        // subsequence match with a small score
                        let mut qi = q.chars().peekable();
                        let mut score = 0;
                        for (i, ch) in name.chars().enumerate() {
                            if qi.peek() == Some(&ch) {
                                qi.next();
                                score += i;
                                if qi.peek().is_none() {
                                    break;
                                }
                            }
                        }
                        (qi.peek().is_none()).then_some((score, id))
                    })
                    .collect();
                scored.sort_by_key(|(s, _)| *s);
                let list: Vec<Id> = scored.into_iter().map(|(_, id)| id).take(40).collect();
                palette.files = list.clone();
                if palette.selected >= list.len() {
                    palette.selected = list.len().saturating_sub(1);
                }
                egui::ScrollArea::vertical().max_height(320.0).auto_shrink(false).show(ui, |ui| {
                    for (i, &id) in list.iter().enumerate() {
                        let path = app.project.path_of(id);
                        let selected = i == palette.selected;
                        let label = egui::RichText::new(path).size(12.5).color(if selected {
                            theme::TEXT
                        } else {
                            theme::TEXT2
                        });
                        if ui.add(egui::Button::new(label).fill(if selected { theme::HOVER } else { egui::Color32::TRANSPARENT })).clicked() {
                            palette.selected = i;
                            activate = true;
                        }
                    }
                });
            }
            PaletteMode::Search => {
                let (n, pos, total) = match &palette.scan {
                    Some(s) => (s.results.len(), s.pos, s.queue.len()),
                    None => (0, 0, 0),
                };
                ui.weak(format!("{n} results — scanning {pos}/{total}"));
                let results: Vec<(Id, usize, String)> = match &palette.scan {
                    Some(s) => s.results.clone(),
                    None => Vec::new(),
                };
                egui::ScrollArea::vertical().max_height(320.0).auto_shrink(false).show(ui, |ui| {
                    for (i, (id, line_no, text)) in results.iter().enumerate() {
                        let path = format!("{}:{}", app.project.path_of(*id), line_no);
                        let label = format!("{path}  {text}");
                        if ui
                            .add(egui::Button::new(egui::RichText::new(label).size(11.5).color(theme::TEXT2)).fill(egui::Color32::TRANSPARENT))
                            .clicked()
                        {
                            palette.selected = i;
                            activate = true;
                        }
                    }
                });
            }
        }
    });
    let _ = close;
    if activate {
        if let Some(palette) = app.palette.take() {
            match palette.mode {
                PaletteMode::Files => {
                    if let Some(&id) = palette.files.get(palette.selected) {
                        app.open_file(id);
                    }
                }
                PaletteMode::Search => {
                    if let Some((id, line_no, _)) = palette.scan.as_ref().and_then(|s| s.results.get(palette.selected)) {
                        let id = *id;
                        app.open_file(id);
                        app.cur_line.insert(id, *line_no);
                        app.scroll_to_cursor = true;
                    } else {
                        app.palette = Some(palette);
                    }
                }
            }
        }
    }
}

impl App {
    /// Advance the incremental find-in-files scan by a small slice per frame.
    pub fn step_scan(&mut self, ctx: &egui::Context) {
        let Some(mut scan) = self.palette.as_mut().and_then(|p| p.scan.take()) else {
            return;
        };
        let query = self.palette.as_ref().map(|p| p.query.clone()).unwrap_or_default();
        if query.len() < 2 {
            // keep the scan slot alive so shortening the query doesn't kill the search
            if let Some(p) = self.palette.as_mut() {
                p.scan = Some(scan);
            }
            return;
        }
        let q = query.to_lowercase();
        let budget = 24;
        let mut done = 0;
        while scan.pos < scan.queue.len() && done < budget {
            let id = scan.queue[scan.pos];
            scan.pos += 1;
            done += 1;
            let Ok(text) = self.project.read_file(id) else { continue };
            if text.len() > 1_000_000 {
                continue;
            }
            for (i, line) in text.lines().enumerate() {
                if line.to_lowercase().contains(&q) {
                    scan.results.push((id, i + 1, line.trim().to_string()));
                    if scan.results.len() >= 300 {
                        scan.pos = scan.queue.len();
                        break;
                    }
                }
            }
        }
        let still_scanning = scan.pos < scan.queue.len();
        if let Some(p) = self.palette.as_mut() {
            p.scan = Some(scan);
        }
        if still_scanning || done == budget {
            ctx.request_repaint();
        }
    }
}
