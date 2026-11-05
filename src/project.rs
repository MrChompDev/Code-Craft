// Project tree model + demo content.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub type Id = usize;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub is_dir: bool,
    pub parent: Option<Id>,
    pub children: Vec<Id>,
    pub content: Option<Vec<String>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ft {
    Js,
    Py,
    Html,
    Css,
    Json,
    Md,
    Txt,
    Rs,
    Other,
}

pub fn ft_of(name: &str) -> Ft {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "js" | "mjs" | "cjs" | "ts" => Ft::Js,
        "py" => Ft::Py,
        "html" | "htm" => Ft::Html,
        "css" => Ft::Css,
        "json" => Ft::Json,
        "md" => Ft::Md,
        "txt" | "gitignore" | "log" => Ft::Txt,
        "rs" => Ft::Rs,
        _ => Ft::Other,
    }
}

pub struct Project {
    pub nodes: Vec<Node>,
    pub root: Id,
    /// When set, the tree mirrors a real directory on disk and every
    /// mutation writes through to the filesystem.
    pub root_path: Option<PathBuf>,
}

impl Project {
    pub fn add(&mut self, parent: Id, name: &str, is_dir: bool, content: Option<&str>) -> Id {
        let id = self.nodes.len();
        self.nodes.push(Node {
            name: name.to_string(),
            is_dir,
            parent: Some(parent),
            children: Vec::new(),
            content: content.map(|c| c.lines().map(|l| l.to_string()).collect()),
        });
        self.nodes[parent].children.push(id);
        id
    }

    pub fn find_child(&self, dir: Id, name: &str) -> Option<Id> {
        self.nodes[dir]
            .children
            .iter()
            .copied()
            .find(|&c| self.nodes[c].name == name)
    }

    pub fn path_of(&self, id: Id) -> String {
        let mut parts = Vec::new();
        let mut cur = Some(id);
        while let Some(i) = cur {
            parts.push(self.nodes[i].name.clone());
            cur = self.nodes[i].parent;
        }
        parts.reverse();
        parts.join("/")
    }

    pub fn find_by_name(&self, name: &str) -> Option<Id> {
        self.nodes.iter().position(|n| !n.is_dir && n.name == name)
    }

    pub fn unique_name(&self, dir: Id, base: &str, is_dir: bool) -> String {
        let mut name = base.to_string();
        let mut n = 2;
        while self.find_child(dir, &name).is_some() {
            if is_dir {
                name = format!("{} {}", base, n);
            } else {
                let (stem, ext) = match base.rfind('.') {
                    Some(i) => (&base[..i], &base[i..]),
                    None => (base, ""),
                };
                name = format!("{} {}{}", stem, n, ext);
            }
            n += 1;
        }
        name
    }

    pub fn walk_files(&self) -> Vec<Id> {
        let mut out = Vec::new();
        let mut stack = vec![self.root];
        while let Some(d) = stack.pop() {
            for &c in &self.nodes[d].children {
                if self.nodes[c].is_dir {
                    stack.push(c);
                } else {
                    out.push(c);
                }
            }
        }
        out.sort();
        out
    }

    pub fn tree_lines(&self, dir: Id, prefix: &str, out: &mut Vec<(String, bool)>) {
        let mut kids: Vec<Id> = self.nodes[dir].children.clone();
        kids.sort_by(|&a, &b| {
            let (da, db) = (self.nodes[a].is_dir, self.nodes[b].is_dir);
            db.cmp(&da)
                .then_with(|| self.nodes[a].name.cmp(&self.nodes[b].name))
        });
        let last = kids.len();
        for (i, &k) in kids.iter().enumerate() {
            let n = &self.nodes[k];
            let branch = if i + 1 == last { "└─ " } else { "├─ " };
            out.push((format!("{}{}{}", prefix, branch, n.name), n.is_dir));
            if n.is_dir {
                let np = if i + 1 == last {
                    format!("{}   ", prefix)
                } else {
                    format!("{}│  ", prefix)
                };
                self.tree_lines(k, &np, out);
            }
        }
    }
}

