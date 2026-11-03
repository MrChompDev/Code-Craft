// CodeCraft — a personal IntelliJ-inspired IDE (Rust + eframe/egui).
mod bottom_ui;
mod chrome;
mod editor_ui;
mod icons;
mod project;
mod theme;
mod tree_ui;

use eframe::egui;
use project::{Id, Project};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BottomTab {
    Terminal,
    Build,
    Debug,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Err,
    Ok,
    Debug,
    Plain,
}

pub struct LogLine {
    pub level: LogLevel,
    pub text: String,
}

impl LogLine {
    fn new(level: LogLevel, text: impl Into<String>) -> Self {
        Self { level, text: text.into() }
    }
}

pub enum TermLine {
    Prompt { cmd: String },
    Out(String),
    Err(String),
    Ok(String),
    Info(String),
}

pub struct Terminal {
    pub lines: Vec<TermLine>,
    pub input: String,
    pub hist: Vec<String>,
    pub hist_idx: Option<usize>,
}

pub enum Pending {
    New { parent: Id, is_dir: bool, name: String },
    Rename { node: Id, name: String },
}

pub struct App {
    // screenshot harness
    pub frames: u32,
    pub shot_sent: bool,
    pub screenshot_path: Option<String>,
    // data
    pub project: Project,
    pub open_tabs: Vec<Id>,
    pub active: Option<Id>,
    pub selected: Option<Id>,
    pub cur_line: HashMap<Id, usize>,
    pub folds: HashMap<(Id, usize), bool>,
    pub fold_action: Option<bool>,
    pub expanded: HashSet<Id>,
    // chrome state
    pub show_tree: bool,
    pub bottom_open: bool,
    pub bottom_tab: BottomTab,
    pub bottom_h: f32,
    pub env: String,
    pub envs: Vec<&'static str>,
    pub running: bool,
    pub about_open: bool,
    pub shortcuts_open: bool,
    pub term_focus: bool,
    pub pending: Option<Pending>,
    // panels
    pub term: Terminal,
    pub build_log: Vec<LogLine>,
    pub dbg_log: Vec<LogLine>,
}

impl App {
    pub fn open_file(&mut self, id: Id) {
        if self.project.nodes[id].is_dir {
            self.selected = Some(id);
            return;
        }
        self.selected = Some(id);
        if !self.open_tabs.contains(&id) {
            self.open_tabs.push(id);
        }
        self.active = Some(id);
        self.cur_line.entry(id).or_insert(1);
    }

    pub fn close_tab(&mut self, id: Id) {
        if let Some(pos) = self.open_tabs.iter().position(|&t| t == id) {
            self.open_tabs.remove(pos);
            if self.active == Some(id) {
                self.active = if pos > 0 {
                    Some(self.open_tabs[pos - 1])
                } else {
                    self.open_tabs.first().copied()
                };
            }
        }
    }

    pub fn active_line(&self) -> usize {
        self.active.and_then(|a| self.cur_line.get(&a).copied()).unwrap_or(1)
    }

    pub fn intent_new_file(&mut self) {
        let parent = match self.selected {
            Some(id) if self.project.nodes[id].is_dir => id,
            Some(id) => self.project.nodes[id].parent.unwrap_or(self.project.root),
            None => self.project.root,
        };
        self.expanded.insert(parent);
        self.pending = Some(Pending::New { parent, is_dir: false, name: String::new() });
    }

    pub fn intent_new_folder(&mut self) {
        let parent = match self.selected {
            Some(id) if self.project.nodes[id].is_dir => id,
            Some(id) => self.project.nodes[id].parent.unwrap_or(self.project.root),
            None => self.project.root,
        };
        self.expanded.insert(parent);
        self.pending = Some(Pending::New { parent, is_dir: true, name: String::new() });
    }

