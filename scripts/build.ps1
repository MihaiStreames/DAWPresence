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
$possiblePaths = @(
    (Join-Path $Root "target\release\$binName"),
    (Join-Path $Root "target\x86_64-pc-windows-gnu\release\$binName"),
    (Join-Path $Root "target\x86_64-pc-windows-msvc\release\$binName")
)

$foundPath = $possiblePaths | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($foundPath) {
    Copy-Item -Force $foundPath (Join-Path $dist $binName)
} else {
    throw "Could not find built binary in any of: $($possiblePaths -join ', ')"
}

Write-Host "Built: $dist\$binName"
