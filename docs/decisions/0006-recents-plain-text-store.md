# ADR-0006: Recent projects persisted as a plain text file

## Status
Accepted

## Date
2026-11-06

## Context
The Open Folder dialog and the File ▸ Open Recent menu need a list of recently
opened projects that survives restarts. The only state so far is in-memory.

## Decision
Recents live in `$XDG_CONFIG_HOME/codecraft/recents.txt` (falling back to
`~/.config/codecraft/recents.txt`), one absolute path per line, newest first,
capped at 8 entries. Loaded at startup (`App::load_recents`), written on every
`open_project_dir` (`remember_recent`). The startup project is argv, else the
newest recents entry, else the demo.

## Alternatives Considered
- **JSON + serde** — deferred: one string list does not justify the dependency
  yet; when real settings arrive (theme, font size, keymap) switch the file to
  JSON and migrate or drop `recents.txt` (it is disposable state).
- **egui persistence (eframe storage)** — rejected: ties user state to the
  egui storage backend and is lost when the app data dir is cleared; a config
  file is inspectable and portable.

## Consequences
- No settings file format is committed to yet; introducing serde later is a
  `feat:` change with a migration note in the changelog.
- Clearing the list from the menu deletes the file.
