#!/bin/sh
# Entry point for the citation check. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_citations.py" "$@"
