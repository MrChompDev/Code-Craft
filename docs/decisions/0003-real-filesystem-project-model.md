# ADR-0003: Real-filesystem project model with in-memory demo fallback

## Status
Accepted

## Date
2026-11-06

## Context
The original CodeCraft was a facade: `Project::new_demo()` built an in-memory
tree, the terminal replayed canned output, and the editor painted read-only
buffers. Building out the IDE meant making the tree, editor, and terminal operate
on real files while keeping the existing chrome and the screenshot harness
working.

## Decision
- `Project` gains `root_path: Option<PathBuf>`. When set, the tree mirrors a real
  directory (`open_disk` scan, skip `.git`/`target`/`node_modules`/hidden), every
  mutation writes through (`create_entry`, `rename_entry`, `delete_entry`,
  `write_file`), and buffers load lazily from disk.
- When `root_path` is `None` (demo mode), content stays in memory so the app
  always starts, even with no project path — the startup path tries argv, then
  the most recent project, then the demo.
- `refresh()` rescans and remaps open tabs/buffers by absolute path so external
  changes converge instead of corrupting state.

## Alternatives Considered
- **VFS crate / virtual filesystem abstraction** — rejected for now: one
  `Option<PathBuf>` covers disk-vs-memory with two code paths that are each
  trivially testable; an abstraction layer would be premature.
- **Always-on-disk only (no demo)** — rejected: the demo keeps the screenshot
  harness and first-run experience working with zero setup.

## Consequences
- `write_file` needs `&mut self` (demo mode mutates node content); disk mode does
  not mutate the tree.
- The caller must keep the tree name in sync after `rename_entry` (done in
  `commit_pending`); the disk op alone does not touch node names.
- Covered by `src/project.rs` `#[cfg(test)] mod tests` (8 tests, tempdir fixtures).
