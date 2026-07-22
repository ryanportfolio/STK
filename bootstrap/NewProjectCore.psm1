Set-StrictMode -Version 2.0

$script:RequiredProjectFiles = @(
    'AGENTS.md',
    '.agents\skills\init-project\SKILL.md',
    '.claude\skills\init-project\SKILL.md',
    '.claude\scripts\sync-codex-skills.mjs'
)

$script:TemplateOnlyPaths = @(
    'bootstrap',
    '.claude-plugin',
    '.github\workflows\validate-template.yml'
)

function Write-NewProjectLog {
    param(
        [scriptblock]$LogAction,
        [string]$Message,
        [string]$Tone = 'out'
    )

    if ($LogAction) {
        & $LogAction $Message $Tone | Out-Null
    }
}

function Invoke-NewProjectNative {
    param(
        [Parameter(Mandatory = $true)]
        [string]$File,

        [string[]]$Arguments = @(),
        [scriptblock]$LogAction,
        [string]$Tone = 'out',
        [switch]$Quiet,
        [switch]$QuietOutput
    )

    if (-not $Quiet) {
        Write-NewProjectLog $LogAction (("> {0} {1}" -f $File, ($Arguments -join ' ')).TrimEnd()) 'cmd'
    }

    $previousPreference = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $lines = @(& $File @Arguments 2>&1)
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previousPreference
    }

    if (-not $Quiet -and -not $QuietOutput) {
        foreach ($line in $lines) {
            Write-NewProjectLog $LogAction ([string]$line) $Tone
        }
    }

    return [pscustomobject]@{
        ExitCode = $exitCode
        Output = @($lines | ForEach-Object { [string]$_ })
    }
}

function Test-NewProjectName {
    [CmdletBinding()]
    param(
        [AllowEmptyString()]
        [string]$Name
    )

    return -not [string]::IsNullOrWhiteSpace($Name) -and
        $Name -match '^[a-zA-Z0-9._-]+$' -and
        $Name -notmatch '^\.+$'
}

function Get-NewProjectMode {
    [CmdletBinding()]
    param(
        [switch]$LocalOnly
    )

    if ($LocalOnly) {
        return [pscustomobject]@{
            Mode = 'local'
            Reason = 'Local-only mode requested. The bundled template is copied without contacting GitHub.'
        }
    }

    $gh = Get-Command gh -ErrorAction SilentlyContinue
    if (-not $gh) {
        return [pscustomobject]@{
            Mode = 'local'
            Reason = 'gh CLI not found. The bundled template is copied locally and manual GitHub steps are shown.'
        }
    }

    $auth = Invoke-NewProjectNative -File 'gh' -Arguments @('auth', 'status') -Quiet
    if ($auth.ExitCode -eq 0) {
        return [pscustomobject]@{
            Mode = 'gh'
            Reason = 'gh CLI detected and authenticated. A private GitHub repo is created from the template.'
        }
    }

    return [pscustomobject]@{
        Mode = 'local'
        Reason = 'gh CLI found but not signed in. The bundled template is copied locally and manual GitHub steps are shown.'
    }
}

function Resolve-NewProjectLocalTemplate {
    param(
        [string]$LocalTemplate
    )

    $candidates = @()
    if (-not [string]::IsNullOrWhiteSpace($LocalTemplate)) {
        $candidates += $LocalTemplate
    }
    $candidates += (Join-Path $PSScriptRoot 'template')
    $candidates += (Split-Path -Parent $PSScriptRoot)

    foreach ($candidate in $candidates) {
        if ($candidate -and
            (Test-Path -LiteralPath (Join-Path $candidate 'AGENTS.md') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $candidate 'CLAUDE.md') -PathType Leaf)) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    throw 'No local template snapshot was found. Re-extract the complete release ZIP or run from the claude-starter repository.'
}

function Assert-NewProjectContract {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Target
    )

    foreach ($required in $script:RequiredProjectFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $Target $required) -PathType Leaf)) {
            throw "Generated project is missing required Codex asset: $required"
        }
    }

    foreach ($forbidden in $script:TemplateOnlyPaths) {
        if (Test-Path -LiteralPath (Join-Path $Target $forbidden)) {
            throw "Generated project still contains template-only asset: $forbidden"
        }
    }

    $scratchDirectories = @(Get-ChildItem -LiteralPath $Target -Directory -Force | Where-Object { $_.Name -like '.tmp*' })
    if ($scratchDirectories.Count -gt 0) {
        throw "Generated project still contains template scratch directory: $($scratchDirectories[0].Name)"
    }
}