    pub fn commit_pending(&mut self) {
        match self.pending.take() {
            Some(Pending::New { parent, is_dir, name }) => {
                let name = name.trim().to_string();
                if name.is_empty() {
                    return;
                }
                let name = self.project.unique_name(parent, &name, is_dir);
                let id = if is_dir {
                    let nid = self.project.nodes.len();
                    self.project.nodes.push(project::Node {
                        name: name.clone(),
                        is_dir: true,
                        parent: Some(parent),
                        children: Vec::new(),
                        content: None,
                    });
                    self.project.nodes[parent].children.push(nid);
                    self.expanded.insert(nid);
                    nid
                } else {
                    self.project.add(parent, &name, false, Some(""))
                };
                self.selected = Some(id);
                if !is_dir {
                    self.open_file(id);
                }
            }
            Some(Pending::Rename { node, name }) => {
                let name = name.trim().to_string();
                if !name.is_empty() {
                    self.project.nodes[node].name = name;
                }
            }
            None => {}
        }
    }

    pub fn delete_node(&mut self, id: Id) {
        if id == self.project.root {
            return;
        }
        self.close_tab(id);
        if let Some(parent) = self.project.nodes[id].parent {
            self.project.nodes[parent].children.retain(|&c| c != id);
        }
        if self.selected == Some(id) {
            self.selected = None;
        }
    }

    pub fn toggle_folds(&mut self, collapse: bool) {
        self.fold_action = Some(collapse);
    }

    pub fn do_build(&mut self) {
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Build;
        let (files, dirs) = self.project.stats();
        let t = now_stamp();
        let log = &mut self.build_log;
        log.push(LogLine::new(LogLevel::Info, format!("[{t}] Scanning project 'demo-project'…")));
        log.push(LogLine::new(LogLevel::Info, format!("[{t}] {files} source files, {dirs} folders")));
        for name in ["src/main.js", "src/app.py", "web/app.js", "web/styles.css"] {
            log.push(LogLine::new(LogLevel::Info, format!("[{t}] Compiling {name}…")));
        }
        log.push(LogLine::new(LogLevel::Warn, "[t] src/utils.js:31 — unused import 'legacy'".replace("[t]", &format!("[{t}]"))));
        log.push(LogLine::new(LogLevel::Info, format!("[{t}] Linking…")));
        log.push(LogLine::new(LogLevel::Ok, format!("[{t}] Build finished in 1.24s — 0 errors, 1 warning")));
    }

    pub fn do_run(&mut self) {
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Build;
        self.running = true;
        let env = self.env.clone();
        let t = now_stamp();
        let log = &mut self.build_log;
        log.push(LogLine::new(LogLevel::Info, format!("[{t}] Running 'demo-project' (target: {env})…")));
        log.push(LogLine::new(LogLevel::Plain, "[runner] 2 tasks in 18ms"));
        log.push(LogLine::new(LogLevel::Plain, "bootstrap done after 600 frames"));
        log.push(LogLine::new(LogLevel::Ok, format!("[{t}] Process exited with code 0")));
        self.running = false;
    }

    pub fn do_debug(&mut self) {
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Debug;
        let env = self.env.clone();
        let t = now_stamp();
        let log = &mut self.dbg_log;
        log.push(LogLine::new(LogLevel::Debug, format!("[{t}] Debugger attached (target: {env})")));
        log.push(LogLine::new(LogLevel::Debug, "[t] Breakpoint hit — src/main.js:37 in bootstrap()".replace("[t]", &format!("[{t}]"))));
        log.push(LogLine::new(LogLevel::Debug, "[t] TaskRunner.queue = [render, makeTimer]".replace("[t]", &format!("[{t}]"))));
        log.push(LogLine::new(LogLevel::Debug, "[t] Stepping… resumed".replace("[t]", &format!("[{t}]"))));
    }

    pub fn do_stop(&mut self) {
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Build;
        self.running = false;
        let t = now_stamp();
        self.build_log.push(LogLine::new(LogLevel::Err, format!("[{t}] Stopped by user (exit code 130)")));
    }
}