const MAIN_JS: &str = r#"// CodeCraft demo — entry point
import { createStore } from './store.js';
import { render } from './render.js';

const TICK_MS = 16;
const MAX_FRAMES = 600;

/** Small task runner used by the build script. */
class TaskRunner {
  constructor(name) {
    this.name = name;
    this.queue = [];
    this.results = new Map();
  }

  add(task) {
    if (typeof task !== 'function') {
      throw new TypeError('task must be a function');
    }
    this.queue.push(task);
    return this;
  }

  async runAll() {
    const started = performance.now();
    for (const task of this.queue) {
      const result = await task();
      this.results.set(task.name || 'anon', result);
    }
    const ms = Math.round(performance.now() - started);
    console.log(`[runner] ${this.queue.length} tasks in ${ms}ms`);
    return this.results;
  }
}

function makeTimer(interval) {
  let ticks = 0;
  const id = setInterval(() => {
    ticks += 1;
    if (ticks >= 5) {
      clearInterval(id);
    }
  }, interval);
  return () => ticks;
}

export function bootstrap(root) {
  const store = createStore({ fps: 0, frames: 0 });
  const runner = new TaskRunner('main');
  runner.add(() => render(root, store));
  runner.add(makeTimer(TICK_MS));
  return runner.runAll().then((results) => {
    console.log('bootstrap done after', MAX_FRAMES, 'frames');
    return results;
  });
}

if (typeof window !== 'undefined') {
  window.addEventListener('DOMContentLoaded', () => {
    bootstrap('#app').catch((err) => console.error(err));
  });
}
"#;

const APP_PY: &str = r##"""CodeCraft demo — tiny task scheduler."""

from dataclasses import dataclass, field
from typing import Callable, List, Optional


@dataclass
class Task:
    name: str
    action: Callable[[], int]
    retries: int = 3
    history: List[int] = field(default_factory=list)


class Scheduler:
    """Runs tasks with a simple retry policy."""

    def __init__(self, max_workers: int = 4):
        self.max_workers = max_workers
        self.tasks: List[Task] = []

    def submit(self, task: Task) -> None:
        if task.retries < 0:
            raise ValueError("retries must be >= 0")
        self.tasks.append(task)

    def run(self) -> dict:
        report: dict = {}
        for task in self.tasks:
            value: Optional[int] = None
            for attempt in range(task.retries):
                try:
                    value = task.action()
                except RuntimeError as err:
                    print(f"retry {attempt}: {err}")
                    continue
                task.history.append(value)
                break
            report[task.name] = value
        return report


def fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)


if __name__ == "__main__":
    scheduler = Scheduler(max_workers=2)
    scheduler.submit(Task("fib-10", lambda: fib(10)))
    scheduler.submit(Task("fib-20", lambda: fib(20)))
    print(scheduler.run())
"##;

const UTILS_JS: &str = r#"// Shared helpers for the demo project.

export function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

export function lerp(a, b, t) {
  return a + (b - a) * t;
}

export function formatBytes(bytes) {
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  let v = bytes;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  return `${v.toFixed(1)} ${units[i]}`;
}

// TODO: replace with a real queue once tasks grow.
export const QUEUE_LIMIT = 64;
"#;

const SERVER_PY: &str = r##"""Minimal dev server for the demo."""

import json
from http.server import BaseHTTPRequestHandler, HTTPServer

PORT = 8787


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        body = json.dumps({"ok": True, "path": self.path}).encode()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        print("[dev-server]", fmt % args)


if __name__ == "__main__":
    print(f"serving on http://localhost:{PORT}")
    HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
"##;

const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>CodeCraft Demo</title>
    <link rel="stylesheet" href="styles.css" />
  </head>
  <body>
    <header class="topbar">
      <h1 class="brand">CodeCraft</h1>
      <nav class="nav">
        <a href="#features">Features</a>
        <a href="#docs">Docs</a>
      </nav>
    </header>
    <main id="app">
      <section class="hero">
        <p>Edit code here. Preview it there.</p>
      </section>
    </main>
    <script type="module" src="app.js"></script>
  </body>
</html>
"##;

