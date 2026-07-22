# Skill Provenance

Where each skill came from, its license, and what this repo changed. Not loaded
into context; it is reference for maintainers and public users.

**Maintenance rule:** when you materially change a forked skill, update its
"Our deltas" cell here. When adding a third-party skill, add a row and keep its
LICENSE/NOTICE files in the skill folder.

## Forked / third-party

| Skill | Upstream | License | Our deltas |
|---|---|---|---|
| `brainstorming` | [obra/superpowers](https://github.com/obra/superpowers) (Jesse Vincent) | MIT (in folder) | Two-lane scope calibration, authorization-safe artifacts, optional visual companion |
| `dispatching-parallel-agents` | obra/superpowers | MIT (in folder) | Stock |
| `executing-plans` | obra/superpowers | MIT (in folder) | Stock |
| `finishing-a-development-branch` | obra/superpowers | MIT (in folder) | Stock |
| `subagent-driven-development` | obra/superpowers | MIT (in folder) | Parallel-dispatch mode; implementer/reviewer prompt files |
| `systematic-debugging` | obra/superpowers | MIT (in folder) | Stock |
| `test-driven-development` | obra/superpowers | MIT (in folder) | Stock |
| `using-git-worktrees` | obra/superpowers | MIT (in folder) | Stock |
| `using-superpowers` | obra/superpowers | MIT (in folder) | Stock |
| `verification-before-completion` | obra/superpowers | MIT (in folder) | Stock |
| `writing-plans` | obra/superpowers | MIT (in folder) | Stock |
| `writing-skills` | obra/superpowers | MIT (in folder) | Stock |
| `caveman` | Community token-compression pattern (viral skill, author attribution unclear) | Reimplemented here | Intensity tiers (lite/full/ultra), output budget, auto-clarity carve-outs, persistence rules |
| `humanizer` | Community "humanizer" de-AI-writing pattern | Reimplemented here | patterns.md tell catalog; voice-matching; review-only mode |

## Homegrown (this repo)

`addskill`, `applying-best-practices`, `conflict`, `enhance-prompt`,
`forge-repo-ui-skill`, `handoff-audit`, `impartial-review`, `init-project`, `lab`, `learning`,
`merge`, `optimize-context`, `pr`, `recall`, `safe-ship`, `sync-starter`,
`why`.

Homegrown skills are MIT, same as the repo (see the root `LICENSE`).

`forge-repo-ui-skill` is an original synthesis workflow. It researches linked
third-party sources as untrusted inputs but does not vendor their skill text,
scripts, datasets, licenses, or configuration.