fn now_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (h, m, s) = clock_of(secs);
    format!("{h:02}:{m:02}:{s:02}")
}

pub fn clock_of(secs: u64) -> (u32, u32, u32) {
    let rem = secs % 86400;
    ((rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

/// Civil date from days since epoch (Howard Hinnant's algorithm).
pub fn date_of(secs: u64) -> (i64, u32, u32, u32) {
    let days = (secs / 86400) as i64;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d, (secs % 86400) as u32)
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

pub fn date_string() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, rem) = date_of(secs);
    let dow = ((secs / 86400 + 3) % 7) as usize; // epoch (Thu) → index 3
    let (h, m, s) = clock_of(secs);
    format!(
        "{} {} {} {:04} {:02}:{:02}:{:02} UTC",
        DAYS[dow], MONTHS[(mo - 1) as usize], d, y, h, m, s
    )
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let screenshot_path = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string());

    let icon = load_icon(include_bytes!("../assets/icon256.png"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("CodeCraft — demo-project")
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    let state = App::new(screenshot_path);
    eframe::run_native(
        "CodeCraft",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(state))
        }),
    )
}

impl App {
    fn new(screenshot_path: Option<String>) -> Self {
        let project = Project::new_demo();
        let root = project.root;
        let main_js = project.find_child(project.find_child(root, "src").unwrap(), "main.js").unwrap();
        let app_py = project.find_child(project.find_child(root, "src").unwrap(), "app.py").unwrap();
        let mut expanded = HashSet::new();
        expanded.insert(root);
        for d in ["src", "web", "data", "tests"] {
            if let Some(id) = project.find_child(root, d) {
                expanded.insert(id);
            }
        }
        let mut term = Terminal {
            lines: Vec::new(),
            input: String::new(),
            hist: Vec::new(),
            hist_idx: None,
        };
        term.lines.push(TermLine::Info("CodeCraft Terminal — type 'help' for commands.".into()));
        term.lines.push(TermLine::Prompt { cmd: "ls".into() });
        term.lines.push(TermLine::Out("src/  web/  data/  tests/  README.md  .gitignore".into()));

        let mut build_log = Vec::new();
        build_log.push(LogLine::new(LogLevel::Info, "[06:31:04] Scanning project 'demo-project'…"));
        build_log.push(LogLine::new(LogLevel::Info, "[06:31:04] 11 source files, 5 folders"));
        build_log.push(LogLine::new(LogLevel::Info, "[06:31:05] Compiling src/main.js…"));
        build_log.push(LogLine::new(LogLevel::Info, "[06:31:05] Compiling src/app.py…"));
        build_log.push(LogLine::new(LogLevel::Warn, "[06:31:05] src/utils.js:31 — unused import 'legacy'"));
        build_log.push(LogLine::new(LogLevel::Info, "[06:31:06] Linking…"));
        build_log.push(LogLine::new(LogLevel::Ok, "[06:31:06] Build finished in 1.24s — 0 errors, 1 warning"));

        let mut dbg_log = Vec::new();
        dbg_log.push(LogLine::new(LogLevel::Debug, "[06:31:10] Debugger attached (target: Current File)"));
        dbg_log.push(LogLine::new(LogLevel::Debug, "[06:31:12] Breakpoint hit — src/main.js:37 in bootstrap()"));

        let envs = vec![
            "Current File",
            "Debug Build",
            "Release Build",
            "Docker: codecraft-dev",
            "Remote: production",
        ];

        Self {
            frames: 0,
            shot_sent: false,
            screenshot_path,
            open_tabs: vec![main_js, app_py],
            active: Some(main_js),
            selected: Some(main_js),
            cur_line: [(main_js, 37), (app_py, 24)].into_iter().collect(),
            folds: HashMap::new(),
            fold_action: None,
            expanded,
            show_tree: true,
            bottom_open: true,
            bottom_tab: BottomTab::Terminal,
            bottom_h: 190.0,
            env: "Current File".into(),
            envs,
            running: false,
            about_open: false,
            shortcuts_open: false,
            term_focus: true,
            pending: None,
            term,
            build_log,
            dbg_log,
            project,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // screenshot harness: after the UI settles, ask eframe to read the
        // painted framebuffer; the image arrives as an input event next frame.
        self.frames += 1;
        if self.screenshot_path.is_some() && !self.shot_sent && self.frames >= 116 {
            self.shot_sent = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let shot_img = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot_img {
            if let Some(path) = self.screenshot_path.clone() {
                let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
                let out = image::RgbaImage::from_raw(img.width() as u32, img.height() as u32, rgba)
                    .unwrap_or_else(|| image::RgbaImage::new(1, 1));
                let _ = out.save(&path);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // global shortcuts
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl;
            if ctrl && i.key_pressed(egui::Key::N) {
                self.intent_new_file();
            }
            if ctrl && i.key_pressed(egui::Key::B) {
                self.show_tree = !self.show_tree;
            }
            if ctrl && i.key_pressed(egui::Key::Backtick) {
                self.bottom_open = !self.bottom_open;
            }
            if i.modifiers.shift && i.key_pressed(egui::Key::F10) {
                self.do_run();
            }
            if i.modifiers.shift && i.key_pressed(egui::Key::F9) {
                self.do_debug();
            }
            if i.key_pressed(egui::Key::Escape) {
                self.about_open = false;
                self.shortcuts_open = false;
                self.term_focus = false;
                self.pending = None;
            }
        });

        // layout
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .frame(egui::Frame::new().fill(theme::BAR_BG))
            .show(ctx, |ui| chrome::status_bar(self, ui));

        egui::TopBottomPanel::top("menu_bar")
            .exact_height(27.0)
            .frame(egui::Frame::new().fill(theme::BAR_BG))
            .show(ctx, |ui| chrome::menu_bar(self, ui));

        egui::TopBottomPanel::top("toolbar")
            .exact_height(42.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BAR_BG)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER)),
            )
            .show(ctx, |ui| chrome::toolbar(self, ui));