const STYLES_CSS: &str = r#"/* CodeCraft demo styles */
:root {
  --bg: #1e1f22;
  --panel: #2b2d30;
  --accent: #3574f0;
  --text: #dfe1e5;
}

body {
  margin: 0;
  font-family: 'JetBrains Mono', monospace;
  background: var(--bg);
  color: var(--text);
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: var(--panel);
}

.brand {
  font-size: 18px;
  letter-spacing: 0.5px;
}

.hero {
  padding: 64px 20px;
  text-align: center;
}

@media (max-width: 600px) {
  .nav {
    display: none;
  }
}
"#;

const WEB_APP_JS: &str = r#"// Web entry — mounts the demo UI.
import { clamp, formatBytes } from '../src/utils.js';

const root = document.querySelector('#app');

function mount() {
  const state = { frames: 0, fps: 0 };
  const el = document.createElement('div');
  el.className = 'stat';
  root.appendChild(el);

  requestAnimationFrame(function loop(ts) {
    state.frames += 1;
    el.textContent = `frames: ${state.frames} • heap: ${formatBytes(1024)}`;
    if (state.frames < 600) {
      requestAnimationFrame(loop);
    }
  });
}

mount();
"#;

const PACKAGE_JSON: &str = r#"{
  "name": "codecraft-demo",
  "version": "1.0.0",
  "description": "Demo project for the CodeCraft editor",
  "main": "src/main.js",
  "scripts": {
    "start": "node src/server.py",
    "build": "codecraft build --release",
    "test": "codecraft test"
  },
  "keywords": [
    "editor",
    "demo"
  ],
  "license": "MIT",
  "dependencies": {
    "tiny-store": "^2.1.0"
  },
  "devDependencies": {
    "miniserve": "^0.4.2"
  }
}
"#;

const CONFIG_JSON: &str = r#"{
  "editor": {
    "fontFamily": "monospace",
    "fontSize": 13,
    "tabSize": 4,
    "lineNumbers": true,
    "foldBlocks": true,
    "highlightActiveLine": true
  },
  "build": {
    "target": "debug",
    "warnings": true,
    "threads": 8
  }
}
"#;

const TEST_PY: &str = r##"""Tests for the demo scheduler."""

from app import Scheduler, Task


def test_runs_simple_task():
    s = Scheduler()
    s.submit(Task("one", lambda: 41 + 1))
    assert s.run() == {"one": 42}


def test_reports_failures():
    def boom():
        raise RuntimeError("nope")

    s = Scheduler()
    s.submit(Task("bad", boom, retries=2))
    assert s.run() == {"bad": None}
"##;

const README_MD: &str = r#"# CodeCraft Demo

A tiny demo project used to try out the **CodeCraft** editor.

## What's inside

- `src/` — JavaScript and Python sources
- `web/` — HTML, CSS and the web entry
- `data/` — JSON configuration
- `tests/` — a couple of unit tests

## Build

```bash
codecraft build --release
codecraft run
```

> Tip: open a file in the project tree to see syntax highlighting,
> line numbers and collapsible code blocks in action.
"#;

const GITIGNORE: &str = r#"node_modules/
dist/
*.log
.env
"#;

impl Project {
    pub fn new_demo() -> Self {
        let mut p = Project {
            nodes: vec![Node {
                name: "demo-project".into(),
                is_dir: true,
                parent: None,
                children: Vec::new(),
                content: None,
            }],
            root: 0,
            root_path: None,
        };
        let src = p.add(0, "src", true, None);
        p.add(src, "main.js", false, Some(MAIN_JS));
        p.add(src, "app.py", false, Some(APP_PY));
        p.add(src, "utils.js", false, Some(UTILS_JS));
        p.add(src, "server.py", false, Some(SERVER_PY));
        let web = p.add(0, "web", true, None);
        p.add(web, "index.html", false, Some(INDEX_HTML));
        p.add(web, "styles.css", false, Some(STYLES_CSS));
        p.add(web, "app.js", false, Some(WEB_APP_JS));
        let data = p.add(0, "data", true, None);
        p.add(data, "package.json", false, Some(PACKAGE_JSON));
        p.add(data, "config.json", false, Some(CONFIG_JSON));
        let tests = p.add(0, "tests", true, None);
        p.add(tests, "test_app.py", false, Some(TEST_PY));
        p.add(0, "README.md", false, Some(README_MD));
        p.add(0, ".gitignore", false, Some(GITIGNORE));
        p
    }