function Remove-NewProjectLocalTemplateFiles {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Target
    )

    foreach ($relativePath in $script:TemplateOnlyPaths) {
        $fullPath = Join-Path $Target $relativePath
        if (Test-Path -LiteralPath $fullPath) {
            Remove-Item -Recurse -Force -LiteralPath $fullPath
        }
    }

    Get-ChildItem -LiteralPath $Target -Directory -Force |
        Where-Object { $_.Name -like '.tmp*' } |
        Remove-Item -Recurse -Force

    $readmePath = Join-Path $Target 'README.md'
    if (Test-Path -LiteralPath $readmePath) {
        Remove-Item -Force -LiteralPath $readmePath
    }
}

function Copy-NewProjectTemplate {
    param(
        [Parameter(Mandatory = $true)]
        [string]$TemplateRoot,

        [Parameter(Mandatory = $true)]
        [string]$Target,

        [scriptblock]$LogAction
    )

    $repoProbe = Invoke-NewProjectNative -File 'git' -Arguments @(
        '-C', $TemplateRoot, 'rev-parse', '--is-inside-work-tree'
    ) -Quiet

    if ($repoProbe.ExitCode -eq 0) {
        $archivePath = Join-Path ([IO.Path]::GetTempPath()) ("new-project-template-" + [guid]::NewGuid().ToString('N') + '.zip')
        try {
            $archive = Invoke-NewProjectNative -File 'git' -Arguments @(
                '-C', $TemplateRoot, 'archive', '--format=zip', '--output', $archivePath, 'HEAD'
            ) -LogAction $LogAction -Tone 'dim'
            if ($archive.ExitCode -ne 0) {
                throw 'Failed to export the tracked template snapshot.'
            }

            New-Item -ItemType Directory -Force -Path $Target | Out-Null
            Add-Type -AssemblyName System.IO.Compression.FileSystem
            [IO.Compression.ZipFile]::ExtractToDirectory($archivePath, $Target)
        }
        finally {
            if (Test-Path -LiteralPath $archivePath) {
                Remove-Item -Force -LiteralPath $archivePath
            }
        }
    }
    else {
        $copy = Invoke-NewProjectNative -File 'robocopy' -Arguments @(
            $TemplateRoot, $Target, '/E',
            '/XD', '.git', 'bootstrap', '.claude-plugin', '.tmp*', 'dist', 'build', '.worktrees', 'worktrees',
            '/XF', '.git', 'README.md', 'validate-template.yml'
        ) -LogAction $LogAction -Tone 'dim' -QuietOutput
        if ($copy.ExitCode -ge 8) {
            throw "Template copy failed (robocopy exit $($copy.ExitCode))."
        }
    }

    Remove-NewProjectLocalTemplateFiles -Target $Target
}

function Initialize-NewProjectReadme {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Target,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    $readmePath = Join-Path $Target 'README.md'
    if (Test-Path -LiteralPath $readmePath) {
        Remove-Item -Force -LiteralPath $readmePath
    }
    [IO.File]::WriteAllText($readmePath, "# $Name`r`n", [Text.Encoding]::ASCII)
}

function Remove-NewProjectTemplateFiles {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Target,

        [Parameter(Mandatory = $true)]
        [string]$Name,

        [scriptblock]$LogAction
    )

    Write-NewProjectLog $LogAction 'Stripping template-only files and replacing README.md ...' 'out'
    $removeArgs = @(
        '-C', $Target, 'rm', '-rq', '--ignore-unmatch', '--',
        'bootstrap', '.claude-plugin', '.github/workflows/validate-template.yml', 'README.md'
    )
    $remove = Invoke-NewProjectNative -File 'git' -Arguments $removeArgs -LogAction $LogAction -Tone 'dim'
    if ($remove.ExitCode -ne 0) {
        throw 'Failed to remove template-only files from the cloned repository.'
    }

    foreach ($relativePath in $script:TemplateOnlyPaths) {
        $fullPath = Join-Path $Target $relativePath
        if (Test-Path -LiteralPath $fullPath) {
            Remove-Item -Recurse -Force -LiteralPath $fullPath
        }
    }

    Initialize-NewProjectReadme -Target $Target -Name $Name
    $add = Invoke-NewProjectNative -File 'git' -Arguments @('-C', $Target, 'add', 'README.md') -LogAction $LogAction -Tone 'dim'
    if ($add.ExitCode -ne 0) {
        throw 'Failed to stage the replacement README.'
    }

    Assert-NewProjectContract -Target $Target
}

