param(
    [Parameter(Mandatory)]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path "$PSScriptRoot\..\..\..").Path
$IssPath = Join-Path $PSScriptRoot "DAWPresence.iss"
$Dist = Join-Path $Root "dist"

iscc "/DAppVersion=$Version" "/DExeDir=..\..\..\dist" $IssPath
if ($LASTEXITCODE -ne 0) { throw "iscc failed with exit code $LASTEXITCODE" }

Get-ChildItem -Path (Join-Path $PSScriptRoot "output") -Filter "*.exe" |
    ForEach-Object {
        Copy-Item -Force $_.FullName (Join-Path $Dist $_.Name)
        Write-Host "Installer: dist\$($_.Name)"
    }
