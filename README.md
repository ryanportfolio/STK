# Agent firmware

A portable operating layer for AI coding agents.

Most agent setups start as a prompt in one repo. Then one repo learns a better
debug loop. Another learns a deployment gotcha. A third learns which rules cost
too much to keep loaded. The lessons stay where they happened.

This starter makes those lessons travel.

It gives each project a small boot layer: standing rules, on-demand skills,
committed memory, session hooks, and sync scripts. Claude Code gets the full
system. Codex gets a safe boundary in `AGENTS.md` and can use the same playbooks
manually.

This began as `claude-starter`. The name still fits the repo. The frame is
larger now: agent firmware, small enough to stay under the work, strong enough
to shape how the agent boots, remembers, checks itself, and ships.

## the problem

A coding agent needs habits. It also needs restraint.

If everything goes into the prompt, every turn gets heavier. If nothing is
stored, every project repeats the same mistakes. If Claude-specific automation
leaks into Codex, useful defaults can become unsafe commands.

Agent firmware splits the work:

- The kernel stays small and always loaded.
- Skills hold the long playbooks until the task calls for them.
- Reference files keep project memory out of the chat transcript.
- Hooks handle cheap startup checks.
- Sync scripts move reusable improvements between projects.
- `context-weight.sh` measures the always-loaded token tax.

One repo learns. The next repo should not start dumb.

## runtime boundary

| Runtime | Entry point | Use it for |
|---|---|---|
| Claude Code | `CLAUDE.md`, `.claude/settings.json`, `.claude/hooks/`, `.claude/skills/` | The full template: kernel rules, slash skills, project memory, session hook, plugin path, and Claude-specific workflow rules. |
| Codex | `AGENTS.md`, `.agents/skills/` | A safe compatibility layer plus native skill discovery, backed by the unchanged canonical Claude workflows. |

Codex discovers thin adapters under `.agents/skills/`. Each adapter delegates to
the matching canonical `.claude/skills/` workflow, so both runtimes use one
source of truth. `AGENTS.md` defines the safety boundary. Codex does not run
Claude SessionStart hooks. The starter intentionally has no project Codex hook:
`AGENTS.md` already loads natively, while command hooks add a separate trust and
platform-failure surface.

## use it

### full Claude Code template

Use this when you want the whole system: kernel, hooks, memory, skills, and
starter sync.

- GitHub UI: **Use this template -> Create a new repository**, clone it, open it
  in Claude Code, then run `/init-project`.
