import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const script = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../skills/long-horizon/scripts/manifest.mjs");
const run = (...args) => {
  const r = spawnSync(process.execPath, [script, ...args], { encoding: "utf8" });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout;
};

function repo(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "lh-manifest-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const root = path.join(dir, "ws");
  fs.mkdirSync(root);
  const git = (...a) => execFileSync("git", ["-C", root, "-c", "user.name=t", "-c", "user.email=t@t", "-c", "core.autocrlf=false", ...a], { encoding: "utf8" });
  const write = (p, s) => {
    fs.mkdirSync(path.dirname(path.resolve(root, p)), { recursive: true });
    fs.writeFileSync(path.resolve(root, p), s);
  };
  git("init", "-q");
  for (const f of ["a.txt", "b.txt", "c.txt", "d.txt", "e.txt", "same.txt", "mv.txt"]) write(f, `${f} v1\n`);
  write(".gitignore", "out/\n");
  git("add", "-A");
  git("commit", "-qm", "init");
  return { dir, root, git, write };
}

test("diff reports exactly the executor's changes, including reverts and commits", (t) => {
  const { dir, root, git, write } = repo(t);
  const outside = path.join(dir, "outside.txt");
  fs.writeFileSync(outside, "outside v1\n");
  // Pre-existing state at baseline: dirty, untracked, staged rename, ignored artifact.
  write("a.txt", "a.txt dirty\n");
  write("d.txt", "d.txt dirty\n");
  write("u.txt", "untracked\n");
  write("out/r.json", "{}\n");
  git("mv", "e.txt", "e2.txt");
  const baseline = path.join(dir, "baseline.json");
  run(root, baseline, "--ref", "refs/long-horizon/t/round-1", "out", outside);

  const base = JSON.parse(fs.readFileSync(baseline, "utf8"));
  assert.equal(git("rev-parse", "refs/long-horizon/t/round-1").trim(), base.snapshot);
  assert.notEqual(base.snapshot, base.head);
  for (const k of ["a.txt", "d.txt", "u.txt", "out/r.json", "e2.txt", outside.replaceAll("\\", "/")]) assert.match(base.files[k], /^[0-9a-f]{64}$/, k);
  assert.equal(base.files["e.txt"], "DELETED");
  assert.equal(base.files["b.txt"], undefined);

  // The executor's round.
  git("checkout", "--", "a.txt"); // dirty at baseline, reverted to HEAD
  write("b.txt", "b.txt v2\n");
  git("commit", "-qm", "executor commit", "--", "b.txt"); // hidden from git status
  git("commit", "-qm", "commit baseline edit as is", "--", "d.txt"); // unchanged content, now clean
  write("same.txt", "same.txt v2\n");
  git("commit", "-qm", "change then restore", "--", "same.txt");
  write("same.txt", "same.txt v1\n"); // dirty against HEAD, equal to the baseline
  git("mv", "mv.txt", "moved.txt");
  git("commit", "-qm", "committed rename"); // rename detection would hide the source
  fs.unlinkSync(path.join(root, "c.txt"));
  write("n.txt", "new\n");
  write("out/r.json", "{\"x\":1}\n");
  fs.unlinkSync(path.join(root, "u.txt"));
  fs.writeFileSync(outside, "outside v2\n");

  const current = path.join(dir, "current.json");
  const d = JSON.parse(run("--diff", baseline, current));
  assert.deepEqual(d.added, ["moved.txt", "n.txt"]);
  assert.deepEqual(d.modified, ["a.txt", "b.txt", outside.replaceAll("\\", "/"), "out/r.json"].sort());
  assert.deepEqual(d.deleted, ["c.txt", "mv.txt", "u.txt"]);
  assert.deepEqual(d.uncovered, []);
  assert.notEqual(d.headNow, d.headAtBaseline);
  assert.ok(fs.existsSync(current));
});

