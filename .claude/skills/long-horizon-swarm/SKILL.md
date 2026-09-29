---
name: long-horizon-swarm
description: 'Peer swarm on a long-horizon run: Opus, Sol and Astra contribute ideas, critique and code throughout; disagreements are flagged to the user, never blocking. Use on /long-horizon-swarm or to bring Sol/Astra into long-horizon.'
---

# long-horizon-swarm: peers working a long-horizon run

This is the `long-horizon` contract plus a swarm. Read `.claude/skills/long-horizon/SKILL.md`
(or `long-horizon-workflows` when that engine is in use) and follow it: state file, rounds,
baseline, fresh auditor, stagnation and completion all apply unchanged except where this file
says otherwise. This file adds who contributes and how their work flows in. Discussing or
editing this skill does not activate it.

Copy, launch, poll, stop and reply mechanics for peers live in
[references/codex-peer.md](references/codex-peer.md).

## Attach or start

Look for `.tmp/long-horizon/*/state.md` in this checkout before anything else.

- Before attaching, check whether another session still manages the run. If this session did
  not run it, and the state file changed in the last 30 minutes, its `Workers:` line lists a
  worker still running, or its `swarm.md` has an S entry with `status: running`, ask the
  user: take over (they stop the other session first) or stay out. Two managers writing one
  state file corrupt it.
- One run matches the task: attach to it. Add `Swarm: on` under its Contract, create
  `swarm.md` beside it, and continue from its current phase. Never start a second state file
  for the same task.
- Several could match: ask which one.
- None: start a long-horizon run per its contract, with `Swarm: on` from round one.
- A round in phase `executing`: only that executor writes the workspace until it finishes.
  Peers work in copies of the round's Baseline snapshot, or wait for Audit if the round has
  none.

At run start, tell the user the path of `swarm.md`.

On attach or resume, before any new dispatch, reconcile every S entry with
`status: running`: collect its output if the process has exited, or stop it. If its status
cannot be established, pause new dispatch and record the recovery needed, as long-horizon
does for workers.

Only this checkout's `.tmp` is visible. A run in another worktree or clone is invisible
unless the user points at it or a `session-hub` hub records it.

## Peers

| Peer | Route | Default |
|---|---|---|
| Opus | `Agent` tool, fresh context | Opus, the latest Fable, or higher; never `sonnet` or `haiku` |
| Sol | `codex exec`, one new process per contribution | the Sol id pinned in `.claude/skills/codex-review/SKILL.md`, `high` effort |
| Astra | `codex exec`, one new process per contribution | `gpt-6-astra`, `medium` effort |

Preflight once: Codex needs an accepted route (a ChatGPT login or a `model_provider` gateway,
as the `codex-review` preflight defines), and flags come from local `codex exec --help`. No
accepted route: tell the user, run with Opus peers only, and label the run as single-vendor. Never switch Codex to an
API key or paid credits, and never bypass its approvals or sandbox. Honor explicit user
model choices. A login check does not prove a model is available: when Codex rejects a
model, mark that entry failed and tell the user. Never drop `-m` or substitute another model.

This session is the **host**. It exists because only it can spawn agents, launch
`codex exec`, keep the state file across compaction, and talk to the user. It dispatches and
keeps the record; its ideas carry no more weight than any peer's. Long-horizon still forbids
it from executing a round's step itself.

The round executor may be any peer: a fresh Opus agent, or Codex run as
`codex exec -C <workspace root> -s workspace-write` with the executor brief on stdin. Record
its agent id or Codex session id under `Workers:`. The workspace-write sandbox can write
anywhere under the root, so Write scope rests on the brief and the auditor's integrity
verdict, as with any executor.

## Touchpoints

The host asks the swarm at three default points in each round:

