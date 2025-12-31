@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" >nul

echo Cleaning build artifacts...
cargo clean

if exist "%ROOT%\dist" (
  rmdir /S /Q "%ROOT%\dist"
  echo Removed dist\
)

popd >nul
endlocal
