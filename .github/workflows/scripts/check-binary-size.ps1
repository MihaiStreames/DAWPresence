param(
    [string]$FilePath = "dist/DAWPresence.exe",
    [int]$MaxSizeMB = 5
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$size = (Get-Item $FilePath).Length
$limitBytes = $MaxSizeMB * 1MB
Write-Host "Binary size: ${size} bytes (limit: ${limitBytes} bytes)"

if ($size -gt $limitBytes) {
    throw "Binary too large: ${size} bytes (limit: ${limitBytes} bytes)"
}