1. **Draft Plan review.** Write the Current round block, phase `planned`, as long-horizon's
   Plan says, and treat its Step and Done-check as a draft. Peers review the draft; the
   `done-check` lens fits. The host revises the block, then takes the Baseline and writes
   both briefs. From there the step and done-check are frozen, and a later finding goes to
   the next round's Plan.
2. **Post-audit cross-model review.** After Audit, a peer from a different model family than
   the auditor reviews the round's delta. Its findings go to the next Plan; they never change
   the audit verdict.
3. **Alternatives after a failed audit.** Peers propose different approaches before the next
   brief. At a long-horizon stagnation trigger this pass replaces the one cross-vendor
   consult; Spend limits it, not the one-consult rule. Check each proposal against the
   current contract before it rewrites Remaining, as long-horizon requires.

The host may skip a touchpoint and logs the reason under Skipped touchpoints. Peers may
contribute at any other phase too.

## Contributing

Any peer may contribute anything useful at any phase: ideas, critique of a plan or diff,
alternative approaches, code, tests, a reproduction, a counterexample. Beyond the
touchpoints, the host decides who to ask and when, by where a second model is likely to see
something the first did not.

- **Code** is written in the contributor's own copy under
  `.tmp/long-horizon/<slug>/swarm/<entry-id>/ws/`. Outside a round's execution the copy comes
  from the live workspace, including uncommitted and untracked changes. While a round is
  `executing` or `awaiting-audit` it comes from that round's Baseline snapshot, never the
  live tree, so no peer builds on the executor's half-finished edits. With Swarm on, Plan
  saves that snapshot as `swarm/baseline-r<N>.tar` right after the Baseline manifest; with
  the Workflow engine, which takes the Baseline inside its script, right before the
  workflow call, with no writer active in between. Never
  use `isolation: "worktree"`: a worktree starts from HEAD, and rounds do not commit, so it
  would miss earlier rounds' verified work. An Opus peer is told to write only inside its
  copy; a Codex peer gets `-C <copy>`. Swarm copies sit outside Write scope.
- **Integration** stays with the round's executor, the only agent that writes the workspace
  between Baseline and Audit. Its brief names the chosen contribution's path, and it applies
  that result within Write scope. A second writer would make the auditor's delta
  unattributable and the round `suspect`. Code the swarm produces after the executor has
  finished goes into the next round.
- **Review and reply.** The reviewer gets the artifact (diff, plan, file paths), not the
  author's reasoning: critique aimed at the artifact finds more than critique aimed at the
  justification. The critique then goes back to the author, by `codex exec resume <id>` for
  a Codex author or `SendMessage` for an Opus author, who concedes or refutes it with
  evidence. The critic gets one counter. A point still split after that opens a D entry.
  A round executor always replies read-only, from a copy, never from the workspace root: a
  change it concedes is the next round's work, so nothing edits an audited round.
- **Brief** each peer with the contract, dead ends, its lens, and the artifact paths it
  needs. A Codex peer shares no memory with this session, so the brief is all it knows. A
  copy has no `.git`; a peer that needs a diff gets it as a file named in the brief.

A pass is one dispatch per chosen lens. The cycle continues only while a pass returns a
finding that would change the artifact, or new code; anything else ends it. There is no
fixed cap. If peers start agreeing without citing new evidence, run one more pass with a
fresh peer on the bare artifact; if that adds nothing, stop.

## Lenses

Copies of one model given one brief return near-identical answers. Give every peer a lens: a
single question it answers about the artifact. Vary model and lens together (Sol on
performance, Astra on the decomposition, Opus on quality), pick only lenses that fit the
task, and rotate them across rounds so each lens meets different models. The number of
peers in a pass equals the number of lenses chosen, not a per-model count.

A lens is one of:

- **A swarm lens** in `lenses/` beside this file, written for this workflow. Each file states
  the question, what to inspect, what counts as a finding, and the return format.
  [done-check.md](lenses/done-check.md) is the first; add one when a run shows an angle the
  pool lacks.
