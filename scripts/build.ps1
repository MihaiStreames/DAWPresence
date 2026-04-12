param(
    [string]$Root = (Resolve-Path "$PSScriptRoot\..").Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location $Root

$dist = Join-Path $Root "dist"
$built = Join-Path $Root "target\release\DAWPresence.exe"

New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "Building tiny-skia variant..."
cargo build --release

if (-not (Test-Path $built)) { throw "Build failed: $built not found" }

Copy-Item -Force $built (Join-Path $dist "DAWPresence-tiny-skia.exe")
Write-Host "Built: dist/DAWPresence-tiny-skia.exe"

Write-Host "Building wgpu variant..."
cargo build --release --no-default-features --features renderer-wgpu

if (-not (Test-Path $built)) { throw "Build failed: $built not found" }

Copy-Item -Force $built (Join-Path $dist "DAWPresence-wgpu.exe")
Write-Host "Built: dist/DAWPresence-wgpu.exe"
