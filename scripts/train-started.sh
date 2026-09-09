#!/usr/bin/env bash
# Prints the Unix time at which a training run started, or 0.
#
# Usage:
#   scripts/train-started.sh RUNDIR
#
# **One declaration site for the start of a run.** Two readers report the
# spend of a run, and each needs the wall clock. A reader that summed the
# per-generation seconds of every strategy reported six times the wall clock,
# because the strategies run at the same time, and it reported zero before the
# first generation finished. The money is real from the first minute, so zero
# is the worse of the two answers.
#
# The launcher builds the run identifier from the UTC clock, so the directory
# name carries the start. That survives the teardown, which removes the state
# file of the instance. The modification time of the state file is the
# fallback for a directory that carries no stamp.
set -uo pipefail

dir="${1:-}"
[ -n "$dir" ] || { echo 0; exit 0; }
dir="${dir%/}"

stamp="$(basename "$dir")"
stamp="${stamp#cachette-train-}"
stamp="${stamp%-*}"
if [ ${#stamp} -eq 15 ]; then
    started="$(date -u -d \
        "${stamp:0:8} ${stamp:9:2}:${stamp:11:2}:${stamp:13:2}" +%s 2>/dev/null \
        || echo 0)"
    if [ "$started" != "0" ]; then
        echo "$started"
        exit 0
    fi
fi

for candidate in instance.env remote.sh; do
    if [ -f "$dir/$candidate" ]; then
        stat -c %Y "$dir/$candidate" 2>/dev/null && exit 0
    fi
done
echo 0
