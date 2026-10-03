---
description: 'Long-horizon rounds run through the Workflow tool: fresh executor, inspector, and judges per round with schema verdicts and a run journal. Use on /long-horizon-workflows or to run a big task in Workflow-audited rounds. Claude Code only.'
disable-model-invocation: true
---

# long-horizon-workflows: audited rounds on the Workflow engine

Manager, Executor, Auditor. You (this ctx) = Manager: hold goal, keep state file true,
delegate every round. Executors/auditors = fresh subagents; fresh ctx per round separates
impl from independent evidence. Discussing/editing this skill ≠ activating it.

`long-horizon` contract, one change: `Workflow` tool exposed → each round's Baseline,
Execute, Audit run as one script: ctx boundary enforced by construction, verdicts forced into
enums, every agent's exact input/output journaled. Claude Code only; Codex sessions use
`long-horizon`. Tool absent in Claude session → same steps as fresh `Agent` calls, contract
unchanged.

## State file

`.tmp/long-horizon/<task-slug>/state.md` (gitignored scratch), created before round one:

```markdown
# Contract  (original retained; explicit user changes recorded as amendments)
Goal: <one paragraph>
Acceptance: <the checks that prove it done, as a numbered list>
Version: <current contract version>

# Amendments
- <version, explicit user instruction, changed checks, affected steps and claims>

# Verified progress
- <claim>: contract version <version>; evidence: <file/command/output the auditor saw>

# Remaining
1. <step sized for one fresh context>; depends on: <step numbers or none>; writes: <paths>

# Current rounds  (one block per active round; scope and checks fixed at Plan, except for explicit user amendments)
Round: <N>   Batch: <B>   Phase: planned | executing | awaiting-audit | audited
Contract version: <version used for this round>
Engine: workflow | agent
Workspace: <absolute root the executor writes; any other workspace the round reads, marked read-only>
Workers: <workflow runId + transcript dir, or agent IDs; role; last observed status>
Step: <the one Remaining step this round works>
Done-check: <commands, cwd, and the expected result; the auditor runs them itself>
Write scope: <paths the executor may change; test and gate definitions only if the step is about them>
Judges: <total verdict count including the inspector, and the one-line reason for that number>
Baseline: <manifest path> + <git ref>  (see Plan; the auditor diffs against this, never HEAD)
Executor brief: <path>  (written at Plan)
Auditor brief: <path>  (written at Plan, dispatched byte for byte)
Residue: <paths a failed earlier round left changed, and whether they were reverted or kept>

# Dead ends  (approaches that failed audit; do not retry without new evidence)
- <approach>: <why it failed, one line>

# Method notes  (rules later rounds must follow, such as how to measure)
- <rule>: source round <N>; unconfirmed / confirmed in round <M> / dropped

# Audit log
- round N (batch B): <step>: <status>/<integrity>/<contract>, <one-line evidence>, <runId or agent IDs>
```

Only audit-passed results enter **Verified progress**. Resume from existing state file;
reconcile w/ real workspace + latest user instructions. Before resuming run this session
didn't start, check if another session still manages it: state file changed in last 30 minutes,
or `Workers:` lists still-running worker/workflow run → ask user: take over (they stop other
session first) or stay out. Two Managers on one state file corrupt it.

Preserve original contract. Explicit user changes → versioned amendments; reassess affected
steps, invalidate affected claims before using them as prerequisites. Never weaken acceptance
just to pass a round. Amendment mid-round → reconcile its workers before further execution,
preserve baseline + old briefs, write new versioned brief from amended contract + raw
artifacts, no executor assessments. Only audit vs current contract accepts affected work.

Dead ends = memory too. Unrecorded failed approach gets re-proposed rounds later; re-walking
costs full round.

## Context boundary

Fresh = no Manager conversation history. State file, workspace, standalone brief carry every
needed fact. Executor gets only its bounded brief; auditor only its prewritten brief. Auditor
never gets executor's turns or report.

Real leak ≠ executor's file list (auditor recovers it from workspace). It's executor's
narrative ("works, checked X, Y was out of scope"): Manager has read it by auditor-brief time,
can paraphrase unnoticed. So auditor brief pre-registered: written at Plan, before executor
exists, from current contract version's acceptance checks, Current round block, workspace
root; saved to block's recorded path; dispatched unchanged. Manager wants to add to brief after Execute → that's Plan defect, not brief defect → next round (exception: explicit user amendment,
handled above). Provenance checkable; "do not paraphrase" isn't.

