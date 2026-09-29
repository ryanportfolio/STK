---
name: compact-review
description: "Use on /compact-review or right before /compact: review the session and return copy-ready custom instructions for /compact that keep what matters. Does not run /compact."
disable-model-invocation: true
---

# Compact review

Review the whole session so far and write the custom instructions the user will paste after `/compact`. The summarizer reads them as directions for what to keep, so write imperatives to it.

Keep, in priority order:

1. The user's goal for the session and any standing instructions or preferences they gave in it (style, scope limits, things to never do).
2. Current state: repo, branch, worktree path, PR URL, last commit SHA, deploy or preview URL, files created or changed.
3. Decisions made and the reason for each, including options the user rejected.
4. What is verified and how, and what is still unverified.
5. Open work: the next concrete step, pending questions to the user, background tasks still running.
6. Failed approaches and gotchas that would otherwise be retried.
7. Exact identifiers to keep verbatim: paths, commands, error strings, IDs, numbers.

Tell the summarizer to drop resolved tangents, raw tool output, superseded plans, and anything re-readable from files or CLAUDE.md. Never invent a fact; if a state item is unknown, leave it out. Keep the block under about 300 words; cut from the bottom of the priority list first.

## Output

Return exactly one fenced `text` block and nothing else except one short line after it if something important could not be captured. The first line of the block is always `Always use caveman ultra.`, whatever style the session is using.

```text
Always use caveman ultra.
Goal: <one line>.
Keep: <standing user instructions>.
State: <branch, worktree, PR, SHA, files>.
Decisions: <decision, why>; ...
Verified: <what, how>. Unverified: <what>.
Next: <next step>; open questions: <...>.
Don't retry: <failed approach, cause>.
Verbatim: <paths, commands, errors>.
Drop tool output, resolved tangents, and superseded plans.
```

Omit any line with nothing to say. Do not run `/compact`, write a file, or change anything else.
