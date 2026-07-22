<div align="center">

# STK — Session Token Killer

**A Claude Code hook that stops oversized file reads from flooding your context.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-win%20%7C%20macos%20%7C%20linux-lightgrey.svg)](#install)

*Sibling to [RTK (Rust Token Killer)](https://github.com/reachingforthejack/rtk). RTK kills tokens per **command**. STK kills them per **session**.*

</div>

---

## The one number that started this

We mined **250 real Claude Code sessions** (27.8 MB of tool output) to find where the tokens actually go. The answer was lopsided:

> **85% of all oversized (>8 KB) context came from a single source — the native `Read` tool.**
> RTK's shell hook never sees it. Nothing did. That's the gap STK fills.

Big file reads are the fattest unfiltered stream left in an agent's context. STK intercepts them before they land.

## What it does

STK is a single Rust binary wired in as a Claude Code `PreToolUse` hook on the `Read` tool. When the agent tries to read a file, STK decides in under 10 ms:

- **Small file** (≤ 16 KB) → **pass through untouched.** No behavior change.
- **Already scoped** (`Read` with `offset`/`limit`) → **pass through.** Never fight a targeted read.
- **Big file, first sight** → **deny with an outline instead.** A line-numbered structure map (functions, classes, headings, or JSON keys) plus exact instructions to fetch any range with `offset`/`limit`. The agent gets the shape of the file for ~2 KB and pulls only the parts it needs.
- **Same file, already seen this session, unchanged** → **deny with a one-line "unchanged" note.** No re-sending 50 KB the model already has.
- **Anything STK can't analyze** (binary, missing, unreadable, malformed input) → **pass through.** Fail-open, always. STK never blocks a read it doesn't understand.

Full output is never lost — the agent re-reads any range on demand. STK trades a guaranteed full dump for a cheap map plus targeted fetches.

### What an outline looks like

Instead of 84 KB of source hitting context, the model sees:

```
stk clamp: src/pipeline.ts — 84.3 KB, 2140 lines (threshold 16 KB).
Outline below; fetch only what you need with Read(file_path, offset, limit).

   1  import { … } (12 import lines)
  40  export interface Config
  92  export class Pipeline
 118    constructor(opts: PipelineOpts)
 143    async run(input: Stream): Promise<Result>
 402    private flush()
 ...
2101  export function main()

Re-read a symbol's body: Read with offset=<line>, limit=<span>.
```

## Install

STK is a single binary with no runtime dependencies.

```bash
cargo install --path .
```

Then print the hook snippet and add it to your Claude Code settings:

```bash
stk init
```

`stk init` prints the exact `PreToolUse` block to paste into `~/.claude/settings.json` (or a project `.claude/settings.json`). **STK never edits your settings for you** — you paste it, so you stay in control of what runs.

Verify the hook is live:

```bash
echo {} | stk hook claude   # prints nothing, exits 0
```

## Usage

| Command | What it does |
|---|---|
| `stk hook claude` | The hook entry point (stdin JSON → decision JSON). Wired via `stk init`. |
| `stk outline <path>` | Print the outline for a file by hand. |
| `stk gain` | Savings so far: clamps, dedup hits, bytes avoided, estimated tokens. |
| `stk gain --json` | Same, machine-readable (totals + per-day series) for dashboards. |
| `stk config` | Show active config and store location. |

### Configuration

All optional, via `stk`'s config file (path shown by `stk config`):

```toml
clamp_threshold   = 16384      # bytes; files at or below this always pass through
outline_max_lines = 80         # cap on outline length
dedup             = true       # dedup identical re-reads within a session
exclude           = ["*.lock"] # globs that always pass through untouched
```

## Honest limitations

STK reports **bytes avoided** — the file bytes it kept out of context minus the small outline it sent. That number is real, but it is an **upper bound**, not a net:

- When the agent needs the actual content, it re-reads specific ranges. Those follow-up reads cost tokens STK can't see from the hook, so **true savings are somewhat lower than the raw counter.** `stk gain` says so in its own output.
- STK only sees the `Read` tool. Shell command output is [RTK](https://github.com/reachingforthejack/rtk)'s job — run both.
- Measured session-level exact-repeat rate was only 3.2%, so STK deliberately ships **no diff/delta engine** — dedup is a cheap exact-hash check, nothing more. We built what the data justified and skipped what it didn't.

## How it fits with RTK

|  | [RTK](https://github.com/reachingforthejack/rtk) | STK |
|---|---|---|
| Scope | Shell command output | `Read` tool output |
| Unit | Per command | Per session |
| Mechanism | Filter & compress | Outline & clamp |
| Loss | Lossless (strips decor) | Lossy but recoverable (re-read on demand) |

They cover different halves of the same problem. Together they clamp the two fattest input streams an agent pays for.

## Development

```bash
cargo build --release
cargo test            # 35 tests: decision matrix, outline generation, dedup, fail-open, JSON contract
```

See [SPEC.md](SPEC.md) for the full design: decision matrix, outline format per file family, session store layout, and the fail-open contract.

## License

[MIT](LICENSE) © ryanportfolio
