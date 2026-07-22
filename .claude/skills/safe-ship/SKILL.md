---
description: Use when the user explicitly asks to ship changes end to end, including scope audit, branch, verification, commit, push, and pull request.
disable-model-invocation: true
---

# safe-ship — branch, stage, verify, commit, push, PR

Run the complete shipping pipeline for the user's working changes. Pass through `$ARGUMENTS` as scope hints if provided (e.g. `/safe-ship the hook changes`).

The pipeline is **fail-loud**: stop at the first violated rule. Never silently push to main. Never silently stage personal files. Never bypass verification.

## Step 1: Audit current state

Run in parallel:
- `git branch --show-current`
- `git status --short`
- `git log --oneline @{u}..HEAD 2>/dev/null` (unpushed commits on current branch)
- `git fetch origin main` (latest main reference)

Report back inline: current branch, count of changed files, whether on main, whether ahead/behind origin/main.

## Step 2: Branch hygiene — refuse main, branch off if needed

If current branch is `main`:

1. **Refuse to commit on main.** State this clearly inline.
2. Determine a branch name:
   - If `$ARGUMENTS` describes the work, derive a name like `feat/<short-kebab-description>` (max 5–6 words, lowercase, hyphens).
   - If no `$ARGUMENTS` context, infer from the file list (e.g. modifications mostly in `src/components/Foo/` → `feat/foo-component-changes`).
   - If still unclear, ask the user inline (not via popup) for a branch name.
3. Pull main if behind (`git pull` while on main with no uncommitted changes — only if working tree is clean; if dirty, pull will fail and that's fine, proceed to branch).
4. Create branch: `git checkout -b feat/<name>`. Working-tree changes follow automatically.

If already on a feature branch (`feat/*`, `claude/*`, etc.), skip branching.

## Step 3: Identify what to stage

List every file in `git status --short`. Categorize:

**Auto-exclude these without asking** (CLAUDE.md and gitignore say so):
- `.env`, `.env.local`, `.env.*.local`
- `.claude/settings.local.json`
- Personal launcher scripts and local log files
- Anything matching the repo's `.gitignore` (run `git check-ignore -v <file>` to confirm)
- Files under `.tmp/` (these are scratch by convention)
- Anything that looks like credentials, tokens, or large binaries (>1MB)

**Stage these by default if they relate to `$ARGUMENTS` scope**:
- Source files in the relevant directory tree
- Tests for changed code
- Schema/migration files when the change is schema-related

**Ask before staging** (inline, not via popup):
- Files in unrelated areas of the codebase that are also modified
- Untracked directories with many files
- `CLAUDE.md` modifications (they're often in-progress; ask "do you want the CLAUDE.md edits in this PR or saved for later?")

Show the user a structured list before running any `git add`:

```
Will stage (auto):
  M  server/foo.ts
  A  server/foo.test.ts

Will skip (excluded):
  M  .claude/settings.local.json    (per-machine, gitignored)
  ?? launch-log.txt                 (personal)

Need decision:
  M  CLAUDE.md                      (looks unrelated — include?)
  ?? .claude/skills/some-skill/     (untracked; intentional?)
```

Wait for the user's response, then proceed. **Never use `git add -A` or `git add .`.** Stage by explicit path:

```
git add server/foo.ts server/foo.test.ts
```

## Step 4: Verify before committing

Run the project's verification command per CLAUDE.md (look in CLAUDE.md's verification section or `package.json` scripts — e.g. a type check, lint, or test suite). If CLAUDE.md scopes verification differently for this environment (no installs, no dev server), honor that.

If verification fails:
- Report the failure inline with the relevant error excerpt
- **Do not commit.** Stop here. Let the user fix or decide to skip verification (which they must do explicitly).

If verification passes, continue.

## Step 5: Commit

Draft a concise commit message based on the staged changes:
- 1-line subject (under 72 chars), imperative mood ("Add X" not "Added X")
- Optional body explaining *why* (not what — the diff shows what)
- Always end with the Co-Authored-By footer

Use heredoc to preserve formatting:

```bash
git commit -m "$(cat <<'EOF'
Subject line under 72 chars

Optional body explaining why this change is needed,
the constraint that motivated it, or the bug it fixes.

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

If a pre-commit hook fails, **fix the underlying issue and create a NEW commit** — never `--amend` or `--no-verify`.

## Step 6: Push the branch

```
git push -u origin <branch-name>
```

If push is rejected because branch protection requires a PR (good, expected on `main` if branch protection is on), this should never happen because we're not on main. If it happens on a feature branch, something is mis-configured — report and stop.

## Step 7: Open the PR

Use `gh pr create` with a heredoc body:

```bash
gh pr create --title "<short title under 70 chars>" --body "$(cat <<'EOF'
## Summary
<1-3 bullets describing what this PR changes>

## Why
<motivation — the constraint or bug, not the diff>

## Test plan
- [x] <verification step you actually ran>
- [ ] <reviewer-side verification, if any>
EOF
)"
```

If `gh` is unavailable, fall back to printing the comparison URL (derive the repo path from `git remote get-url origin`): `https://github.com/<owner>/<repo>/compare/<branch-name>` and tell the user to open it.

## Step 8: Report and stop

Final output to the user is **the PR URL**. That's the deliverable.

**Do not** offer to:
- Watch CI
- Auto-respond to review comments
- Run further verification post-PR
- Suggest follow-up commits

Per CLAUDE.md: "After creating a PR, report the URL and stop."

## Anti-patterns

- **Never commit to main.** Even if the change is "tiny." Even if branch protection isn't yet enabled. The rule is the rule.
- **Never use `git add -A` or `git add .`.** Stage explicit paths.
- **Never `--amend` an existing commit** unless the user explicitly asked. If a hook fails, the commit didn't happen — make a NEW commit after fixing.
- **Never `--no-verify`** to bypass hooks. If a hook fails, fix the underlying issue.
- **Never skip Step 3's "Need decision" pause.** Auto-staging unrelated files is how secrets and personal config leak.
- **Never violate CLAUDE.md naming/copy policies in commits, PR titles, or PR bodies.** If the project constrains what names or wording can appear in user-visible text, those rules apply to PRs too.
- **Never skip verification** without an explicit user override. "It's just a one-liner" is not an override.
- **Never push directly to main**, including via force-push, including for "fixing" a previous bad push. If main is wrong, open a revert PR.
- **Never offer to babysit the PR after creation.** Print URL, stop.
