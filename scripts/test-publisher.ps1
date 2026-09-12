param([Parameter(Mandatory = $true)][string]$StkDirectory)
$ErrorActionPreference = "Stop"
$testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("stk-publisher-test-" + [guid]::NewGuid())
$oldPath = $env:PATH
$oldData = $env:STK_DATA_DIR
try {
    New-Item -ItemType Directory -Path $testRoot | Out-Null
    $env:PATH = [System.IO.Path]::GetFullPath($StkDirectory) + ";" + $oldPath
    $env:STK_DATA_DIR = $testRoot
    @'
{"ts":1784592000,"file":"old","file_bytes":1000,"sent_bytes":100,"kind":"clamp"}
{"ts":1784592000,"client":"claude","file":"a","file_bytes":3000,"sent_bytes":1000,"kind":"clamp"}
{"ts":1784592000,"client":"codex","file":"b","file_bytes":5000,"sent_bytes":1000,"kind":"clamp"}
'@ | Set-Content -LiteralPath (Join-Path $testRoot "stats.jsonl") -Encoding ASCII
    $outputFile = Join-Path $testRoot "snapshot.json"
    & (Join-Path $PSScriptRoot "publish-stats.ps1") -SnapshotOnly -OutputPath $outputFile
    $snapshot = Get-Content -LiteralPath $outputFile -Raw | ConvertFrom-Json
    if ($snapshot.stk.clients.codex.bytes_avoided -ne 4000 -or
        $snapshot.stk.clients.claude.bytes_avoided -ne 2000 -or
        $snapshot.stk.clients.legacy.bytes_avoided -ne 900 -or
        $snapshot.stk.bytes_avoided -ne 6900 -or
        $snapshot.stk.clamps -ne 3) { throw "Publisher lost client data" }
    Write-Host "PASS: publisher preserves Codex, Claude, legacy, and combined totals"
} finally {
    $env:PATH = $oldPath
    $env:STK_DATA_DIR = $oldData
    $resolved = [System.IO.Path]::GetFullPath($testRoot)
    $expectedParent = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\')
    if ((Split-Path -Parent $resolved) -eq $expectedParent -and
        (Split-Path -Leaf $resolved).StartsWith("stk-publisher-test-")) {
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
