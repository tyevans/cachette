#!/usr/bin/env bash
# Checks that each priority index lists every open thing exactly once.
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 scripts/check_priority.py
