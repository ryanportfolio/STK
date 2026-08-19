---
name: unslop
description: "Always-on AI-tell stripper: apply its pattern check to everything written for humans (chat prose, commits, PR bodies, docs, UI text). Also use when the user says /unslop, \"unslop this\", or points at text or a file to clean."
---

# Unslop

Strip AI patterns from writing. Write clean first: never generate the tell and fix it after. The cleanup-afterward pass has been measured to fail, so the bad sentence must not be produced in the first place.

## Always-on contract

This skill is a session default (see CLAUDE.md). It applies at write time to every artifact a human will read: chat replies, commit messages, PR titles and bodies, docs, READMEs, UI text, summaries, issue comments.

Exempt: code, symbol/function/API names, error strings, quoted text, file contents reproduced verbatim.

Coexists with other modes:

- **caveman** compresses; unslop strips tells. Both apply to chat prose. Invoking caveman activates this contract, and the caveman skill carries a copy of the core-tells digest so no separate load is needed.
- **humanizer** is the on-request deep pass: voice-matching, file rewrites, soul-adding for long-form drafts. Unslop is the always-on floor. For "humanize this in my voice", load humanizer.

Two condensed copies of the core tells exist outside this file: the "Core tells, banned at write time" section in CLAUDE.md and the "Unslop rides this skill" section in `.claude/skills/caveman/SKILL.md`. They are verbatim duplicates. Changing a core tell here means updating both.

## On-demand mode (`/unslop <text or path>`)

1. Scan for the patterns below.
2. Rewrite. Preserve meaning and every fact; match intended tone.
3. Self-audit: "What makes this obviously AI generated?" Fix remaining tells.
4. For files: targeted section edits, show the diff, never silently overwrite.

## Patterns to detect and never write

### Content

1. **Puffery.** "pivotal moment", "testament to", "evolving landscape", "setting the stage for", "indelible mark". State what happened.
2. **Superficial -ing phrases.** "highlighting...", "ensuring...", "showcasing...", "fostering...". Delete or expand with real substance.
3. **Promotional language.** "nestled", "vibrant", "breathtaking", "groundbreaking", "renowned", "stunning". Neutral description.
4. **Vague attributions.** "Experts believe", "Industry reports suggest". Name the source or delete.
5. **Formulaic challenges.** "Despite challenges... continues to thrive." Specific facts.

### Language

6. **AI vocabulary.** Additionally, crucial, delve, enduring, enhance, fostering, garner, interplay, intricate, landscape (abstract), pivotal, showcase, tapestry (abstract), testament, underscore, vibrant. Plain words.
7. **Fancy ways to say "is".** "serves as", "stands as", "boasts", "features". Say "is" or "has".
8. **"Not just X, but Y."** State the point directly.
9. **Rule of three.** Forcing ideas into groups of three. Use the natural number.
10. **Synonym cycling.** Protagonist, main character, central figure in one paragraph. Pick one, repeat it.
11. **False ranges.** "from X to Y" where X and Y aren't on a meaningful scale. List directly.

### Style

12. **Em dashes.** Never. Use `.` `,` `:` `;` `·`. No parentheses or en dashes as substitutes; end the sentence or use a comma.
13. **Colon as mid-sentence connector.** Colons before a list or example are fine. As a rhetorical hinge ("If you're coming from X: instead of...") rewrite so the point stands alone.
14. **Boldface overuse.** Don't bold every proper noun or acronym.
15. **Inline-header lists.** Bold label + colon restating the line ("**Performance:** Performance improved...") is a tell; convert to prose. A bold lead-in ending in a period followed by genuinely new detail is fine.
16. **Title case headings.** Sentence case.
17. **Decorative emojis** in headings and bullets. Remove.
18. **Curly quotes.** Straight quotes.

### Communication artifacts

19. **Chatbot phrases.** "I hope this helps!", "Let me know if...", "Of course!", "Certainly!", "Found the smoking gun!" Remove.
20. **Sycophancy.** "Great question! You're absolutely right!" Respond directly.
21. **Cutoff disclaimers.** "While specific details are limited..." Find the fact or remove.

### Filler

22. **Filler phrases.** "In order to" → "To". "Due to the fact that" → "Because". "It is important to note that" → delete.
23. **Hedging stacks.** "could potentially possibly be argued that it might" → "may".
24. **Generic conclusions.** "The future looks bright." Specific plans or facts, or nothing.

### Jargon

25. **Abstract metaphor nouns.** Substrate, wedge, vector, locus, nexus, primitive (noun), harness (metaphor), surface ("API surface"), bedrock, scaffolding (metaphor), paradigm, gold-plating, ratchet (metaphor), north star, flywheel, endgame. Pick the concrete word: "substrate" → "base", "wedge in" → "add", "endgame" → "the last phase".

### Plain speech

26. **Say what it does, not how it feels.** "SQL you can read", "types that follow your schema" name a feeling. Name the mechanism or a number instead: "`.toSQL()` returns the exact string sent to the database", "a column rename fails the build". If a sentence can't be restated as a concrete instruction, fact, or number, cut it. If it could appear unchanged in another project's docs, it says nothing about this one; cut it.
27. **One idea per sentence.** If the reader backtracks to parse, split or drop clauses.
28. **Active voice.** "queries are validated" → "the compiler validates queries". Passive only when the actor is unknown or genuinely irrelevant.
29. **Cut adverbs or use the number.** "runs quickly" → "is fast" or the measurement. "significantly improves" → the measured delta.
30. **Plain word over fancy synonym.** "utilize" → "use", "leverage" → "use", "facilitate" → "help", "numerous" → "many", "in the event that" → "if".

## Code diffs (`/unslop the diff`)

Code has its own slop. When pointed at a diff (default: against main), remove AI patterns introduced in the branch:

- Extra comments that are unnecessary or inconsistent with local style.
- Defensive checks or try/catch blocks abnormal for trusted code paths.
- Casts to `any` used only to bypass type issues.
- Deeply nested code that early returns would simplify.
- Other patterns inconsistent with the file and surrounding codebase.

Guardrails: keep behavior unchanged unless fixing a clear bug; minimal focused edits over broad rewrites; summary in 1-3 sentences.

## Anti-patterns

- Don't drop a fact, caveat, or qualifier to remove a tell. Accuracy beats cleanliness.
- Don't apply to code, identifiers, error strings, or quoted material.
- Don't run a visible "cleanup pass" over your own fresh prose; write it clean the first time.
- Don't replace humanizer for voice-matching or long-form rewrites; that's its job.

---
Adapted from the `unslop` skill in [cursor/plugins pstack](https://github.com/cursor/plugins/tree/main/pstack) (MIT, by poteto); code-diff section from cursor-team-kit's `deslop` (MIT).
