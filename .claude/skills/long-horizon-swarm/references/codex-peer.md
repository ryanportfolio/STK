# Peer copy, launch and reply mechanics (Git Bash on Windows)

Used by `long-horizon-swarm`. Checked against codex-cli 0.156.1 and GNU tar 1.35 in Git Bash.
Read `codex exec --help` and `codex exec resume --help` again when the CLI version changes.

## Shell rules

- Each Bash tool call starts a new shell: variables and cwd do not carry over. Put absolute
  paths into every call.
- Use Git Bash paths (`/c/Users/...`; `cygpath -u` converts). GNU tar reads `C:` in an
  archive name as a remote host and fails with `Cannot connect to C: resolve failed`.
- Launch one peer per Bash call. A call that sets a variable and then starts two
  `( ... ) &` groups can lose the variable in the second group.

Below, `<root>` is the workspace root in Git Bash form, `<slug>` the run's task slug, and
`S<n>` the entry id.

## Entry directory

One per contribution: `<root>/.tmp/long-horizon/<slug>/swarm/S<n>/`. It holds `brief.md`,
the copy `ws/`, and after launch `report.md`, `run.log`, `pid`, `winpid` and `exit.code`.
Write the S entry in `swarm.md` with `status: running` and `output:` set to this directory
when you launch.

## Baseline snapshot

With Swarm on, Plan takes this right after the Baseline manifest, before the executor is
dispatched. With the Workflow engine, whose script takes the Baseline and dispatches the
executor in one call, take it right before the workflow call, with no writer active:

```bash
mkdir -p <root>/.tmp/long-horizon/<slug>/swarm && tar -C <root> --exclude='./.git' --exclude='./.tmp/long-horizon/*/swarm' -cf <root>/.tmp/long-horizon/<slug>/swarm/baseline-r<N>.tar .
```

## Copy

While the round is `executing` or `awaiting-audit`, copy from the Baseline snapshot:

```bash
mkdir -p <root>/.tmp/long-horizon/<slug>/swarm/S<n>/ws && tar -C <root>/.tmp/long-horizon/<slug>/swarm/S<n>/ws -xf <root>/.tmp/long-horizon/<slug>/swarm/baseline-r<N>.tar
```

In any other phase, copy the live workspace, including untracked and ignored inputs:

```bash
set -o pipefail; mkdir -p <root>/.tmp/long-horizon/<slug>/swarm/S<n>/ws && tar -C <root> --exclude='./.git' --exclude='./.tmp/long-horizon/*/swarm' -cf - . | tar -C <root>/.tmp/long-horizon/<slug>/swarm/S<n>/ws -xf -
```

Both omit Git metadata and earlier swarm entries. A worktree's `.git` is a pointer file back
to the original repository, so it must stay out of the copy.

The copy has no `.git`: `codex exec` needs `--skip-git-repo-check`, and the peer cannot run
`git diff`. When its lens needs a diff, the host writes one into the entry directory and
names it in the brief, e.g. `git -C <root> diff <base> > .../S<n>/diff.patch`. `git diff`
leaves out untracked files; list them from `git -C <root> ls-files --others
--exclude-standard` so the peer can read them in `ws/`.

## Brief

`brief.md` carries the contract and amendments, the round and the peer's lens (absolute path
of the lens file or skill, or the stated standard for the built-in `security-review`), the
verified facts and dead ends it needs, artifact paths, the task, whether it may edit `ws/`,
and the return format. A code peer reports the paths it changed. Never name the audited
workspace as a write target.

## Launch a Codex peer

Model: the Sol id pinned in `.claude/skills/codex-review/SKILL.md` at `high`, or
`gpt-6-astra` at `medium`. Sandbox: `-s read-only` for ideas and critique,
`-s workspace-write` for code. Both run with `-C` set to the copy.