    /// Count of non-dir nodes (used by the build log).
    pub fn stats(&self) -> (usize, usize) {
        let mut files = 0;
        let mut dirs = 0;
        for n in &self.nodes {
            if n.is_dir {
                dirs += 1;
            } else {
                files += 1;
            }
        }
        (files, dirs)
    }

    /// Sorted listing for `ls`.
    pub fn ls(&self, dir: Id) -> Vec<String> {
        let mut items: BTreeMap<String, bool> = BTreeMap::new();
        for &c in &self.nodes[dir].children {
            let n = &self.nodes[c];
            let mut name = n.name.clone();
            if n.is_dir {
                name.push('/');
            }
            items.insert(name, n.is_dir);
        }
        items.into_keys().collect()
    }
}

// ---------------- real filesystem layer ----------------
// When `root_path` is set the tree mirrors a directory on disk: opening a
// file reads its bytes, mutations write through, and refresh resyncs.

impl Project {
    /// Directories never scanned into the tree.
    fn skip_dir(name: &str) -> bool {
        matches!(
            name,
            ".git"
                | "target"
                | "node_modules"
                | "__pycache__"
                | ".idea"
                | ".vscode"
                | "dist"
                | "build"
        ) || (name.starts_with('.') && name != ".github")
    }

    /// Open a real directory: scan it into the tree. Content is loaded lazily
    /// per file by the editor, so large projects cost nothing up front.
    pub fn open_disk(root: &Path) -> std::io::Result<Self> {
        // canonicalize so "." / "~" / trailing dots resolve to a real name
        let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let name = canonical
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| canonical.to_string_lossy().to_string());
        let mut p = Project {
            nodes: vec![Node {
                name,
                is_dir: true,
                parent: None,
                children: Vec::new(),
                content: None,
            }],
            root: 0,
            root_path: Some(canonical),
        };
        p.scan_dir(0, &root)?;
        Ok(p)
    }

    fn scan_dir(&mut self, parent: Id, dir: &Path) -> std::io::Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    !Self::skip_dir(&name)
                } else {
                    true
                }
            })
            .collect();
        entries.sort_by_key(|e| (e.path().is_file(), e.file_name()));

        for e in entries {
            let name = e.file_name().to_string_lossy().to_string();
            let path = e.path();
            let is_dir = path.is_dir();
            let id = self.add(parent, &name, is_dir, None);
            if is_dir {
                self.scan_dir(id, &path)?;
            }
        }
        Ok(())
    }

    /// Absolute path of a node on disk (only meaningful when root_path is set).
    pub fn disk_path(&self, id: Id) -> Option<PathBuf> {
        self.root_path.as_ref().map(|root| {
            let rel = self.path_of(id);
            // path_of starts with the root folder's own name; strip it.
            let rel = rel.split_once('/').map(|(_, r)| r).unwrap_or("");
            if rel.is_empty() {
                root.clone()
            } else {
                root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
            }
        })
    }

    /// Rescan from disk, keeping `keep` node ids alive by relative path.
    /// Returns a map old_id -> new_id for surviving nodes.
    pub fn refresh(&mut self, keep: &[Id]) -> std::collections::HashMap<Id, Id> {
        let mut mapping = std::collections::HashMap::new();
        let Some(root) = self.root_path.clone() else {
            return mapping;
        };
        let keep_paths: Vec<(Id, String)> = keep
            .iter()
            .filter_map(|&id| {
                self.disk_path(id)
                    .map(|p| (id, p.to_string_lossy().to_string()))
            })
            .collect();
        if let Ok(fresh) = Self::open_disk(&root) {
            for (old, path) in keep_paths {
                for (i, n) in fresh.nodes.iter().enumerate() {
                    if !n.is_dir {
                        if let Some(dp) = fresh.disk_path(i) {
                            if dp.to_string_lossy() == path {
                                mapping.insert(old, i);
                                break;
                            }
                        }
                    }
                }
            }
            *self = fresh;
        }
        mapping
    }

    /// Extensions we never try to render as text.
    fn binary_ext(name: &str) -> bool {
        let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        matches!(
            ext.as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "ico"
                | "webp"
                | "bmp"
                | "exe"
                | "dll"
                | "so"
                | "dylib"
                | "zip"
                | "tar"
                | "gz"
                | "xz"
                | "7z"
                | "rar"
                | "pdf"
                | "bin"
                | "o"
                | "a"
                | "rlib"
                | "woff"
                | "woff2"
                | "ttf"
                | "otf"
                | "pak"
                | "mp3"
                | "mp4"
                | "wav"
                | "ogg"
                | "class"
                | "jar"
                | "pyc"
                | "wasm"
        )
    }

    /// Heuristic: extension blocklist plus a NUL-byte sniff of the head.
    pub fn looks_binary(&self, id: Id) -> bool {
        if Self::binary_ext(&self.nodes[id].name) {
            return true;
        }
        match self.disk_path(id) {
            Some(p) => {
                use std::io::Read;
                let mut head = [0u8; 8192];
                match std::fs::File::open(p).and_then(|mut f| f.read(&mut head)) {
                    Ok(n) => head[..n].contains(&0),
                    Err(_) => false,
                }
            }
            None => false,
        }
    }

    pub fn read_file(&self, id: Id) -> std::io::Result<String> {
        match self.disk_path(id) {
            Some(p) => fs::read_to_string(p),
            // demo project: content is stored inline
            None => Ok(self.nodes[id]
                .content
                .as_ref()
                .map(|l| l.join("\n"))
                .unwrap_or_default()),
        }
    }

    pub fn write_file(&mut self, id: Id, body: &str) -> std::io::Result<()> {
        match self.disk_path(id) {
            Some(p) => fs::write(p, body),
            None => {
                // demo project: update in-memory content only
                self.nodes[id].content = Some(body.lines().map(|l| l.to_string()).collect());
                Ok(())
            }
        }
    }

    pub fn create_entry(&self, parent: Id, name: &str, is_dir: bool) -> std::io::Result<()> {
        match self.disk_path(parent) {
            Some(p) => {
                let target = p.join(name);
                if is_dir {
                    fs::create_dir(target)
                } else {
                    fs::write(target, "")
                }
            }
            None => Ok(()), // demo: tree add alone is enough
        }
    }

    pub fn rename_entry(&self, id: Id, new_name: &str) -> std::io::Result<()> {
        match self.disk_path(id) {
            Some(p) => {
                let target = p.with_file_name(new_name);
                fs::rename(p, target)
            }
            None => Ok(()),
        }
    }

    pub fn delete_entry(&self, id: Id) -> std::io::Result<()> {
        match self.disk_path(id) {
            Some(p) => {
                if p.is_dir() {
                    fs::remove_dir_all(p)
                } else {
                    fs::remove_file(p)
                }
            }
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Build a small on-disk fixture:
    /// root/
    ///   src/            (main.rs, lib.rs)
    ///   .git/           (must be skipped)
    ///   target/         (must be skipped)
    ///   README.md
    fn fixture() -> (tempfile::TempDir, Project) {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub struct S;\n").unwrap();
        fs::create_dir_all(dir.path().join(".git/objects")).unwrap();
        fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/master\n").unwrap();
        fs::create_dir_all(dir.path().join("target/debug")).unwrap();
        fs::write(dir.path().join("target/debug/marker"), "x").unwrap();
        fs::write(dir.path().join("README.md"), "# fixture\n").unwrap();
        let project = Project::open_disk(dir.path()).expect("open_disk");
        (dir, project)
    }

    fn child<'a>(p: &'a Project, dir: Id, name: &str) -> Id {
        p.find_child(dir, name)
            .unwrap_or_else(|| panic!("missing child {name}"))
    }

    #[test]
    fn open_disk_scans_tree_and_skips_vcs_and_build_dirs() {
        let (_dir, p) = fixture();
        let root = p.root;
        assert_eq!(
            p.nodes[root].children.len(),
            2,
            "only src/ and README.md visible"
        );
        assert!(p.find_child(root, ".git").is_none(), ".git must be skipped");
        assert!(
            p.find_child(root, "target").is_none(),
            "target must be skipped"
        );
        let src = child(&p, root, "src");
        assert!(p.nodes[src].is_dir);
        assert_eq!(p.nodes[src].children.len(), 2, "main.rs + lib.rs");
    }

    #[test]
    fn open_disk_lists_directories_before_files() {
        let (_dir, p) = fixture();
        let kids = &p.nodes[p.root].children;
        assert!(
            p.nodes[kids[0]].is_dir,
            "first entry should be the src/ dir"
        );
        assert!(!p.nodes[kids[1]].is_dir, "second entry should be README.md");
    }

    #[test]
    fn disk_path_maps_nodes_to_absolute_paths() {
        let (dir, p) = fixture();
        let readme = child(&p, p.root, "README.md");
        let expected = dir.path().join("README.md");
        assert_eq!(p.disk_path(readme).unwrap(), expected);
        // root itself maps to the project root
        assert_eq!(p.disk_path(p.root).unwrap(), dir.path());
    }

    #[test]
    fn read_and_write_files_roundtrip_through_disk() {
        let (_dir, mut p) = fixture();
        let readme = child(&p, p.root, "README.md");
        assert_eq!(p.read_file(readme).unwrap(), "# fixture\n");
        p.write_file(readme, "# changed\n").unwrap();
        assert_eq!(p.read_file(readme).unwrap(), "# changed\n");
        assert_eq!(
            fs::read_to_string(_dir.path().join("README.md")).unwrap(),
            "# changed\n"
        );
    }

    #[test]
    fn create_rename_delete_write_through_to_disk() {
        let (dir, mut p) = fixture();
        // create
        p.create_entry(p.root, "notes.txt", false).unwrap();
        assert!(dir.path().join("notes.txt").is_file());
        let id = p.add(p.root, "notes.txt", false, None);
        // rename on disk, then keep the tree in sync (as commit_pending does)
        p.rename_entry(id, "notes.md").unwrap();
        p.nodes[id].name = "notes.md".into();
        assert!(!dir.path().join("notes.txt").exists());
        assert!(dir.path().join("notes.md").is_file());
        // delete
        p.delete_entry(id).unwrap();
        assert!(!dir.path().join("notes.md").exists());
    }

    #[test]
    fn refresh_survives_external_changes_and_remaps_tabs() {
        let (dir, mut p) = fixture();
        let readme = child(&p, p.root, "README.md");
        // externally delete src/lib.rs, then refresh with README open
        fs::remove_file(dir.path().join("src/lib.rs")).unwrap();
        let mapping = p.refresh(&[readme]);
        assert!(!mapping.is_empty(), "README.md should survive the refresh");
        assert!(p.find_child(p.root, "README.md").is_some());
        let src = child(&p, p.root, "src");
        assert!(
            p.find_child(src, "lib.rs").is_none(),
            "externally deleted file must disappear"
        );
    }

    #[test]
    fn demo_project_stays_in_memory_without_root_path() {
        let mut p = Project::new_demo();
        assert!(p.root_path.is_none());
        let main_js = p
            .find_child(p.find_child(p.root, "src").unwrap(), "main.js")
            .unwrap();
        // read returns inline content; write updates the in-memory node
        assert!(p.read_file(main_js).unwrap().contains("CodeCraft"));
        p.write_file(main_js, "hello\n").unwrap();
        assert_eq!(p.read_file(main_js).unwrap(), "hello");
    }

    #[test]
    fn file_type_detection_covers_common_languages() {
        assert_eq!(ft_of("main.rs"), Ft::Rs);
        assert_eq!(ft_of("app.PY"), Ft::Py);
        assert_eq!(ft_of("script.test.ts"), Ft::Js);
        assert_eq!(ft_of("data.json"), Ft::Json);
        assert_eq!(ft_of("notes.md"), Ft::Md);
        assert_eq!(ft_of("Makefile"), Ft::Other);
    }
}
