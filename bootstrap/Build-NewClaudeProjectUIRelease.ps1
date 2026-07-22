# Build a standalone New Claude Project UI ZIP.
# The archive includes a template snapshot so local-only mode works offline.

param(
    [string]$OutputPath = (Join-Path (Split-Path -Parent $PSScriptRoot) 'dist\New-ClaudeProject-UI.zip')
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$outputFullPath = [IO.Path]::GetFullPath($OutputPath)
$outputDirectory = Split-Path -Parent $outputFullPath
$stage = Join-Path ([IO.Path]::GetTempPath()) ("new-claude-project-ui-" + [guid]::NewGuid().ToString('N'))
$templateStage = Join-Path $stage 'template'
$snapshotArchive = Join-Path ([IO.Path]::GetTempPath()) ("claude-starter-template-" + [guid]::NewGuid().ToString('N') + '.zip')

if (-not (Test-Path -LiteralPath $outputDirectory)) {
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
}
if (Test-Path -LiteralPath $outputFullPath) {
    Remove-Item -Force -LiteralPath $outputFullPath
}

try {
    New-Item -ItemType Directory -Force -Path $stage, $templateStage | Out-Null

    foreach ($asset in @('New-ClaudeProject-UI.cmd', 'new-claude-project-ui.ps1', 'NewProjectCore.psm1')) {
        Copy-Item -Force -LiteralPath (Join-Path $PSScriptRoot $asset) -Destination (Join-Path $stage $asset)
    }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    & git -C $repoRoot archive --format=zip --output $snapshotArchive HEAD
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to export the tracked template snapshot.'
    }
    [IO.Compression.ZipFile]::ExtractToDirectory($snapshotArchive, $templateStage)

    foreach ($templateOnlyPath in @('bootstrap', '.claude-plugin', '.github\workflows\validate-template.yml', 'README.md')) {
        $fullPath = Join-Path $templateStage $templateOnlyPath
        if (Test-Path -LiteralPath $fullPath) {
            Remove-Item -Recurse -Force -LiteralPath $fullPath
        }
    }
    Get-ChildItem -LiteralPath $templateStage -Directory -Force |
        Where-Object { $_.Name -like '.tmp*' } |
        Remove-Item -Recurse -Force

    [IO.Compression.ZipFile]::CreateFromDirectory(
        $stage,
        $outputFullPath,
        [IO.Compression.CompressionLevel]::Optimal,
        $false
    )
}
finally {
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -Recurse -Force -LiteralPath $stage
    }
    if (Test-Path -LiteralPath $snapshotArchive) {
        Remove-Item -Force -LiteralPath $snapshotArchive
    }
}

Write-Output $outputFullPath
