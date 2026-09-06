#!/bin/sh
# Entry point for the orientation check. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_orientation.py" "$@"
