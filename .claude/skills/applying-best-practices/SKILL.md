---
description: Use before non-trivial features, refactors, performance work, bug fixes, or applying a performance-review finding in web or TypeScript code.
---

# Applying best practices

This is a checklist consulted during implementation work. It does two things:

1. **Lists the practices** that commonly apply to web/TypeScript stacks. **Tune
   this catalog to the project**: delete rules that don't apply to this stack,
   add stack-specific ones, and record real project examples as they accumulate
   (via `/recall save`).
2. **Encodes the discipline** of investigating intent before "fixing"
   apparent violations — because code that looks wrong is often intentional.

## The discipline (read this first)

When you spot what looks like a best-practice violation:

1. **Open the surrounding code.** Read the function, the callers, the file's
   history if relevant. A 5-minute investigation prevents an hour of
   regression debugging.
2. **Ask "why might this be intentional?"** Common reasons:
   - Sequential awaits because order matters (caching, rate limits, side effects).
   - Eager imports because the code path runs on every page.
   - `useState` + `useEffect` because the source is async or external.
   - Raw `fetch` because the call is one-shot (e.g., file export, not a query).
3. **Classify the fix's risk:**
   - **Zero-risk:** purely additive (`{ passive: true }`, hoisting a regex,
     unifying a query key). Apply freely.
   - **Low-risk:** semantic-preserving refactor with a clear rollback (N+1 →
     single query that returns the same shape). Apply with a sanity check.
   - **Behavioral:** changes timing, ordering, or side-effects. Stop and
     check with the user.
4. **Never bundle "fixes" that span risk categories** into one commit —
   you lose the ability to bisect a regression to the actual cause.

If a "fix" requires comments like "TODO: verify this still works" or
"should be equivalent" — you haven't verified enough yet.

## The catalog (generic web/TS baseline — tune per project)

### Async / IO (highest leverage)

- **Run independent awaits in `Promise.all`** — especially in route handlers
  that hit multiple external services or DB queries.
- **Cheap sync checks before expensive awaits** — auth gates, feature flags,
  early-return validation should short-circuit before any DB/network call.
- **No N+1 over fat rows** — if a route loops `await getX(id)` per parent
  record, it's almost always a single grouped query in disguise
  (LEFT JOIN + COUNT, `WHERE id IN (...)`).

### Server caching

- **Module-level cache for hot read-only data** (config files, status data).
- **No request-scoped state in module variables** — keep in-memory maps
  keyed by stable IDs, TTL-evicted.
- **Don't re-read static files per request** — hoist file loads to module init.

### Bundle size

- **Route-level code splitting via `React.lazy` + `Suspense`** (or the
  framework's equivalent) for top-level pages.
- **Heavy components behind dynamic `import()`** — 3D, file-upload, PDF libs.
- **Granular imports** — `import { X } from 'lib'`, not barrels, for
  icon/utility libraries that support it.
- **Manual chunking for route-specific heavy deps** if the bundler supports it.

### Client data fetching (query-cache libraries)

- **Stable, parameterized query keys** — `['things', { limit }]` not
  `['things']` when params vary. Different params = different cache entry.
- **One key per logical resource across components** — if two components
  fetch the same data, they MUST use the same key, or the cache desyncs.
- **Don't use raw `fetch` for cacheable GETs** — use the query library. OK for
  one-shot user-triggered actions (file exports, form submits) where
  dedup/caching aren't wanted.

### React re-renders

- **Derive state during render or in `useMemo`**, not in `useState` + `useEffect`.
- **`{ passive: true }` on scroll/touch listeners** that never call
  `preventDefault`.
- **Hoist regex/Set/Map construction** out of hot render or callback paths.
- **Use `IntersectionObserver` instead of scroll listeners** when you
  only care about a threshold crossing, not continuous position.
- **`memo()` for list items** when a list parent re-renders frequently and
  item props are stable.

### Rendering

- **Long lists need either virtualization or `content-visibility: auto`** —
  don't render 1000 DOM nodes if 50 are visible.
- **`useTransition` / `useDeferredValue`** for filter inputs over large lists.
- **No components defined inside other components' render** — they're
  recreated each render and lose state.

### JS perf (micro)

- **`Set` / `Map` for repeated lookups** — `.find()` or `.includes()` in
  a render loop is O(n²).
- **Combine `.filter().map().filter()` chains** into one loop for large arrays.
- **Cache property access in hot loops.**

## Project-specific gotchas the catalog doesn't cover

See `.claude/reference/pitfalls.md` for this project's accumulated traps.
Add new ones there via `/recall save` — not here.

For verification constraints (what this sandbox can and can't run), see
CLAUDE.md's verification section.

## When to fire this skill

- BEFORE implementing a non-trivial feature, refactor, or perf fix.
- When the user asks to "optimize", "make X faster", "fix this perf issue".
- When applying a code-review finding — to make sure the "fix" doesn't
  break the thing the original code was doing intentionally.
- When you spot what looks like a best-practice violation in unfamiliar
  code — pause, investigate, then decide.
