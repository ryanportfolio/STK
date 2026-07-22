---
name: conflict
description: Use only when the user invokes /conflict and a merge, rebase, or cherry-pick has unmerged paths or conflict markers, especially after parallel edits.
disable-model-invocation: true
---

# conflict — review and resolve an in-progress merge conflict

Resolve the conflict already sitting in the working tree. This skill does **not** start a merge/rebase — if no operation is in progress, stop and tell the user there's nothing to resolve.

Pass through `$ARGUMENTS` as a scope hint if given (e.g. `/conflict the PageY changes`).

## The Iron Rule

**Never blind-pick a side.** Every `git checkout --ours` / `--theirs` on a whole file, and every "I'll just take origin/main," is forbidden until you have read both sides and classified the hunk. Each conflicted hunk is one of two things — handle them differently:

| Hunk type | What it means | Action |
|---|---|---|
| **Separable** | The two sides changed *different, independent things* in the same region (one added a tooltip, the other added a Print button) | **Auto-merge**: union both. Preserve every line of intent from both sides. |
| **Contradiction** | The two sides set the *same thing* to *different values* (`maxBullets: 7` vs `5`; same className, different value; same function rewritten two ways) | **STOP. Do not pick.** Surface both values to the user and checkpoint. Inventing a reason one side "wins" is the failure this skill exists to prevent. |

When two sessions edited the same page, expect both: some separable (auto), some genuine contradictions (checkpoint). Triage per hunk, not per file.

## Procedure

1. **Confirm an op is in progress.** `git status` — look for "You have unmerged paths" / "rebase in progress" / "cherry-pick". If none, stop: nothing to resolve.
2. **Note the operation** — it flips the sides:
   - **merge**: `HEAD`/`ours` = your branch, `>>>>>>>` side = incoming.
   - **rebase**: **reversed** — `ours` = the branch you're rebasing *onto* (usually main), `theirs` = your commits. Re-read the labels every time; don't assume.
3. **List conflicts:** `git diff --name-only --diff-filter=U`.
4. **Per file, per hunk:** read the full hunk (both sides, plus surrounding context — `git show :1:path` is the merge base if you need it). Classify separable vs contradiction (table above).
   - Separable → merge preserving both, no markers left.
   - Contradiction → leave it, collect it for the checkpoint.
5. **Hazard files** (don't hand-merge — see table below).
6. **Checkpoint** any contradictions with the user before committing (see Commit).
7. **Sweep:** grep the whole repo for leftover markers — `<<<<<<<`, `=======`, `>>>>>>>` — must be zero. (`=======` can false-positive on markdown rules / `====` comment banners; eyeball hits.)
8. **Stage + verify + commit** (below).

## Hazard files — regenerate or preserve, never hand-merge

| File | Why | Do |
|---|---|---|
| Lockfiles (`package-lock.json` etc.), build artifacts, generated output | Hand-merging corrupts them | Take one side, then regenerate per the project's install policy (check CLAUDE.md — some projects route installs through a separate agent/environment). Don't silently ship a hand-stitched lock. |
| Locale/translation JSONs | Often fixed EOL (CRLF); reformatting rewrites every line | Resolve only the conflicted keys, preserve the file's EOL style, never re-stringify the whole file. New UI strings must land in **all** locales if the project has no fallback. |
| Schema files (ORM table definitions) | Schema drift vs the live DB | Resolve carefully; if a column/table differs, follow the project's migration policy (CLAUDE.md) — don't assume the merge fixed the database. |

## Verification

Run the project's verification command if the environment allows it (check CLAUDE.md's verification section). If the sandbox can't run type-checks/builds:
- Self-review the resolved diff: imports still used, no dangling JSX tags, both intents actually present.
- If your resolution plausibly changed types, **flag it as a regression risk** and name the authoritative check (deploy log, CI). Do **not** claim "verified" or "type-checks pass."

## Commit

- **Trivial-only resolution** (all hunks were separable, plus hazard files handled) → resolve, stage the conflicted paths explicitly, commit per the project's auto-commit rule. Standard merge/rebase continue (`git commit` with no `-m` keeps the merge message, or `git rebase --continue`).
- **Any contradiction present** → show the user a per-contradiction summary (file, both values, your recommendation + why) and get an explicit OK before committing. Merge commits are painful to unwind → this is a deliberate checkpoint, not caveman-terse.
- **Never** violate the project's naming/copy policies in the merge/commit message (if CLAUDE.md constrains what names may appear in user-visible text, that applies to commits too).
- If a pre-commit hook fails, fix the cause and make a new commit — never `--no-verify`, never `--amend` unasked.

## Common mistakes (observed)

- **Silent arbitrary pick on a contradiction.** Baseline behavior: faced with `7` vs `5`, the model takes one and writes a confident rationale ("upstream baseline these should track") that is pure invention. One session's deliberate change vanishes. → If you can't tell which value is *correct* (not which is "upstream"), it's a contradiction: checkpoint.
- **Union-ing a real contradiction.** Two incompatible rewrites of the same function don't concatenate. If both sides can't coexist, it's a checkpoint, not a merge.
- **Leaving a marker.** Always sweep after. `>>>>>>>` compiling is luck, not success.
- **Forgetting rebase reverses ours/theirs** → you "kept your work" and actually kept main's.
- **Reformatting a CRLF locale file** → diff explodes, real change buried.

## Red flags — stop

- About to `git checkout --ours/--theirs` a whole file without reading it
- Writing a sentence explaining why one side "wins" a same-value contradiction
- Claiming the result type-checks
- Committing a contradiction resolution without showing the user both values first
