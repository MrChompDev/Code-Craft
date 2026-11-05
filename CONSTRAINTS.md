# Constraints

Last reviewed: 2026-11-06 by @danielb + agent (adopted with skill defaults; adjust freely)

## Floor (always enforced, no setup required)

- No new suppression attributes (`#[allow(...)]`, `#![allow(...)]`) without a reason in a comment or commit message
- No unimplemented stubs in shipped paths: `unimplemented!()`, `todo!()`, empty `catch`-equivalents (`match _ => {}` swallowing errors)
- No skipped or deleted tests (`#[ignore]`, removed `#[test]`) without a reason in the commit message
- No secrets in source (signing keys, tokens, credentials)
- This file does not get weakened to make a change pass

## Enforced with numbers

| Dimension | Rule | Checked by | Runs at |
|-----------|------|-----------|---------|
| Build | `cargo check` exits 0, zero errors | `cargo check` | every edit |
| Warnings | 0 compiler warnings (today: 0) | `cargo check` | every edit |
| Tests | `cargo test` green — 8 passing today, must not fall | `cargo test` | task end, CI |
| Clippy | ≤ 13 warnings (today: 13, ratchet down) | `cargo clippy` | task end, CI |
| Format | `cargo fmt --check` clean on changed files | `cargo fmt --check` | task end, CI |

## Measured, not yet enforced

| Metric | Today | Direction |
|--------|-------|-----------|
| Clippy warnings | 13 (3× map_or simplifiable, 3× empty if-branch, 2× manual prefix strip, …) | must not rise; drive toward 0 |
| Test count | 8 (project engine) | must not fall; grows with each engine feature |

## Exceptions

| ID | Rule | Path | Reason | Owner | Expires |
|----|------|------|--------|-------|---------|
| (none) | | | | | |

## Notes

- CI-only dimensions (need tooling not installed locally): dependency audit
  (`cargo-audit` / `osv-scanner`), secret scan (`gitleaks`), code scan
  (Semgrep with Rust rules) — wired in `.github/workflows/ci.yml`.
- Desktop GUI: no Lighthouse/axe page metrics apply (no browser); UI verification
  uses the built-in `--screenshot` harness against real projects.
- Coverage of changed lines is deferred until `cargo-llvm-cov` is installed;
  the test suite currently covers the project engine (`src/project.rs`).
