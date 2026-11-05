// CodeCraft — a personal IntelliJ-inspired IDE (Rust + eframe/egui).
mod bottom_ui;
mod chrome;
mod dialogs;
mod editor_ui;
mod icons;
mod palette;
mod project;
mod theme;
mod tree_ui;

use eframe::egui;
use palette::{Palette, PaletteMode, ScanState};
use project::{Id, Project};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::sync::mpsc::Receiver;

/// An open document: text loaded from disk plus dirty tracking.
pub struct Buffer {
    pub text: String,
    pub dirty: bool,
}

/// Background terminal job output. 0 = stdout, 1 = stderr, 2 = exit status line.
pub type TermMsg = (u8, String);

/// "Open Folder" dialog state.
pub struct OpenDialog {
    pub path: String,
    pub error: Option<String>,
}

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
    // real engine state
    /// Open document text keyed by node id. Buffers load lazily from disk.
    pub buffers: HashMap<Id, Buffer>,
    /// Working directory for terminal commands (project root by default).
    pub cwd: PathBuf,
    /// PID of a terminal-spawned process (for the Stop button).
    pub child_pid: Option<u32>,
    /// Background job output feeding the terminal.
    pub term_rx: Option<Receiver<TermMsg>>,
    /// Background cargo build output feeding the Build tab.
    pub build_rx: Option<Receiver<TermMsg>>,
    pub open_dialog: Option<OpenDialog>,
    pub palette: Option<Palette>,
    /// Recently opened project folders, newest first (persisted).
    pub recent: Vec<String>,
    /// Window title needs re-applying to the viewport.
    pub title_dirty: bool,
    /// Active editor should scroll its cursor into view this frame.
    pub scroll_to_cursor: bool,
    /// Memoized syntax tokens for the active editor: id -> (text hash, tokens).
    /// Avoids re-tokenizing the whole file on every layout pass.
    pub token_cache: HashMap<Id, (u64, Vec<Vec<crate::editor_ui::Tok>>)>,
}

impl App {
    pub fn open_file(&mut self, id: Id) {
        if self.project.nodes[id].is_dir {
            self.selected = Some(id);
            return;
        }
        self.selected = Some(id);
        if !self.buffers.contains_key(&id) {
            if self.project.looks_binary(id) {
                let t = now_stamp();
                self.build_log.push(LogLine::new(
                    LogLevel::Info,
                    format!(
                        "[{t}] {} looks binary — not opened as text",
                        self.project.path_of(id)
                    ),
                ));
                return;
            }
            match self.project.read_file(id) {
                Ok(text) => {
                    self.buffers.insert(id, Buffer { text, dirty: false });
                }
                Err(e) => {
                    let t = now_stamp();
                    self.build_log.push(LogLine::new(
                        LogLevel::Err,
                        format!("[{t}] Cannot open {}: {e}", self.project.path_of(id)),
                    ));
                    self.bottom_open = true;
                    self.bottom_tab = BottomTab::Build;
                }
            }
        }
        if !self.open_tabs.contains(&id) {
            self.open_tabs.push(id);
        }
        self.active = Some(id);
        self.cur_line.entry(id).or_insert(1);
        self.term_focus = false;
    }

    /// Buffer text for a tab (empty when not loaded).
    pub fn buffer_text(&self, id: Id) -> &str {
        self.buffers.get(&id).map(|b| b.text.as_str()).unwrap_or("")
    }

    pub fn mark_dirty(&mut self, id: Id, dirty: bool) {
        if let Some(b) = self.buffers.get_mut(&id) {
            b.dirty = dirty;
        }
    }

