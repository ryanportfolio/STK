---
description: Use once in a spawned starter repo when the user asks to initialize or configure it, or when FILL IN markers remain; never auto-run in a claude-starter template checkout.
---

# init-project — configure a freshly spawned starter project

The starter template ships with placeholder sections that MUST be configured before the kernel rules are trustworthy. This skill is the guided path: detect what's detectable, ask only what isn't, write it down, and verify the setup.

Run it ONCE per spawned project. If `grep -c "FILL IN" CLAUDE.md` returns 0, the project is already configured — say so and exit.

## Step 0: Confirm this is a spawned project

Check `git remote get-url origin` before changing files. If it points to `ryanportfolio/AI-Firmware` (or the old name `ryanportfolio/claude-starter`), this is the canonical template: do not initialize, delete template assets, prune skills, or rewrite its generic defaults. Explain that the markers are intentional and stop.

If the repo still has the canonical `# Agent firmware` README plus both `bootstrap/` and `.claude-plugin/`, ask whether this is a template checkout or maintenance fork versus a spawned project, regardless of its origin name. Do not delete those assets until the user confirms it is spawned. Their answer is authoritative; repository names alone are ambiguous.

## Step 1: Detect the stack (no questions yet)

Look before asking. Gather from the repo itself:

- `ls` the root — what scaffold exists? (`package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `*.csproj`, nothing yet?)
- If `package.json` exists: read `scripts` (build/dev/test/check commands), `dependencies` (framework, router, query lib, ORM), `devDependencies` (bundler, test runner).
- Equivalent manifest reads for other ecosystems.
- `git remote get-url origin` — the repo path for PR links.

If the repo is EMPTY (fresh spawn, no app code yet), either configure honest undecided defaults now or defer initialization without changing files until after scaffolding. Do not partially initialize and then promise a second run; configured projects intentionally exit on rerun.

## Step 2: Ask what detection can't answer

Plain chat, numbered (popup tools are banned). Only ask what's actually unknown — skip questions the scaffold already answered:

1. **Profile:** which best describes this project — **web-app** (has UI), **backend/CLI/library** (code, no UI), **data/notebooks**, or **writing/docs**? Lead with the guess detection supports ("scaffold says web-app — confirm?"), only truly ask when the repo is empty.
2. **Deploy target:** where will this run? (host/platform, database, where secrets live)
3. **Sandbox capabilities:** can sessions in this environment run installs, builds, type-checks, tests meaningfully? Can the user reach a dev server the session starts? Is there a browser?
4. **Authoritative verification:** what's the final word that a change works — local test suite, CI, a deploy log?
5. **Hard lines:** anything that must ALWAYS go through the user (installs, migrations, deploys, destructive ops)?

If the user doesn't know yet (brand-new project), write the honest default: "not yet decided — ask before installs/migrations/deploys" and move on. Don't stall setup on undecided infrastructure.

## Step 3: Fill in CLAUDE.md

- Replace the **verification** FILL IN section with the real answer from Step 2 (what this sandbox can/can't verify, what the authoritative signal is, what to flag-as-risk instead of claim).
- Replace the **Environment & Deploy Target** FILL IN section with the deploy target, install policy, migration policy, and hard lines.
- Delete the `STARTER TEMPLATE NOTE` comment block and every `FILL IN` comment.
- Delete `.claude-plugin/`, `bootstrap/`, and `.github/workflows/validate-template.yml` — they maintain or distribute the template and must not run in spawned projects. Remove empty `.github/workflows/` and `.github/` directories afterward.
- Keep the section structure — future sessions navigate by those headings.

## Step 4: Seed the reference files

- `commands.md` — the detected scripts/commands, verbatim and runnable.
- `tech-stack.md` — detected framework + any non-default picks the user names (and WHY, if they say).
- `deployment.md` — the deploy answers from Step 2.
- Leave `pitfalls.md` / `architecture.md` / `secrets.md` skeletal — they fill organically via `/recall save`.

## Step 5: Apply the profile — prune skills + tune best-practices

**Skill pruning.** The profile from Step 2 disables skills that will never fire in this project, via `skillOverrides` in committed `.claude/settings.json` (`"off"` = hidden from the picker AND the per-turn skills list; re-enable any time by removing the key). Project skills use the bare directory name as the key:

| Profile | Disable (`"off"`) |
|---|---|
| web-app | — (full set) |
| backend / CLI / library | `forge-repo-ui-skill`, `lab` |
| data / notebooks | `forge-repo-ui-skill`, `lab` |
| writing / docs | `forge-repo-ui-skill`, `lab`, `test-driven-development`, `subagent-driven-development` |

The table is a floor, not a ceiling — offer obvious extras ("no frontend planned, also drop `humanizer`? it's for prose deliverables"). Each `off` saves its description from every turn (`bash .claude/scripts/context-weight.sh` shows per-skill weight); takes effect next session.

**Best-practices catalog.** Open `.claude/skills/applying-best-practices/SKILL.md` — it ships as a generic web/TS baseline:

- Non-web or non-JS project → cut the React/bundle/query-cache sections entirely; keep the discipline section and the Async/IO + JS-perf generics that still apply (or their ecosystem equivalents).
- Web project → trim to the actual stack (e.g. drop query-cache rules if there's no query library yet; note the framework's idioms).
- Empty repo → leave as-is, note in the file that it's untuned.

## Step 6: README

If `README.md` is the spawn stub (`# <name>` only) or still starts with the canonical `# Agent firmware` heading, ask the user for a one-line project description and replace it minimally: name, one-liner, how to run (from `commands.md`). Don't write aspirational docs for code that doesn't exist.

## Step 6b: Sync Codex skill adapters

Run `node .claude/scripts/sync-codex-skills.mjs --write` after applying the
profile. The generator mirrors active skill metadata into thin
`.agents/skills/` adapters while leaving the canonical Claude skills unchanged.
Disabled skills are omitted according to `skillOverrides`.

## Step 7: Wire the starter remote

```
git remote get-url starter || git remote add starter https://github.com/ryanportfolio/AI-Firmware.git
```

This pre-wires `/sync-starter` and lets the session-start hook surface template drift ("starter differs on N files"). Remotes are local git config, not committed — mention that a new clone on another machine needs this line re-run (or the hook's URL-fallback fetch covers it when credentials allow).

## Step 8: Finish and optionally ship

Verify that no `FILL IN` markers or template-only paths from Step 3 remain. In Claude Code, follow the project's configured Git rule. In Codex, initialization alone does not authorize commit, push, PR, or merge: perform only the Git actions the user explicitly requested, otherwise leave the verified setup uncommitted and report it.

## Anti-patterns

- Don't invent deploy facts or sandbox capabilities — wrong kernel rules are worse than FILL IN markers. Ask, or write the honest "undecided" default.
- Don't run installs just to probe the stack — read manifests instead.
- Don't leave any `FILL IN` marker behind. `grep -n "FILL IN" CLAUDE.md` must return nothing at the end.
- Don't initialize a canonical or forked `AI-Firmware` (formerly `claude-starter`) template checkout; its markers and template assets are intentional.
- Don't leave template CI or distribution files in a spawned project.
- Don't pad the reference files with boilerplate prose — they're lookup tables for future sessions, not documentation theater.
- Don't copy full skills into `.agents/skills/` — generated adapters keep
  `.claude/skills/` as the single source of truth for both runtimes.
- Don't run this twice. Configured projects evolve via `/recall save` and direct CLAUDE.md edits, not re-initialization.
