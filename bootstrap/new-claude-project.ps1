# new-claude-project.ps1 - console front end for the shared project generator.
#
# Usage:
#   .\new-claude-project.ps1
#   .\new-claude-project.ps1 -Name my-app
#   .\new-claude-project.ps1 -Name my-app -Dest D:\code
#   .\new-claude-project.ps1 -Name my-app -LocalOnly

param(
    [string]$Name,
    [string]$Dest = "$HOME\code",
    [string]$Template = 'ryanportfolio/AI-Firmware',
    [switch]$LocalOnly
)

$ErrorActionPreference = 'Stop'

$modulePath = Join-Path $PSScriptRoot 'NewProjectCore.psm1'
Import-Module -Force -Name $modulePath

if (-not $Name) {
    $Name = Read-Host 'New project name (kebab-case, becomes folder and repo name)'
}

$logger = {
    param($message, $tone)

    $color = switch ($tone) {
        'cmd' { 'DarkGray' }
        'dim' { 'DarkGray' }
        'ok' { 'Green' }
        'err' { 'Red' }
        'head' { 'Cyan' }
        default { 'Gray' }
    }
    Write-Host ([string]$message) -ForegroundColor $color
}

try {
    $result = Invoke-NewProject -Name $Name -Dest $Dest -Template $Template -LocalOnly:$LocalOnly -LogAction $logger
}
catch {
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

Write-Host ''
if ($result.Mode -eq 'gh') {
    Write-Host 'DONE. Private repo created and cloned:' -ForegroundColor Green
    Write-Host "  Local:  $($result.LocalPath)"
    if ($result.RemoteUrl) { Write-Host "  Remote: $($result.RemoteUrl)" }
}
else {
    Write-Host "DONE (local only). Folder ready: $($result.LocalPath)" -ForegroundColor Green
    Write-Host ''
    Write-Host 'To put it on GitHub manually:' -ForegroundColor Cyan
    Write-Host "  1. Create a PRIVATE repo named '$Name' at https://github.com/new"
    Write-Host '  2. Then run:'
    Write-Host "       cd `"$($result.LocalPath)`""
    Write-Host "       git remote add origin https://github.com/<your-username>/$Name.git"
    Write-Host '       git push -u origin main'
}

Write-Host ''
Write-Host 'Next: open the folder in Claude Code and run /init-project.'
Write-Host 'Codex users: open the folder in Codex and select the init-project skill.'
