# Stats publisher recovery implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use the available Codex execution workflow to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore the live STK chart without losing unpublished snapshots, then make future remote divergence visible and self-recovering.

**Architecture:** Preserve the current local history, rebase it onto GitHub's current `main`, and publish it. Update the PowerShell publisher to synchronize before generating a snapshot and to turn every failed native Git command into a nonzero task result.

**Tech Stack:** Git, Windows PowerShell 5.1, Windows Task Scheduler, GitHub Pages

---

### Task 1: Preserve and synchronize unpublished snapshots

**Files:**
- No file changes

- [ ] Disable `STK stats publish` during repair.
- [ ] Create backup branch `codex/stats-backlog-20260822` at the current local `main` tip.
- [ ] Fetch `origin/main` and confirm the split: local snapshot commits versus remote PR merges.
- [ ] Rebase the local snapshot chain onto `origin/main`.
- [ ] Confirm `docs/data/gain.json` still ends on `2026-08-22`.

### Task 2: Harden publisher behavior

**Files:**
- Modify: `scripts/publish-stats.ps1`

- [ ] Add a native Git helper that throws when `$LASTEXITCODE` is nonzero.
- [ ] Fetch and rebase onto `origin/main` before generating the snapshot.
- [ ] Route `git add`, `git commit`, and `git push` through the checked helper.
- [ ] Parse-check the script with the Windows PowerShell parser.
- [ ] Test the failure path with a stub `git` executable and confirm a nonzero script exit.

### Task 3: Publish and verify

**Files:**
- Modify: `.claude/reference/deployment.md`
- Modify: `.claude/reference/pitfalls.md`
- Add: `docs/superpowers/plans/2026-08-22-stats-publisher-recovery.md`

- [ ] Record the synchronization rule and the 2026-08-19 failure mode.
- [ ] Commit only the hardened publisher, references, and this plan.
- [ ] Push the rebased history to `origin/main`.
- [ ] Fast-forward the primary checkout to the pushed commit while preserving its untracked VBS wrapper.
- [ ] Re-enable and run the scheduled task once.
- [ ] Confirm remote `gain.json`, GitHub Pages origin, and `savetokens.tips/stk/` show the current date.
