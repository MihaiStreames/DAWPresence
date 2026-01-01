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
set "BUILT_PATH=%ROOT%\target\release\%BIN_NAME%"

if exist "%BUILT_PATH%" (
    copy /Y "%BUILT_PATH%" "%DIST%\%BIN_NAME%" >nul
    echo Built: %DIST%\%BIN_NAME%
) else (
    echo Could not find built binary at: %BUILT_PATH%
    exit /b 1
)

popd >nul
endlocal
