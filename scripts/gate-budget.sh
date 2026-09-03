#!/usr/bin/env bash
# Runs a gate command, times it, and reports the cost against the local
# budget.
#
# The budget lives in one place: the development budget register. One reader
# gets it from there and both cost reports call that reader. A second copy of
# the figure would decay, because nothing fails when two copies disagree.
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
# 3. The budget reader. scripts/gate-budget-figure.sh
# 4. The per-recipe timing harness. scripts/gate-times.sh
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
arch="$(uname -m)"

start="$SECONDS"
"$@"
status="$?"
elapsed="$((SECONDS - start))"

seconds="$("$root/scripts/gate-budget-figure.sh" "$arch")"

printf '\n'
printf 'Gate suite cost: %s s on %s.\n' "$elapsed" "$arch"

if [ -z "$seconds" ]; then
    printf 'No budget row for %s in the development budget register.\n' "$arch"
    printf 'Measure this machine and add a row before you read this figure.\n'
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
printf 'Run scripts/gate-times.sh to see which recipe holds the cost.\n'

exit "$status"