- Windows one-click: double-click `bootstrap/New-ClaudeProject.cmd`.
- Windows visual launcher: download and extract
  [`New-ClaudeProject-UI.zip`](https://github.com/ryanportfolio/AI-Firmware/releases/latest/download/New-ClaudeProject-UI.zip),
  then double-click `New-ClaudeProject-UI.cmd`. Keep the extracted launcher,
  PowerShell module, and `template/` folder together; the bundled snapshot makes
  local-only creation work without GitHub access.
- Windows CLI: `.\bootstrap\new-claude-project.ps1 -Name my-app -Dest C:\code`

`/init-project` detects the stack, asks a short Q&A, fills the verification and
deploy sections, seeds reference files, prunes irrelevant skills, removes
spawn-only template files, and commits the setup.

### Codex-compatible use

Use this when opening the starter or a spawned project in Codex.

1. Open the repo in Codex.
2. Let Codex read `AGENTS.md` as the authoritative Codex instruction boundary.
3. Use `.claude/reference/` for shared project memory.
4. Let Codex discover the generated repo skills under `.agents/skills/`.
5. Do not run Claude hooks or inherit Claude auto-commit/auto-merge rules unless
   the user explicitly asks in the current Codex session.

For a fresh project, ask Codex to initialize the starter or select the
`init-project` skill. Its adapter delegates to the same canonical workflow
Claude Code uses.

### Claude plugin skills only

Use this when you want the Claude skills in an existing project without cloning
the template:

```text
/plugin marketplace add ryanportfolio/AI-Firmware
/plugin install claude-starter@claude-starter
```

This path requires the repository to be public. Plugin skills are namespaced,
for example `/claude-starter:recall`. Projects spawned from the template ship the
same skills without the namespace.

## what's inside

| Path | Purpose |
|---|---|
| `CLAUDE.md` | Claude Code kernel rules loaded every turn: verification, git workflow, subagent discipline, and context restraint. Two placeholder sections are filled per project by `/init-project`. |
| `AGENTS.md` | Codex boundary. Inherits project facts without inheriting Claude-only hooks or automatic git behavior. |
| `.claude/skills/` | Canonical Markdown playbooks used by Claude Code and Codex. |
| `.agents/skills/` | Generated Codex-native adapters; metadata only, no duplicated workflow bodies. |
| `.claude/reference/` | Durable project memory: secrets, architecture, pitfalls, commands, tech stack, and deployment notes. |
| `.claude/hooks/session-start.sh` | Claude Code SessionStart hook for drift checks, overlap warnings, and Claude-specific defaults. |
| `.claude/scripts/context-weight.sh` | Prints always-loaded context weight, including a per-skill breakdown. |
| `.claude/settings.json` | Claude Code hook wiring plus a Bash permission allowlist. |
| `.claude-plugin/` | Claude plugin and marketplace manifests. Template-only for spawned projects. |
| `bootstrap/` | Project creation, fork retargeting, and machine setup scripts. |

## work loop

Use the starter while you work, then feed the useful parts back into it.

- `/recall save <text>` records a project gotcha in the right reference file.
- `/learning` captures what mattered after a multi-attempt debug arc.
- `/sync-starter` moves a generic improvement back to the starter or pulls a
  starter improvement into a spawned project.
- `bash .claude/scripts/context-weight.sh` shows what the always-loaded layer
  costs per turn.
- `/optimize-context` is the playbook for cutting context that no longer earns
  its place.

The loop is simple: capture the lesson, keep it out of the kernel unless it must
be loaded every turn, and make useful generic work travel.

## skill set

See `.claude/skills/PROVENANCE.md` for forked skill provenance, licenses, and
local deltas.

- **Lifecycle:** `init-project`, `sync-starter`, `addskill`, `optimize-context`,
  `forge-repo-ui-skill`.
- **Workflow:** `recall`, `learning`, `safe-ship`, `pr`, `merge`, `advocate`,
  `caveman`, `enhance-prompt`, `handoff-audit`, `why`, `lab`, `conflict`.
- **Discipline:** `brainstorming`, `writing-plans`, `executing-plans`,
  `systematic-debugging`, `test-driven-development`,
  `verification-before-completion`, `impartial-review`,
  `subagent-driven-development`, `dispatching-parallel-agents`,
  `using-git-worktrees`, `using-superpowers`, `writing-skills`,
  `applying-best-practices`, `finishing-a-development-branch`.
- **Craft:** `humanizer`.

## safety model

The starter is supposed to travel, so defaults must stay safe outside one
person's machine.

- Runtime-specific rules stay runtime-specific. Claude hooks and Claude popup
  constraints do not become Codex standing orders.
- Starter files must not ship private checkout paths, maintainer-only workflow
  mandates, secrets, tokens, or local-machine assumptions.
- Git automation stages explicit paths and protects against direct pushes to
  `main`, force-pushes, secret files, and unverified completion claims.
- Installs, migrations, deploys, deletes, branch merges, and edits outside the
  current workspace require explicit user authority for the current session.
- Verification claims must name the check that actually ran. If the real signal
  is CI, deploy logs, or the user's machine, say that instead of pretending.

## forking this template

One command retargets functional upstream references to your fork:

```bash
bash bootstrap/retarget-fork.sh <you>/<your-fork>
```

Review the diff and commit. LICENSE attribution is intentionally left untouched.

## dotfiles for Claude

Machine-level `~/.claude` files do not travel with any repo. Keep your copies in
`bootstrap/machine/home-claude/` in your fork, then on a new machine run:

```powershell
.\bootstrap\setup-machine.ps1
```

The script copies missing files only. `-Force` overwrites. `-DryRun` previews.

## requirements

- Claude Code for the full template and plugin workflow.
- Codex uses `AGENTS.md` and native `.agents/skills/` adapters. It does not run
  Claude SessionStart hooks or need a project hook to load repository guidance.
- `gh` CLI is optional for the Windows project creator. Without it, the script
  falls back to a local copy plus printed GitHub steps.
- Bootstrap is Windows-first PowerShell. The Claude session hook is Bash and is
  validated under Ubuntu CI.

## provenance and license

MIT. See `LICENSE`.

Several skills are forked from upstream work, notably Jesse Vincent's
`superpowers` skills (MIT). `.claude/skills/PROVENANCE.md` tracks forked origins,
licenses, and local changes. Per-skill LICENSE and NOTICE files ship in
third-party skill folders when required.
