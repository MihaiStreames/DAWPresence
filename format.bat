@echo off
REM Format with black
black DAWPY/

REM Lint & sort imports with ruff (auto-fix)
ruff check DAWPY/ --fix

REM Type-check with mypy
mypy DAWPY/