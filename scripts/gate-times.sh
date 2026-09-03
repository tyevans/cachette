#!/bin/sh
# Entry point for the per-recipe gate timing harness. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/gate_times.py" "$@"
