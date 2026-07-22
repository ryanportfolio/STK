---
description: Use when the user asks to review, audit, or stress-test recent code changes with fresh independent agents; requires exposed multi-agent tools.
---

# Impartial review

You are reviewing work that was probably written too fast — possibly by you. Your job is to find what's wrong, not validate what's right. **If you wrote the code yourself, look harder, not softer.** Bias toward finding real issues, even at the cost of being uncomfortable.

The hardest bias to overcome is defending code you just wrote. The fix is mechanical: dispatch the actual review to fresh-context Sonnet 4.6 subagents that have not seen the conversation that produced the code. Your job in the main session is to gather the diff, brief the subagents, and consolidate findings.

This is a two-stage split, and the stages have opposite jobs. The subagents maximize **coverage** — find everything, including uncertain and low-severity issues. You — running on the strongest available model — supply **precision**, verifying each finding before it reaches the human. Over-reporting upstream of a strong verifier is the design, not a flaw: it is the main model's job to review the subagents' work, not to rubber-stamp it.

## Step 1: Identify scope

Use `$ARGUMENTS` if the user named a specific scope (file path, PR number, "the Q&A changes", commit SHA, etc.). Otherwise default to recent work in this priority order:

1. `git status` and `git diff` — uncommitted changes
2. `git log -1 --stat` and `git show HEAD` — most recent commit
3. If there's an open PR on the current branch, include the full PR diff (`git diff origin/main...HEAD`)

State the scope you're reviewing in your first sentence so the user can redirect if it's wrong. Also count the changed lines (`git diff <range> --stat | tail -1`) — you'll need this for Step 2.

## Step 2: Pick review mode

- **Tiny diff (< 50 changed lines, single file, no schema/auth/cache code):** review inline in the main session. Spawning 5 subagents for a 20-line CSS tweak is wasteful.
- **Everything else:** dispatch **five parallel Sonnet 4.6 subagents** (see Step 3). This is the default.

If you're not sure, dispatch. Subagents are cheap relative to a missed bug.

## Step 3: Dispatch five parallel Sonnet 4.6 review subagents

Send all five `Agent` tool calls **in a single message** so they run concurrently. Each uses:

- `subagent_type: "general-purpose"`
- `model: "sonnet"` (resolves to Sonnet 4.6 — the current Sonnet alias)
- A self-contained prompt — subagents do not inherit your context

**Reviewer model tier.** Sonnet 4.6 is the right default — cheap, fast, and your main-session verification (Step 6) is the safety net that catches its misses. Escalate the reviewers to Opus only for the highest-stakes diffs (schema/migrations, auth, core routing logic, anything touching money or PII), where a missed bug is expensive enough to justify the cost.

Each prompt must include:

1. The exact diff to review (paste it inline if < ~1500 lines; otherwise give the exact `git` command and the commit range/branch).
2. Their assigned category bucket (below).
3. The verification rule from Step 4.
4. The severity scheme from Step 5.
5. The per-finding output format from Step 6.
6. An instruction to **only** report findings in their bucket — the main session deduplicates and merges.

Bucket E (project-aware) gets an **extended** prompt — see its section below. The other four use the standard template.

### The five buckets

**Bucket A — Correctness & types**
- Bugs, off-by-one, edge cases, logic errors
- Conditions that look right but aren't (`||` vs `??`, missing `await`, mutating loops, truthy/falsy traps)
- ORM table ↔ validation schema drift; inferred type drift
- Consumers of a changed type that no longer compile
- Optional fields added without updating callers

**Bucket B — Data flow, compatibility, error handling**
- Where input comes from, where output goes, what cache rows look like
- Concurrent access; stale or wrong-shape cache rows
- Old data in the DB; old rows in changed-shape tables; old cache rows; old API requests
- Required migrations; backwards-incompatible changes
- Failure paths, timeouts, retries; external APIs returning null/empty/wrong shape
- Values assumed present that can be undefined

**Bucket C — Perf, security, observability**
- Extra DB queries per request, prompt size growth, latency, N+1, full-table scans, unbounded loops
- Cross-user data leakage, prompt injection, auth bypasses, PII/secrets in logs, public endpoints
- Logs on paths that matter; silent failure modes (parser returns empty, fallback fires, cache miss) made visible

**Bucket D — Things the author missed** (highest-value bucket — undivided attention, push hard)

This is the single highest-leverage category and gets its own dedicated reviewer. The agent should treat the diff as a list of incomplete changes and look for what wasn't done.