Inspect exposed tools + capacity before dispatch. Auditor: fresh ctx; never
`subagent_type: fork` or any option inheriting Manager history. No fresh independent ctx →
record gap; inherited ctx or self-review can't establish audit gate. Continue useful
authorized work not depending on it. Count Manager + other active workers vs capacity; short
→ smaller batches or sequential fresh rounds OK.

Workspace = executor's for round duration. Anyone else's edits between Baseline and Audit →
attribution impossible; auditor reports integrity `suspect`, no guessing whose change.

## Parallel rounds

Max safe concurrency. Each Plan: every ready step able to run beside others → own round, all
dispatched as one batch. Ready step waits for later batch only if rule below forbids; note
rule in its Remaining entry. Sequential run = batches of one.

Ready = all dependencies in Verified progress. Ready steps share batch only if:

- write scopes don't overlap; neither reads path other writes;
- no two executions/done-checks contend for one resource: port, dev server, browser profile,
  DB, GPU, or timing/perf measurement parallel load would skew;
- capacity covers them, counting Manager + every agent of every running round, judges incl.;
- every Write scope path inside workspace root. Step writing outside → runs alone, in main
  workspace: separate checkout can't isolate outside path, edits reach main env whatever
  audit says.
  In-place round skips batch integration steps 1-4 (nothing to copy; its edits already differ from Baseline by design) → normal Integrate on its own audit.

Each batch round keeps whole contract: own Current round block, workspace, Baseline, briefs,
executor, audit. Parallel executors never share workspace: one writer's edits land in other's
manifest diff → both audits `suspect`. Build each round's workspace from main at Plan:
`git stash create` in main workspace (empty output: use HEAD), `git worktree add --detach <path> <sha>`,
then copy in all main's untracked files + any ignored artifacts step needs (deps, build output).
Never worktree at HEAD: drops earlier rounds' uncommitted verified work. Never plain copy of
Git checkout: copied `.git` file still points at original's index + HEAD. Plain copies only
for non-Git workspaces. Baseline taken inside round's own workspace. Before checkout, record main's hashes for round's Write scope paths (`manifest.mjs` on main w/ those paths): conflict reference for integration. Round Baseline stays executor-attribution reference; checkout filters (e.g. `core.autocrlf`) can make round bytes differ from main's.

Workflow engine: one Workflow call per round, own `args` (`workspace` = round's workspace);
all batch calls in one msg. Agent engine: all executors in one msg; each auditor once its
executor stops. Integrate after whole batch audited:

1. Each passed round: apply manifest diff to main workspace verbatim: copy each added/modified path from round workspace, delete each deleted path. Main file not matching main's pre-checkout hashes (or present where none recorded) = conflict → step back to Remaining for later batch; not Dead end.
2. Failed round's delta stays out of main. Save as patch under `evidence/round-<N>/` (recovery brief may cite); record in Residue as not applied.
3. Before removing any round workspace: stop every process its round started (per `processes.log`), confirm gone; else old server answers step-4 check or holds its port. Step 4 starts any server it needs from main. Then copy every cited evidence file living in it (done-check output, ignored artifacts) into `evidence/round-<N>/`. Then unlink dep links in it (`rmdir` on junction/symlink), then `git worktree remove --force <path>`: dirty by design, so plain `git worktree remove` refuses; forcing before unlinking deletes through link into shared target.
4. Separate-workspace verdicts don't prove steps work together or survive workspace removal (e.g. link into removed worktree). After removal, one fresh auditor runs every applied round's done-check in main workspace before any enters Verified progress. Step failing there → Remaining w/ that output; its applied paths → Residue w/ revert-or-keep decision.

Within round, executor runs independent reads/searches/commands at once; may fan out
read-only subagents. Parallel-writer work = several steps: split in Remaining, run as
parallel rounds.

## Round engine: Workflow

Invoking skill = user opt-in to `Workflow` tool for its rounds, nothing else. Engine chosen
once at task start from tool exposure, recorded under `Engine:` in Current round block.
Mid-task switch only if tool disappears; log switch in Audit log.

Script gives: spawned agents never inherit Manager history (boundary by construction);
`schema` forces three verdicts into enums, not prose; run persists script + journal of every
agent's exact input/return value (provenance state file can't give); `resumeFromRunId`
replays finished executor from cache after crash, no rerun.

