---
description: Use only when the user explicitly asks to enable session-wide automatic commit, push, PR, and merge; not for one-shot shipping requests.
---

# Merge — Auto-Merge Mode (Session-Wide)

> Note: inside a git worktree this skill may be exposed under a directory-scoped name (e.g. `.claude/worktrees/<name>:merge`). Invoke the scoped name — same skill, same behavior.

Invoking `/merge` does NOT do a one-off merge. It **flips on Auto-Merge Mode for the rest of the session**, like `/caveman` persists. From the moment it is on, every time a task is complete and verified (to the extent this environment allows), you run the **integration cycle** below automatically — no waiting to be asked, no per-merge confirmation.

Invoking `/merge` IS the user's standing authorization to merge into `main` repeatedly for the session. That is why there is no per-merge confirm gate (see [Why no confirm](#why-no-per-merge-confirm)).

## Step 0: Activate the mode

On `/merge`, announce activation in **plain prose** (not caveman), so the user can immediately correct a misread of this standing authorization. Say, concisely:

> **Auto-Merge Mode is ON for this session.** From now on, when a task is complete I will, without asking: commit the touched files, push, ensure a PR exists, and merge it into `main` (resolving conflicts where unambiguous). The session branch is kept the whole session. Say "stop merge" to turn this off.

Then continue the current work. The cycle fires on the **next** task completion (and every one after), not retroactively.

## The Integration Cycle

Run this whenever a task is complete and verified. "Complete" = the requested change is finished and verified to the extent this environment allows (read code / logs / headless rasterize) — NOT mid-task, exploratory, or throwaway work. Never fabricate verification to trigger the cycle.

### 1. Identify the branch
- `git branch --show-current`.
- If on `main` (should not happen mid-session): create a session branch first, never commit to `main` directly. The one session branch is reused for the whole session.

### 2. Commit + push the work
- Stage **only the files this task touched** — never blanket-commit unrelated changes (`git status --short` to see what's there).
- Commit with a clear message; end with the standard `Co-Authored-By:` trailer.
- `git push` (set upstream on first push of the branch).

### 3. Ensure a PR exists — reuse the open one, never open a second
- Check for an existing **open** PR on this branch first: `gh pr list --head "$(git branch --show-current)" --state open --json number,url`.
- If one is open, that IS the PR for this unit of work — the `git push` in step 2 already updated it. Do not open another. **One open PR per unit of work.**
- Open a fresh PR only when none is open — the branch has no PR yet, or its prior PR is `MERGED`/`CLOSED` (a reused branch's old PR closes after each merge): `gh pr create --base main --fill` (or use the `pr` skill). Confirm `baseRefName` is `main`.

### 4. Sync with main + check conflicts
- `git fetch origin`.
- Inspect `mergeable` / `mergeStateStatus`. `main` advances fast (other sessions land work), so expect occasional divergence.
- If clean (`MERGEABLE`), go to step 6.

### 5. Resolve conflicts (like normal)
If `mergeable` is `CONFLICTING` or the merge is blocked by divergence:
- `git merge origin/main` into the session branch.
- Resolve conflicts the normal way: open each conflicted file, keep both sides' intent, remove markers, `git add`, commit the merge, `git push`.
- **Auto-clarity carve-out:** resolve only conflicts where the correct resolution is unambiguous. If both sides changed the same logic and the right merge is a real judgment call (risk of silently dropping someone's work), **stop, report the conflicted hunks in plain prose, and ask** before committing. Do not guess on semantic conflicts.
- Re-check `mergeable`, then proceed.

### 6. Merge into main
```
gh pr merge <number> --squash
```
- `--squash` → squash-merge is the **default** for this repo; the PR's commits collapse into one commit on `main`.
- **No `--delete-branch`** — the one session branch is kept until the session is done.
- **No `--merge` / `--rebase`** unless the user explicitly asked — squash is the default.
- **No `--admin`** — do not bypass branch protection or failing required checks. If the merge is blocked by checks/protection, report why and stop (pause the cycle for that task); do not force it.

### 7. Report
Confirm the merge landed, give the PR URL, note the branch was kept. If anything blocked it (failing checks, protection, unresolved/ambiguous conflict), report the exact `gh`/`git` output and the reason — never claim success you did not verify.

## Why no per-merge confirm

Merging into `main` is outward-facing and hard to fully undo. The single confirmation is **turning the mode on** — that is the explicit, standing authorization for the session. After that, per-merge prompts would defeat the purpose. The safety valves that remain:
- the mode only fires on genuinely-complete, verified work;
- ambiguous/semantic conflicts still stop and ask;
- branch protection / required checks are still respected (no `--admin`);
- the user can say "stop merge" at any time.

## Deactivation

Turn the mode OFF when the user says "stop merge", "stop auto-merge", "normal mode for merging", or the session ends. The session branch is **not** deleted on deactivation — clean up manually only when the session's work is truly done.

## Anti-patterns

- Don't merge mid-task, exploratory, or unverified work — "complete + verified" is the gate.
- Don't fabricate verification just to trigger the cycle.
- Don't blanket-commit unrelated files — stage only what the task touched.
- Don't open a second PR for a branch that already has an open one — reuse it (`gh pr list --head <branch>` first). One open PR per unit of work.
- Don't push straight to `main` via `git push` — always integrate through `gh pr merge`.
- Don't delete the branch (`--delete-branch`) — one branch for the whole session.
- Don't switch away from squash (to `--merge`/`--rebase`) on your own — squash is the default; other methods need an explicit ask.
- Don't bypass protections/checks (`--admin`) without an explicit ask — report the block and stop.
- Don't guess on semantic merge conflicts — resolve the unambiguous ones, stop and ask on the rest.
- Don't fabricate success — report the real `gh pr merge` / `git merge` outcome.