- Updated one of two related code paths and forgot the other (e.g., streaming + non-streaming, server + client, English + i18n locales)
- Changed a type/interface but not all consumers
- Captured data but forgot to persist or read it back
- Added a feature but forgot the tear-down (cleanup, expiry, eviction, cache invalidation)
- Fixed the streaming version but left the non-streaming version broken (or vice versa)
- Added a config option / env var / flag but forgot to thread it through to the code that actually uses it
- Added a new model / provider / route but didn't register it in the dispatcher, switch, or admin list
- Added a column / field but didn't include it in serializers, exports, or display
- Renamed something but left old references (search the diff for the old name)
- Added handling for the success path but not the error path (or vice versa)
- Added a test but didn't run the suite that includes it
- Added docs/README text asking a HUMAN to maintain an invariant ("don't install X alongside Y", "remember to replace these refs") — a disclaimer that could be a detection hook or a script is a finding: automate it or explain why it can't be

For this bucket specifically: greppability beats cleverness. The agent should `grep` the changed identifiers across the codebase and look at every hit to see if anything was missed.

**Bucket E — Project-aware violations** (the structural blind spot of fresh-context review)

The other four reviewers are deliberately context-free — that's the source of their impartiality, and also why they can't catch violations of *this codebase's specific rules*. Bucket E exists to close that gap. The agent reads project reference material first, then reviews the diff against it.

The reviewer is told to read these files before looking at the diff:

- `CLAUDE.md` (root) — the project's kernel rules (naming/copy policies, install paths, migration protocol, verification carve-outs, etc.)
- `.claude/reference/pitfalls.md` — accumulated project-specific gotchas
- Any other `.claude/reference/*.md` file relevant to the diff's surface area (match topic file names to the code the diff touches — e.g. env vars / `process.env.X` → `secrets.md`, cross-cutting flow → `architecture.md`)

The reviewer's job is to find places where the diff violates rules encoded in those files. High-value patterns (substitute this project's actual rules):

- User-facing copy (toasts, banners, errors, modals, tooltips) that violates a naming or terminology policy in `CLAUDE.md`.
- Diffs that bypass the project's migration or schema-change protocol.
- UI changes that satisfy only one of multiple required themes/modes.
- New env vars / `process.env.X` reads without a corresponding entry in the env-var reference.
- Dependency installs in code or scripts that should have followed the project's install policy instead.
- Anything else flagged in `pitfalls.md`.

The reviewer should cite the specific rule (file + section) it's enforcing for each finding so the human can verify the rule actually says what the agent claims.

### Subagent prompt template (Buckets A–D)

```
You are an impartial code reviewer. Fresh context — you did not write this code.
Your job: find what's wrong, not validate what's right. Bias toward finding real issues.

Your job at this stage is coverage, not filtering. Report every issue you find,
including ones you are uncertain about or consider low-severity. A separate merge
step on a stronger model ranks and verifies — it is better to surface a finding
that later gets filtered out than to silently drop a real bug. Tag each finding
with a confidence level and a severity so the merge step can rank it. (Don't
invent issues to pad the list — fabrication is the only thing to omit.)

## Scope
[paste the diff here, OR give the exact git command + range]

## Your bucket: [A / B / C / D — name]
Review ONLY these categories:
[paste the bucket's bullets]

Do NOT report findings outside your bucket. The main session merges with four
other reviewers covering the rest.

## Verification rule
For every issue you suspect, run a real check before asserting.
- grep for actual call sites before claiming code is unused or that a function does X
- Read the file before claiming a function's behavior or signature
- Don't say "this might break Y" — open Y, look, then say either "Y breaks
  because [specific reason]" or "Y is fine because [specific reason]"
- Distinguish "I haven't checked X" from "I checked X and it's fine"
Plausible-sounding-but-unchecked claims are the most common review failure.
Do not produce them.

## Severity tags
🔴 BLOCKING — Real bug, regression, schema drift, security/privacy issue, data correctness
🟡 SHOULD-FIX — Edge case that will bite, observability gap, inconsistency, parity issue
🟢 NITPICK — Style, future polish, deferable

## Confidence (tag every finding, separate from severity)
HIGH — verified: I read the file / grepped the call sites.
MED  — likely, but I only partially checked.
LOW  — suspected; I could not fully verify within my time budget.

Report LOW-confidence findings too — tag them LOW and let the main-session merge
step adjudicate. Never drop a real finding because you're unsure; that call
belongs downstream, on a stronger model.

If every finding is 🟢, you didn't look hard enough. Go back and push harder
on your bucket — especially if you're Bucket D, where 🟢-only output usually
means you didn't grep aggressively enough for missed paths.

## Output format
Return ONLY a list of findings in this format, severity-ordered (🔴 first):

## 🔴 Short title  ·  confidence: HIGH|MED|LOW
`path/to/file.ts:123`

[Concrete description: what's wrong, what triggers it, what the impact is.
Reference specific code, not abstract worries.]

**Fix:** [Specific edit. Not "consider improving X" — say what to change and where.]

After the findings, add a section:

## Things I checked and verified fine
- [Item that looked suspicious but you confirmed is OK, with a one-line reason.]

If you genuinely found nothing after running every category check, say so
explicitly: "Ran through [list categories]; no issues at any severity in my
bucket." Don't fabricate issues to look productive — but don't suppress real
findings because they seem minor or uncertain either. Report real
low-severity/low-confidence findings and tag them honestly; the merge step filters.
```

