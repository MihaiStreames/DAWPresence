#!/usr/bin/env bash
set -e

# Format with black
black DAWPY/

# Lint & sort imports with ruff (auto-fix)
ruff check DAWPY/ --fix

# Type-check with mypy
mypy DAWPY/