#!/bin/sh
# Entry point for the decision record checks. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_adrs.py" "$@"