function Invoke-NewProject {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,

        [Parameter(Mandatory = $true)]
        [string]$Dest,

        [string]$Template = 'ryanportfolio/AI-Firmware',
        [string]$LocalTemplate,
        [switch]$LocalOnly,
        [scriptblock]$LogAction
    )

    if (-not (Test-NewProjectName -Name $Name)) {
        throw "Invalid name '$Name' - use letters, digits, dot, dash, or underscore; all-dot names are not allowed."
    }
    if ([string]::IsNullOrWhiteSpace($Dest)) {
        throw 'Destination folder is required.'
    }

    $target = Join-Path $Dest $Name
    if (Test-Path -LiteralPath $target) {
        throw "Folder already exists: $target"
    }
    if (-not (Test-Path -LiteralPath $Dest)) {
        New-Item -ItemType Directory -Force -Path $Dest | Out-Null
        Write-NewProjectLog $LogAction "Created destination: $Dest" 'dim'
    }

    $mode = Get-NewProjectMode -LocalOnly:$LocalOnly
    Write-NewProjectLog $LogAction $mode.Reason 'head'
    Write-NewProjectLog $LogAction '' 'out'

    if ($mode.Mode -eq 'gh') {
        Write-NewProjectLog $LogAction ("Creating private repo '{0}' from template {1} ..." -f $Name, $Template) 'head'
        Push-Location -LiteralPath $Dest
        try {
            $create = Invoke-NewProjectNative -File 'gh' -Arguments @(
                'repo', 'create', $Name, '--template', $Template, '--private', '--clone'
            ) -LogAction $LogAction
        }
        finally {
            Pop-Location
        }

        if ($create.ExitCode -eq 0) {
            if (-not (Test-Path -LiteralPath $target -PathType Container)) {
                throw "GitHub creation reported success, but the clone was not found at $target"
            }

            Remove-NewProjectTemplateFiles -Target $target -Name $Name -LogAction $LogAction
            $status = Invoke-NewProjectNative -File 'git' -Arguments @('-C', $target, 'status', '--porcelain') -Quiet
            if ($status.ExitCode -ne 0) {
                throw 'Failed to inspect template cleanup changes.'
            }
            if ($status.Output.Count -gt 0) {
                Write-NewProjectLog $LogAction 'Committing and pushing template cleanup ...' 'out'
                $commit = Invoke-NewProjectNative -File 'git' -Arguments @(
                    '-C', $target, 'commit', '-qm', 'Strip template files, add README stub'
                ) -LogAction $LogAction
                if ($commit.ExitCode -ne 0) { throw 'Failed to commit template cleanup.' }

                $push = Invoke-NewProjectNative -File 'git' -Arguments @('-C', $target, 'push', '-q') -LogAction $LogAction
                if ($push.ExitCode -ne 0) { throw 'Failed to push template cleanup.' }
            }

            $loginResult = Invoke-NewProjectNative -File 'gh' -Arguments @('api', 'user', '--jq', '.login') -Quiet
            $remoteUrl = $null
            if ($loginResult.ExitCode -eq 0 -and $loginResult.Output.Count -gt 0) {
                $login = ([string]$loginResult.Output[0]).Trim()
                if ($login) { $remoteUrl = "https://github.com/$login/$Name" }
            }

            return [pscustomobject]@{
                Mode = 'gh'
                LocalPath = $target
                RemoteUrl = $remoteUrl
            }
        }

        Write-NewProjectLog $LogAction 'GitHub project creation failed; switching to local mode.' 'err'
        if (Test-Path -LiteralPath $target) {
            throw "GitHub creation left a partial folder at $target. Remove or inspect it before retrying."
        }
    }

    $templateRoot = Resolve-NewProjectLocalTemplate -LocalTemplate $LocalTemplate
    Write-NewProjectLog $LogAction ("Copying template from {0} ..." -f $templateRoot) 'out'
    Copy-NewProjectTemplate -TemplateRoot $templateRoot -Target $target -LogAction $LogAction

    Initialize-NewProjectReadme -Target $target -Name $Name
    Assert-NewProjectContract -Target $target

    $init = Invoke-NewProjectNative -File 'git' -Arguments @('-C', $target, 'init', '-b', 'main') -LogAction $LogAction -Tone 'dim'
    if ($init.ExitCode -ne 0) { throw 'Failed to initialize the local repository.' }
    $addAll = Invoke-NewProjectNative -File 'git' -Arguments @('-C', $target, 'add', '-A') -LogAction $LogAction -Tone 'dim'
    if ($addAll.ExitCode -ne 0) { throw 'Failed to stage the local template copy.' }
    $commitLocal = Invoke-NewProjectNative -File 'git' -Arguments @(
        '-C', $target, 'commit', '-qm', 'Initialize from claude-starter template'
    ) -LogAction $LogAction
    if ($commitLocal.ExitCode -ne 0) { throw 'Failed to commit the local template copy.' }

    return [pscustomobject]@{
        Mode = 'local'
        LocalPath = $target
        RemoteUrl = $null
    }
}

Export-ModuleMember -Function Test-NewProjectName, Get-NewProjectMode, Invoke-NewProject
