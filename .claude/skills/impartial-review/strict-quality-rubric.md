# Strict quality rubric ("thermo-nuclear" mode)

Load only when strict mode is requested. Condensed from cursor-team-kit's `thermo-nuclear-code-quality-review` (MIT). This is a maintainability audit on top of the normal buckets — it does not replace correctness review.

## Core stance

Be ambitious about structural simplification. Do not stop at "this could be a bit cleaner." Look for "code judo" moves: restructurings that preserve behavior while whole branches, helpers, modes, conditionals, or layers disappear entirely. Prefer the solution that makes the code feel inevitable in hindsight. If you see a path to delete complexity rather than rearrange it, push hard for that path.

## Non-negotiables

1. **1k-line rule.** Do not let a PR push a file from under 1k lines to over 1k lines without a very strong reason. Treat crossing the threshold as a strong smell; prefer extracting helpers, subcomponents, or modules first. Waive only with a compelling structural reason and a still-organized result.
2. **No spaghetti growth.** New ad-hoc conditionals, scattered special cases, or one-off branches in unrelated flows are a design problem, not a stylistic nit. Push the logic into a dedicated abstraction, helper, state machine, or module.
3. **Clean the design, don't just accept working code.** Same behavior with meaningfully cleaner structure beats "it works". Prefer simplifications that remove moving pieces over refactors that spread the same complexity around.
4. **Direct, boring, maintainable over hacky or magical.** Flag thin abstractions, identity wrappers, and pass-through helpers that add indirection without buying clarity; be skeptical of generic mechanisms hiding simple data-shape assumptions.
5. **Type and boundary cleanliness.** Question unnecessary optionality, `unknown`, `any`, and cast-heavy code where a clearer type boundary could exist. A branch relying on silent fallback to paper over an unclear invariant should make the boundary explicit instead.
6. **Canonical layer and helpers.** Call out feature logic leaking into shared paths, and bespoke helpers where a canonical utility exists. Push code to the package/module/layer that owns the concept.
7. **Orchestration and atomicity.** Flag needless sequential orchestration of independent work and partial-update logic that leaves state half-applied, when the cleaner structure is obvious. Don't over-index on micro-optimizations.

## Flag aggressively

A cleaner reframing that would delete whole categories of complexity · refactors that move code without reducing concepts a reader holds · a file crossing 1000 lines due to the PR · new conditionals bolted onto unrelated paths · one-off booleans/nullable modes complicating control flow · copy-pasted logic instead of extracted helpers · edge-case handling in the middle of an already busy function · "temporary" branching likely to become permanent debt.

## Preferred remedies

Delete a layer of indirection rather than polishing it · reframe the state model so conditionals disappear · turn special-case logic into a simpler default flow with fewer exceptions · replace condition chains with a typed model or explicit dispatcher · separate orchestration from business logic · split the large file into focused modules · reuse the canonical helper.

Not satisfied with "maybe rename this" when the real issue is structural. Not satisfied with a merely cleaner version of the same messy idea when a much simpler idea is plausible.

## Tone

Direct, serious, demanding; not rude, but never soften major maintainability issues into mild suggestions. Say things like:

- "this pushes the file past 1k lines. can we decompose this first?"
- "this adds another special-case branch into an already busy flow. can we move this behind its own abstraction?"
- "i think there's a code-judo move here that makes this much simpler. can we reframe this so these branches disappear?"
- "this refactor moves complexity around, but doesn't really delete it. is there a way to make the model itself simpler?"

## Output priority

1. Structural regressions → 2. missed dramatic-simplification opportunities → 3. spaghetti/branching growth → 4. boundary/type-contract problems → 5. file-size/decomposition → 6. modularity → 7. legibility. Few high-conviction comments beat a long cosmetic list.

## Approval bar

Do not approve merely because behavior seems correct. Presumptive blockers unless clearly justified by the author:

- Preserves a lot of incidental complexity when a plausible code-judo move would delete it.
- Pushes a file from below 1000 lines to above.
- Adds ad-hoc branching that tangles an existing flow.
- Scatters feature checks across shared code to solve a local problem.
- Adds an unnecessary abstraction, wrapper, or cast-heavy contract.
- Duplicates an existing helper or puts logic in the wrong layer.
