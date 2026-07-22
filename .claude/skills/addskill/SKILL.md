---
description: Use when the user asks to add, install, or create a repo-local skill that must work in future Claude Code and Codex sessions.
---

# Add skill — install a skill into this repo

Skills only appear in a Claude Code web session if they're committed to `<repo>/.claude/skills/<name>/`. Personal installs (`~/.claude/skills/`) and CLI-only plugins do NOT follow the user to the web. The fix is always: put the skill folder in the repo and commit it.

## Step 1: Confirm the skill source

Ask the user (or infer from `$ARGUMENTS`) where the skill content comes from:

- **A name + description** the user wants you to author from scratch
- **An existing local folder** (e.g. `~/.claude/skills/<name>/`) to copy in
- **A third-party skill** (e.g. `superpowers`) — these usually ship via a Claude Code plugin marketplace. Ask the user for the source URL/repo. Do NOT invent contents.

If the source is ambiguous or the user only gave a name you don't recognize, ask once before authoring.

## Step 2: Create the skill folder

Skill location is **always**:

```
/<repo-root>/.claude/skills/<skill-name>/SKILL.md
```

- Folder name is the skill name as it will appear after `/` (kebab-case, no spaces).
- The file MUST be named `SKILL.md` (capitalized exactly).
- Sub-files (helper scripts, references) are allowed in the same folder.

```
mkdir -p .claude/skills/<skill-name>
```

## Step 3: Author SKILL.md

Required structure — YAML frontmatter then markdown body:

````markdown
---
description: One-paragraph description. Lead with what the skill does, then the trigger phrases ("Use when the user says /<name>, asks to ..."). The harness uses this string to decide when to surface the skill, so trigger phrases matter.
---

# <Skill name> — short tagline

Brief intro: what the skill produces and when it runs.

## Step 1: ...
## Step 2: ...
## Step N: ...

## Anti-patterns

- Don't ...
- Don't ...
````

Rules of thumb:

- **Description is the routing signal.** Include concrete trigger phrases the user is likely to say. Mention the slash command form (`/<name>`) explicitly.
- **Be concise.** Existing skills in this repo (`pr`, `enhance-prompt`, `impartial-review`) are good length references — short numbered steps, anti-patterns at the end.
- **No emojis** unless the user asks.
- **Don't reference platform-specific tools** in the body (e.g. "use the Bash tool"). Say "run this command" instead. Skills should work across CLI and web.
- **`$ARGUMENTS`** is available inside the skill body — that's how the user passes input via `/skillname some text`.

## Step 4: Classify and generate the Codex adapter

Add the new skill exactly once to `.agents/CODEX-SKILL-COMPATIBILITY.md` as Native, Adapted, Capability-gated, Dangerous, or Claude-only. Keep the routing description at 240 characters or fewer and limited to trigger conditions; the full repo catalog must remain small enough for Codex's initial skills budget.

Run:

```bash
node .claude/scripts/sync-codex-skills.mjs --write
node .claude/scripts/test-codex-contract.mjs
```

This creates `.agents/skills/<name>/SKILL.md`, a thin Codex-native adapter that
delegates to the unchanged canonical Claude skill.

## Step 5: Verify the skill is wired up

After writing, sanity-check:

- File exists at `.claude/skills/<name>/SKILL.md`
- Frontmatter uses one `---` block at the top and passes the supported routing-structure checks
- `description` field is present and non-empty
- Declared skill name, folder, and slash command match
- Generated adapter exists at `.agents/skills/<name>/SKILL.md`
- Compatibility matrix contains the skill exactly once
- Codex contract check passes

You can grep existing skills for shape comparison:
```
ls .claude/skills/
head -5 .claude/skills/pr/SKILL.md
```

## Step 6: Commit and land on main

Skills only become visible in **future** sessions once they're committed **and merged into `main`** — both web sandboxes and the local CLI read from `main`, not from a feature branch. Committing to the branch alone is not enough.

Stage only the new skill file(s) — never `git add -A`:

```
git add .claude/skills/<skill-name>/SKILL.md .agents/skills/<skill-name>/SKILL.md .agents/CODEX-SKILL-COMPATIBILITY.md
```

Then invoke the `/merge` skill to commit, push, open the PR, and merge to `main` (with conflict handling and the local main-checkout pull). `/merge` owns the land-on-main logic — don't duplicate it here.

If the user prefers a one-shot land instead of session-wide auto-merge, do the equivalent manually: commit, push, `gh pr merge`, then `git pull` in the local `main` checkout so the file the user actually runs is updated.

Tell the user the skill appears in the **next** session — the available-skills list loads at session start, so the current session won't see it until reload.

## Anti-patterns

- Don't put skills in `~/.claude/skills/` — they won't follow to the web sandbox.
- Don't put skills in the repo root or a random subfolder — only `.claude/skills/<name>/SKILL.md` is loaded.
- Don't fabricate the contents of a third-party skill (`superpowers`, etc.) you don't have the source for. Ask the user for the source.
- Don't skip adapter generation — Codex discovers repo skills from `.agents/skills/`.
- Don't leave the new skill unclassified or let its routing description push the Codex catalog over budget.
- Don't stop at pushing the branch. An unmerged branch skill is invisible — sessions read from `main`. Land it via `/merge`.
- Don't use `git add -A` or `git add .` — stage only the new skill file(s).
- Don't claim the skill is "now available" in the current session — it isn't until the session reloads.
