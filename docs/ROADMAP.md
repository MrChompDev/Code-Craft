# CodeCraft Roadmap

Ordered by dependency and risk. Each item is one reviewable slice with its
verification named. Constraints in `CONSTRAINTS.md` apply to every item.

## Milestone 1 — trustworthy editing core (current)
- [x] Real project open/save/refresh, write-through tree mutations
- [x] Editable TextEdit editor with syntax layouter, gutter, dirty tabs
- [x] Quick-open (Ctrl+P), find-in-project (Ctrl+Shift+F)
- [x] Real terminal (sh -c, streamed, Stop), cargo build/run wiring
- [x] Engine test suite (8 tests), CI, ADRs, constraints
- [ ] Undo/redo for the editor (snapshot stack on idle debounce; Ctrl+Z/Ctrl+Y)
      — verify: unit tests on the stack, manual edit-session screenshot
- [ ] Tab close prompts when dirty; "Reopen closed tab" (Ctrl+Shift+T)
- [ ] Editor scroll persistence per tab; cursor restore on tab switch

## Milestone 2 — IDE surfaces
- [ ] Search results as a dedicated bottom tab (replace palette-only results)
- [ ] Status-bar diagnostics summary (errors/warnings from build log parse)
- [ ] Project templates: `cargo new`, `npm init` behind the New Project dialog
- [ ] Settings file (JSON via serde): theme variant, font size, keymap;
      migrate `recents.txt` into it (ADR-0006 follow-up)
- [ ] LSP experiment: rust-analyzer as a child process, diagnostics + hover in
      the editor (largest item — spike first, keep behind a feature flag)

## Milestone 3 — craft parity & polish
- [ ] Folding returns (tree of fold ranges over the TextEdit via layouter
      elision — needs an egui feasibility spike)
- [ ] Run configurations (the env dropdown becomes real: Debug/Release args)
- [ ] Debug console wired to a real debugger adapter (LLDB/DAP) — large;
      only after LSP pattern is proven
- [ ] Packaging: `cargo build --release` + AppImage/deb; version + changelog
      per the release checklist in `docs/decisions/`

## Standing constraints on new work
- Every slice: check → test → verify (screenshot for UI) → commit
- egui 0.32 API verified against vendored source when in doubt
- No feature lands with `todo!()` stubs or weakened constraints
