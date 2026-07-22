#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptDir, "..", "..");
const failures = [];
const maxDescriptionChars = 240;
const maxCatalogChars = 7000;

function read(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function exists(relativePath) {
  return fs.existsSync(path.join(root, relativePath));
}

function frontmatter(relativePath) {
  const text = read(relativePath);
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);
  if (!match) {
    failures.push(`${relativePath}: missing YAML frontmatter`);
    return { name: "", description: "" };
  }

  const lines = match[1].split(/\r?\n/);
  for (const [index, line] of lines.entries()) {
    if (!line || /^\s/.test(line) || /^#/.test(line) || /^[A-Za-z0-9_-]+:\s*/.test(line)) continue;
    failures.push(`${relativePath}:${index + 2}: invalid unindented frontmatter continuation`);
  }

  const value = (key) => {
    const index = lines.findIndex((line) => line.startsWith(`${key}:`));
    if (index < 0) return "";
    const raw = lines[index].slice(key.length + 1).trim();
    if (raw === ">" || raw === "|") {
      const block = [];
      for (let cursor = index + 1; cursor < lines.length && /^\s/.test(lines[cursor]); cursor += 1) {
        block.push(lines[cursor].trim());
      }
      return block.join(" ").trim();
    }
    if (raw.startsWith('"')) return JSON.parse(raw);
    if (raw.startsWith("'") && raw.endsWith("'")) return raw.slice(1, -1).replaceAll("''", "'");
    return raw;
  };

  return { name: value("name"), description: value("description") };
}

const settings = JSON.parse(read(".claude/settings.json"));
const disabled = new Set(
  Object.entries(settings.skillOverrides ?? {})
    .filter(([, state]) => state === "off")
    .map(([name]) => name),
);

const skillsRoot = path.join(root, ".claude", "skills");
const skills = fs.readdirSync(skillsRoot, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && !disabled.has(entry.name))
  .filter((entry) => fs.existsSync(path.join(skillsRoot, entry.name, "SKILL.md")))
  .map((entry) => {
    const relativePath = `.claude/skills/${entry.name}/SKILL.md`;
    const metadata = frontmatter(relativePath);
    if (metadata.name && metadata.name !== entry.name) {
      failures.push(`${relativePath}: declared name ${metadata.name} does not match directory ${entry.name}`);
    }
    return {
      directory: entry.name,
      name: metadata.name || entry.name,
      description: metadata.description,
    };
  });

let catalogChars = 0;
for (const skill of skills) {
  if (!skill.description) failures.push(`${skill.directory}: missing description`);
  if (skill.description.length > maxDescriptionChars) {
    failures.push(`${skill.directory}: description is ${skill.description.length} chars (max ${maxDescriptionChars})`);
  }
  catalogChars += skill.name.length + skill.description.length;
}
const duplicateNames = skills
  .map((skill) => skill.name)
  .filter((name, index, names) => names.indexOf(name) !== index);
for (const name of new Set(duplicateNames)) failures.push(`${name}: duplicate active skill name`);
if (catalogChars > maxCatalogChars) {
  failures.push(`Codex skill catalog is ${catalogChars} chars (max ${maxCatalogChars})`);
}

const compatibility = read(".agents/CODEX-SKILL-COMPATIBILITY.md");
const classifications = new Map();
for (const line of compatibility.split(/\r?\n/)) {
  const match = line.match(/^\|\s*(Native|Adapted|Capability-gated|Dangerous|Claude-only)\s*\|(.+)\|$/);
  if (!match) continue;
  for (const skill of match[2].matchAll(/`([^`]+)`/g)) {
    const statuses = classifications.get(skill[1]) ?? [];
    statuses.push(match[1]);
    classifications.set(skill[1], statuses);
  }
}

for (const skill of skills) {
  const statuses = classifications.get(skill.directory) ?? [];
  if (statuses.length !== 1) {
    failures.push(`${skill.directory}: expected one compatibility classification, found ${statuses.length}`);
  }
}
for (const skill of classifications.keys()) {
  if (!skills.some((entry) => entry.directory === skill)) {
    failures.push(`${skill}: compatibility classification has no active canonical skill`);
  }
}

const initProject = read(".claude/skills/init-project/SKILL.md");
for (const target of [".claude-plugin/", "bootstrap/", ".github/workflows/validate-template.yml"]) {
  if (!initProject.includes(target)) failures.push(`init-project: cleanup contract omits ${target}`);
}

const corePath = "bootstrap/NewProjectCore.psm1";
if (!exists(corePath)) {
  failures.push(`${corePath}: shared project generator is missing`);
} else {
  const core = read(corePath);
  for (const target of ["bootstrap", ".claude-plugin", ".github\\workflows\\validate-template.yml"]) {
    if (!core.includes(target)) failures.push(`${corePath}: cleanup contract omits ${target}`);
  }
  for (const command of ["Test-NewProjectName", "Get-NewProjectMode", "Invoke-NewProject"]) {
    if (!core.includes(command)) failures.push(`${corePath}: does not expose ${command}`);
  }
  if (!core.includes("'archive'")) failures.push(`${corePath}: local repo copies must use a tracked Git archive`);
}

for (const wrapper of ["bootstrap/new-claude-project.ps1", "bootstrap/new-claude-project-ui.ps1"]) {
  const text = read(wrapper);
  if (!text.includes("NewProjectCore.psm1")) failures.push(`${wrapper}: does not load the shared project generator`);
  for (const duplicatedCommand of ["gh repo create", "robocopy"]) {
    if (text.includes(duplicatedCommand)) failures.push(`${wrapper}: duplicates core command ${duplicatedCommand}`);
  }
}

const releaseBuilderPath = "bootstrap/Build-NewClaudeProjectUIRelease.ps1";
if (!exists(releaseBuilderPath)) {
  failures.push(`${releaseBuilderPath}: reproducible UI release builder is missing`);
} else {
  const releaseBuilder = read(releaseBuilderPath);
  for (const asset of ["New-ClaudeProject-UI.cmd", "new-claude-project-ui.ps1", "NewProjectCore.psm1", "template"]) {
    if (!releaseBuilder.includes(asset)) failures.push(`${releaseBuilderPath}: release manifest omits ${asset}`);
  }
  if (!releaseBuilder.includes("archive")) failures.push(`${releaseBuilderPath}: template snapshot is not sourced from tracked Git files`);
}

const generatorSmokePath = "bootstrap/tests/Test-NewProjectGenerator.ps1";
if (!exists(generatorSmokePath)) {
  failures.push(`${generatorSmokePath}: local generator smoke test is missing`);
} else {
  const workflow = read(".github/workflows/validate-template.yml");
  if (!workflow.includes("windows-latest")) failures.push("validate-template.yml: generator smoke test needs a Windows runner");
  if (!workflow.includes("Test-NewProjectGenerator.ps1")) failures.push("validate-template.yml: generator smoke test is not wired into CI");
}

if (failures.length) {
  for (const failure of failures) console.error(`FAIL: ${failure}`);
  process.exit(1);
}

console.log(`Codex contract checks passed (${skills.length} skills, ${catalogChars} catalog chars).`);
