#!/bin/sh
# Entry point for the register numbering checks. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_registers.py" "$@"
