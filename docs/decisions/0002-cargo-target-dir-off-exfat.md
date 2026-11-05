# ADR-0002: Keep Cargo target directory off the exFAT project drive

## Status
Accepted

## Date
2026-11-06

## Context
The repository lives on `/run/media/danielb/DNB` (exFAT). `cargo check` failed with
`Text file busy (os error 26)` while executing winit's build script: exFAT does not
provide the POSIX semantics cargo relies on when executing freshly written build
script binaries. Home (`/home`, ext4) has ~345 GB free.

## Decision
`.cargo/config.toml` in the repository sets `build.target-dir` to
`/home/danielb/.cache/codecraft/target`. All build artifacts (debug and release)
land on the native filesystem; sources stay on the data drive.

## Alternatives Considered
- **Move the repository to /home** — rejected: the user keeps projects on the data
  drive by choice; moving files out of the working directory breaks the craft-tool
  layout (`app/`, `shots/`, `craft.json`).
- **Retry builds on exFAT** — rejected: the error is intermittent but reproducible;
  build-script execution on exFAT is unsound, not just slow.

## Consequences
- Every clone of this repository on this machine needs the same `target-dir`
  override (the checked-in `.cargo/config.toml` provides it).
- `cargo clean` clears `/home/danielb/.cache/codecraft/target`, not `app/target`.
- The stale `app/target/` directory created before the fix can be deleted.
