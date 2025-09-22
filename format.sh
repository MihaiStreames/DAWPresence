#!/usr/bin/env bash
set -e

# Format with black
black DAWPY/

# Lint & sort imports with ruff
ruff check DAWPY/ --fix --unsafe-fixes

# Type-check with mypy
cd DAWPY
mypy .
cd ..