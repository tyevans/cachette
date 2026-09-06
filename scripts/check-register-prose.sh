#!/bin/sh
# Entry point for the register prose check. Run from anywhere.
set -e
exec python3 "$(dirname "$0")/check_register_prose.py" "$@"
