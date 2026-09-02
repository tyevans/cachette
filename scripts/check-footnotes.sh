#!/bin/sh
# Entry point for the footnote check. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_footnotes.py" "$@"