    /// Write the active buffer to disk. Returns true on success.
    pub fn save_active(&mut self) -> bool {
        let Some(id) = self.active else { return false };
        let Some(buf) = self.buffers.get(&id) else { return false };
        let text = buf.text.clone();
        let t = now_stamp();
        match self.project.write_file(id, &text) {
            Ok(()) => {
                self.mark_dirty(id, false);
                let name = self.project.nodes[id].name.clone();
                self.term.lines.push(TermLine::Ok(format!("saved {name}")));
                true
            }
            Err(e) => {
                self.build_log.push(LogLine::new(
                    LogLevel::Err,
                    format!("[{t}] Save failed {}: {e}", self.project.path_of(id)),
                ));
                self.bottom_open = true;
                self.bottom_tab = BottomTab::Build;
                false
            }
        }
    }

    pub fn save_all(&mut self) {
        let ids: Vec<Id> = self
            .buffers
            .iter()
            .filter(|(_, b)| b.dirty)
            .map(|(&id, _)| id)
            .collect();
        for id in ids {
            let prev_active = self.active;
            self.active = Some(id);
            self.save_active();
            self.active = prev_active;
        }
    }

    /// Open a real directory as the project; falls back to logging the error.
    pub fn open_project_dir(&mut self, path: &str) {
        let trimmed = path.trim();
        let expanded_path = if let Some(rest) = trimmed.strip_prefix('~') {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}{rest}")
        } else {
            trimmed.to_string()
        };
        let root = PathBuf::from(&expanded_path);
        match Project::open_disk(&root) {
            Ok(p) => {
                let (files, dirs) = p.stats();
                let name = p.nodes[p.root].name.clone();
                self.cwd = root.clone();
                self.project = p;
                self.buffers.clear();
                self.open_tabs.clear();
                self.active = None;
                self.selected = Some(self.project.root);
                self.cur_line.clear();
                self.folds.clear();
                self.expanded = HashSet::new();
                self.expanded.insert(self.project.root);
                let t = now_stamp();
                self.build_log.push(LogLine::new(LogLevel::Info, format!("[{t}] Opened project '{name}'")));
                self.build_log
                    .push(LogLine::new(LogLevel::Info, format!("[{t}] {files} files, {dirs} folders")));
                self.build_log.push(LogLine::new(
                    LogLevel::Info,
                    format!("[{t}] Root: {}", root.display()),
                ));
                self.remember_recent(&root.to_string_lossy());
                self.title_dirty = true;
                self.palette = None;
            }
            Err(e) => {
                let t = now_stamp();
                self.build_log.push(LogLine::new(
                    LogLevel::Err,
                    format!("[{t}] Cannot open '{trimmed}': {e}"),
                ));
                self.bottom_open = true;
                self.bottom_tab = BottomTab::Build;
            }
        }
    }

    fn settings_dir() -> Option<PathBuf> {
        let base = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("codecraft"))
    }

    fn load_recents() -> Vec<String> {
        let Some(dir) = Self::settings_dir() else { return Vec::new() };
        let Ok(body) = std::fs::read_to_string(dir.join("recents.txt")) else { return Vec::new() };
        body.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
    }

    fn remember_recent(&mut self, path: &str) {
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_string());
        self.recent.truncate(8);
        if let Some(dir) = Self::settings_dir() {
            let _ = std::fs::create_dir_all(&dir);
            let body = self.recent.join("\n");
            let _ = std::fs::write(dir.join("recents.txt"), body);
        }
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
                let t = now_stamp();
                // write through to disk first so the tree only shows real entries
                if let Err(e) = self.project.create_entry(parent, &name, is_dir) {
                    self.build_log.push(LogLine::new(
                        LogLevel::Err,
                        format!("[{t}] Create failed: {e}"),
                    ));
                    self.bottom_open = true;
                    self.bottom_tab = BottomTab::Build;
                    return;
                }
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
                    let t = now_stamp();
                    if let Err(e) = self.project.rename_entry(node, &name) {
                        self.build_log.push(LogLine::new(
                            LogLevel::Err,
                            format!("[{t}] Rename failed: {e}"),
                        ));
                        self.bottom_open = true;
                        self.bottom_tab = BottomTab::Build;
                    } else {
                        self.project.nodes[node].name = name;
                    }
                }
            }
            None => {}
        }
    }

    pub fn delete_node(&mut self, id: Id) {
        if id == self.project.root {
            return;
        }
        let t = now_stamp();
        if let Err(e) = self.project.delete_entry(id) {
            self.build_log
                .push(LogLine::new(LogLevel::Err, format!("[{t}] Delete failed: {e}")));
            self.bottom_open = true;
            self.bottom_tab = BottomTab::Build;
            return;
        }
        self.close_tab(id);
        self.buffers.remove(&id);
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

    /// Rescan the project from disk, keeping open tabs that still exist.
    pub fn refresh_tree(&mut self) {
        let keep: Vec<Id> = self.open_tabs.clone();
        let mapping = self.project.refresh(&keep);
        let t = now_stamp();
        // remap open tabs to their new ids
        self.open_tabs = self
            .open_tabs
            .iter()
            .filter_map(|&old| mapping.get(&old).copied())
            .collect();
        self.active = self
            .active
            .and_then(|old| mapping.get(&old).copied())
            .or(self.open_tabs.first().copied());
        self.selected = self
            .selected
            .and_then(|old| mapping.get(&old).copied())
            .or(Some(self.project.root));
        // remap buffers + cursor lines
        let mut buffers = HashMap::new();
        for (old, buf) in self.buffers.drain() {
            if let Some(&new) = mapping.get(&old) {
                buffers.insert(new, buf);
            }
        }
        self.buffers = buffers;
        self.cur_line = self
            .cur_line
            .iter()
            .filter_map(|(&old, &l)| mapping.get(&old).map(|&new| (new, l)))
            .collect();
        let (files, dirs) = self.project.stats();
        self.build_log.push(LogLine::new(
            LogLevel::Info,
            format!("[{t}] Refreshed: {files} files, {dirs} folders"),
        ));
        self.expanded = HashSet::new();
        self.expanded.insert(self.project.root);
    }

    /// Root-level Cargo.toml means we have a real build target.
    pub fn has_cargo_target(&self) -> bool {
        self.project
            .root_path
            .as_ref()
            .map(|p| p.join("Cargo.toml").is_file())
            .unwrap_or(false)
    }

    /// Spawn a child process, stream its stdout/stderr into the terminal or
    /// build log, and report its exit status. Returns true when running.
    fn spawn_pipeline(&mut self, program: &str, args: &[String], dir: &Path, to_build: bool) -> bool {
        let spawned = Command::new(program)
            .args(args)
            .current_dir(dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
        match spawned {
            Ok(mut child) => {
                self.child_pid = Some(child.id());
                self.running = true;
                let (tx, rx) = mpsc::channel::<TermMsg>();
                let mut out = child.stdout.take().expect("stdout");
                let mut err = child.stderr.take().expect("stderr");
                let tx_out = tx.clone();
                thread::spawn(move || {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(&mut out);
                    for line in reader.lines().map_while(Result::ok) {
                        if tx_out.send((0u8, line)).is_err() {
                            break;
                        }
                    }
                });
                let tx_err = tx.clone();
                thread::spawn(move || {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(&mut err);
                    for line in reader.lines().map_while(Result::ok) {
                        if tx_err.send((1u8, line)).is_err() {
                            break;
                        }
                    }
                });
                thread::spawn(move || {
                    let code = child.wait().ok().and_then(|s| s.code());
                    let _ = tx.send((2u8, format!("{:?}", code)));
                });
                if to_build {
                    self.build_rx = Some(rx);
                } else {
                    self.term_rx = Some(rx);
                }
                true
            }
            Err(e) => {
                let t = now_stamp();
                let msg = format!("[{t}] spawn failed: {e}");
                if to_build {
                    self.build_log.push(LogLine::new(LogLevel::Err, msg));
                } else {
                    self.term.lines.push(TermLine::Err(format!("spawn failed: {e}")));
                }
                self.running = false;
                false
            }
        }
    }

    fn spawn_job(&mut self, argv: &[&str], dir: &Path, to_build: bool) {
        let t = now_stamp();
        let shown = argv.join(" ");
        if to_build {
            self.build_log
                .push(LogLine::new(LogLevel::Info, format!("[{t}] $ {shown}")));
            self.bottom_open = true;
            self.bottom_tab = BottomTab::Build;
        } else {
            self.term.lines.push(TermLine::Info(format!("$ {shown}")));
        }
        let args: Vec<String> = argv[1..].iter().map(|s| s.to_string()).collect();
        self.spawn_pipeline(argv[0], &args, dir, to_build);
    }

    pub fn do_build(&mut self) {
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Build;
        let t = now_stamp();
        if !self.has_cargo_target() {
            self.build_log.push(LogLine::new(
                LogLevel::Warn,
                format!("[{t}] No Cargo.toml at project root — nothing to build."),
            ));
            return;
        }
        let dir = self.project.root_path.clone().unwrap_or_else(|| self.cwd.clone());
        self.spawn_job(&["cargo", "build"], &dir, true);
    }

    pub fn do_run(&mut self) {
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Terminal;
        let _t = now_stamp();
        if !self.has_cargo_target() {
            self.term.lines.push(TermLine::Err(
                "No Cargo.toml at project root — use the terminal to run this project.".into(),
            ));
            return;
        }
        let dir = self.project.root_path.clone().unwrap_or_else(|| self.cwd.clone());
        self.spawn_job(&["cargo", "run"], &dir, false);
    }

    pub fn do_debug(&mut self) {
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Debug;
        let env = self.env.clone();
        let t = now_stamp();
        let log = &mut self.dbg_log;
        log.push(LogLine::new(LogLevel::Debug, format!("[{t}] Debugger attach requested (target: {env})")));
        log.push(LogLine::new(
            LogLevel::Debug,
            format!("[{t}] Debug adapter not wired yet — build with 'cargo build' and run from the terminal."),
        ));
    }

    pub fn do_stop(&mut self) {
        if let Some(pid) = self.child_pid.take() {
            let _ = Command::new("kill").arg(pid.to_string()).status();
        }
        self.running = false;
    }

    /// Execute a terminal command line. Builtins run inline; anything else
    /// spawns through `sh -c` with output streamed to the terminal.
    pub fn exec_command(&mut self, raw: &str) {
        let cmd = raw.trim();
        self.term.lines.push(TermLine::Prompt { cmd: cmd.to_string() });
        if cmd.is_empty() {
            return;
        }
        self.term.hist.push(cmd.to_string());
        self.term.hist_idx = None;
        let mut parts = cmd.split_whitespace();
        let verb = parts.next().unwrap_or("");
        match verb {
            "help" => {
                for l in [
                    "help                 show this list",
                    "cd <dir>             change directory (supports ..)",
                    "pwd                  print working directory",
                    "open <path>          open a folder as the project",
                    "clear                clear the terminal",
                    "exit                 close the terminal panel",
                    "anything else        runs in sh from the working directory",
                ] {
                    self.term.lines.push(TermLine::Out(l.into()));
                }
            }
            "clear" => self.term.lines.clear(),
            "exit" | "close" => self.bottom_open = false,
            "pwd" => self.term.lines.push(TermLine::Out(self.cwd.display().to_string())),
            "date" => self.term.lines.push(TermLine::Out(date_string())),
            "version" => self.term.lines.push(TermLine::Out("CodeCraft 1.1.0".into())),
            "whoami" => self
                .term
                .lines
                .push(TermLine::Out(std::env::var("USER").unwrap_or_else(|_| "user".into()))),
            "cd" => {
                let arg = parts.next().unwrap_or("~");
                let target = if arg == "~" {
                    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()))
                } else if arg == ".." {
                    self.cwd.parent().map(|p| p.to_path_buf()).unwrap_or(self.cwd.clone())
                } else {
                    let p = PathBuf::from(arg);
                    if p.is_absolute() {
                        p
                    } else {
                        self.cwd.join(p)
                    }
                };
                match std::fs::canonicalize(&target) {
                    Ok(p) if p.is_dir() => self.cwd = p,
                    _ => self
                        .term
                        .lines
                        .push(TermLine::Err(format!("cd: no such directory: {arg}"))),
                }
            }
            "open" => {
                let arg = parts.next().unwrap_or("");
                if arg.is_empty() {
                    self.term.lines.push(TermLine::Err("usage: open <path>".into()));
                } else {
                    self.open_project_dir(arg);
                    self.term
                        .lines
                        .push(TermLine::Ok(format!("project: {}", self.cwd.display())));
                }
            }
            "echo" => {
                let rest = cmd.split_once(' ').map(|(_, r)| r).unwrap_or("");
                self.term.lines.push(TermLine::Out(rest.to_string()));
            }
            "history" => {
                for (i, h) in self.term.hist.iter().enumerate() {
                    self.term.lines.push(TermLine::Out(format!("  {:3}  {}", i + 1, h)));
                }
            }
            _ => {
                // real execution through sh, streamed into the terminal
                let shell_cmd = format!("exec {}", cmd);
                let args = vec!["-c".to_string(), shell_cmd];
                let dir = self.cwd.clone();
                self.spawn_pipeline("sh", &args, &dir, false);
            }
        }
    }

    /// Drain finished background jobs into the terminal / build log.
    pub fn drain_jobs(&mut self, ctx: &egui::Context) {
        let mut active = false;
        if let Some(rx) = &self.term_rx {
            loop {
                match rx.try_recv() {
                    Ok((kind, line)) => match kind {
                        0 => self.term.lines.push(TermLine::Out(line)),
                        1 => self.term.lines.push(TermLine::Err(line)),
                        2 => {
                            self.term.lines.push(TermLine::Ok(format!("process exited with code {line}")));
                            self.running = false;
                            self.child_pid = None;
                        }
                        _ => {}
                    },
                    Err(mpsc::TryRecvError::Empty) => {
                        active = true;
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.term_rx = None;
                        break;
                    }
                }
            }
        }
        if let Some(rx) = &self.build_rx {
            loop {
                match rx.try_recv() {
                    Ok((kind, line)) => {
                        let level = match kind {
                            1 => LogLevel::Err,
                            _ => LogLevel::Plain,
                        };
                        self.build_log.push(LogLine::new(level, line));
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        active = true;
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.build_rx = None;
                        self.running = false;
                        self.child_pid = None;
                        let t = now_stamp();
                        self.build_log
                            .push(LogLine::new(LogLevel::Ok, format!("[{t}] Build process finished")));
                        break;
                    }
                }
            }
        }
        if active {
            ctx.request_repaint();
        }
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
    let (y, mo, d, _rem) = date_of(secs);
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
    // First positional argument: a project folder to open at startup.
    let open_path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-') && std::path::Path::new(a.as_str()).is_dir())
        .cloned();

    let icon = load_icon(include_bytes!("../assets/icon256.png"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("CodeCraft")
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    let state = App::new(screenshot_path, open_path);
    eframe::run_native(
        "CodeCraft",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(state))
        }),
    )
}

