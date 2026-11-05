# ADR-0005: Terminal executes commands via `sh -c` with streamed pipes

## Status
Accepted

## Date
2026-11-06

## Context
The original terminal replayed canned output. A real IDE terminal must run
arbitrary commands, stream output while the process runs, support Stop, and keep
the UI responsive.

## Decision
- Typed commands run through `sh -c "exec <cmd>"` with the working directory
  `App.cwd` (project root by default, changeable via `cd`).
- stdout and stderr are piped; reader threads forward lines over an
  `mpsc::channel` as `(stream, line)` messages; a waiter thread sends the exit
  code. The UI drains the channel every frame (`drain_jobs`) and requests
  repaint while a job is active — the UI thread never blocks on a process.
- Builtins handled inline: `help`, `clear`, `exit`, `pwd`, `cd` (canonicalized),
  `open <path>` (opens a project), `echo`, `date`, `version`, `whoami`,
  `history`.
- `Stop` sends SIGTERM via `kill <pid>` (`child_pid`).
- `cargo build` / `cargo run` reuse the same pipeline (`spawn_pipeline`) when the
  project root has a `Cargo.toml`; build output routes to the Build tab.

## Trust Model
The terminal runs arbitrary commands with the user's privileges. This is
identical to any terminal emulator (GNOME Terminal, Konsole) and is the feature,
not a vulnerability. No command is executed except from the terminal input box
or the Run/Build buttons.

## Alternatives Considered
- **PTY (pseudo-terminal) integration** — deferred: gives TTY semantics
  (colors, curses apps) but needs a crate (e.g. portable-pty) and a vte
  parser; the pipe model covers the first real milestone.
- **Async runtime (tokio)** — rejected: std threads + mpsc are sufficient and
  keep the dependency tree tiny.

## Consequences
- Programs requiring a TTY (progress bars, `vim`) degrade to line-based output.
- A build and a terminal command can run concurrently; each owns its own
  channel and `child_pid` is shared (Stop targets the most recent process).
