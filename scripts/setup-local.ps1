# setup-local.ps1 — one-shot local activation for STK's meter pipeline.
#
# Does two things, both idempotent:
#   1. Adds the STK PreToolUse hook (matcher "Read" -> "stk hook claude") to
#      %USERPROFILE%\.claude\settings.json, merging into existing hooks.
#      A timestamped backup of settings.json is written first.
#   2. Registers a daily Windows scheduled task "STK stats publish" that runs
#      scripts\publish-stats.ps1 from this repo checkout at 21:00.
#
# Note: the stk BINARY never edits settings (see `stk init`). This script is
# repo tooling the operator runs on purpose; it edits settings so you don't
# paste by hand. Re-running is safe: existing hook and task are detected.
#
# Usage:  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\setup-local.ps1
#         [-Time 21:00]  (scheduled-task time, HH:mm)

param(
    [string]$Time = "21:00"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$SettingsPath = Join-Path $env:USERPROFILE ".claude\settings.json"
$PublishScript = Join-Path $RepoRoot "scripts\publish-stats.ps1"
$TaskName = "STK stats publish"

# --- 0. sanity: stk answers the hook contract ---
$null = "{}" | & stk hook claude
if ($LASTEXITCODE -ne 0) { throw "echo {} | stk hook claude failed (exit $LASTEXITCODE); is stk on PATH?" }
Write-Host "[ok] stk hook claude answers (exit 0)"

# --- 1. merge hook into settings.json ---
if (-not (Test-Path $SettingsPath)) { throw "settings.json not found at $SettingsPath" }
$settings = Get-Content $SettingsPath -Raw | ConvertFrom-Json

$hasHook = $false
if ($settings.hooks -and $settings.hooks.PreToolUse) {
    foreach ($entry in $settings.hooks.PreToolUse) {
        if ($entry.matcher -eq "Read") {
            foreach ($h in $entry.hooks) {
                if ($h.command -match "stk\s+hook\s+claude") { $hasHook = $true }
            }
        }
    }
}

if ($hasHook) {
    Write-Host "[ok] STK hook already present in settings.json; not touching it"
} else {
    $backup = "$SettingsPath.bak-$(Get-Date -Format yyyyMMdd-HHmmss)"
    Copy-Item $SettingsPath $backup
    Write-Host "[ok] settings.json backed up to $backup"

    $stkEntry = [pscustomobject]@{
        matcher = "Read"
        hooks   = @([pscustomobject]@{ type = "command"; command = "stk hook claude" })
    }

    if (-not $settings.hooks) {
        $settings | Add-Member -NotePropertyName hooks -NotePropertyValue ([pscustomobject]@{})
    }
    if (-not $settings.hooks.PreToolUse) {
        $settings.hooks | Add-Member -NotePropertyName PreToolUse -NotePropertyValue @()
    }
    $settings.hooks.PreToolUse = @($settings.hooks.PreToolUse) + $stkEntry

    $json = $settings | ConvertTo-Json -Depth 12
    Set-Content -Path $SettingsPath -Value $json -Encoding UTF8
    Write-Host "[ok] STK PreToolUse hook added to settings.json (new sessions pick it up)"
}

# --- 2. daily scheduled task for the stats snapshot ---
if (-not (Test-Path $PublishScript)) { throw "publish script not found at $PublishScript" }
$action = "powershell -NoProfile -ExecutionPolicy Bypass -File `"$PublishScript`""
& schtasks /Create /F /TN $TaskName /SC DAILY /ST $Time /TR $action | Out-Null
if ($LASTEXITCODE -ne 0) { throw "schtasks /Create failed (exit $LASTEXITCODE)" }
Write-Host "[ok] scheduled task '$TaskName' set: daily $Time -> $PublishScript"

Write-Host ""
Write-Host "Done. The meter starts moving after your next Claude Code sessions clamp a read;"
Write-Host "the site updates when the nightly task pushes the next snapshot."
