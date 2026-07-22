---
description: Use when the user explicitly asks to lab or prototype a visual, UI, motion, or game-feel element with live tuning before production implementation.
---

# lab — live-tune an element in an isolated sandbox, then port + delete

LLMs guess at feel; the human eye knows it. A "lab" is a single throwaway HTML file with live controls that renders the element in isolation, so the **user** dials in the numbers instead of you guessing. Once they're happy, you port the exact values into the real code and throw the lab away. This bridges "looks right in my head" → deterministic, agreed-on values in the actual codebase.

Works for **any** visual or feel element — game OR website.

## The contract (read first)

A lab is **scratch tooling, never product**:

1. **Self-contained.** One HTML file. Inline vanilla JS + CSS. NO external imports, NO build step, NO framework, NO bundler. It must open and run as a plain file.
2. **Seeded at parity.** Every control starts at the element's CURRENT real value, so the lab opens looking exactly like the live thing. The user tunes *away* from the baseline — they can always see what changed.
3. **1:1 key mapping.** The "Copy Settings" JSON uses keys that match the real constant names, so porting is a paste-map, not a translation.
4. **Never committed.** Do not `git add` the lab file. It does not ship.
5. **Deleted when done.** After porting, delete the lab and verify it's gone. Forgotten labs accumulate into served-folder junk — don't start the pile.

## Workflow

### 1. Scope the knobs
Read the real code for the element. List every tunable parameter and its **current value** — these become the lab's controls. A knob is anything the user might want to feel out: durations, magnitudes, counts, radii, easing, colors, alphas, spacing, font sizes, delays, thresholds. Note the real constant name for each (you'll reuse it as the JSON key).

Decide the template (see `templates.md`):
- **Game / canvas** — anything drawn to a `<canvas>`: particles, flashes, shake, trails, sprites, motion, timing. Mock the world procedurally (dummy actors, auto-fire) — do NOT import the real engine.
- **Web / DOM** — a component, layout, type scale, color, spacing, motion/transition, a redesign. Mock the markup with placeholder content.

### 2. Build the lab
Start from the matching skeleton in `templates.md`. For each knob, add a labeled control (range slider, number, color picker, or toggle) bound **live** to the preview — moving it updates the render immediately, no reload. Seed each control's default at the current real value (contract rule 2). Group related knobs. Keep the preview large and the panel compact.

### 3. Add "Copy Settings" — and design for the `file://` clipboard gotcha
A button that serializes the current control values to JSON. Keys = real constant names (contract rule 3).

**CRITICAL gotcha (bit a real session — "I click copy settings but it's not copying"):** every lab here runs from `file://` (step 4), and on a `file://` page `navigator.clipboard` is frequently **`undefined`** (the Clipboard API needs a secure context). `navigator.clipboard.writeText(...)` then throws a **synchronous `TypeError`** — NOT a rejected promise — so a naive `.catch(() => {})` never runs, the click handler aborts, and the visible JSON mirror never gets written → the button looks dead AND nothing is copyable.

So make the JSON readable **without** relying on the clipboard:

1. **Live mirror.** A `<pre>`/box that shows the JSON and updates on **every input** (not just on click), so the current settings are always on screen — the user can read/screenshot/paste them even if copy is 100% blocked. Seed it at load so it's never empty.
2. **Write the box first, then select it.** On click, set the box text BEFORE touching the clipboard, then auto-select its contents (Range + Selection) so a manual Ctrl+C works.
3. **Guard + fall back.** `if (navigator.clipboard && navigator.clipboard.writeText)` → try it; else fall back to `document.execCommand("copy")` on the selection. Wrap BOTH in `try/catch` so neither can abort the handler.
4. **Never** let any clipboard call execute before the visible mirror is written.

The skeletons in `templates.md` already implement this pattern — copy them rather than reinventing the naive `navigator.clipboard.writeText(json)` one-liner.

### 4. Place it
- **Default: a local `file://`-openable `.tmp/<name>.html`.** `.tmp/` is gitignored, so the lab can never be committed or deployed by accident. Hand off the absolute on-disk path; the user opens it directly with `file://`.
- **Only consider a served lab** (e.g. a public/static folder on a dev server) when the user can actually reach that dev server from their machine — check the project's CLAUDE.md environment notes first. If a lab genuinely needs app-served assets/fonts, inline/mock them instead so the local file still works standalone.

Name it whatever the user asks. They may want a generic name with no element word in it — honor an explicit name/title request exactly, including the page `<title>` and any `<h*>` heading. Default fallback when they don't specify: `<element>-lab.html` (e.g. `bomb-lab.html`).

### 5. Hand off to the user
Give them the **absolute on-disk path** to open with `file://` (e.g. `C:\...\.tmp\test-labs.html`) — NOT a dev-server URL (they can't reach it; see step 4). Tell them plainly: **tune the sliders → read/copy the JSON from the live box → paste it back here.** Mention the box updates live and that they can just paste/screenshot it if the Copy button can't reach the OS clipboard (common on `file://`). Then wait.

Note: editing the lab file re-loads it in the user's preview/browser, which **resets their in-progress tune to the seeded defaults**. Avoid editing the lab after handoff unless necessary; if you must (e.g. to fix the copy button), warn them their current values will reset. You cannot read their live in-memory slider state — the preview panel is a viewer, not an eval bridge (`preview_list` returns no server for a `file://` lab), so the live JSON box is the only channel back.

### 6. Port the values
When the user pastes the JSON, map each key to its real constant and make the edits in the actual code. Flag any knob that does NOT map cleanly so the user knows the lab and the real thing will differ slightly — e.g. the lab had a per-effect duration knob but the real code shares one duration constant across effects, so that knob can't carry without a bigger change. Be honest about these gaps.

### 7. Tear down
Delete the lab file. Verify it's gone (`ls`) and that it was never committed (it shouldn't appear in `git status` / `git ls-files`). The lab has served its purpose.

## General notes

- **Naming:** honor the user's requested filename/title verbatim. They may ask for a generic name and require the element word to appear nowhere — not the filename, not the `<title>`, not a visible heading. Check all three.
- **Determinism boundary:** if the real code has a deterministic core (a sim engine, a pure-function reducer), keep it that way when porting — sim-affecting knobs become plain constants in the core; cosmetic-only knobs go in the renderer. Never port lab-style `Math.random` into a deterministic core.
- **Mock, don't import:** labs reproduce the *look* with throwaway code; they don't import the real engine/components. Keep them dependency-free so they open instantly.
- **Multiple render surfaces:** if the element renders in more than one shell/page, port knobs into the shared component so every surface inherits them.

See `templates.md` for copy-paste lab skeletons (canvas + DOM).