Manager keeps Plan + Integrate. Script has no filesystem, clock, or user → can't pick step,
freeze done-check, weigh dead ends, decide residue, absorb amendment. One round per workflow
call. Rework after failed audit = next round, planned inline; never retry loop in script.

Agents per round: 3 fixed roles + variable judges, chosen at Plan:

- Fixed: baseline, executor, inspector. Baseline + inspector = mechanical roles contract
  already requires. One executor b/c round = one step in one fresh ctx; parallel executors in
  one workspace → manifest diff unattributable. Step wanting N parallel workers = decomposed
  wrong: split into N steps → parallel rounds, one workflow call each.
- `Judges:` = total verdict count incl. inspector. Pick per round, record reason:
  - 1 (inspector alone): mechanical step, small write scope, deterministic done-check (tests,
    build, hash compare).
  - 2 to 3: normal step; delta spans >1 subsystem, or done-check needs interpretation
    (visual result, log inspection, "no regressions").
  - 4 or more: rework round after failed audit, step touches test/gate definitions,
    irreversible side effects, or stagnation trigger fired.
  - Extra judges rotate distinct lenses (scope integrity, check validity, contract drift),
    not identical copies.

Script rules:

- Every prompt from `args` + constants only. Executor return kept for Audit log, never
  concatenated into audit prompt. `agent(auditBrief + executorReport)` = the leak this skill
  prevents, one keystroke away.
- Briefs travel as paths, not strings. Agents read file Manager wrote at Plan; on-disk file =
  byte-for-byte record. Wrapper text around path = template constant, not per-round Manager
  prose.
- Inspector raw artifacts at paths derived from `args`. Judges get paths from `args`, never
  from inspector's return; never see its verdict.
- One agent runs done-check. Parallel auditors each running it → concurrent writes contaminate
  manifest diff. Judges read delta + check output, score; no re-run.
- No `isolation: 'worktree'` for anyone: worktree starts from HEAD, loses executor's
  uncommitted edits. Parallel round's workspace comes from Manager at Plan via
  `args.workspace`.
- `agent()` returns `null` when user skips or API dies. Null = `blocked` w/ integrity
  `suspect`, never `complete`.
- Pass no `model`. Agents inherit session model → quality floor holds. `effort: 'low'` OK for
  baseline agent only.
- Under `+Nk` budget directive, `agent()` throws at ceiling. Catch, return
  `blocked: budget`, checkpoint.
- Pass = unanimity: every status `complete`, every integrity `clean`, every contract
  `aligned`. Any `blocked` verdict blocks round. Disagreement = `suspect`.

Template. Manager writes Current round block + both briefs first, then calls Workflow w/
`args`:

