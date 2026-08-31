#!/bin/sh
# Entry point for the backlog checks. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_backlog.py" "$@"
