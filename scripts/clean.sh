#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

find "$PROJECT_ROOT" -type d -name ".svelte-kit" -exec rm -rf {} + 2>/dev/null || true
find "$PROJECT_ROOT" -type d -name "node_modules" -exec rm -rf {} + 2>/dev/null || true
find "$PROJECT_ROOT" -type d -name "target" -exec rm -rf {} + 2>/dev/null || true
find "$PROJECT_ROOT" -type d -name "build" -exec rm -rf {} + 2>/dev/null || true
find "$PROJECT_ROOT" -type d -name "dist" -exec rm -rf {} + 2>/dev/null || true
