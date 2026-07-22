---
description: Use when the user asks for a self-contained audit prompt to paste into a separate fresh session for independent verification.
---

# Handoff audit — draft a prompt for a fresh session to verify your work

The deliverable is a single fenced markdown block the user copies into a new
session. That session has **zero context** from this one: no memory of what
changed, why, or what you verified. The prompt must carry everything the auditor
needs to independently falsify the change — and it must invite them to find
breaks, not rubber-stamp.

The value of a separate session is independence. Do NOT spawn a subagent or run
the audit yourself here — that defeats the purpose. Your job is only to write the
prompt.

## Step 1: Determine scope

Use `$ARGUMENTS` if the user named a target (a commit, a PR, a file, "the auth
refactor"). Otherwise default to the work just completed on this branch:

- `git branch --show-current`
- `git log --oneline main..HEAD` (the commits under audit)
- `git diff --stat main..HEAD` (files touched)

Capture the exact branch name, the relevant commit SHA(s) newest-last, and the
repo (`git remote get-url origin`). The auditor needs these verbatim.

## Step 2: Extract the CLAIMS (the heart of the prompt)

An audit is only useful if it can falsify something specific. State what the
change asserts is TRUE — so the auditor can try to prove it false. Pull these
from the diff and the conversation:

- **What changed** — moved/added/deleted/renamed, in concrete terms.
- **What MUST still hold** — invariants the change relied on. ("These 9 symbols
  stay live", "this object was dead", "behavior X is unchanged", "no other caller
  depends on the old signature").
- **Why each risky decision was safe** — the reasoning the auditor should
  independently re-derive, not take on faith. (e.g. "I deleted this because it had
  zero importers" → auditor re-greps importers themselves.)

If the change was a refactor/move, list the exact symbols/files involved. Vague
claims produce vague audits.

## Step 3: Pin a pre-change reference for fidelity checks

The strongest check for a move/refactor is a byte-diff of old vs new. Give the
auditor the git handle for the BEFORE state so they don't trust your summary:

- `git show <base-sha>:path/to/file` is the pre-change version.
- For moved code: "extract function/block X from `<base-sha>:oldfile` and diff
  against its new home in `newfile` — they should differ ONLY in <expected
  context>; flag any change to logic, data, or values."

This catches silent truncation/alteration that a scripted edit can introduce.

## Step 4: Derive concrete, falsifiable checks

Translate each claim into a check with explicit pass/fail criteria. Common ones:

- **No dangling consumers.** For every deleted/moved symbol, grep the whole repo
  (`grep -rw`, word-boundary — warn about substring false positives, e.g.
  `User` matching inside `UserProfile`, or `get` inside `getCached`). Anything
  still importing it from the old
  location outside the intended file(s) is a BREAK.
- **Deleted-thing-was-really-dead.** Independently re-grep importers of anything
  deleted. If found, the deletion was wrong.
- **Kept-thing-still-wired.** For symbols deliberately retained, confirm they're
  still defined, exported, and referenced by their live consumers.
- **Fidelity diff** (Step 3) for moved code.
- **Completeness.** Every reference site updated; no half-renamed call.
- **Syntax/balance.** Brace/tag balance, no orphaned imports, no duplicate imports.
- **Comment-noise warning.** If you left `// X moved to …` pointer comments, tell
  the auditor those are NOT real references so they don't false-alarm.

Tailor the list to the actual change — drop irrelevant checks, add domain-specific
ones (schema, i18n coverage, naming/copy policies, etc.).

## Step 5: State the environment constraints

So the auditor doesn't fabricate verification it can't do. Pull the real
constraints from the project's CLAUDE.md (e.g. whether the sandbox can run
installs, type-checks, or a dev server, and what the authoritative verification
signal is — deploy log, CI, local test suite). Tell them to flag
plausible-but-unprovable risks rather than assert false confidence.

## Step 6: Specify the output contract

- PASS/FAIL per check, each with `file:line` evidence.
- Before reporting any FAIL, re-verify it with a second grep/read — confirm it's a
  real consumer, not a substring match or a pointer comment. A false alarm wastes
  the handoff; a missed break defeats it.
- Fix ONLY real breaks; don't refactor or "improve" beyond the audit.
- If fixing, commit only the touched files; otherwise commit nothing.
- If everything's clean, say so plainly — no padding.

## Step 7: Emit the prompt

Output the whole thing as ONE fenced code block (use `````markdown` fencing so any
inner code blocks survive copy/paste). Structure it so the auditor can't confuse
the facts under audit with the instructions: open with a role and a grounding line,
then wrap each content type in its own XML tag.

- **Role — adversarial.** Open with a falsify-first stance, not a neutral one:
  "You are a skeptical code auditor. Assume this change is wrong until your own
  grep/read proves otherwise — your job is to break it, not to confirm it." The
  whole value of the audit is this stance; a soft "please review" framing forfeits it.
- **Grounding.** State plainly that every verdict comes from the auditor's own
  investigation: "Ground every PASS/FAIL in files you open and greps you run
  yourself. Do not trust this prompt's summary of what changed — re-derive it from
  the diff and the code."
- **Tagged sections, in order:**
  - `<audit_target>` — repo URL, branch name, commit SHA(s) newest-last, verbatim.
  - `<claims>` — the assertions from Step 2 the auditor must try to falsify.
  - `<fidelity_reference>` — the pre-change git handles from Step 3 (moves/refactors only).
  - `<checks>` — the concrete pass/fail checks from Step 4.
  - `<constraints>` — the environment limits from Step 5.
  - `<output_contract>` — the reporting format from Step 6.

End your chat reply (outside the block) with a 1–2 line note on which checks carry
the most real downside, so the user knows where the risk concentrates.

Do not commit anything in this session — the prompt is the only artifact, and it
goes to the user, not a file (unless they ask to save it).

## Anti-patterns

- Don't run the audit yourself or spawn a subagent — independence requires a
  separate session with no shared context.
- Don't write a vague "please review my changes" prompt. No claims = no audit.
  The auditor needs specific, falsifiable assertions and the git handles to test
  them.
- Don't omit the pre-change git reference for moves/refactors — "trust my summary"
  is exactly what the audit exists to avoid.
- Don't tell the auditor to run builds/type-checks the sandbox can't run, or let
  them claim runtime verification they didn't do.
- Don't forget the substring/word-boundary and pointer-comment caveats — they are
  the most common sources of false alarms in grep-based audits.
