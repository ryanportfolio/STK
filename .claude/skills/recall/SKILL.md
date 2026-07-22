---
description: Use before unfamiliar project-area edits, when the user asks what the project knows or its pitfalls, or when saving durable project-specific learning.
---

# recall — project memory

Topical project reference lives in `.claude/reference/<topic>.md`. CLAUDE.md carries only cross-cutting safety and process rules; everything else (env vars, architecture quirks, area pitfalls) is in the reference. This skill is the read/write interface to that store.

## When to invoke

**Lookup (most common):**
- User says `/recall <topic>` or `/recall <question>`.
- User asks "what do we know about X", "any pitfalls with Y", "remind me how Z works".
- BEFORE editing in an area you don't already have loaded — any subsystem with its own reference file, plus secrets/env wiring.

**Capture:**
- User says `/recall save <text>`, "remember this", "save this learning", "add to reference".
- A quirk just bit you (or just bit the user) and the lesson belongs in the next session's context.

## Step 1: Look up

1. List the available topics:
   ```
   ls .claude/reference/
   ```
2. Match the user's query to a topic. Starter topics (the project may have grown more — trust the `ls`):

   | File | Covers |
   |---|---|
   | `secrets.md` | Env var names + what they key |
   | `architecture.md` | System flow, auth, state strategy |
   | `pitfalls.md` | Accumulated cross-cutting gotchas |
   | `commands.md` | Build / dev / test scripts |
   | `tech-stack.md` | Non-default library choices and why |
   | `deployment.md` | Deploy target, build output, asset paths |

3. Read the matched file(s). If the query spans multiple topics, read each.
4. If nothing in the table fits the query, grep the directory for keywords:
   ```
   grep -rn -i '<keyword>' .claude/reference/
   ```
5. Summarize the relevant entries to the user with file:line references.

If nothing relevant exists, say so plainly — don't fabricate from memory or guesses.

## Step 2: Capture

When the user wants to save a learning:

1. **Pick the topic file.** Use an existing file when the topic fits. Create a new file only if no existing topic fits AND the topic is durable (worth a permanent home, not a one-off).
2. **Append at the bottom** under a dated header:
   ```markdown
   ### YYYY-MM-DD: <short title>

   <Body — 1 to 5 sentences. State the symptom and the fix, not just the fix. Include `file:line` refs where relevant. Don't quote large blocks of code.>
   ```
3. **Date format:** today's date. Check the conversation context's `# currentDate` block first; otherwise run `date +%Y-%m-%d`.
4. **If creating a new topic file,** also add a row to the index table in CLAUDE.md's "Project Reference Library" section so future sessions discover it.
5. **Commit on the current branch** with a message like `recall: <short title>`. Don't bundle unrelated changes.
6. **Generic-check.** If the lesson would bite ANY project — tooling/shell/git traps, Claude Code behavior quirks, workflow improvements — say so and offer to push it to the claude-starter template via `/sync-starter` (Direction B). Capture locally either way; push-back is additive, not instead-of. Project-specific lessons (this stack, this deploy target, this codebase) stay local — don't offer.

## Step 3: Stay disciplined

- **Never duplicate kernel rules.** Cross-cutting safety/process rules stay in CLAUDE.md. The reference is for area-specific info that only matters when working on a specific subsystem.
- **Don't let `pitfalls.md` become a junk drawer.** If it grows past ~200 lines, propose splitting by area (`pitfalls-<area>.md`) and update the CLAUDE.md index.
- **Keep entries terse.** A learning is a flag for the next session, not a tutorial. 1–5 sentences. Link to code, don't quote it at length.
- **Date everything.** Future-you needs to know which entries might be stale.

## Anti-patterns

- Don't move safety-critical rules out of CLAUDE.md. Those need to be active every session, not loaded on demand by description matching.
- Don't append to `.claude/reference/` without committing — uncommitted entries can vanish with the sandbox.
- Don't fabricate entries. If you don't know what's already in a file, read it; don't summarize from training-data guesses.
- Don't trigger this skill for tasks that aren't about this codebase (generic library questions, design discussions, unrelated CLI help).
- Don't expand an entry on re-read. Replace stale entries; don't accrete over time.
- Don't use `git add -A` or `git add .` when committing a recall entry — stage only the reference file you touched.
