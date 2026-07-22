# stk: Session Token Killer

Sibling to RTK (Rust Token Killer). RTK compresses shell command output; stk clamps the
fattest remaining context stream: the native `Read` tool. Measured on 250 real Claude Code
sessions: 85% of oversized (>8KB) tool-result bytes came from `Read` (9.1MB of 10.7MB),
which RTK's Bash hook can never see.

## Mechanism

Claude Code `PreToolUse` hook matched on `Read`. The hook receives JSON on stdin:

```json
{
  "session_id": "abc123",
  "tool_name": "Read",
  "tool_input": { "file_path": "C:\\path\\to\\file.ts", "offset": 100, "limit": 50 }
}
```

Decision logic (in order):

1. `tool_input` has `offset` or `limit` → **allow** (model already scoping; never fight it).
2. File missing / unreadable / not a regular file → **allow** (let Read produce its own error).
3. File is binary (NUL byte in first 8KB) or an image/PDF/notebook extension → **allow**.
4. File size ≤ `clamp_threshold` (default 16 KiB) → **allow** + record hash in session store.
5. Same `file_path` + same content hash already recorded this `session_id` → **deny**, reason:
   `"stk: file unchanged since stk last saw it this session (hash match). Re-read with offset/limit if you need to re-check a specific range."`
6. Else (big file, first sight) → **deny**, reason = generated **outline** + footer.

Allow = exit 0, no stdout (passthrough). Deny = exit 0, stdout JSON:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "<outline text>"
  }
}
```

Never exit non-zero for policy decisions (non-zero = hook error, pollutes context).
On ANY internal error: allow (fail-open). The hook must never block a read it cannot analyze.

## Outline format (the deny payload)

Header + structure map + retrieval instructions. Target ≤ 60 lines / ≤ 2.5KB. Example:

```
stk clamp: C:\repo\src\big.ts, 84.3 KB, 2140 lines (threshold 16 KB). Outline below;
fetch only what you need with Read(file_path, offset, limit).

   1  import { … } (12 import lines)
  40  export interface Config
  92  export class Pipeline
 118    constructor(opts: PipelineOpts)
 143    async run(input: Stream): Promise<Result>
 402    private flush()
 ...
2101  export function main()

Re-read a symbol's body: Read with offset=<line>, limit=<span>. Whole file only if truly
needed: re-Read with offset=1, limit=2140.
```

Outline generator (deterministic, no model):
- Line-numbered skeleton: lines matching structural patterns per extension family.
  - code (rs/ts/js/py/go/java/cs/c/cpp/rb/php…): top-level + indented decl lines
    (fn/class/struct/impl/interface/def/export/pub/function), attribute of `#[test]`-style
    lines skipped; collapse consecutive imports to one count line.
  - markdown: heading lines (`#`…), fenced-code-block count.
  - json: top-level keys (+ child keys to depth 2), array lengths, never values.
  - other text: first 10 lines + last 5 lines + total count.
- Hard cap: if outline would exceed 80 lines, keep first 60 + `… (+N more entries)`.
- Always include exact total line count so `offset=1, limit=N` full fetch is expressible.

## Session store

`%LOCALAPPDATA%\stk\sessions\<session_id>.jsonl` (append-only records):
`{"ts":…,"file":…,"size":…,"hash":"sha1 of content","action":"allow|clamp|dup"}`
Hash computed only for files ≤ 4 MiB (else skip dup layer). Store read = scan lines for
latest record per path. Prune: on startup, delete session files older than 14 days.

`%LOCALAPPDATA%\stk\stats.jsonl` (one record per clamp/dup):
`{"ts":…,"file":…,"file_bytes":…,"sent_bytes":<outline len>,"kind":"clamp|dup"}`

## CLI surface

```
stk hook claude        # stdin JSON → decision JSON (the hook entry point)
stk outline <path>     # print outline for a file (manual/debug)
stk gain               # aggregate stats: clamps, dup hits, bytes avoided, est. tokens (bytes/4)
stk init               # print the settings.json hook snippet + install instructions (does NOT edit settings)
stk config             # print active config (TOML at %APPDATA%\stk\config.toml, all keys optional)
```

Config keys: `clamp_threshold` (bytes, default 16384), `outline_max_lines` (80),
`dedup` (bool, true), `exclude` (glob list, e.g. `["*.lock"]` always allowed through).

## Non-goals (v1)

- No Bash/shell output filtering (RTK owns that).
- No diff/delta emission (measured repeat rate 3.2%, not worth a diff engine).
- No PostToolUse rewriting (platform doesn't support output replacement).
- No editing of user settings files; `stk init` prints, user installs.

## Engineering constraints

- Rust 2021, single binary, clap for CLI, serde/serde_json, sha1 or sha2 crate, toml, glob.
  No async runtime (hook path must be <10ms typical; plain std).
- Windows-first (dev box) but cross-platform paths via `dirs` crate; no unix-only calls.
- `cargo test` covers: decision matrix (all 6 rules), outline generation per file family,
  JSON in/out contract (golden samples), store dup detection, fail-open on malformed stdin.
- Honest metric: `gain` reports bytes-avoided = file_bytes − sent_bytes per clamp, and
  separately counts "re-fetch follow-ups not measurable from here" caveat in output.
```