/// Choose the file to open on startup: README first, then the first
/// text-looking source file, else nothing.
fn pick_initial_tab(project: &Project) -> Option<Id> {
    if let Some(readme) = project.find_by_name("README.md") {
        return Some(readme);
    }
    project
        .walk_files()
        .into_iter()
        .find(|&id| !project.looks_binary(id))
}

impl App {
    fn new(screenshot_path: Option<String>, open_path: Option<String>) -> Self {
        // Either the user-provided path or the most recent project.
        let recents = Self::load_recents();
        let start = open_path.or_else(|| recents.first().cloned());
        let project = start
            .as_deref()
            .map(|p| {
                let expanded = if let Some(rest) = p.strip_prefix('~') {
                    PathBuf::from(format!(
                        "{}{rest}",
                        std::env::var("HOME").unwrap_or_default()
                    ))
                } else {
                    PathBuf::from(p)
                };
                Project::open_disk(&expanded).ok()
            })
            .flatten()
            .unwrap_or_else(Project::new_demo);
        let root_path = project.root_path.clone();
        let root = project.root;
        let project_name = project.nodes[root].name.clone();
        let cwd = root_path.clone().unwrap_or_else(|| PathBuf::from("."));
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

        let mut build_log = Vec::new();
        {
            let (files, dirs) = project.stats();
            let t = now_stamp();
            build_log.push(LogLine::new(LogLevel::Info, format!("[{t}] Project '{project_name}' ready")));
            build_log
                .push(LogLine::new(LogLevel::Info, format!("[{t}] {files} files, {dirs} folders")));
            match &root_path {
                Some(p) => build_log.push(LogLine::new(LogLevel::Info, format!("[{t}] Root: {}", p.display()))),
                None => build_log.push(LogLine::new(
                    LogLevel::Warn,
                    format!("[{t}] In-memory demo project — open a folder to work on real files"),
                )),
            }
        }

        let envs = vec!["Current File", "Debug Build", "Release Build"];

        let initial_tab = pick_initial_tab(&project).unwrap_or(root);

        Self {
            frames: 0,
            shot_sent: false,
            screenshot_path,
            open_tabs: if initial_tab != root { vec![initial_tab] } else { vec![] },
            active: (initial_tab != root).then_some(initial_tab),
            selected: Some(initial_tab),
            cur_line: HashMap::new(),
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
            dbg_log: Vec::new(),
            project,
            buffers: HashMap::new(),
            cwd,
            child_pid: None,
            term_rx: None,
            build_rx: None,
            open_dialog: None,
            palette: None,
            recent: recents,
            title_dirty: true,
            scroll_to_cursor: false,
            token_cache: HashMap::new(),
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

        // background job output → panels
        self.drain_jobs(ctx);
        self.step_scan(ctx);

        // window title follows the project
        if self.title_dirty {
            self.title_dirty = false;
            let name = self.project.nodes[self.project.root].name.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("CodeCraft — {name}")));
        }

