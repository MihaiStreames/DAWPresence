param(
    [string]$Root = (Resolve-Path "$PSScriptRoot\..").Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location $Root

$binName = "DAWPresence.exe"
$dist = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "Building release binary..."
cargo build --release

Write-Host "Copying to dist\..."
Copy-Item -Force (Join-Path $Root "target\release\$binName") (Join-Path $dist $binName)

Write-Host "Built: $dist\$binName"