test("clean workspace diffs empty; links are recorded, not followed", (t) => {
  const { dir, root, write } = repo(t);
  const shared = path.join(dir, "shared");
  fs.mkdirSync(shared);
  fs.writeFileSync(path.join(shared, "big.js"), "x");
  write("out/keep.txt", "k\n");
  fs.symlinkSync(shared, path.join(root, "out", "node_link"), process.platform === "win32" ? "junction" : "dir");
  const baseline = path.join(root, "lh-baseline.json"); // inside the workspace, untracked
  run(root, baseline, "out");
  const base = JSON.parse(fs.readFileSync(baseline, "utf8"));
  assert.equal(base.snapshot, base.head);
  assert.equal(base.files["lh-baseline.json"], undefined);
  assert.match(base.files["out/node_link"], /^link:/);
  assert.equal(Object.keys(base.files).some((k) => k.includes("big.js")), false);
  const d = JSON.parse(run("--diff", baseline));
  assert.deepEqual([d.added, d.modified, d.deleted], [[], [], []]);
});

test("an unreadable snapshot is reported as uncovered, not as a clean result", (t) => {
  const { dir, root, git, write } = repo(t);
  const baseline = path.join(dir, "baseline.json");
  run(root, baseline);
  write("b.txt", "b.txt v2\n");
  git("commit", "-qam", "executor commit");
  const base = JSON.parse(fs.readFileSync(baseline, "utf8"));
  fs.writeFileSync(baseline, JSON.stringify({ ...base, snapshot: "0".repeat(40) }));
  const d = JSON.parse(run("--diff", baseline));
  assert.match(d.uncovered.join("\n"), /snapshot 0{40} unavailable/);
});

test("odd names, a folder replaced by a junction, and a path unreadable at baseline", (t) => {
  const { dir, root, git, write } = repo(t);
  write("..config", "v1\n");
  git("add", "-A");
  git("commit", "-qm", "dotdot");
  write("out/a.txt", "a\n");
  write("logs/secret.txt", "s\n");
  const baseline = path.join(dir, "baseline.json");
  run(root, baseline, "out", "logs");
  // Simulate a path the baseline could not read.
  const base = JSON.parse(fs.readFileSync(baseline, "utf8"));
  delete base.files["logs/secret.txt"];
  base.uncovered.push("unreadable: logs/secret.txt (EACCES)");
  fs.writeFileSync(baseline, JSON.stringify(base));

  write("..config", "v2\n");
  write("__proto__", "p\n");
  write("constructor", "c\n");
  const shared = path.join(dir, "shared");
  fs.mkdirSync(shared);
  fs.writeFileSync(path.join(shared, "a.txt"), "outside content\n");
  fs.rmSync(path.join(root, "out"), { recursive: true });
  fs.symlinkSync(shared, path.join(root, "out"), process.platform === "win32" ? "junction" : "dir");

  const d = JSON.parse(run("--diff", baseline));
  assert.deepEqual(d.added, ["__proto__", "constructor", "out"]);
  assert.deepEqual(d.modified, ["..config"]);
  assert.deepEqual(d.deleted, ["out/a.txt"]);
  assert.match(d.uncovered.join("\n"), /unreadable at baseline: logs\/secret\.txt/);
});

test("a directory outside git is walked whole and flagged", (t) => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "lh-manifest-nogit-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const root = path.join(dir, "ws");
  fs.mkdirSync(path.join(root, "node_modules"), { recursive: true });
  fs.writeFileSync(path.join(root, "f.txt"), "1");
  fs.writeFileSync(path.join(root, "node_modules", "m.js"), "1");
  const baseline = path.join(dir, "baseline.json");
  run(root, baseline);
  const base = JSON.parse(fs.readFileSync(baseline, "utf8"));
  assert.deepEqual(Object.keys(base.files), ["f.txt"]);
  assert.equal(base.snapshot, null);
  assert.equal(base.uncovered.length, 1);
  fs.writeFileSync(path.join(root, "f.txt"), "2");
  fs.writeFileSync(path.join(root, "g.txt"), "1");
  const d = JSON.parse(run("--diff", baseline));
  assert.deepEqual([d.added, d.modified, d.deleted], [["g.txt"], ["f.txt"], []]);
});

test("bad arguments exit 2", () => {
  assert.equal(spawnSync(process.execPath, [script], { encoding: "utf8" }).status, 2);
  assert.equal(spawnSync(process.execPath, [script, ".", "x.json", "--ref"], { encoding: "utf8" }).status, 2);
});
