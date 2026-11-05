# AGENTS.md — CodeCraft (`app/`)

Read `CONSTRAINTS.md` before writing code. Do not weaken it to make a change pass.

## What this is

CodeCraft is a personal IntelliJ-inspired desktop IDE written in Rust with
egui/eframe 0.32. It operates on **real files**: the project tree mirrors a
directory on disk, the editor edits and saves real buffers, the terminal runs
real commands. An in-memory demo project (`Project::new_demo`) exists only as a
fallback when no folder is opened.

## Architecture (see `docs/decisions/` for the why)

| Module | Responsibility |
|--------|----------------|
| `main.rs` | `App` state, engine actions (open/save/project/terminal), keyboard map |
| `project.rs` | Tree model + real-filesystem layer (`open_disk`, write-through ops, `refresh`) |
| `editor_ui.rs` | Tabs + TextEdit code view with syntax layouter and gutter |
| `palette.rs` | Quick-open (Ctrl+P) and find-in-project (Ctrl+Shift+F) |
| `bottom_ui.rs` | Terminal / Build Output / Debug Console panels |
| `chrome.rs` | Menu bar, toolbar, activity rail, status bar |
| `tree_ui.rs` | Project explorer rows and context menus |
| `dialogs.rs` | Open Folder, About, Shortcuts modals |
| `theme.rs` | Dark palette + text styles (register `editor` style before using it) |

## Conventions

- Rust 2021, one binary crate; modules own their UI; `App` owns state and actions.
- egui 0.32: verify API details against the vendored source in
  `~/.cargo/registry/src/*/egui-0.32.3/` when unsure — minor versions break APIs.
- `Ui::horizontal_centered` allocates the **full remaining height** — never use it
  inside a full-height panel (this caused the invisible-tree bug).
- New engine behavior gets tests in the same change (`cargo test`).
- Build artifacts live at `/home/danielb/.cache/codecraft/target` (see
  `.cargo/config.toml`) — the project drive is exFAT and cannot execute build
  scripts; do not remove that config.

## Verification commands

```
cargo check      # floor: zero errors, zero warnings
cargo test       # engine tests must stay green
cargo clippy     # ≤ 13 warnings today, ratcheting down
cargo fmt --check
cargo build && ./.../target/debug/codecraft --screenshot shots/<name>.png <project-dir>
```

The `--screenshot` harness renders N frames (currently 116), writes the PNG, and
exits — use it to verify UI changes without a display interaction.
