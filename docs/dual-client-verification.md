# Dual-client verification

Verified on Windows on 2026-09-12 with Codex CLI 0.154.0, Claude Code 2.1.268,
and Rust 1.96.1. The same optimized STK executable passed both runtime tests.

## Checks

- `cargo test --offline`: 41 unit tests and 4 executable-level tests passed.
- `cargo clippy --offline --all-targets -- -D warnings`: passed.
- `cargo build --release --offline`: passed.
- `git diff --check`: passed.
- `scripts/test-codex-runtime.py --stk target/release/stk.exe`: passed against
  the actual Codex CLI with installer-generated hooks.
- `scripts/test-claude-runtime.py --stk target/release/stk.exe`: passed against
  the actual Claude Code CLI with installer-generated hooks.

The runtime fixtures use local scripted model endpoints. They verify hook
registration, execution, and feedback handling without paid model calls. They
do not measure how an autonomous model chooses follow-up reads. Codex may
contact its plugin registry during startup.

Both clients attempted a whole-file read of a 3,000-line Rust fixture. STK
blocked that read and returned an outline. The next tool call recovered lines
2100 and 2101, without line 2102. The executable and fixture paths included
spaces. The Windows Codex check caught a missing `commandWindows` entry during
development; the installer now emits it explicitly.

Executable tests also verify byte-preserving range recovery, Claude dedup,
Codex repeated outlines, scoped reads, exclusions, binary pass-through,
malformed settings, backups, preservation of other hooks, idempotent setup,
and uninstall.

## Boundaries

Codex interception covers the simple shell reads listed in the README. It
passes through unfamiliar commands and keeps dedup disabled because shared
session IDs and compaction do not reliably identify retained context.

Live user settings and the globally installed STK executable were not changed.
Codex requires review and trust of its newly installed hook. macOS and Linux
runtime behavior has not been exercised in this verification.
