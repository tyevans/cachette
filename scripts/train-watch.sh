#!/usr/bin/env bash
# Prints one screen of what a training run is doing. Made for `watch -n1`.
#
# Usage:
#   watch -n1 scripts/train-watch.sh            the newest run
#   watch -n1 scripts/train-watch.sh RUNDIR     that run
#
# It reads the log from the instance over one connection, and it keeps that
# connection open between calls, so a call costs a few tens of milliseconds
# rather than a new handshake every second.
#
# When the instance has gone, it falls back to the copy in the run directory
# and says so. A run that ended still reports what it did.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dir="${1:-}"
if [ -z "$dir" ]; then
    dir="$(ls -td "$root"/runs/graviton/*/ 2>/dev/null | head -1)"
fi
[ -n "$dir" ] || { echo "No run directory. Pass one."; exit 0; }
dir="${dir%/}"

[ -f "$dir/instance.env" ] || {
    # The run ended and its teardown removed the state file. The log it
    # brought back is still worth reading.
    if [ -f "$dir/learn/train.log" ]; then
        exec "$root/scripts/train_watch.py" "$dir/learn/train.log" \
            --facts "$(basename "$dir")  (ended, reading the collected log)"
    fi
    echo "No instance.env and no collected log in $dir"
    exit 0
}
# shellcheck disable=SC1091
. "$dir/instance.env"

socket="$dir/watch.sock"
options=(
    -i "$dir/key.pem"
    -o StrictHostKeyChecking=no
    -o UserKnownHostsFile="$dir/known_hosts"
    -o ConnectTimeout=5
    -o ControlMaster=auto
    -o ControlPath="$socket"
    -o ControlPersist=120
    -o LogLevel=ERROR
)

fetched="$dir/watch.log"
raw="$(ssh "${options[@]}" "ec2-user@$HOST" \
    'cat cachette/runs/learn/train.log 2>/dev/null; printf "@@FACTS@@\n"; \
     uptime; cat /tmp/marker 2>/dev/null' 2>/dev/null)"

if [ -n "$raw" ]; then
    printf '%s' "${raw%%@@FACTS@@*}" > "$fetched"
    facts_body="$(printf '%s' "${raw#*@@FACTS@@}" | tr '\n' ' ')"
    load="$(printf '%s' "$facts_body" | sed -n 's/.*load average: \([0-9.]*\).*/\1/p')"
    marker="$(printf '%s' "$facts_body" | awk '{print $NF}')"
    facts="$(basename "$dir")  ${INSTANCE_TYPE:-} ${ZONE:-} \$${PRICE:-0}/hr  load ${load:-?}  ${marker:-}"
else
    facts="$(basename "$dir")  (no answer from the instance)"
    [ -f "$fetched" ] || fetched="$dir/train.log"
fi

started=0
if [ -f "$dir/instance.env" ]; then
    started="$(stat -c %Y "$dir/instance.env" 2>/dev/null || echo 0)"
fi

exec "$root/scripts/train_watch.py" "$fetched" \
    --price "${PRICE:-0}" \
    --generations "${TOTAL_GENERATIONS:-100}" \
    --started "$started" \
    --facts "$facts"
