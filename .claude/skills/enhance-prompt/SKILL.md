---
description: Use when the user asks to rewrite a rough request into a polished, copy/paste-ready prompt for another agent or a fresh session.
---

# Enhance prompt

Rough request in `$ARGUMENTS` → one polished prompt a cold agent can act on. Platform-neutral: works in fresh coding-agent session, IDE assistant, or generic chat LLM.

**Prime directive: every sentence must change receiver behavior. No behavior change → cut.** 200 words all signal > 800 words repeating.

**This skill file is compressed; the output prompt is NOT.** Receiver is cold → output prompt = plain clear prose, full sentences, nothing abbreviated.

## 1: Extract real goal

Look past surface wording:

- "Review X" → usually audit + propose, not edit. Phase it.
- "Add feature X" → needs scope, edge cases, UI placement
- "Fix bug X" → needs repro steps, current vs expected behavior
- "Refactor X" → needs scope boundaries (which files, which patterns to keep)
- "Make it better / cleaner / faster" → needs concrete success criteria

Genuinely ambiguous on a point that materially changes output → ask user once before drafting. Else proceed; surface assumptions as `<TODO: confirm>` placeholders.

## 2: Gather receiver-cold context

Receiver: no conversation memory, maybe no codebase view. Add:

- **File paths + line numbers** — greppable → include. `Edit client/src/pages/Foo.tsx:142` 10× more actionable than "edit the foo page."
- **Existing patterns to match** — one codebase reference ("follow the pattern in `bar.tsx:80-110`")
- **Current vs desired state** — bug fixes/changes: spell both out concretely
- **No invented facts.** User didn't say it + can't verify → `<TODO: user fills in>`, never fabricate.
- **No big code dumps.** Reference file path; receiver with repo access reads it. Quote minimum only when receiver clearly lacks repo access.

## 3: Write like prompt engineer

Craft rules for prompt body:

- **Clear + direct.** Write for brilliant new employee, zero context on your norms. Golden rule: colleague with minimal context confused → receiver confused.
- **Explicit action verbs.** Models follow literally: "can you suggest changes" → suggestions, not edits. Want edits → "Change/Implement/Fix X". Want no edits → "Propose/List, do not edit". Act-vs-advise never implicit.
- **Say do, not avoid.** "Write flowing prose paragraphs" > "don't use markdown". Convert negatives to positives; keep negatives only as scope guards ("don't refactor unrelated files").
- **Why on non-obvious constraints.** "Never use ellipses — the output is read by a text-to-speech engine" → receiver generalizes correctly. Bare rule → literal-minded misfires. Obvious constraint needs no why — padding.
- **Calm imperative tone.** No "CRITICAL:", "YOU MUST", ALL-CAPS — modern models overtrigger on aggressive language, reads as noise. Plain "Do X" followed just as reliably. Strong emphasis ≤ 1 genuinely blocking rule.
- **Quality modifiers when quality is the point.** Above-and-beyond wanted → say concretely: "Include as many relevant features and interactions as possible; go beyond the basics." Vague → on-distribution median output.
- **XML tags when content types mix.** Instructions + pasted data/logs + examples → wrap each (`<instructions>`, `<context>`, `<input>`, `<example>`) so data ≠ directive. Short single-purpose prompt → no tags, no ceremony.
- **Long pasted content top, task bottom.** 1k+ tokens of logs/docs/data → material first, instructions/question after — measurably better responses. Very long docs → add "quote the relevant parts before answering."
- **Role line only if it changes behavior.** "You are a senior security engineer reviewing for OWASP Top 10" focuses the review; "You are a helpful assistant" dead weight.
- **Examples when format matters.** Specific deliverable shape (severity-tagged findings, table layout) → 1-2 short `<example>` tags beat prose description. Obvious format → skip.
- **Self-check for verifiable work.** "Before you finish, verify the change against [the failing test / a type-check / the listed criteria]". Match check to task; no generic "double-check your work" bolted onto everything.
- **General solutions, not test-passers.** Tests/specific examples present → add: "Implement the actual logic that solves the problem generally — do not hard-code values or special-case the given examples. If a test or requirement is itself wrong, say so rather than working around it."
- **Grounding for codebase questions.** Task = answering questions about existing code → add "read the relevant files before making claims about them; don't speculate about code you haven't opened."

## 4: Phase risky work

Touches user-visible copy, DB schemas, public APIs, model routing, payment logic, or large refactor → split into phases, receiver STOPs between:

