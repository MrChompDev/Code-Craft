// Project tree model + demo content.
use std::collections::BTreeMap;

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
        self.nodes
            .iter()
            .position(|n| !n.is_dir && n.name == name)
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
            db.cmp(&da).then_with(|| self.nodes[a].name.cmp(&self.nodes[b].name))
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
