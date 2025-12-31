@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" >nul

echo Building release binary...
cargo build --release

set "DIST=%ROOT%\dist"
if not exist "%DIST%" mkdir "%DIST%"

copy /Y "target\release\DAWPresence.exe" "%DIST%\DAWPresence.exe" >nul
echo Built: %DIST%\DAWPresence.exe

popd >nul
endlocal
