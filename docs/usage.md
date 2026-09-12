# Using STK

STK supports Claude Code and Codex with one binary. For the quick start, see the [README](../README.md).

## Setup

STK is a single binary with no runtime dependencies.

```bash
cargo install --path .
```

Configure both detected clients:

```bash
stk init --auto
```

Detection checks for `.claude` and `.codex` settings directories. You can select either client explicitly:

```bash
stk init --claude
stk init --codex
stk init --auto --dry-run
stk init --auto --uninstall
```

Setup merges STK handlers into Claude's `settings.json` and Codex's `hooks.json`, preserves unrelated settings, and backs up existing files before changing them. Repeating setup is safe. Uninstall removes only STK's handlers. `CLAUDE_CONFIG_DIR` and `CODEX_HOME` are honored; `--home <directory>` uses an alternate home for testing or portable setup.

Restart the clients after setup. In Codex, review and trust the generated hook through `/hooks`. Installing a hook does not grant trust. Codex must support `PreToolUse` for shell commands; see the [Codex hook documentation](https://learn.chatgpt.com/docs/hooks).

Install the binary in its permanent location before setup. The generated hook uses that executable's absolute path. If you move it, run setup again. Put `stk` on PATH so agents can run range reads.

Running `stk init` without flags retains the legacy Claude snippet-only behavior.

### Codex coverage

The Codex adapter recognizes single-file `cat`, `rtk read`, and `Get-Content` calls, including literal quoted paths and PowerShell `-Path`, `-LiteralPath`, and `-Raw`. Hooks report these shell calls as `Bash`, including on Windows.

Pipelines, compound commands, shell wrappers, interpolation, globs, multiple paths, and unfamiliar options pass through. Scoped reads such as `Get-Content -TotalCount` also pass through. This release does not intercept Python reads, arbitrary scripts, or MCP reads.

Codex receives an outline and instructions to recover content:

```bash
stk read src/pipeline.ts --offset 143 --limit 40
stk read src/pipeline.ts --offset 1
```

Offsets are 1-based. Either range flag bypasses outlining; `--offset 1` returns the entire file. Small and excluded files are returned unchanged. Binary or oversized files that cannot be outlined also pass through. Missing files and zero offsets or limits produce an error. An offset beyond the end of the file returns empty output.

Codex dedup is disabled: subagents can share a session ID, and compaction can discard an earlier outline. Returning the map again preserves access. Claude retains its existing session dedup behavior; a scoped read always bypasses it.

## Usage

| Command | What it does |
|---|---|
| `stk hook claude` | Claude native Read hook. |
| `stk hook codex` | Codex shell-read hook. |
| `stk read <path>` | Read a small file or outline a large file. |
| `stk read <path> --offset 100 --limit 40` | Return an exact line range. |
| `stk outline <path>` | Print the outline for a file by hand. |
| `stk gain` | Savings so far: clamps, dedup hits, bytes avoided, estimated tokens. |
| `stk gain --json` | Same, machine-readable (totals + per-day series) for dashboards. |
| `stk config` | Show active config and store location. |

### Configuration

All optional, via `stk`'s config file (path shown by `stk config`):

```toml
clamp_threshold   = 16384      # bytes; files at or below this always pass through
outline_max_lines = 80         # cap on outline length
dedup             = true       # Claude session dedup; disabled for Codex
exclude           = ["*.lock"] # globs that always pass through untouched
```

`STK_CONFIG_FILE` overrides the configuration file; `STK_DATA_DIR` overrides the session and stats directory. Both support isolated tests.

## Honest limitations

STK reports **bytes avoided**: the file bytes it kept out of context minus the small outline it sent. That number is real, but read it as an **upper bound**. Here is what it does not capture:

- Net session savings remain unmeasured. Follow-up reads cost tokens and can reduce or erase the bytes avoided.
- Claude coverage is the native `Read` tool; Codex coverage is the conservative command subset above. Other command output remains [RTK](https://github.com/rtk-ai/rtk)'s job.
- Stats cover hook decisions. Direct `stk read` calls and follow-up range reads are not counted.

## Build and test

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
node scripts/readme/build.mjs --check
```

The optional runtime fixtures drive installed clients with local scripted model endpoints:

```bash
python scripts/test-codex-runtime.py --stk target/release/stk.exe
python scripts/test-claude-runtime.py --stk target/release/stk.exe
```

On Unix, use `target/release/stk`. These checks use temporary settings and make no paid model calls. Codex may contact its plugin registry during startup. A nested Windows sandbox may prevent the Codex fixture from launching its own sandbox.

The [verification record](dual-client-verification.md) distinguishes tested Windows behavior from untested platforms. See [SPEC.md](../SPEC.md) for the decision matrix and storage format.

## README artwork

The Markdown stays hand-maintained. `node scripts/readme/build.mjs` rebuilds four theme and viewport variants of the hero from the client enum, default threshold, and declaration lines in the source. CI checks for stale art, missing links, missing assets, and a missing installation section.