```js
export const meta = {
  name: 'long-horizon-round',
  description: 'One long-horizon round: baseline, executor, inspector, independent judges',
  phases: [{ title: 'Baseline' }, { title: 'Execute' }, { title: 'Audit' }],
}
// args: { taskSlug, round, roundDir, workspace, writeScope, executorBrief, auditorBrief, judges }
// workspace = absolute root this round's executor writes (its own one in a parallel batch).
// roundDir = .tmp/long-horizon/<slug>/round-<N>. Briefs are absolute paths written at Plan.
// judges = total verdict count including the inspector, from the Current round block.
const VERDICT = {
  type: 'object',
  properties: {
    status: { enum: ['complete', 'incomplete', 'blocked'] },
    blockedReason: { type: 'string' },
    integrity: { enum: ['clean', 'suspect', 'violation'] },
    contract: { enum: ['aligned', 'drifted'] },
    contractVersion: { type: 'string' },
    evidence: { type: 'string' },
    deltaPaths: { type: 'array', items: { type: 'string' } },
    // On incomplete only: yes = mechanical fault from the auditor's own check run, no = the
    // approach failed. Judges read the raw check output, never the executor's report.
    // diagnostic is the fault as seen in that output; empty on complete or blocked.
    repairable: { enum: ['yes', 'no', 'n/a'] },
    diagnostic: { type: 'string' },
  },
  required: ['status', 'integrity', 'contract', 'contractVersion', 'evidence', 'repairable', 'diagnostic'],
}
const BASELINE = {
  type: 'object',
  properties: {
    manifestPath: { type: 'string' },
    gitRef: { type: 'string' },
    fileCount: { type: 'integer' },
    uncovered: { type: 'array', items: { type: 'string' } },
  },
  required: ['manifestPath', 'gitRef', 'fileCount'],
}
const LENSES = ['scope integrity', 'check validity', 'contract drift']
const a = args
const manifest = `${a.roundDir}/baseline-manifest.json`
const delta = `${a.roundDir}/audit/delta.md`
const checkOut = `${a.roundDir}/audit/check-output.txt`
const artifacts = { manifest, delta, checkOut }
const blocked = (reason) => ({
  status: 'blocked', blockedReason: reason, integrity: 'suspect',
  contract: 'aligned', contractVersion: 'n/a', evidence: reason, repairable: 'n/a', diagnostic: '',
})
// Stage results live outside the try so a throw mid-round still returns what was collected.
let base = null, executorReport = null, executed = false, inspector = null, judges = []

try {
  phase('Baseline')
  base = await agent(
    `Take a long-horizon baseline of the workspace at ${a.workspace}. ` +
    `Write scope: ${JSON.stringify(a.writeScope)}. ` +
    `Write a manifest to ${manifest}: path and content hash for every file under write scope ` +
    `(including paths outside the repo), every untracked file, every tracked file with ` +
    `uncommitted changes, and deleted paths. In a git workspace also run \`git stash create\` ` +
    `(empty output means clean: use HEAD) and ` +
    `\`git update-ref refs/long-horizon/${a.taskSlug}/round-${a.round} <sha>\`. ` +
    `Change nothing else. List coverage you could not take under uncovered.`,
    { schema: BASELINE, effort: 'low' })
  if (!base) return { verdict: blocked('baseline agent returned null'), executed, executorReport, artifacts }

  phase('Execute')
  executed = true
  executorReport = await agent(
    `Read ${a.executorBrief} and do exactly what it says. Apply fable-mode discipline. ` +
    `Return what changed and how to check it.`)
  // Kept for the Audit log only. Never passed to an audit agent.
  // Null means skipped or died, possibly after partial edits: the Manager reconciles this as
  // an interrupted execution against the baseline; the workspace is never audited as complete.
  if (executorReport === null) {
    return { verdict: blocked('executor returned null; reconcile as interrupted execution'), executed, executorReport, baseline: base, artifacts }
  }

  phase('Audit')
  inspector = await agent(
    `Read ${a.auditorBrief} and follow it. Work in this order. ` +
    `1: rebuild the manifest with the same coverage as ${manifest}, diff it (added, modified, ` +
    `deleted), append \`git diff ${base.gitRef} --stat\`, and write the result to ${delta}. ` +
    `2: run the done-check from the recorded cwd and write the complete raw output to ${checkOut}. ` +
    `3: return your verdicts with evidence.`,
    { schema: VERDICT, phase: 'Audit' })
  if (!inspector) return { verdict: blocked('inspector returned null'), executed, executorReport, baseline: base, artifacts }

  const judgeCount = Math.max(0, (a.judges ?? 1) - 1)
  judges = (await parallel(Array.from({ length: judgeCount }, (_, i) => () => agent(
    `Independent audit judge ${i + 1}, lens: ${LENSES[i % LENSES.length]}. ` +
    `Read ${a.auditorBrief}, then read ${delta} and ${checkOut}. ` +
    `Do not run anything and do not modify files. Return your own verdicts. ` +
    `Default to incomplete or suspect when the evidence is unclear.`,
    { schema: VERDICT, phase: 'Audit' })))).filter(Boolean)
  if (judges.length < judgeCount) log(`${judgeCount - judges.length} judge(s) returned null; integrity capped at suspect`)

  const all = [inspector, ...judges]
  const blockedVote = all.find(v => v.status === 'blocked')
  const verdict = blockedVote ? blockedVote : {
    status: all.every(v => v.status === 'complete') ? 'complete' : 'incomplete',
    integrity: all.some(v => v.integrity === 'violation') ? 'violation'
      : (judges.length === judgeCount && all.every(v => v.integrity === 'clean')) ? 'clean' : 'suspect',
    contract: all.every(v => v.contract === 'aligned') ? 'aligned' : 'drifted',
    contractVersion: inspector.contractVersion,
    evidence: all.map((v, i) => `[${i === 0 ? 'inspector' : `judge ${i}`}] ${v.evidence}`).join('\n'),
    deltaPaths: inspector.deltaPaths ?? [],
    // Any judge calling the failure unrepairable wins; a repairable claim needs everyone,
    // plus a non-empty inspector diagnostic, since the recovery brief has to carry it.
    repairable: all.every(v => v.status === 'complete') ? 'n/a'
      : all.some(v => v.repairable === 'no') ? 'no'
      : (all.every(v => v.repairable === 'yes') && (inspector.diagnostic ?? '').trim()) ? 'yes' : 'no',
    diagnostic: inspector.diagnostic ?? '',
  }
  return { verdict, votes: all, executed, executorReport, baseline: base, artifacts }
} catch (error) {
  // agent() throws at the +Nk budget ceiling and on runtime faults. Return what exists so the
  // Manager can checkpoint; the executor may already have changed files.
  const reason = `blocked: agent() threw (budget ceiling or runtime error): ${error && error.message ? error.message : String(error)}`
  log(reason)
  return { verdict: blocked(reason), votes: [inspector, ...judges].filter(Boolean), executed, executorReport, baseline: base, artifacts }
}
```

After return: runId + transcript dir → Workers; `verdict` + `votes` → Audit log; then
Integrate (below). `blocked` w/ `executed: true` = executor ran, or may have, before stop
(null return, budget ceiling, runtime fault) → reconcile as interrupted execution vs recorded
baseline, never clean round. Cached return on resume ≠ evidence until `journal.jsonl` in
transcript dir shows agent's actual output.

## Round loop

1. **Plan**: read state file; pick batch (every ready step Parallel rounds allows; each round
   works ONE step); set each round's judge count; write each Current round block to state
   file, phase `planned`, before spawning anything. Then per round: auditor brief → its
   recorded path; executor brief = contract excerpt, Current round block, only verified facts
   step needs, every dead end touching step, Method notes. Done-check frozen from here; wrong
   one fixed in next round's Plan, never after reading executor's report.

   Hours-long execution or done-check (browser work, GPU timing, long batches) → draft
   review after Current round block written, before briefs freeze it. One fresh read-only
   peer, prefer other model family (custom-prompt `codex exec -s read-only` run, preflight +
   CLI mechanics per `codex-review` skill; else fresh Claude subagent outside round script),
   reads draft Step, Done-check + code the check exercises. One question: can check pass while
   work wrong, fail while right, or not run as written? Concrete findings only: wrong impl
   that passes (stub returning expected value, test never reaching changed path, check
   reading file executor can write), correct result failing check read literally, or command
   failing as written w/ cwd + output. "Could be stronger" ≠ finding. Manager fixes draft,
   ≤1 follow-up review of changed wording, then freezes. Peer sees only draft + code, never
   executor report; auditor brief carries only frozen block, never peer critique. Cheap
   rounds skip; inspector's `blocked: invalid check` covers them.

   State file + briefs: write/edit via file-edit tool; shell/script string layers drop
   backslashes, backticks. Re-read each saved brief before dispatch. Every brief: worker
   appends each long-lived process it starts (pid, port, command) to `processes.log` in task
   dir. Hours-long job brief: executor checks between batches that workspaces still whole
   (`git worktree list`, sentinel file); mismatch → stop + report, no rebuild. Evidence from
   another revision (line numbers, patch map) names that revision; executor finds cited code
   by anchor text, not line number.

   Baseline = workspace before this executor ran. Rounds never commit between selves → HEAD always wrong reference: would attribute earlier rounds' verified edits + pre-existing user changes to
   this executor. Take as manifest file under task's `.tmp` dir: path + content hash for
   every file under Write scope (incl. outside repo), every untracked file, every tracked
   file w/ uncommitted changes. Git workspace: also pin tracked snapshot: `git stash create`
   (touches neither tree nor index; empty output means clean, use HEAD) +
   `git update-ref refs/long-horizon/<task-slug>/round-<N> <sha>` so gc can't prune it across
   sessions. Git ops unavailable → equivalent immutable snapshot + content manifest valid.
   Include relevant ignored generated artifacts explicitly; name unavailable coverage, don't
   call it clean. Record deleted paths. Sizes/mtimes ≠ baseline; hashes are.
   `.claude/skills/long-horizon/scripts/manifest.mjs` builds
   (`<root> <out.json> --ref <ref> [Write scope and ignored paths]`) + diffs
   (`--diff <out.json>`). Workflow engine: baseline agent takes it as script's first stage,
   at manifest path + ref name already in Current round block. Agent engine: inline before
   spawning executor.
2. **Execute**: phase `executing`. Workflow engine: call round script; executor = stage 2.
   Agent engine: fresh subagent w/ brief alone, no Manager conversation history. Either way
   record run/agent ID as soon as dispatch returns. Executor does step, reports what changed
   + how to check. Confirm it stopped writing, record status, phase `awaiting-audit`.
3. **Audit**: confirm all writers to scope finished/stopped. Workflow engine: inspector +
   judges = stage 3, run only after executor agent returned. Agent engine: second fresh
   subagent w/ prewritten auditor brief, nothing else; record its ID. Order fixed b/c its own
   done-check run writes files too:
   1. Rebuild manifest now, diff vs Baseline (added, modified, deleted) + `git diff <ref> --stat`
      for tracked files. This delta = executor's work.
   2. Run done-check from recorded cwd; compare w/ expected result.
   3. Return three verdicts w/ evidence:
   - status: complete / incomplete / blocked, from own done-check run. Evidence the auditor
     didn't produce this round counts only if it fetched it itself from authenticated source (CI run by
     URL, receipt from external system); executor-produced logs/test output = claims. Check
     itself broken → `blocked: invalid check` = Plan defect, not Dead end. On `incomplete`:
     add `repairable: yes` or `repairable: no` w/ diagnostic from inspector's own done-check
     run (raw check output under round dir). Yes = mechanical fault; approach survives it (build
     error, missing dep, harness/resource failure); no = approach itself failed. Diagnostic
     only in executor's report = claim, can't make step repairable. Several judges: any `no`
     is `no`.
   - integrity: clean / suspect / violation. Clean only if step-1 delta touches nothing
     outside Write scope AND every promised artifact exists. Delta reaching test/gate
     definitions step didn't own = `suspect` at best: passing check proves nothing if
     executor could edit it. Unclear evidence = suspect.
   - contract: aligned / drifted, w/ inspected contract version + acceptance checks.
   Executor report = claim; auditor inspection = evidence. Only complete + clean + aligned
   enters Verified progress. Phase `audited`.
4. **Integrate**: batches follow Parallel rounds order. Pass: step → Verified progress w/
   auditor's evidence, round's brief paths, baseline ref, runId (re-examinable later); copy
   cited result files outside task dir into its `evidence/round-<N>/` (workspace can
   vanish). Fail: keep unaffected Verified progress, mark affected claims stale; append audit
   findings; delta's paths → Residue, revert-or-keep decision each; next round per combined
   `repairable`. `yes`: one recovery round, same approach, brief carries inspector's
   diagnostic; counts as step's second attempt under Stagnation. `no`: approach → Dead ends
   now; next brief changes approach. `invalid check` = Plan defect → neither. One recovery
   per step: failed recovery = step's second failure → Stagnation forces new approach
   whatever second diagnostic says. Either way: archive Current round block into Audit log,
   clear it (stale block feeds next auditor wrong done-check). Rules for later rounds stated
   in executor's report → Method notes, unconfirmed until later done-check covers them.

Update state file every round. 3 rounds w/o state-file write = drift: stop, rebuild file from
real workspace.

Round building/changing verification tool (harness, probe, diff or measurement script) later
rounds use as evidence → one cross-vendor code review of tool after it passes audit, before
any later done-check depends on it: `codex-review`, or `codex-fullreview` if tool large, run
between rounds on uncommitted work holding it. Audit judged step, not whether tool measures
correctly. Confirmed tool findings → next round's step; others → Remaining.

End of each phase (contract milestone, or unit user asked to ship as one PR):
`codex-fullreview` on phase diff, or `codex-review` if diff small. Surviving findings fixed in
audited round before merge, never patched by Manager directly. Other pre-merge reviews repo
requires still run. One PR per phase requested → phase ends after last audited round: commit,
open PR, run these reviews, record `Waiting: merge of <PR>` in state file. Next phase plans
first round in fresh workspace from merged default branch; Baseline taken there.

After compaction/restart: read state, reconcile workspace + latest user instructions, inspect
recorded workers before touching round. Stop `processes.log` processes whose worker no longer
runs. Confirm old writers finished/stopped before auditing or replacing them. Missing IDs or
lost handles ≠ proof of completion; writer status not established → pause affected work, record
recovery needed. Workflow-engine round resumes w/ `resumeFromRunId`, same script, same
`args`: finished agents replay from cache → completed executor not rerun. Read run's
`journal.jsonl` before trusting any cached return.

Validate baseline manifest + Git ref in every phase. Recover missing pieces only from trusted
pre-execution artifacts. Execution may have started + recovery fails → keep partial edits,
integrity `suspect`, attribution unavailable. Never replace old baseline w/ current content
or accept round as clean.

Then reconcile phase:
- `planned`: no execution + no work present → finish/rebuild Plan before dispatch. Execution
  may have occurred → keep original baseline, reconcile as interrupted execution.
- `executing` or `awaiting-audit`: once writers stopped + baseline valid, audit existing work;
  don't repeat execution or overwrite its baseline.
- `audited`: integrate only if inspected content + contract version still apply; else
  invalidate affected evidence, re-audit.

Record recovery actions + pending checks before continuing.

## Stagnation

Round count alone can't resolve stall. Watch repeated failures directly:

- Same step fails audit twice in a row: next brief must change approach, not retry. Move
  failed approach to Dead ends first.
- 3 batches in a row w/ nothing new in Verified progress: stop spawning, rewrite Remaining.
  Decomposition is suspect, not executor. Keep consumed attempts; new decomposition doesn't
  reset user budget.

Count both triggers from Audit log, never memory; recovery round = attempt. Remaining rewrite
may route stuck step via `arena` (parallel candidates, pick, graft); arena runs inside
executor agent: Manager never reads candidates, picks or grafts; inspector sees only
workspace result. Candidates need state file's Contract + Dead ends copied in. Worktree starts
from HEAD → use `.tmp/arena-*` copies or commit WIP first; else earlier rounds' uncommitted
edits lost.

Either trigger may escalate to cross-vendor supervisor. Manager, executor, auditor all Claude
→ shared blindspots, which is exactly how plateau looks from inside. Codex = different model
family, never saw this session:

```bash
codex login status
```

Logged in, or a `model_provider` gateway set in `config.toml` in `$CODEX_HOME` (default `~/.codex`): one `codex exec` run (custom prompt, no scope selector) carrying contract, audit log,
Dead ends; asks plateau diagnosis + different strategy. Current local preflight, CLI
mechanics, run identity: `codex-review` skill. Answer = opinion: check vs current contract version +
acceptance checks before it rewrites Remaining; drop anything that drifts. Neither, or run
fails: skip; rewrite rules above stand alone.

One consult per trigger. Each run bills user's Codex subscription → stagnation-triggered
only, not every round.

## Completion

Per-round verdicts prove each step vs workspace as it was then; later round can regress
earlier one. So before reporting: one last fresh auditor w/ current contract, amendments,
workspace root runs every current acceptance check vs final workspace. Workflow engine: one
more script call, final-audit brief, no executor stage. Failures → back to Remaining.

Then answer from Verified progress alone. Unfinished = valid report: state verified +
remaining, incl. missing independent checks. Bind final evidence to inspected revision +
content manifest; later relevant changes → revalidate.

## Guardrails

- Honor explicit user model choices + required quality floors; else inherit configured
  session model. Check actual exposure before dispatch; requested model/floor unavailable →
  disclose; never silently downgrade or claim it ran.
- Size step so one fresh ctx finishes it: one slice, one migration, one bug. Split
  independent work into separate steps → parallel rounds.
- Audit independence = the point: verdicts from auditor's own inspection in fresh subagent,
  never this Manager ctx.
- Executors + auditors use fable-mode discipline inside round; fable-mode governs one ctx,
  this skill governs work spanning many.
- Under ~3 dependent steps: skip harness, run fable-mode directly.
- At `max(5, 2 * initial step count)` rounds: reassess strategy + remaining work before
  continuing. This default = reassessment threshold, not completion or abandonment rule. At reassessment: tag every Remaining item continue, reserve, or close w/ one-line reason; reserved item reopens only
  via final auditor's failed checks or user instruction. Track executor attempts, auditor
  calls, retries separately. Explicit user round/time/cost limits binding; checkpoint before
  exceeding, report unfinished checks. Budget zero → inspection allowed, no budgeted
  execution.
- Unavailable required check blocks that step + dependents; finish independent authorized
  work, ask only for missing user-owned decisions or authority. Invocation doesn't authorize
  publication, installation, deployments, or external messages.
