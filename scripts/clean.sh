#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $(basename "$0") [-a|--all] [-h|--help]"
  echo ""
  echo "Options:"
  echo "  -a, --all   remove target and installer output"
  echo "  -h, --help  show this message"
}

ALL=false
for arg in "$@"; do
  case "$arg" in
    -a|--all)  ALL=true ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Error: unknown option '$arg'"; usage; exit 1 ;;
  esac
done

cd "$(dirname "$0")/.."

[[ -d dist ]] && echo "dist" && rm -rf dist

if [[ "$ALL" == true ]]; then
  [[ -d installer/Output ]] && echo "installer/Output" && rm -rf installer/Output
  cargo clean
fi

echo "Clean complete"
