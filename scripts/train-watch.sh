#!/usr/bin/env bash
# Prints one screen of what a training run is doing. Made for a watch loop.
#
# Usage:
#   scripts/train-watch.sh              one screen of the newest run
#   scripts/train-watch.sh RUNDIR       one screen of that run
#   just train-screen                   the watch loop, in colour
#
# It reads the log from the instance over one shared connection, and it keeps
# a copy on disk. A call inside the refresh window opens nothing at all, so a
# one second loop does not open a connection every second.
#
# **The control socket lives outside the run directory, and that is the fix
# for the defect this script was written around.** OpenSSH creates the master
# socket under a temporary name of the control path plus seventeen bytes, and
# a Unix domain socket path holds 107 bytes. A socket inside the run
# directory reached 91 bytes, the temporary name reached 108, and every call
# failed with `unix_listener: path too long`. `ControlMaster=auto` does not
# fall back when the listener fails, so ssh exited 255 with no output and the
# screen said the instance did not answer while the run was healthy.
#
# It also never writes inside a run directory. A watcher is a reader.
#
# Every axis is a parameter:
#   CACHETTE_WATCH_REFRESH   seconds between remote reads, 5 by default
#   CACHETTE_WATCH_STATE     where the cache and the socket live
#   NO_COLOR                 set it for no colour, whatever else says
#   CLICOLOR_FORCE           set it for colour through a pipe
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dir="${1:-}"
if [ -z "$dir" ]; then
    dir="$(ls -td "$root"/runs/graviton/*/ 2>/dev/null | head -1)"
fi
[ -n "$dir" ] || { echo "No run directory. Pass one."; exit 0; }
dir="${dir%/}"
run_id="$(basename "$dir")"

# One script owns the start of a run, and both readers of a run call it. The
# progress feed and this screen disagreed on the elapsed time before that.
started="$("$root/scripts/train-started.sh" "$dir")"
now="$(date +%s)"
elapsed=0
[ "$started" != "0" ] && elapsed=$(( now - started ))

state="${CACHETTE_WATCH_STATE:-${XDG_RUNTIME_DIR:-/tmp}/cachette-train-watch}"
mkdir -p "$state" 2>/dev/null
cache="$state/$run_id.log"
socket="$state/$run_id.s"
fails="$state/$run_id.fails"
verdict="$state/$run_id.verdict"

render() {
    exec "$root/scripts/train_watch.py" "$@" \
        --elapsed "$elapsed" \
        --recent "${CACHETTE_WATCH_RECENT:-12}"
}

if [ ! -f "$dir/instance.env" ]; then
    # The run ended and its teardown removed the state file. The log it
    # brought back is still worth reading, and it is not a lost instance.
    collected="$dir/learn/train.log"
    [ -f "$collected" ] || collected="$dir/train.log"
    if [ -f "$collected" ]; then
        quiet=$(( now - $(stat -c %Y "$collected" 2>/dev/null || echo "$now") ))
        # The teardown removed the price and the generation target with the
        # state file, so this screen names neither. A guessed target reads
        # like a measured one.
        render "$collected" --link local --log-quiet "$quiet" \
            --generations "${CACHETTE_WATCH_GENERATIONS:-0}" \
            --facts "$run_id  (ended, reading the collected log)"
    fi
    echo "No instance.env and no collected log in $dir"
    exit 0
fi
# shellcheck disable=SC1091
. "$dir/instance.env"

