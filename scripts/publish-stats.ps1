# publish-stats.ps1 — snapshot `stk gain --json` (+ RTK's own numbers) into
# docs/data/gain.json and push it, so the GitHub Pages site renders real data.
#
# Intended to run from a daily scheduled task on the author's machine:
#   schtasks /Create /TN "STK stats publish" /SC DAILY /ST 21:00 /TR ^
#     "powershell -NoProfile -ExecutionPolicy Bypass -File <repo>\scripts\publish-stats.ps1"
#
# Safe to run by hand. Commits only docs/data/gain.json, only when it changed.

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$OutFile = Join-Path $RepoRoot "docs\data\gain.json"

# --- STK: the authoritative JSON contract ---
# Scheduled tasks often run without the user's PATH; fall back to the install path.
$stkExe = "stk"
if (-not (Get-Command stk -ErrorAction SilentlyContinue)) {
    $stkExe = Join-Path $env:USERPROFILE ".local\bin\stk.exe"
}
$stkRaw = & $stkExe gain --json
if ($LASTEXITCODE -ne 0) { throw "stk gain --json failed" }
$stk = $stkRaw | ConvertFrom-Json

# --- RTK: parse its human output; labeled on the site as RTK's own accounting ---
$rtk = $null
try {
    $rtkRaw = & rtk gain 2>$null | Out-String
    if ($LASTEXITCODE -eq 0 -and $rtkRaw) {
        $cmds = if ($rtkRaw -match 'Total commands:\s+([\d,]+)') { [long]($Matches[1] -replace ',', '') } else { $null }
        $saved = $null; $pct = $null
        if ($rtkRaw -match 'Tokens saved:\s+(\S+)\s+\(([\d.]+%)\)') { $saved = $Matches[1]; $pct = $Matches[2] }
        if ($cmds -ne $null) {
            $rtk = [ordered]@{
                commands     = $cmds
                tokens_saved = $saved
                reduction    = $pct
                source       = "rtk gain (RTK's own accounting)"
            }
        }
    }
} catch { }

$snapshot = [ordered]@{
    generated_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    stk          = $stk
    rtk          = $rtk
}

New-Item -ItemType Directory -Force -Path (Split-Path $OutFile) | Out-Null
$json = $snapshot | ConvertTo-Json -Depth 6
Set-Content -Path $OutFile -Value $json -Encoding UTF8

# --- commit + push only if the snapshot changed ---
Push-Location $RepoRoot
try {
    $status = git status --porcelain -- docs/data/gain.json
    if (-not $status) {
        Write-Host "gain.json unchanged; nothing to publish."
        return
    }
    git add docs/data/gain.json
    git commit -m "chore: stats snapshot $($snapshot.generated_at)" | Out-Null
    git push
    Write-Host "Published snapshot $($snapshot.generated_at)."
} finally {
    Pop-Location
}