### Subagent prompt template (Bucket E only)

```
You are an impartial code reviewer with project context. Fresh context — you
did not write this code. Your job: find places where the diff violates rules
encoded in this project's reference material. You are the ONLY reviewer
seeing project-specific rules; the other four are deliberately context-free.

This stage is coverage, not filtering: report every violation you find,
including uncertain or low-severity ones, tagged with confidence and severity.
The main-session merge step on a stronger model verifies and ranks.

## Required reading (do this BEFORE looking at the diff)

1. Read `CLAUDE.md` in the repo root — the project's kernel rules (naming/copy
   policies, install paths, migration protocol, verification carve-outs, etc.).
2. Read `.claude/reference/pitfalls.md` — accumulated project-specific gotchas.
3. List `.claude/reference/` and read whichever topic files match the diff's
   surface area (e.g. `secrets.md` for env var / API key code,
   `architecture.md` for schema / cross-cutting changes, `tech-stack.md`,
   `commands.md`, `deployment.md` if relevant).

## Scope (the diff to review)
[paste the diff here, OR give the exact git command + range]

## What to look for

Find places where the diff violates rules in the files you just read. Examples
(non-exhaustive — let the reference files drive you):

- User-facing copy (toasts, banners, errors, modals, tooltips, exports)
  violating a naming or terminology policy in CLAUDE.md.
- Diffs that bypass the project's migration or schema-change protocol.
- UI changes that satisfy only one of multiple required themes/modes.
- New `process.env.X` reads without a corresponding `secrets.md` entry.
- Dependency installs in code or scripts that should have followed the
  project's install policy.
- Anything in `pitfalls.md`.

Cite the specific rule (file + section quote) you're enforcing for each
finding. The human will verify the rule actually says what you claim.

## Verification rule
For every issue you suspect, run a real check before asserting.
- grep for actual call sites before claiming code is unused or that a function does X
- Read the file before claiming a function's behavior or signature
- Quote the project rule you're invoking — don't paraphrase if a verbatim
  quote is short enough
- Distinguish "I haven't checked X" from "I checked X and it's fine"

## Severity tags
🔴 BLOCKING — Clear violation of an explicit project rule with user-visible
  or correctness impact (e.g., a naming-policy leak in a toast, a diff that
  bypasses the migration protocol)
🟡 SHOULD-FIX — Project-pattern inconsistency (e.g., new env var without
  reference entry, single-theme styling)
🟢 NITPICK — Style alignment with project conventions, deferrable

## Confidence (tag every finding, separate from severity)
HIGH — verified: I read the file / grepped, and quoted the rule verbatim.
MED  — likely, but I only partially checked the code or the rule.
LOW  — suspected; I could not fully verify within my time budget.

Report LOW-confidence findings too — tag them LOW and let the merge step
adjudicate. Never drop a real violation because you're unsure.

## Output format
Return ONLY a list of findings in this format, severity-ordered (🔴 first):

## 🔴 Short title  ·  confidence: HIGH|MED|LOW
`path/to/file.ts:123`
Rule: [file:section, with a short quote of the rule]

[Concrete description: what the diff does, why it violates the rule, what
the user-visible or correctness impact is.]

**Fix:** [Specific edit. Reference the project pattern the fix conforms to.]

After the findings, add a section:

## Things I checked and verified fine
- [Item that looked like a violation but is OK, with a one-line reason.]

If you genuinely found nothing, say so explicitly: "Read [list reference
files]; no project-rule violations in this diff." Don't fabricate violations to
look productive — but report real low-severity/low-confidence ones and tag them;
the merge step filters.
```

## Step 4: Verification rule (applies inline too)

For every potential issue, run a real check before asserting. `grep` for call sites. `Read` the file. Open the consumer and look. Plausible-sounding-but-unchecked claims waste the human's time when they re-investigate and find the claim was wrong.

