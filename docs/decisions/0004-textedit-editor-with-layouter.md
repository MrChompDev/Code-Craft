# ADR-0004: Editor as egui TextEdit with a custom layouter and token memo

## Status
Accepted

## Date
2026-11-06

## Context
The original editor custom-painted text through `Painter::text` per line — good
for pixel-perfect folds and colors, but read-only: no caret, no selection, no
keyboard editing. Making the editor real meant choosing between extending the
custom painter into a full text-editing engine (caret hit-testing, IME,
selection, undo — weeks of work) or moving to egui's built-in editing primitive.

## Decision
The code view is an `egui::TextEdit::multiline` with a custom layouter that
applies the existing per-line tokenizer (`tokens_for`) as a `LayoutJob`, plus a
painter-drawn gutter inside the same `ScrollArea` so line numbers scroll in sync.
- The gutter prepends nothing to the text; the job's text is byte-identical to
  the buffer so cursor mapping stays exact.
- Files above 200 KB render plain (no tokenizing).
- Tokens are memoized per buffer in `App.token_cache` keyed by a `DefaultHasher`
  of the text; a mid-frame edit colorizes fresh and the cache catches up next
  frame. Line folding is dropped for now (the old fold painter is in git
  history); dirty state shows as a dot in the tab.

## Alternatives Considered
- **Extend the custom painter into an editor engine** — rejected: correctness
  risk and time; egui's TextEdit already handles caret, selection, IME, and
  clipboard.
- **syntect for highlighting** — rejected for now: heavy dependency and syntax
  bundles; the existing hand tokenizer covers the project's languages (Rust, JS,
  Python, HTML, CSS, JSON, Markdown).

## Consequences
- Folding, multi-cursor, and column selection are not available in the editor
  until a later pass — recorded as roadmap items, not regressions.
- `scroll_to_me(Some(Align::Center))` is how search results scroll the caret
  into view.
- egui 0.32 layouter signature takes `&dyn TextBuffer` and returns
  `Arc<Galley>` via `ui.fonts(|f| f.layout_job(job))`.
