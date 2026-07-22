$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$modulePath = Join-Path $root 'bootstrap\NewProjectCore.psm1'
$consolePath = Join-Path $root 'bootstrap\new-claude-project.ps1'
$builderPath = Join-Path $root 'bootstrap\Build-NewClaudeProjectUIRelease.ps1'
$scratch = Join-Path ([IO.Path]::GetTempPath()) ("new-project-smoke-" + [guid]::NewGuid().ToString('N'))
$destination = Join-Path $scratch 'projects'
$projectName = 'codex-contract-smoke'
$target = Join-Path $destination $projectName
$zipPath = Join-Path $scratch 'New-ClaudeProject-UI.zip'

$identityVariables = @(
    'GIT_AUTHOR_NAME',
    'GIT_AUTHOR_EMAIL',
    'GIT_COMMITTER_NAME',
    'GIT_COMMITTER_EMAIL'
)
$originalIdentity = @{}

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )

    if (-not $Condition) { throw $Message }
}

try {
    New-Item -ItemType Directory -Force -Path $scratch | Out-Null
    foreach ($variable in $identityVariables) {
        $originalIdentity[$variable] = [Environment]::GetEnvironmentVariable($variable, 'Process')
    }
    [Environment]::SetEnvironmentVariable('GIT_AUTHOR_NAME', 'Generator Smoke Test', 'Process')
    [Environment]::SetEnvironmentVariable('GIT_AUTHOR_EMAIL', 'generator-smoke@example.invalid', 'Process')
    [Environment]::SetEnvironmentVariable('GIT_COMMITTER_NAME', 'Generator Smoke Test', 'Process')
    [Environment]::SetEnvironmentVariable('GIT_COMMITTER_EMAIL', 'generator-smoke@example.invalid', 'Process')

    Import-Module -Force -Name $modulePath
    Assert-True (Test-NewProjectName -Name 'valid-project_1.0') 'Valid project name was rejected.'
    Assert-True (-not (Test-NewProjectName -Name '..')) 'All-dot project name was accepted.'
    Assert-True (-not (Test-NewProjectName -Name 'bad name')) 'Project name containing spaces was accepted.'

    $consoleOutput = @(& $consolePath -Name $projectName -Dest $destination -LocalOnly)
    Assert-True $? 'Console generator reported a failure.'
    $null = $consoleOutput

    foreach ($required in @(
        'AGENTS.md',
        '.agents\skills\init-project\SKILL.md',
        '.claude\skills\init-project\SKILL.md',
        '.claude\scripts\sync-codex-skills.mjs',
        'README.md'
    )) {
        Assert-True (Test-Path -LiteralPath (Join-Path $target $required) -PathType Leaf) "Generated project is missing $required"
    }
    foreach ($forbidden in @('bootstrap', '.claude-plugin', '.github\workflows\validate-template.yml', '.tmp-video-7m')) {
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $target $forbidden))) "Generated project retained $forbidden"
    }

    $readme = [IO.File]::ReadAllText((Join-Path $target 'README.md'), [Text.Encoding]::ASCII)
    Assert-True ($readme -eq "# $projectName`r`n") 'Generated README content or encoding contract changed.'

    $status = @(& git -C $target status --porcelain)
    Assert-True ($LASTEXITCODE -eq 0) 'Could not inspect the generated repository.'
    Assert-True ($status.Count -eq 0) 'Generated repository is not clean after its initial commit.'
    $gitDir = [IO.Path]::GetFullPath(([string](& git -C $target rev-parse --absolute-git-dir)).Trim())
    Assert-True ($LASTEXITCODE -eq 0) 'Could not resolve the generated repository Git directory.'
    Assert-True ($gitDir.StartsWith($target, [StringComparison]::OrdinalIgnoreCase)) 'Generated repository points at the source worktree Git directory.'

    $builtZip = [string](& $builderPath -OutputPath $zipPath)
    Assert-True ($builtZip.Trim() -eq $zipPath) 'Release builder returned an unexpected output path.'
    Assert-True (Test-Path -LiteralPath $zipPath -PathType Leaf) 'Release builder did not create the ZIP.'

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($zipPath)
    try {
        $entries = @($archive.Entries | ForEach-Object { $_.FullName.Replace('\', '/') })
        foreach ($requiredEntry in @(
            'New-ClaudeProject-UI.cmd',
            'new-claude-project-ui.ps1',
            'NewProjectCore.psm1',
            'template/AGENTS.md',
            'template/.agents/skills/init-project/SKILL.md'
        )) {
            Assert-True ($entries -contains $requiredEntry) "Release ZIP is missing $requiredEntry"
        }
        Assert-True (-not ($entries | Where-Object { $_ -eq 'template/.git' -or $_ -like 'template/.git/*' })) 'Release ZIP contains source Git metadata.'
        Assert-True (-not ($entries | Where-Object { $_ -like 'template/.tmp*' })) 'Release ZIP contains template scratch files.'
    }
    finally {
        $archive.Dispose()
    }

    Write-Host 'Generator smoke test passed.'
}
finally {
    foreach ($variable in $identityVariables) {
        [Environment]::SetEnvironmentVariable($variable, $originalIdentity[$variable], 'Process')
    }
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -Recurse -Force -LiteralPath $scratch
    }
}