This rule is repeated inside each subagent prompt, but it also applies to your inline review for tiny diffs and to the merging step in Step 6 — don't paper over a subagent's unverified claim by passing it through.

## Step 5: Severity tags

🔴 **BLOCKING** — Real bug, regression, schema drift, security/privacy issue, or data correctness problem. Should not merge.

🟡 **SHOULD-FIX** — Edge case that will eventually bite, observability gap, inconsistency, minor parity issue between code paths. Should be fixed but not blocking.

🟢 **NITPICK** — Style preference, future polish, deferable consideration. Mention it but make clear it can be skipped.

## Step 6: Merge subagent findings and present

When the five agents return — **this is the precision stage.** The subagents over-reported on purpose (coverage); your job is to verify and rank so the human gets a trustworthy list. You're running on the strongest available model, and this verification is exactly where it earns its cost. Reviewing the subagents' work is the point — do not rubber-stamp it.

1. **Deduplicate.** Two agents may flag the same issue from different angles — merge into one finding, keep the higher severity. Bucket E findings often overlap with A/B/C/D (e.g., a cover-identity leak is also a correctness issue) — merge but preserve E's rule citation so the human sees *why* it's a violation.
2. **Verify every finding you intend to surface — across all severities, not just 🔴.** The finding stage deliberately over-reported, including LOW-confidence items; turning that into precision is your job. For each finding, run a real `grep`/`Read` to confirm before passing it to the human (for Bucket E, open the cited rule file and confirm the rule actually says what the agent claimed — paraphrased rules are the most common Bucket E failure mode). Treat the 🔴s adversarially: a fresh-context subagent in a hurry is exactly the kind of reviewer that produces plausible-but-wrong blockers, so try to *refute* each one before you accept it.
   - **Own the confidence filter — but drop only on evidence.** A finding tagged LOW-confidence gets *confirmed* (verify, then promote and re-tag), *refuted* (drop it — optionally note it under "checked and verified fine"), or *kept as LOW* with a one-line note on the residual uncertainty. Drop a finding **only because you checked and it isn't real** — never because it "seems minor" or "seems unlikely." Filtering on vibes here re-introduces exactly the recall loss the coverage-first finding stage was built to prevent.
3. **Severity-order globally.** All 🔴 first across all buckets, then all 🟡, then 🟢 — not bucket-by-bucket and not in the order agents returned.
4. **Present in this format:**

```
## 🔴 Short title of the issue
`path/to/file.ts:123`

[Concrete description: what's wrong, what triggers it, what the impact is.]

**Fix:** [Specific edit to make.]
```

After the findings, include:

```
## Things I checked and verified fine

- [Suspicious-looking item that's actually OK, with a one-line reason. Merge
  these from all five subagents so the human doesn't re-investigate.]

## Recommendation

[Which fixes are blocking merge, which can be a follow-up, which can be skipped.
Be concrete about merge readiness.]
```

If every subagent returned zero findings and your verification confirms: say so explicitly. "Five Sonnet 4.6 subagents reviewed buckets A/B/C/D/E; all returned zero findings and I confirmed the highest-suspicion items. Recommend merge." Don't manufacture nitpicks — but be extra skeptical of zero findings from Bucket D (rare on non-trivial diffs) and Bucket E (rare on any diff that touches areas the project's reference files cover — schema, theming, env vars, error surfaces).

## Anti-patterns to avoid

- **Don't skip subagent dispatch to save tokens on a non-tiny diff.** The whole point of this skill is fresh context. Reviewing in the same session that wrote the code reintroduces the bias the skill exists to defeat.
- **Don't dispatch for a 20-line CSS tweak.** Use judgment — the tiny-diff inline path exists for a reason. (Caveat: if the CSS tweak touches only one theme, Bucket E would have caught it — for theme/CSS work, dispatch even on small diffs.)
- **Don't pass subagent findings through unchecked.** Verify every finding you intend to surface — not just the 🔴s, now that the finding stage over-reports by design. If a subagent hallucinates a function name or misreads the diff, the human pays the cost.
- **Don't praise the implementation.** "This looks well-structured" is not useful — find what's wrong.
- **Don't list findings in the order subagents returned them.** Severity-order globally so the human can triage top-down.
- **Don't mix "I haven't checked" with "I checked and it's fine."** They're different. State which.
- **Don't give generic advice ungrounded in the code.** Point at the specific line and say what to change.
- **Don't be defensive of code you wrote.** That's the easiest trap. Dispatching to fresh-context subagents is the structural fix; don't undermine it by overruling their findings without verification.
