---
name: arena
description: "Spawn N parallel candidate attempts at one task, pick the strongest as base, graft the losers' best parts in. Use when the user says /arena, \"arena this\", or when one attempt at a non-trivial artifact would lock in the wrong shape."
---

# Arena

Fan out N parallel attempts at the same task. Read every candidate end to end. Pick the strongest as the base. Graft the best ideas from the others into it. Verify the synthesized result.

Not the same as wow-loop (one implementer iterated under adversarial critique) or impartial-review (review of an existing diff). Arena is a bakeoff plus synthesis: use it when the *shape* of the solution is the open question.

## Start

Open a todo list with one entry per phase before launching anything. The arena runs autonomously and the list keeps phases from silently disappearing.

1. Frame
2. Fan out
3. Cross-judge
4. Pick
5. Graft
6. Verify

## Phase A: Frame

The N candidates receive the same prompt, so the prompt is the contract. Get it right before spawning anything.

1. State the artifact each candidate is producing.
2. Derive the rubric. State what success looks like for *this* task, then turn it into 3-6 concrete gradeable criteria. Concrete: "Adds a --dry-run flag that skips writes". Vague: "code is correct". The rubric is the picker's tool in Phase D; candidates only see the task.
3. Pick the runners. Default: 3 subagents on the session model (never Haiku), each prompted from a distinct angle (e.g. simplest-thing-that-works, robustness-first, user-experience-first) so diversity comes from framing, not chance. When the Codex CLI is available and the task warrants cross-vendor diversity, one candidate may run there (see codex-review for driving it). Spawn more candidates when the arena covers multiple design directions.
4. Assign output paths. Each candidate writes to its own location: a git worktree where possible (worktree isolation when spawning), otherwise `.tmp/arena-<slug>/candidate-<n>/`. N candidates writing to the same path is shared mutable state and corrupts the comparison.

## Phase B: Fan out

Spawn all N subagents in one message so they run concurrently, each with the task, the path to any shared grounding, its own output path, and instructions to produce both the artifact and a short rationale.

The rationale is mandatory. Without it, the parent cannot tell whether a candidate's structure is principled or accidental, which makes Phase E grafting unreliable. Each rationale names the alternatives the candidate considered and what it rejected.

If a candidate fails to produce output, proceed with N-1 and note the dropout in the synthesis record.

## Phase C: Cross-judge

After all Phase B candidates complete, spawn one fresh read-only judge subagent. Prefer a different vendor from the candidates (Codex CLI) when available; otherwise a fresh same-model subagent still removes the parent's authorship bias. The judge sees the rubric and the candidates by path label only — never which angle or vendor produced which — scores each criterion, and recommends a base with rationale. It runs in parallel with the parent's own reading in Phase D, not with the candidates themselves: spawning while candidates are still writing means the judge sees partial outputs and reports them as dropouts.

## Phase D: Pick a base

Read every candidate end to end before picking. Skimming N candidates surfaces only the candidate whose surface looks most familiar.

Score each candidate against the rubric criterion by criterion, not on holistic feel. Compare against the cross-judge. Agreement on the base confirms the pick. Disagreement means one of you is biased or the rubric was ambiguous; read both rationales before deciding.

Pick the base on which candidate a future maintainer can extend most easily without breaking invariants. Prefer the cleaner boundary or smaller surface area when two feel tied.

Record the pick and the reason in a short synthesis note alongside the base artifact, including the cross-judge's verdict.

## Phase E: Graft

Walk each losing candidate once more and identify what is worth porting into the base. The signal is usually one or two things per candidate, not most of it.

Fold each graft in by hand, redesigning it to fit the base's shape. Don't paste mechanically. The result has to remain coherent under one mental model.

Record what was grafted, from which candidate, and what was rejected and why. The rejection notes are the highest-signal part of the record: future readers learn from what you considered and dropped, not just what you kept.

When N candidates converge on the same shape, that is a strong agreement signal. Note the convergence in the record and ship the consensus shape; no graft needed. When N candidates wildly diverge, Phase A was under-specified. Reframe and re-run rather than averaging the divergence.

## Phase F: Verify

The synthesized artifact has to hold up under the same scrutiny as any other output. The arena does not earn a verification pass: run the real check the artifact claims to satisfy.

If verification surfaces a problem the arena did not catch, either Phase A was wrong (re-frame and re-run) or one candidate caught it and you missed the graft (go back to Phase E). Don't paper over.

## Outputs

One synthesized artifact. One short synthesis note alongside, naming the base, the grafts (with source candidate), the rejections, the dropouts if any, and the verification result. Scratch candidate outputs stay in `.tmp/` or their worktrees; only the synthesis ships.

## Anti-patterns

- Don't average divergent candidates into a hybrid nobody designed. Reframe and re-run.
- Don't let the judge see angle or vendor labels; sanitized path labels only.
- Don't skip reading a candidate because the judge scored it low; grafts hide in losers.
- Don't run an arena on trivial work; one attempt suffices when the shape is obvious.

---
Adapted from the `arena` skill in [cursor/plugins pstack](https://github.com/cursor/plugins/tree/main/pstack) (MIT, by poteto).
