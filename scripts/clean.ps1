param(
    [switch]$All,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($Help) {
    Write-Host "Usage: clean.ps1 [-All] [-Help]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -All   remove target and installer output"
    Write-Host "  -Help  show this message"
    exit 0
}

Set-Location (Resolve-Path "$PSScriptRoot\..")

if (Test-Path dist) {
    Write-Host "dist"
    Remove-Item -Recurse -Force dist
}

if ($All) {
    if (Test-Path "installer\Output") {
        Write-Host "installer\Output"
        Remove-Item -Recurse -Force "installer\Output"
    }
    cargo clean
}

Write-Host "Clean complete"