        // global shortcuts
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl;
            if ctrl && i.key_pressed(egui::Key::N) {
                self.intent_new_file();
            }
            if ctrl && i.key_pressed(egui::Key::S) && !i.modifiers.shift {
                self.save_active();
            }
            if ctrl && i.key_pressed(egui::Key::S) && i.modifiers.shift {
                self.save_all();
            }
            if ctrl && i.key_pressed(egui::Key::P) && !i.modifiers.shift {
                self.palette = Some(Palette {
                    mode: PaletteMode::Files,
                    query: String::new(),
                    files: Vec::new(),
                    selected: 0,
                    scan: None,
                });
            }
            if ctrl && i.key_pressed(egui::Key::F) && i.modifiers.shift {
                self.palette = Some(Palette {
                    mode: PaletteMode::Search,
                    query: String::new(),
                    files: Vec::new(),
                    selected: 0,
                    scan: Some(ScanState {
                        queue: self.project.walk_files(),
                        pos: 0,
                        results: Vec::new(),
                    }),
                });
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
                self.palette = None;
                self.open_dialog = None;
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
                    .stroke(egui::Stroke::new(1.0_f32, theme::BORDER)),
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
                    .frame(egui::Frame::new().fill(theme::PANEL_BG).stroke(egui::Stroke::new(1.0_f32, theme::BORDER)));
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
        dialogs::modals(self, ctx);
    }
}

fn load_icon(bytes: &[u8]) -> egui::IconData {
    let img = image::load_from_memory(bytes).expect("icon").to_rgba8();
    egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
}
