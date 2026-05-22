param(
	[Parameter(Mandatory)]
	[string]$Version
)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path "$PSScriptRoot\..").Path
$IssPath = Join-Path $Root "installer\DAWPresence.iss"
$OutputDir = Join-Path $Root "installer\output"
$Dist = Join-Path $Root "dist"

iscc "/DAppVersion=$Version" "/DExeDir=..\dist" $IssPath
if ($LASTEXITCODE -ne 0) { exit 1 }

Get-ChildItem -Path $OutputDir -Filter "*.exe" | ForEach-Object {
	Copy-Item -Force $_.FullName (Join-Path $Dist $_.Name)
}
