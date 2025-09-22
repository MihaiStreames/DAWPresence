@echo off
REM Format with black
black DAWPY/

REM Lint & sort imports with ruff
ruff check DAWPY/ --fix --unsafe-fixes

REM Type-check with mypy
cd DAWPY
mypy .
cd ..