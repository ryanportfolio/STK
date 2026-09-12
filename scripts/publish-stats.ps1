# Snapshot `stk gain --json` and publish docs/data/gain.json.
# RTK remains in the JSON for existing consumers; the site shows STK totals.
# The scheduled job runs from the clean main checkout.
# Use -SnapshotOnly -OutputPath <file> to export without Git operations.

param(
    [switch]$SnapshotOnly,
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$OutFile = Join-Path $RepoRoot "docs\data\gain.json"
if ($SnapshotOnly) {
    if (-not $OutputPath) { throw "-SnapshotOnly requires -OutputPath" }
    $OutFile = [System.IO.Path]::GetFullPath($OutputPath)
} elseif ($OutputPath) { throw "-OutputPath requires -SnapshotOnly" }

function Invoke-Git {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$GitArgs
    )

    $output = & git @GitArgs
    if ($LASTEXITCODE -ne 0) {
        throw "git $($GitArgs -join ' ') failed (exit $LASTEXITCODE)"
    }
    $output
}

Push-Location $RepoRoot
try {
    if (-not $SnapshotOnly) {
    # Pull remote changes before generating data. If a push loses a race, the
    # next run rebases the unpublished snapshot and retries automatically.
    $branch = (Invoke-Git -GitArgs @("branch", "--show-current") | Out-String).Trim()
    if ($branch -ne "main") {
        throw "stats publisher must run from main; current branch is '$branch'"
    }

    $trackedChanges = Invoke-Git -GitArgs @("status", "--porcelain", "--untracked-files=no")
    if ($trackedChanges) {
        throw "tracked working-tree changes found; refusing to publish"
    }

    Invoke-Git -GitArgs @("fetch", "origin", "main") | Out-Host
    Invoke-Git -GitArgs @("rebase", "origin/main") | Out-Host
    }

    # --- STK: the authoritative JSON contract ---
    # Scheduled tasks often run without the user's PATH; fall back to the install path.
    $stkExe = "stk"
    if (-not (Get-Command stk -ErrorAction SilentlyContinue)) {
        $stkExe = Join-Path $env:USERPROFILE ".local\bin\stk.exe"
    }
    $stkRaw = & $stkExe gain --json
    if ($LASTEXITCODE -ne 0) { throw "stk gain --json failed (exit $LASTEXITCODE)" }
    $stk = $stkRaw | ConvertFrom-Json
    if (-not $stk.clients -or $null -eq $stk.clients.codex -or $null -eq $stk.clients.claude) {
        throw "STK lacks client accounting; upgrade the installed binary before publishing"
    }

    # --- RTK: retain its own accounting in the JSON for compatibility ---
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
    if ($SnapshotOnly) {
        Write-Host "Snapshot written to $OutFile (no Git changes or publication)."
        return
    }

    # --- commit + push only if the snapshot changed ---
    $status = Invoke-Git -GitArgs @("status", "--porcelain", "--", "docs/data/gain.json")
    if (-not $status) {
        Write-Host "gain.json unchanged; nothing to publish."
        return
    }

    Invoke-Git -GitArgs @("add", "docs/data/gain.json") | Out-Null
    Invoke-Git -GitArgs @("commit", "-m", "chore: stats snapshot $($snapshot.generated_at)") | Out-Null
    Invoke-Git -GitArgs @("push", "origin", "main") | Out-Host
    Write-Host "Published snapshot $($snapshot.generated_at)."
} finally {
    Pop-Location
}