options=(
    -i "$dir/key.pem"
    -o StrictHostKeyChecking=no
    -o UserKnownHostsFile="$dir/known_hosts"
    -o BatchMode=yes
    # **A loaded machine answers slowly.** The instance runs every core at
    # full tilt, so its sshd takes seconds to answer.
    -o ConnectTimeout=15
    -o ServerAliveInterval=30
    -o LogLevel=ERROR
)
# The guard against the defect in the header. A socket that cannot hold the
# temporary name loses the multiplexing rather than the whole connection.
if [ $(( ${#socket} + 17 )) -le 107 ]; then
    options+=(
        -o ControlMaster=auto
        -o ControlPath="$socket"
        -o ControlPersist=120
    )
fi

# The remote read. It brings the log, then the facts the screen needs: the
# clock of the instance, when the run last wrote a line, the core count and
# the load average. Every one of them is read, never guessed.
remote='cat cachette/runs/learn/train.log 2>/dev/null
printf "@@FACTS@@\n"
date +%s
stat -c %Y cachette/runs/learn/train.log 2>/dev/null || echo 0
nproc
cat /proc/loadavg'

refresh="${CACHETTE_WATCH_REFRESH:-5}"
cache_age=-1
if [ -f "$cache" ]; then
    cache_age=$(( now - $(stat -c %Y "$cache" 2>/dev/null || echo "$now") ))
fi

link=cached
asked="$cache_age"
if [ "$cache_age" -lt 0 ] || [ "$cache_age" -ge "$refresh" ]; then
    raw="$(ssh "${options[@]}" "ec2-user@$HOST" "$remote" 2>/dev/null)"
    if [ -z "$raw" ] && [ ! -f "$cache" ]; then
        # One retry, and only with nothing to fall back on. A refresh window
        # covers a single lost answer once a copy exists.
        raw="$(ssh "${options[@]}" "ec2-user@$HOST" "$remote" 2>/dev/null)"
    fi
    if [ -n "$raw" ]; then
        printf '%s' "${raw%%@@FACTS@@*}" > "$cache.part"
        printf '%s' "${raw#*@@FACTS@@}" > "$cache.facts.part"
        mv -f "$cache.part" "$cache"
        mv -f "$cache.facts.part" "$cache.facts"
        printf '0' > "$fails"
        rm -f "$verdict"
        link=live
        asked=0
    else
        streak=$(( $(cat "$fails" 2>/dev/null || echo 0) + 1 ))
        printf '%s' "$streak" > "$fails"
        link=silent
        # **A silent instance is not a lost one, and a lost one is not
        # silent.** These are spot instances, so a reclaim is a real
        # outcome. Ask the control plane, but only after a streak of
        # failures and only once every half minute, because the answer costs
        # an API call and a watch loop runs every second.
        if [ "$streak" -ge 3 ] && command -v aws >/dev/null 2>&1; then
            verdict_age=-1
            if [ -f "$verdict" ]; then
                verdict_age=$(( now - $(stat -c %Y "$verdict" 2>/dev/null \
                    || echo "$now") ))
            fi
            if [ "$verdict_age" -lt 0 ] || [ "$verdict_age" -ge 30 ]; then
                aws ec2 describe-instances \
                    --region "${REGION:-us-west-2}" \
                    --instance-ids "${INSTANCE_ID:-none}" \
                    --query 'Reservations[].Instances[].State.Name' \
                    --output text > "$verdict" 2>/dev/null
            fi
            case "$(cat "$verdict" 2>/dev/null)" in
                *terminated*|*shutting-down*|*stopped*|*stopping*) link=gone ;;
            esac
        fi
    fi
fi

load=-1
cores=0
log_quiet=-1
if [ -f "$cache.facts" ]; then
    mapfile -t facts_lines < <(tr -s ' \n' '\n' < "$cache.facts" | grep -v '^$')
    remote_now="${facts_lines[0]:-0}"
    log_written="${facts_lines[1]:-0}"
    cores="${facts_lines[2]:-0}"
    load="${facts_lines[3]:--1}"
    if [ "$remote_now" -gt 0 ] 2>/dev/null && [ "$log_written" -gt 0 ] 2>/dev/null
    then
        # The age of the newest line, measured against the clock of the
        # instance. A local clock would fold this watcher's own lag into it.
        log_quiet=$(( remote_now - log_written + (asked > 0 ? asked : 0) ))
    fi
fi

render "$cache" \
    --price "${PRICE:-0}" \
    --generations "${TOTAL_GENERATIONS:-100}" \
    --link "$link" \
    --asked "$asked" \
    --log-quiet "$log_quiet" \
    --load "$load" \
    --cores "$cores" \
    --facts "$run_id  ${INSTANCE_TYPE:-} ${ZONE:-} \$$(printf '%.4f' \
        "${PRICE:-0}")/hr"