        egui::SidePanel::left("rail")
            .exact_width(40.0)
            .frame(egui::Frame::new().fill(theme::BAR_BG))
            .show(ctx, |ui| chrome::rail(self, ui));

        if self.show_tree {
            egui::SidePanel::left("tree_panel")
                .resizable(true)
                .default_width(252.0)
                .width_range(170.0..=440.0)
                .frame(egui::Frame::new().fill(theme::PANEL_BG))
                .show(ctx, |ui| tree_ui::draw(self, ui));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::EDITOR_BG))
            .show(ctx, |ui| {
                let open = self.bottom_open;
                let mut panel = egui::TopBottomPanel::bottom("bottom_panel")
                    .resizable(open)
                    .min_height(if open { 90.0 } else { 26.0 })
                    .max_height(if open { 520.0 } else { 26.0 })
                    .frame(egui::Frame::new().fill(theme::PANEL_BG).stroke(egui::Stroke::new(1.0, theme::BORDER)));
                if open {
                    panel = panel.default_height(self.bottom_h);
                } else {
                    panel = panel.exact_height(26.0);
                }
                let resp = panel.show_inside(ui, |ui| {
                    if open {
                        bottom_ui::draw(self, ui);
                    } else {
                        bottom_ui::draw_collapsed(self, ui);
                    }
                });
                if open {
                    self.bottom_h = resp.response.rect.height().max(90.0);
                }

                editor_ui::draw(self, ui);
            });

        // modals
        modals(self, ctx);
    }
}

fn modals(app: &mut App, ctx: &egui::Context) {
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

fn load_icon(bytes: &[u8]) -> egui::IconData {
    let img = image::load_from_memory(bytes).expect("icon").to_rgba8();
    egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
}

