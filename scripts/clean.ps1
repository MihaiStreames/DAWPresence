param(
    [string]$Root = (Resolve-Path "$PSScriptRoot\..").Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location $Root

Write-Host "Cleaning build artifacts..."
cargo clean

if (Test-Path dist) {
    Remove-Item -Recurse -Force dist
    Write-Host "Removed dist/"
}
