# Deployment

> Deploy target, build output, asset paths, publish flow.

## Site (GitHub Pages)

- Public URL: https://savetokens.tips/stk/ (the repo homepage field; canonical in index.html).
  Served by the savetokens Vercel project via rewrite `/stk/:path(.*)` ->
  `https://ryanportfolio.github.io/STK/:path` (see savetokens repo's deployment.md).
- Origin: https://ryanportfolio.github.io/STK/ — pushes here update /stk with no savetokens deploy.
- Source: `main` branch, `/docs` folder, legacy (Jekyll) build — no workflow file.
  Any push to `main` touching `docs/` redeploys; builds take ~1–2 min.
- Static only: `docs/index.html` + `styles.css` + `app.js`, no build step.
  Design language shared with fullbuild.ai/prototype/burn-in.

## Stats pipeline

- `docs/data/gain.json` = committed snapshot: `{generated_at, stk: <stk gain --json>, rtk: parsed from rtk gain}`.
  The page fetches it at load; missing/zero data renders the cold-start STANDBY state.
- `scripts/publish-stats.ps1` regenerates it and commits+pushes only on change.
  Runs from whatever checkout it lives in — the scheduled task points at the
  MAIN checkout `C:\Users\Home\CoreWise\STK`, never a worktree.
- Before generating data, the publisher requires a clean `main`, fetches
  `origin/main`, and rebases any unpublished snapshot commits. Every Git command
  is exit-checked, so a failed sync or push reaches Task Scheduler as a failure.
- Windows scheduled task "STK stats publish", daily 21:00 (registered 2026-07-21
  by `scripts/setup-local.ps1`, which also merged the STK PreToolUse hook into
  `~/.claude/settings.json` with a timestamped backup).
