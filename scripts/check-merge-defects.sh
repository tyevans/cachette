#!/bin/sh
# Entry point for the merge defect checks. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_merge_defects.py" "$@"
