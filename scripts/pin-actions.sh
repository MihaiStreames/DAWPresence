#!/usr/bin/env bash
set -euo pipefail

CHECK=false

for arg in "$@"; do
  case "$arg" in
  -c | --check) CHECK=true ;;
  -h | --help)
    echo "Usage: $(basename "$0") [-c|--check] [-h|--help]"
    exit 0
    ;;
  *)
    echo "error: unknown option '$arg'"
    exit 1
    ;;
  esac
done

cd "$(dirname "$0")/.."

if [[ "$CHECK" == true ]]; then
  pinact run --check
else
  pinact run
fi
