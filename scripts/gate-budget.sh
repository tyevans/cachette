#!/usr/bin/env bash
# Runs a gate command, times it, and reports the cost against the local
# budget.
#
# The budget lives in one place: the development budget register. This script
# reads it from there. A second copy of the figure would decay, because
# nothing fails when two copies disagree.
#
# The budget belongs to one architecture. A figure taken on x86-64 is not a
# budget for arm64, and neither is evidence about the target platform. The
# script therefore reports without a comparison when the register holds no
# row for the architecture that runs it.
#
# The report never fails the build. Wall clock on a loaded machine is not a
# gate, and a timing assertion teaches everyone to ignore a red pipeline.
#
# References
#
# 1. Development budgets, the local register. docs/reference/development-budgets.md
# 2. Testing rules, section 3. .claude/rules/testing.md
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
register="$root/docs/reference/development-budgets.md"
arch="$(uname -m)"

start="$SECONDS"
"$@"
status="$?"
elapsed="$((SECONDS - start))"

budget="$(awk -F'|' -v arch="$arch" '
    /^\| Whole gate suite, budget/ {
        gsub(/^[ \t]+|[ \t]+$/, "", $5)
        gsub(/^[ \t]+|[ \t]+$/, "", $3)
        if ($5 == arch) { print $3 }
    }
' "$register")"

printf '\n'
printf 'Gate suite cost: %s s on %s.\n' "$elapsed" "$arch"

if [ -z "$budget" ]; then
    printf 'No budget row for %s in the development budget register.\n' "$arch"
    printf 'Measure this machine and add a row before you read this figure.\n'
    exit "$status"
fi

seconds="$(printf '%s' "$budget" | tr -cd '0-9')"
if [ -z "$seconds" ]; then
    printf 'The budget row for %s holds no number.\n' "$arch"
    exit "$status"
fi

printf 'Budget for %s: %s s.\n' "$arch" "$seconds"
if [ "$elapsed" -gt "$seconds" ]; then
    printf 'The suite is over its budget by %s s.\n' "$((elapsed - seconds))"
    printf 'Read the register, find the gate that grew, and file the work.\n'
else
    printf 'The suite is inside its budget by %s s.\n' "$((seconds - elapsed))"
fi

printf 'This figure describes a development machine. It is not evidence\n'
printf 'about the target platform.\n'

exit "$status"
