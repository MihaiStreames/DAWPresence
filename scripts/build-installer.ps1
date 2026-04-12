param(
    [Parameter(Mandatory)]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path "$PSScriptRoot\..").Path
$IssPath = Join-Path $Root "installer\DAWPresence.iss"
$Dist = Join-Path $Root "dist"
$OutputDir = Join-Path $Root "installer\output"

Write-Host "Building installer..."
iscc "/DAppVersion=$Version" "/DExeDir=..\dist" $IssPath
if ($LASTEXITCODE -ne 0) { throw "iscc failed" }

Get-ChildItem -Path $OutputDir -Filter "*.exe" |
    ForEach-Object {
        Copy-Item -Force $_.FullName (Join-Path $Dist $_.Name)
        Write-Host "Installer: dist\$($_.Name)"
    }
