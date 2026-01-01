@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" >nul

set "DIST=%ROOT%\dist"
if not exist "%DIST%" mkdir "%DIST%"

echo Building release binary...
cargo build --release

echo Copying to dist\...
copy /Y "target\release\DAWPresence.exe" "%DIST%\DAWPresence.exe" >nul
echo Built: %DIST%\DAWPresence.exe

popd >nul
endlocal
