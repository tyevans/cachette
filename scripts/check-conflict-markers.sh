#!/bin/sh
# Entry point for the merge conflict marker check. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_conflict_markers.py" "$@"
