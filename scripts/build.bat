@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" >nul

set "BIN_NAME=DAWPresence.exe"
set "DIST=%ROOT%\dist"
if not exist "%DIST%" mkdir "%DIST%"

echo Building release binary...
cargo build --release

echo Copying to dist\...
set "FOUND="
for %%P in (
    "target\release\%BIN_NAME%"
    "target\x86_64-pc-windows-gnu\release\%BIN_NAME%"
    "target\x86_64-pc-windows-msvc\release\%BIN_NAME%"
) do (
    if exist "%%~P" if not defined FOUND (
        set "FOUND=%%~P"
    )
)

if defined FOUND (
    copy /Y "%FOUND%" "%DIST%\%BIN_NAME%" >nul
    echo Built: %DIST%\%BIN_NAME%
) else (
    echo Could not find built binary
    exit /b 1
)

popd >nul
endlocal
