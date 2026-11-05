# ADR-0001: Rust + egui (eframe) as the CodeCraft stack

## Status
Accepted

## Date
2026-11-03 (recorded 2026-11-06)

## Context
CodeCraft's goal is a personal, IntelliJ-inspired desktop IDE. Requirements:
single native binary, fast startup, immediate-mode UI iteration, and a stack the
developer can own end to end. The craft session (`../craft.json`) recorded the
stack choice as "Rust + egui (eframe)".

## Decision
Rust with eframe/egui (pinned to 0.32 — the version the original UI was written
against), with the `image` crate only for the window icon. No serde yet: settings
persistence uses a plain-text recents file to keep the dependency tree minimal.

## Alternatives Considered
- **Electron / web UI** — rejected: memory footprint and startup time; the README
  drafted on GitHub mentions Electron, but the working implementation is Rust and
  the README should be updated to match (noted, not silently rewritten).
- **GTK / Qt bindings** — rejected: heavier FFI story and less fun iteration loop
  for a from-scratch IDE shell.

## Consequences
- egui's immediate-mode model shapes the whole codebase (per-frame drawing, no
  retained widget tree). The editor is a `TextEdit` with a custom layouter.
- eframe 0.32 API quirks are documented in code comments (e.g.
  `Ui::horizontal_centered` allocates the full remaining height — the tree panel
  bug); verify against the vendored registry source when unsure.
- Upgrades past 0.32 must be treated as feature work with the test suite green
  before and after (egui breaks API between minor versions).
