param(
    [Parameter(Mandatory)]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path "$PSScriptRoot\..").Path
$IssPath = Join-Path $Root "installer\DAWPresence.iss"
$Dist = Join-Path $Root "dist"
$OutputDir = Join-Path $Root "installer\output"

foreach ($renderer in @("tiny-skia", "wgpu")) {
    $exeDir = Join-Path $Dist $renderer
    New-Item -ItemType Directory -Force -Path $exeDir | Out-Null

    $src = Join-Path $Dist "DAWPresence-$renderer.exe"
    Copy-Item -Force $src (Join-Path $exeDir "DAWPresence.exe")

    Write-Host "Building $renderer installer..."
    iscc "/DAppVersion=$Version" "/DExeDir=..\dist\$renderer" "/DRenderer=$renderer" $IssPath
    if ($LASTEXITCODE -ne 0) { throw "iscc failed for $renderer" }
}

Get-ChildItem -Path $OutputDir -Filter "*.exe" |
    ForEach-Object {
        Copy-Item -Force $_.FullName (Join-Path $Dist $_.Name)
        Write-Host "Installer: dist\$($_.Name)"
    }
