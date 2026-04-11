param(
    [string]$Root = (Resolve-Path "$PSScriptRoot\..").Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location $Root

$dist = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "Building release binary..."
cargo build --release

$builtPath = Join-Path $Root "target\release\DAWPresence.exe"
if (-not (Test-Path $builtPath)) {
    throw "Could not find built binary at: $builtPath"
}

Copy-Item -Force $builtPath (Join-Path $dist "DAWPresence.exe")
Write-Host "Built: dist/DAWPresence.exe"
