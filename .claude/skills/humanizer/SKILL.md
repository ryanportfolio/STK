---
name: humanizer
description: Use when the user asks to humanize, de-AI, de-slop, voice-match, or review prose for AI tells before publishing.
---

# Humanizer

Edit text to remove AI patterns and add human voice. Preserve meaning and every real fact.

## When to use

Load when the user asks to humanize, de-AI, de-slop, unslop, or un-ChatGPT text; rewrite a draft to sound natural; match their voice; or review for AI tells before publishing.

**Scoped self-application:** apply to **your own** long-form user-facing prose (release notes, PR bodies, blog drafts, public docs, long summaries). Do **not** apply to short chat replies, code comments, commit subject lines, or tool descriptions.

**Book writing:** if the user wants narrative craft (scene, economy, delayed revelation), they may prefer the **Book writing** chat skill. Humanizer is general de-slop; Book writing is craft-first.

## Input modes

1. **Inline** — user pastes text. Rewrite in the reply.
2. **File** — user points at a path. Read it; prefer targeted section edits over full-file rewrites unless the whole file is slop. Show a diff or changed sections; never silently overwrite.
3. **Voice sample** — user provides their own writing (inline or file). Read the sample first; mirror rhythm, word choice, and punctuation. See Voice calibration below.

## Workflow

1. Read the input (and voice sample if provided).
2. Scan the **Pattern checklist** below. Open `patterns.md` only when you need before/after examples for a specific tell.
3. Rewrite: remove tells, preserve facts, match tone or sample voice.
4. **Add soul** — sterile "clean" prose is still a tell (see below).
5. **Internal self-audit:** "What still reads as obviously AI?" Revise once more.
6. Deliver per **Output** below.

## Voice calibration

When a sample is provided, note before rewriting:

- Sentence length and rhythm (short, long, mixed?)
- Register (casual, academic, in between?)
- Paragraph openings and transitions
- Punctuation habits (dashes, asides, semicolons?)
- Recurring phrases or tics

Match the sample. Do not "upgrade" casual voice to corporate prose. With no sample, use the Adding soul defaults.

## Adding soul

Removing patterns is half the job.

- **Have opinions.** React to facts; don't only list neutral pros and cons.
- **Vary rhythm.** Short sentences. Then longer ones. Mix it up.
- **Acknowledge complexity.** Mixed feelings beat one-note praise.
- **Use "I" when it fits.** First person isn't unprofessional.
- **Let some mess in.** Perfect symmetry feels algorithmic.
- **Be specific.** Prefer concrete mechanism, instruction, or number over mood metaphors ("stays close at hand" → what it actually does or returns).

### Soulless → alive (same facts, no new sources)

**Before:**
> The experiment produced interesting results. The agents generated 3 million lines of code. Some developers were impressed while others were skeptical. The implications remain unclear.

**After:**
> I don't really know what to make of this one. Three million lines of code, generated while people were presumably asleep. Half the dev timeline is losing their minds; half is explaining why it doesn't count. The boring truth is probably in the middle, but I keep thinking about those agents running overnight.

## Pattern checklist

Scan for these. Numbers map to `patterns.md` for examples.

| # | Category | Scan for |
|---|----------|----------|
| 1–6 | Content | Significance inflation; notability name-drops; promotional language; vague attribution; formulaic "despite challenges…" sections |
| 7–12 | Language | AI vocabulary; copula dodge (serves as → is); -ing filler; "not just X, it's Y"; tail negations; rule of three; synonym cycling; false ranges |
| 13 | Grammar | Passive voice and subjectless fragments when active voice is clearer |
| 14–19 | Style | Em dash overuse; colon as mid-sentence crutch; mechanical boldface; inline-header bullets that restate the line; Title Case headings; decorative emojis; curly quotes |
| 20–22 | Chatbot | "I hope this helps", cutoff disclaimers, sycophantic openers |
| 23–25 | Filler | "In order to", excessive hedging, generic upbeat endings |
| 26 | Modifiers | Stacked uniform hyphenated compounds (keep required hyphens) |
| 27–29 | Framing | Authority tropes (at its core); signposting (let's dive in); fragmented headers |
| 30–34 | Plain speech | Abstract jargon nouns (substrate, wedge, paradigm); vague product copy; dense sentences; weak adverbs; fancy synonyms (utilize → use) |

**Compact scan lines** (use every pass):

- **Content** — pivotal moment, testament, evolving landscape, nestled, experts say, despite challenges… continues to thrive.
- **Language** — delve, crucial, showcase, foster, leverage, serves as, highlighting…/ensuring…, not just X it's Y, rule of three, from X to Y.
- **Style** — do not use excessive em dashes (—); use periods, commas, semicolons, or hyphens instead. Also cut colon comparison crutches, **Speed:** Speed improved…, Great question! / I hope this helps!
- **Plain speech** — substrate/wedge/harness/paradigm; feelings instead of mechanism; split sentences that need a second read; utilize/leverage/facilitate.

**Inline-header nuance:** bad = `**Speed:** Speed improved…` (label restates the line). OK = `**Schema in TypeScript.** Tables live in one file.` (label names; next sentence adds detail).

## Output

### Default (most requests)

Lead with the **rewritten text**, ready to paste or send.

- Do **not** invent quotes, statistics, or sources. Remove unsourced claims or mark `[needs source]` if the user must fill a gap.
- Keep required hyphens (`cross-functional team`).
- Don't trade one tell for another (semicolon avalanches, parenthesis piles).
- Add a brief **Notes:** line only if something factual was ambiguous or worth flagging. No edit commentary otherwise.

### Review mode

Use when the user asks to **review**, **audit**, **show your work**, or **explain changes**:

1. Draft rewrite
2. Brief bullets: "What still reads as AI?"
3. Final rewrite
4. Optional one-line summary of change categories

### File edits

Apply targeted edits; show diff or changed sections.

## Short example

**Before:**
> Great question! AI-assisted coding serves as a testament to the transformative potential of LLMs, marking a pivotal moment in the evolving landscape of software development. At its core, it's not just about autocomplete; it's about unlocking seamless, intuitive experiences. Industry observers have noted widespread adoption. I hope this helps!

**After:**
> AI coding assistants can speed up boilerplate: configs, test scaffolding, repetitive refactors. They're also good at sounding right while being wrong if you stop reviewing. Adoption is real, but the productivity numbers are hard to compare across teams because everyone measures different things.

## Anti-patterns

- Don't strip grammatically required hyphens — see pattern #26 in `patterns.md`.
- Don't replace one tell with another (em dashes → semicolon storms).
- Don't sanitize into neutral, voiceless prose.
- Don't apply to short replies, code comments, or commit subjects.
- Don't silently overwrite files.
- Don't fabricate quotes, stats, or sources to sound human — invented specificity is worse than vague AI prose.

## Reference

Full before/after catalog (Wikipedia WikiProject AI Cleanup-derived, plus plain-speech extensions): **`patterns.md`** in this folder.