1. **Phase 1: audit / propose only.** Findings → `.tmp/<task>-plan.md` or inline. No file edits.
2. **Phase 2: user reviews.**
3. **Phase 3 (separate prompt or continuation): implement approved subset.**

Safe mechanical work (typo fix, rename one internal variable, add console log) → single phase.

Doubt → phase. Skipped phase cheap; bad edit expensive to undo.

## 5: Specify deliverable

Always include:

- **Artifacts to produce** (file edits, markdown audit, new component, verification script, etc.)
- **Format if structured output expected** (table layout, severity-tagged list, JSON shape, fenced sections with specific headings)
- **Verification step** — type-check, screenshot, dry-run script — matched to task
- **Scope guards** — common: "don't refactor unrelated files", "don't add tests unless asked", "don't install packages without confirmation", "don't change user-facing copy outside the listed strings". Gold-plating-prone task → add: "keep the solution minimal — no extra abstractions, configurability, or defensive code beyond what the task needs."
- **Escape hatch** — receiver hits genuine blocker (missing access, contradictory requirement, `<TODO>` unresolved) → should stop and ask, not guess. One line: "If anything here is ambiguous or blocked, ask before proceeding rather than guessing."

## 6: Bake in project constraints

`CLAUDE.md` in working dir → skim for rules touching this task, **inline only those, stated as plain constraints**. Receiver may lack CLAUDE.md — never write "read the CLAUDE.md", "follow the project guidelines", or "CLAUDE.md says X". Rule matters → verbatim constraint in prompt. Doesn't → stays out.

Frequent categories when applicable:

- **Naming/terminology policies** — error surfaces, banners, provider/product names in user-facing copy
- **Manual package install policy** — task may add dependencies
- **Manual DB migration policy** — schema changes
- **Translation / i18n approval flow** — user-facing wording changes
- **Settings duplicated across files** — config living in multiple places (default model lists, feature flags)
- **Multi-theme / multi-mode UI requirements** — new visual elements

## 7: Platform-neutral output

Prompt must work regardless of executor.

- ❌ "use the Bash tool", "via the Task subagent", "as Claude Code", any named agent platform
- ❌ Platform UI affordances ("click the X button in the sidebar")
- ✅ "run the following command", "search the codebase for X", "edit the file at `path:line`", "produce a markdown file at `<path>`"
- ✅ Outcomes + artifacts, not the path there

Receiver uses own tools. Give the *what*; prescribe the *how* only when the *how* is itself the point.

## 8: Self-review pass

Draft done → reread once as the cold receiver. Colleague test: minimal-context colleague confused anywhere → rewrite that line. Then per sentence: changes receiver behavior? No → cut. Catch leftover conversation-context leaks ("as discussed", "the file we looked at") — receiver has none.

## 9: Output format

One outer-fenced markdown block, copied verbatim. Four backticks outer so triple-backtick blocks inside render:

````
```markdown
[the polished prompt goes here]
```
````

Above block: 1-2 sentence note on what changed from user's input — e.g., "I added a phase split, three file path references, and the copy-policy constraint since this touches user-facing wording."

Do NOT execute the prompt. Do NOT edit files while running this skill. Deliverable = the prompt, nothing more.

## Anti-patterns

- **No instructions the environment already handles.** Never: "read the CLAUDE.md / project guidelines first", "use your available tools", "explore the codebase to understand it", "be thorough and careful", "think step by step". Agent harness injects these automatically; chat LLM has neither. Dead weight either way.
- **No padding.** Sentence doesn't change receiver behavior → cut. Includes goal restated in new words, generic quality exhortations, prompt summarizing itself.
- **No invented constraints.** Typo fix needs no 5-phase audit, XML tags, examples, or role line. Prompt complexity = job complexity — Step 3 techniques conditional, apply only when task calls for them.
- **No shouting.** "CRITICAL", "IMPORTANT!!", MUST-in-caps → overtriggering on modern models + dilutes the one rule that might actually be blocking.
- **No CLAUDE.md dumps.** Only task-relevant rules, as plain constraints.
- **No overriding user's clear choices.** Decided → encode as-is. Left open → ask or flag `<TODO>`.
- **No placeholders for verifiable facts.** Can grep the path / read the code → do it, don't write `<file path>`.
- **No platform-specific tools or UI.** Output must be tool-agnostic.
- **No pre-writing receiver's reply.** "the agent will respond with X" / "you should answer Y" leak frame; specify what receiver should produce, nothing more.
