# Configure detected Claude Code/Codex hooks and register the local stats publisher.
# Usage: powershell -File scripts/setup-local.ps1 [-Time 21:00]

param(
    [string]$Time = "21:00"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$PublishScript = Join-Path $RepoRoot "scripts\publish-stats.ps1"
$TaskName = "STK stats publish"

# Install detected clients through the shared, backup-preserving installer.
& stk init --auto
if ($LASTEXITCODE -ne 0) { throw "stk init --auto failed (exit $LASTEXITCODE); upgrade STK first" }
Write-Host "[ok] detected client hooks configured; review Codex trust with /hooks"

# --- 2. daily scheduled task for the stats snapshot ---
if (-not (Test-Path $PublishScript)) { throw "publish script not found at $PublishScript" }
$action = "powershell -NoProfile -ExecutionPolicy Bypass -File `"$PublishScript`""
& schtasks /Create /F /TN $TaskName /SC DAILY /ST $Time /TR $action | Out-Null
if ($LASTEXITCODE -ne 0) { throw "schtasks /Create failed (exit $LASTEXITCODE)" }

# schtasks can't set these: catch up ASAP after a missed run, and run on battery.
$taskSettings = New-ScheduledTaskSettingsSet -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
Set-ScheduledTask -TaskName $TaskName -Settings $taskSettings | Out-Null
Write-Host "[ok] scheduled task '$TaskName' set: daily $Time -> $PublishScript"
Write-Host "     (catches up as soon as possible if the $Time run is missed; runs on battery)"

Write-Host ""
Write-Host "Done. The meter starts moving after Claude Code or Codex sessions clamp a read;"
Write-Host "the site updates when the nightly task pushes the next snapshot."
