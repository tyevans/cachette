#!/usr/bin/env bash
# Prints the gate suite budget, in seconds, for one architecture.
#
# The budget lives in one place: the development budget register. Two scripts
# report a gate cost against it, and both read the figure from here rather
# than from a copy of their own. A second copy would decay, because nothing
# fails when two copies disagree.
#
# The budget belongs to one architecture. A figure taken on x86-64 is not a
# budget for arm64, and neither is evidence about the target platform. This
# script prints nothing and exits 1 when the register holds no row for the
# architecture it is given, so a caller reports without a comparison rather
# than borrowing another machine's row.
#
# Usage: gate-budget-figure.sh [architecture]
# The architecture defaults to the output of `uname -m`.
#
# References
#
# 1. Development budgets, the local register. docs/reference/development-budgets.md
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
register="$root/docs/reference/development-budgets.md"
arch="${1:-$(uname -m)}"

value="$(awk -F'|' -v arch="$arch" '
    /^\| Whole gate suite, budget/ {
        gsub(/^[ \t]+|[ \t]+$/, "", $5)
        gsub(/^[ \t]+|[ \t]+$/, "", $3)
        if ($5 == arch) { print $3 }
    }
' "$register")"

seconds="$(printf '%s' "$value" | tr -cd '0-9')"
if [ -z "$seconds" ]; then
    exit 1
fi

printf '%s\n' "$seconds"