- **An existing skill's criteria**, such as `wow-loop` (quality bar), `perf-loop` (measured
  cost), `dare` (problem and decomposition) or `impartial-review` (defects). The peer reads
  that skill's standard from its `SKILL.md` and applies it in one pass. It does not run the
  skill's loop: those need fresh critics and repeated rounds a peer may not be able to
  spawn, and a loop nested in every peer multiplies cost.
- **`security-review`** (trust boundaries) is a built-in Claude Code command, not a skill
  file. Its brief states the standard itself: untrusted input reaching a shell, SQL, file
  paths or HTML; missing authentication or authorization checks; secrets in code or logs;
  data crossing a trust boundary it should not.

A code-writing peer also gets a constraint lens: smallest diff, a different approach from
the current one, or test-first. Code peers in one pass get different constraints.

When a lens finds something that needs the full loop (performance far off target, visual
work well below the bar), the host makes that loop a round step, the way long-horizon runs
`arena` inside a round.

Brief each peer with its lens file or skill path; a Codex peer reads it from the absolute
path.

## The audit stays cold

The auditor brief is written at Plan, after the draft review and before the executor
exists, from the contract and the frozen Current round block, and dispatched byte for byte
as long-horizon requires. It never contains `swarm.md`, a peer's critique, or a peer's
opinion of the work; a done-check a peer helped tighten enters only as the host's frozen
check. A step spec built from swarm findings reaches the auditor as the list of changes
only, stripped of sources and evidence. Every auditor brief, per round and final, excludes
`.tmp/long-horizon/<slug>/swarm/` and `.tmp/long-horizon/<slug>/swarm.md` by path, from both
its delta and its reading. The auditor never receives the executor's or any peer's report.
Forbid the auditor billed Codex runs, not the CLI: it needs local help output to check
flags.

Contract Acceptance must be checkable without `swarm.md`. Swarm agreement is not evidence;
only the auditor's own inspection moves work into Verified progress.

## Log

`.tmp/long-horizon/<slug>/swarm.md`, kept by the host. Write each S entry at dispatch with
`status: running`, and update it when the peer returns or fails:

```markdown
# Contributions
- S<n> round <N> | <peer> <model>/<effort> | lens: <name> | <agent id, or codex session id> | idea / critique / code / alternative / reply / executor
  status: running / returned / failed   output: <entry directory>
  target: <plan, step, file or entry id>   artifact: <path>
  outcome: taken / partly taken / left, with the reason

# Skipped touchpoints
- round <N> | <touchpoint> | <reason>

# Disagreements
- D<n> round <N> | <topic> | status: open / decided by user
  positions: <party>: <claim + evidence>; <party>: <claim + evidence>
  following: <which position the work follows now, and why>
  user decision: <verbatim or pointer, once given>

# Codex runs
<count per model, updated every entry>
```

## Disagreements

Flag, never block. A D entry opens when a point stays split after review and reply, or when
the host leaves a contribution that no peer refuted with evidence; the host is then a party
with its own reason. Continue on the position with the stronger evidence (tie: the one
cheaper to reverse), and keep the other side's artifact.

When a D entry opens, post one line in chat: `D<n> opened: <topic>; following <position>.`
Report open disagreements to the user in every progress update and in the final handover,
with each position's evidence and what changes if they pick the other side.

A user decision that changes Goal or Acceptance is a long-horizon amendment. A disagreement
that touches an irreversible action (publish, deploy, delete, migrate) stops that action
until the user decides; the rest of the work continues.

## Spend

Every `codex exec` run, resumes included, bills the user's subscription. Log each under
Codex runs and state the counts in every progress update. Explicit user budgets for runs,
rounds or time are binding; checkpoint before exceeding one.

## Handover

Long-horizon's completion rules apply. Add: the Codex run counts, every open disagreement
with both positions, and which peer's work each Verified progress claim came from. The
kernel's `codex-review` before merge still applies; swarm review does not replace it.
