---
name: optimize-context
description: Use when the user asks to reduce per-turn context or token load, trim kernels, skills, or connectors, or propagate a generic context optimization to the starter.
---

# optimize-context — cut per-turn context cost, propagate to the starter

The dominant token cost in long sessions is **input that reloads every turn**, not output prose. This skill is the playbook for finding and cutting it, plus porting the generic wins to `ryanportfolio/AI-Firmware` (formerly `claude-starter`) so every future repo inherits them.

**First principle:** compression is **lossless when you relocate, not delete.** Always-loaded files become thin hooks; full detail moves to a subfile (`.claude/reference/`, a MEMORY subfile) read on demand. Accuracy/rules never trade against brevity — only fluff and duplication die.

## The lever catalog (ranked by payoff)

### 1. MCP connectors — biggest single win, NOT scriptable
Each connected MCP server injects its tool-name list AND any instruction block into every turn. `claude mcp list` shows them. Disconnect ones this project never uses.
- Account/marketplace connectors (claude.ai → Settings → Connectors) are **account-level** — `claude mcp remove` can't touch them (they re-sync); the user toggles them in the UI. Reversible (reconnect anytime).
- This is the user's action, not a file edit. Identify the dead weight, name it, let them disconnect. Flag if a disconnect removes a capability the project sometimes needs (e.g. Context7 live docs → say so when you'd have used it).

### 2. Skill visibility — `skillOverrides` (project/bundled skills ONLY)
The harness injects every visible skill's name + description every turn. Hide unused **project or bundled** skills in **committed** `.claude/settings.json`:
```json
"skillOverrides": { "lab": "off" }
```
- Values (schemastore-verified): `on` | `name-only` | `user-invocable-only` | `off`. **`off`** = gone from Claude's context AND the `/` picker.
- **Keys are BARE skill names** (the project skill's directory name).
- **PLUGIN SKILLS ARE NOT AFFECTED — hard limitation, schema-explicit.** Any key targeting a plugin skill (colon-prefixed or bare) is a silent no-op; 17 such keys sat in this template as dead config until 2026-07. Trim plugin skills at their source instead:
  - Account-synced skills plugins (`@inline`, from claude.ai) → disable the individual skills or the sync in claude.ai settings. Per-account, not a repo file.
  - Marketplace plugins → `claude plugin disable <name>` (all projects) or uninstall. No per-project plugin disable exists.
- `off` takes effect **next session**. Scope precedence: `settings.local.json` > project `settings.json` > `~/.claude/settings.json`.

### 3. Kernel (CLAUDE.md) + index (MEMORY.md) compression
- **Caveman-ultra the prose**, keep every rule exact.
- **Thin hooks, detail in subfiles.** A fat index/kernel line taxes every turn; the subfile is free until recalled. Push PR-history / mechanism / proof to `.claude/reference/<topic>.md` (or a MEMORY subfile); leave a one-line hook + pointer. Before pointing a hook at a subfile, confirm the detail is actually there — relocate, don't dangle.
- **Don't restate what the harness already injects every turn:** the available-skills list, the environment block (OS/shell/cwd/git/model), tool-doc behavior. Cut them — keep only the project's value-add (non-obvious implication, version pins not in env, the project-specific rule).
- **Remove untrue / stale / irrelevant rules** outright (with the user's confirmation when it's a behavioral rule).

### 4. Verify the cut
- **Measure first, then again after:** `bash .claude/scripts/context-weight.sh` prints the file-measurable per-turn weight (kernel + global CLAUDE.md + every skill's injected description) with per-skill breakdown. Run it before touching anything so the saving is a real delta, not a guess.
- Byte counts before/after (`wc -c`) for kernel/index edits.
- For skill disables / MCP disconnects: confirm **next session** the items are gone (current session's context is fixed).
- Never claim a per-turn saving you didn't measure.

## Gotchas (burned in)
- **Durable vs ephemeral carrier:** a rule that looks duplicated by a SessionStart-hook message is NOT safe to cut — hook output is a one-time early message (droppable at compaction); CLAUDE.md re-injects every turn, so the kernel is the durable home (e.g. the caveman default stays in CLAUDE.md).
- **`skillOverrides` is a silent no-op for plugin skills** — no error, the skill just keeps loading. A `skillUsage` entry like `superpowers:writing-skills` proves the skill RAN, not that an override key in that form works. Verify a disable by checking the skills list in a NEXT session, never by the write succeeding.
- **Don't trust an agent's schema claim un-verified** — confirm enum values / property names against the real schema (`schemastore.org/claude-code-settings.json`) or the running config.

## Propagation to claude-starter
Only the **generic, portable** wins go up — run them through `sync-starter` Direction B (it owns the genericize + apply-at-`~/code/AI-Firmware` + PR mechanics):
- **Portable:** `skillOverrides` disabling broadly-unused bundled skills (lean default; a project that needs `docx`/`pdf` re-enables it), CLAUDE.md structural conventions (caveman default, thin-hooks, don't-restate-injected-content), a leaner skill file (e.g. the trimmed `caveman`).
- **NOT portable:** account-level MCP connector disconnects (per-account, not a repo file), project-specific reference/memory content, anything named after the current project.
- Genericize first: strip project names/paths/URLs. If it can't be genericized, it stays local.

## Anti-patterns
- Deleting detail instead of relocating it to a subfile.
- Cutting a rule that must persist all session (durable-carrier trap).
- Bare-naming a `skillOverride` that has a project dupe.
- Disabling a bundled skill a future project might need without leaving it re-enableable (the template default is lean; document the re-enable).
- Pushing project-flavored content to the template — genericize or leave it.
- Running this on every session — it's occasional, user-triggered maintenance.
