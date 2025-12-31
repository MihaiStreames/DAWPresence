param(
    [string]$Root = (Resolve-Path "$PSScriptRoot\..").Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location $Root

Write-Host "Building release binary..."
cargo build --release

$binName = "DAWPresence.exe"
$dist = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item -Force (Join-Path $Root "target\release\$binName") (Join-Path $dist $binName)

Write-Host "Built: $dist\$binName"
