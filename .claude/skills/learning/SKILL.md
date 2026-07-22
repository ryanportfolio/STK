---
name: learning
description: Use after confirmed multi-attempt problem solving reveals durable gotchas, or when the user asks to capture or save session learnings.
---

# learning — retrospective synthesis of session lessons

## Overview

Looks back at the conversation that just finished, extracts the generalizable lessons (not the specific fix), routes each to the right `.claude/reference/<topic>.md` file, and writes dated entries to it.

## When to use

- User invokes `/learning`, asks to "capture learnings", "save learnings from this session", "what did we learn"
- A debug or implementation arc just concluded successfully (tests pass, behavior verified, user accepted)
- The path to the fix took **>1 attempt**, OR contradicted an initial assumption, OR revealed a wrong/stale/missing reference entry

## When NOT to use

- Short, surprise-free sessions — nothing to extract, exit
- Sessions that ended ambiguously or unverified — never capture lessons from outcomes you are not sure about
- The only insight is a user preference about Claude's behavior — that belongs in CLAUDE.md or personal config, not topical reference
- The lesson is already documented with the same content — touch the date instead, do not duplicate

## Process

### Step 1: Filter

Look back at the conversation context. A candidate must clear at least one bar:

- **Took >1 attempt** to get right (an incorrect path was tried first)
- **Contradicted an assumption** — Claude's or the user's
- **Revealed a wrong, stale, or missing entry** in `.claude/reference/` or `CLAUDE.md`
- **A workaround a future session needs to know about**

Skip routine implementation, "built X, worked first try" cases, lucky guesses, one-off debugging that does not generalize. **A high bar is the point.** If nothing clears it, say so and exit. Do not invent.

### Step 2: Draft entries

For each survivor, write one entry in this format:

```markdown
### YYYY-MM-DD: <short title>

<1–5 sentences. Symptom + underlying cause + the generalizable rule. Include `file:line` refs where relevant. Do not quote large code blocks. The fix is the least interesting part — the WHY is what makes it transferable.>
```

Today's date: check the conversation context's `# currentDate` block first; otherwise `date +%Y-%m-%d`.

### Step 3: Route to a topic file

Pick the right `.claude/reference/<topic>.md`:

| Lesson is about… | File |
|---|---|
| Cross-cutting gotcha, recurring footgun | `pitfalls.md` |
| Env var wiring, secret keys | `secrets.md` |
| Cross-cutting flow, system structure | `architecture.md` |
| Library-choice gotcha | `tech-stack.md` |
| Build / dev / deploy commands | `commands.md` |
| Deploy target, build artifacts | `deployment.md` |

(Check `ls .claude/reference/` — this project may have grown additional topic files beyond the starter set.)

If no existing topic fits and the lesson is durable, propose a new topic file *and* an index-row update to CLAUDE.md's "Project Reference Library" section so future sessions can find it.

### Step 4: De-dupe

For each candidate, grep the target file before writing:

```bash
grep -in "<keyword from candidate>" .claude/reference/<topic>.md
```

If a near-duplicate exists, **update the existing entry's date and append the new example as a one-line addendum** instead of creating a second entry. Prevents `pitfalls.md` from becoming a junk drawer.

### Step 5: Write

Apply the edits to the reference files.

## Anti-patterns

- **Fabricating learnings.** If nothing in the actual conversation supports the claim, do not write it. Reference what was actually attempted and what actually fixed it.
- **Lesson = the fix.** A lesson is the generalizable rule. "Changed line 42 from X to Y" is not a learning. "OpenRouter requests without the `:free` suffix silently bill against paid tiers" is.
- **Touching CLAUDE.md content.** This skill writes to `.claude/reference/`. Only touch CLAUDE.md to add an index row for a brand-new topic file. CLAUDE.md is hand-curated kernel content.
- **Running on unverified work.** If the user has not confirmed the fix actually works, exit and tell them to come back when it is confirmed.

## Hygiene

If `pitfalls.md` exceeds ~200 lines after a commit, propose splitting it by area (`pitfalls-<area>.md`) and update the CLAUDE.md index. Junk-drawer files defeat the point.
