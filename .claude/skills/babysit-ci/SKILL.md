---
name: babysit-ci
description: "Watch a PR's checks and iterate on failures until green. Use for /babysit-ci, \"watch CI\", \"fix CI\", \"get the checks green\", or when a PR is waiting on failing or pending checks."
---

# Babysit CI

Drive a PR's checks to green: watch, diagnose, apply the smallest fix, push, repeat.

`gh pr checks` is the source of truth. It includes all PR-attached checks; `gh run list` only covers GitHub Actions and misses external checks.

## Workflow

1. Resolve the active PR: `gh pr view --json number,url,headRefName`.
2. Inspect current checks before waiting: `gh pr checks --json name,bucket,state,workflow,link`.
3. Checks already failed → diagnose those first. For a GitHub Actions check, `gh run view <run-id> --log-failed` and extract the first actionable error; otherwise follow the check's link to identify the failing command or service.
4. Checks pending → watch with `gh pr checks --watch --fail-fast`. For long waits, run the watch in a background task and continue other work; report when it resolves.
5. Apply the smallest safe fix for one failure cause. Push.
6. Re-run `gh pr checks --json name,bucket,state,workflow,link` after every push — the check set itself can change — and repeat until green.

## Guardrails

- Fix one actionable failure at a time; prefer minimal, low-risk changes before broader refactors.
- Never bypass hooks (`--no-verify`) to force progress.
- Failure clearly unrelated to the PR and already fixed on main → merge latest main instead of bloating the PR with unrelated fixes.
- Flaky failure → retry once and report the flake evidence; don't silently re-run until green.
- Verify a fix locally when a cheap local repro of the failing command exists, before spending a CI round trip.

## Output

- Current CI status.
- Failure summary and fixes applied, in iteration order.
- PR URL once checks are green; next action if blocked.

---
Merged from the `loop-on-ci` + `fix-ci` skills and `ci-watcher` agent in [cursor/plugins cursor-team-kit](https://github.com/cursor/plugins/tree/main/cursor-team-kit) (MIT).