```bash
E=<root>/.tmp/long-horizon/<slug>/swarm/S<n>; ( codex exec -C "$E/ws" --skip-git-repo-check -s <read-only|workspace-write> -m <sol id> -c model_reasoning_effort=high -o "$E/report.md" - < "$E/brief.md" > "$E/run.log" 2>&1; printf '%s\n' "$?" > "$E/exit.code" ) < /dev/null > /dev/null 2>&1 & printf '%s\n' "$!" > "$E/pid"; cat "/proc/$!/winpid" > "$E/winpid"
```

An Opus peer runs through the `Agent` tool without `isolation`; its brief names the copy as
its only write target. Record its agent id in the S entry.

## Collect

A Codex peer is done when `exit.code` exists. Accept `report.md` only with exit code 0 and a
nonempty file, then set `status: returned`; otherwise set `status: failed` and keep
`run.log`. Read the `model:`, `reasoning effort:`, `sandbox:` and `session id:` lines from
the `run.log` header and add the session id to the S entry. A model rejection
(`model_not_found`, `unknown provider for model`) is a failed entry reported to the user;
never rerun it without `-m` or on another model.

## Stop

`kill` on the recorded `pid` ends only the Git Bash subshell; `codex.exe` keeps running.
Stop the whole process tree through its Windows PID:

```bash
taskkill //PID "$(cat <root>/.tmp/long-horizon/<slug>/swarm/S<n>/winpid)" //T //F
```

No `exit.code` is written. Set `status: failed` with reason `stopped`.

## Reply to a critique

The critique goes back to a Codex author by resuming its session. `codex exec resume` has
no `-C` or `-s`, so run it from the author's copy and set the sandbox through config. Put
the critique and the ask (concede, or refute with evidence) in `reply-<k>-in.md`:

```bash
E=<root>/.tmp/long-horizon/<slug>/swarm/S<n>; ( cd "$E/ws" && codex exec resume --skip-git-repo-check -c 'sandbox_mode="read-only"' -m <model> -c model_reasoning_effort=<effort> -o "$E/reply-<k>.md" <session id> - < "$E/reply-<k>-in.md" > "$E/reply-<k>.log" 2>&1; printf '%s\n' "$?" > "$E/reply-<k>.exit" ) < /dev/null > /dev/null 2>&1 & printf '%s\n' "$!" > "$E/reply-<k>.pid"; cat "/proc/$!/winpid" > "$E/reply-<k>.winpid"
```

A code peer may keep `workspace-write`, since `ws/` is its own copy. A round executor always
replies with `sandbox_mode="read-only"`: first make a live-workspace copy in
`executor-r<N>/ws/` (see Copy) and resume from there, never from the workspace root. What it
concedes becomes the next round's work. Check the `workdir:` and `sandbox:` lines in the reply log header; if either
differs from the original run, stop it and brief a fresh peer with the artifact and the
critique instead. Log the resume as its own S entry of kind `reply`.

## Codex as round executor

The executor works on the workspace root, which is a Git checkout, so it needs no
`--skip-git-repo-check`. Keep its files under `<root>/.tmp/long-horizon/<slug>/swarm/executor-r<N>/`
with the executor brief as `brief.md`:

```bash
X=<root>/.tmp/long-horizon/<slug>/swarm/executor-r<N>; ( codex exec -C <root> -s workspace-write -m <model> -c model_reasoning_effort=<effort> -o "$X/report.md" - < "$X/brief.md" > "$X/run.log" 2>&1; printf '%s\n' "$?" > "$X/exit.code" ) < /dev/null > /dev/null 2>&1 & printf '%s\n' "$!" > "$X/pid"; cat "/proc/$!/winpid" > "$X/winpid"
```

Record the `session id:` from its `run.log` header under `Workers:`. Its report is for the
host only: it sits under `swarm/`, which every auditor brief excludes by path.

Do not use `codex exec review` for any of this: peers and executors need a custom brief,
not a review scope selector.
