# setup-machine.ps1 - "dotfiles for Claude": seed ~/.claude on a new machine.
#
# Projects carry their own .claude/ config, but machine-level files do not
# travel with any repo: the global ~/.claude/CLAUDE.md, keybindings.json,
# and personal (non-project) skills. Keep your copies of those files in
# bootstrap/machine/home-claude/ in YOUR fork/clone of this template, and
# run this script once per new machine.
#
# What it does:
#   1. Mirrors bootstrap/machine/home-claude/** into ~/.claude/**
#   2. Never overwrites an existing file unless -Force is passed.
#   3. Skips *.example files (they are documentation, not config).
#
# Usage:
#   .\bootstrap\setup-machine.ps1           # copy missing files only
#   .\bootstrap\setup-machine.ps1 -Force    # overwrite existing files too
#   .\bootstrap\setup-machine.ps1 -DryRun   # show what would happen

param(
    [switch]$Force,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

$src = Join-Path $PSScriptRoot 'machine\home-claude'
$dst = Join-Path $HOME '.claude'

if (-not (Test-Path $src)) {
    Write-Host "Nothing to do: $src does not exist." -ForegroundColor Yellow
    Write-Host "Put your machine-level files there (CLAUDE.md, keybindings.json, skills\...) and re-run."
    exit 0
}

$files = Get-ChildItem $src -Recurse -File | Where-Object { $_.Name -notlike '*.example' }
if (-not $files) {
    Write-Host "Nothing to do: $src has no non-example files yet." -ForegroundColor Yellow
    exit 0
}

if (-not (Test-Path $dst) -and -not $DryRun) {
    New-Item -ItemType Directory -Force $dst | Out-Null
}

$copied = 0
$skipped = 0
foreach ($f in $files) {
    $rel = $f.FullName.Substring($src.Length).TrimStart('\', '/')
    $out = Join-Path $dst $rel
    if ((Test-Path $out) -and -not $Force) {
        Write-Host "skip (exists): $rel"
        $skipped++
        continue
    }
    if ($DryRun) {
        Write-Host "would copy:    $rel"
        $copied++
        continue
    }
    $outDir = Split-Path $out -Parent
    if (-not (Test-Path $outDir)) { New-Item -ItemType Directory -Force $outDir | Out-Null }
    Copy-Item $f.FullName $out -Force
    Write-Host "copied:        $rel" -ForegroundColor Green
    $copied++
}

Write-Host ""
Write-Host "Done. $copied copied, $skipped skipped (already present)." -ForegroundColor Cyan
if ($skipped -gt 0 -and -not $Force) {
    Write-Host "Re-run with -Force to overwrite existing files."
}
