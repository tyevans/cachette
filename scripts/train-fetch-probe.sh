#!/usr/bin/env bash
# Prove that the training launcher brings the weights back, and that a broken
# connection cannot destroy the ones already here.
#
# The launcher copies the resume point and the best centre of every strategy
# to this machine at every poll.[^1] A reclaimed spot instance therefore costs
# the work of one poll. Before that fetch existed the launcher copied nothing
# until `--stop` ran, and a reclaim lets `--stop` copy nothing at all: one run
# lost the best policy this project has measured.[^2]
#
# Nothing here rents a machine. The probe puts a stand-in for `scp` on the
# path, points it at a directory of files, and drives the real functions of
# the launcher over it. The stand-in can fail and can truncate, so the probe
# reaches the case a dying connection produces.
#
# **A case that demands the files stay unchanged is the load-bearing one.**
# The defect this guards against is silent: a truncated weight file that
# overwrote a whole one reads as a successful fetch. The truncation case below
# therefore proves first that the stand-in really truncates, and then that the
# launcher discards what it truncated.
#
# Add a case when the fetch changes. A case is cheap and a lost run is not.
#
# References
#
# [^1]: The training launcher. `scripts/graviton-train.sh`
# [^2]: Findings register, FND-746. `docs/FINDINGS.md`
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
launcher="$root/scripts/graviton-train.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

passed=0
failed=0

report() {
    if [ "$2" = "0" ]; then
        printf 'ok   %s\n' "$1"
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' "$1"
        failed=$((failed + 1))
    fi
}

# ------------------------------------------------------- the stand-in for scp

# It maps a remote specification onto FAKE_REMOTE. SCP_FAIL makes every copy
# fail, and SCP_TRUNCATE makes it deliver the first bytes of each file, which
# is what a connection that dies part way through leaves behind.
# SCP_FAIL_MATCH is a pattern, and a copy whose remote path matches it fails
# while every other copy succeeds.
mkdir -p "$work/bin"
cat > "$work/bin/scp" <<'SHIM'
#!/usr/bin/env bash
set -u
sources=()
while [ $# -gt 0 ]; do
    case "$1" in
        -i|-o) shift 2 ;;
        -r|-q) shift ;;
        *) sources+=("$1"); shift ;;
    esac
done
count=${#sources[@]}
dest="${sources[$((count - 1))]}"
unset 'sources[count-1]'
status=0
for spec in "${sources[@]}"; do
    path="${spec#*:}"
    if [ -n "${SCP_FAIL_MATCH:-}" ]; then
        case "$path" in
            $SCP_FAIL_MATCH) status=1; continue ;;
        esac
    fi
    for file in $FAKE_REMOTE/$path; do
        if [ ! -e "$file" ]; then
            status=1
            continue
        fi
        target="$dest"
        if [ -d "$dest" ]; then
            target="$dest/$(basename "$file")"
        fi
        if [ -d "$file" ]; then
            cp -r "$file/." "$target/"
            continue
        fi
        if [ "${SCP_TRUNCATE:-0}" = "1" ]; then
            head -c 4 "$file" > "$target"
        else
            cp "$file" "$target"
        fi
    done
done
if [ "${SCP_FAIL:-0}" = "1" ]; then
    exit 1
fi
exit $status
SHIM
chmod +x "$work/bin/scp"
PATH="$work/bin:$PATH"
export PATH

# ------------------------------------------------- the functions under test

# The launcher is one script that runs from the top, so the probe cannot
# source it. It takes the definitions by name instead, which is the same text
# the launcher runs.
take() {
    awk -v name="$1" '
        $0 ~ "^" name "\\(\\) \\{" { inside = 1 }
        inside { print }
        inside && $0 == "}" { exit }
    ' "$launcher"
}

# **A silent extraction is the way this probe goes green while proving
# nothing.** The launcher can be reindented, and a function whose body no
# longer ends on a bare closing brace comes out truncated. Each name below must
# yield a body that opens and closes, and the probe stops when one does not.
definitions="$work/definitions.sh"
for name in fetch_file fetch_checkpoints report_local collect refuse_out_argument \
    refuse_start_from; do
    body="$work/$name.body"
    take "$name" > "$body"
    if [ "$(head -1 "$body")" != "$name() {" ] || [ "$(tail -1 "$body")" != "}" ]
    then
        printf 'The probe could not take %s out of the launcher.\n' "$name" >&2
        exit 1
    fi
    cat "$body" >> "$definitions"
    printf '\n' >> "$definitions"
done
# shellcheck disable=SC1090
. "$definitions"

say() { printf '=== %s\n' "$1" >&2; }
ssh_options=()
remote="ec2-user@fake"
RUN_ID="probe-run"
export RUN_ID

# ------------------------------------------------------------- the fake world

# The trainer writes a resume point and a best centre for each strategy, a
# copy of the resume point for each generation, the report of the run, and the
# log. The names below are the names it uses.
#
# **The trainer writes each weight file under a temporary name first.** That
# name ends in `.part`, and the fake world holds one, as an instance does in
# the middle of a save. A fetch must never take it.
FAKE_REMOTE="$work/remote"
export FAKE_REMOTE
mkdir -p "$FAKE_REMOTE/cachette/runs/learn"
printf 'the resume point of alpha, generation twelve\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/alpha-latest.npz"
printf 'the resume point of alpha, generation eleven\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/alpha-gen011.npz"
printf 'the resume point of alpha, generation twelve\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/alpha-gen012.npz"
printf 'half of a resume point\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/alpha-latest.npz.4242.part"
printf 'the best validated centre of alpha\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/alpha.npz"
printf 'the resume point of beta, generation twelve\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/beta-latest.npz"
printf '{"held_out": 0.42}\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/report.json"
printf 'run attempt 1 running\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/status"
printf 'generation 12\n' \
    > "$FAKE_REMOTE/cachette/runs/learn/train.log"
printf 'the console of the run\n' > "$FAKE_REMOTE/run.log"

out_dir="$work/run"
mkdir -p "$out_dir"

# --------------------------------------------------------- case: a good fetch

fetch_output="$(fetch_checkpoints 2>&1)"
missing=""
for name in alpha-latest.npz alpha.npz beta-latest.npz report.json status; do
    [ -f "$out_dir/learn/$name" ] || missing="$missing $name"
done
if [ -n "$missing" ]; then
    report "a fetch brings back every file the trainer wrote (missing:$missing)" 1
else
    report "a fetch brings back every file the trainer wrote" 0
fi

if [ -f "$out_dir/learn/alpha-gen011.npz" ] \
    && [ -f "$out_dir/learn/alpha-gen012.npz" ]; then
    report "a fetch brings back the copy of every generation" 0
else
    report "a fetch brings back the copy of every generation" 1
fi

if compgen -G "$out_dir/learn/*.part" > /dev/null; then
    report "a fetch never takes a weight file the trainer is still writing" 1
else
    report "a fetch never takes a weight file the trainer is still writing" 0
fi

if grep -q 'fetched 5 weight files and 2 other files' <<<"$fetch_output"; then
    report "the fetch counts the weights apart from the other files" 0
else
    printf '  the output was: %s\n' "$fetch_output"
    report "the fetch counts the weights apart from the other files" 1
fi

if [ -s "$out_dir/last-fetch" ]; then
    report "the fetch records when it happened" 0
else
    report "the fetch records when it happened" 1
fi

# **Two mechanisms carry the files, so the cases below assert on both.** The
# weight files arrive as a group through a staging directory, and the report
# and the status arrive one at a time through a temporary name. A case that
# named only a weight file passed while the report path had no protection at
# all, and the probe found that by putting the defect back.
before="$(cat "$out_dir/learn/alpha.npz")"
before_report="$(cat "$out_dir/learn/report.json")"

# --------------------------------------------- case: the stand-in truncates

# **This case proves the fixture reaches the failure.** Without it the case
# below could pass because nothing was ever copied, rather than because the
# launcher threw the partial copy away.
SCP_TRUNCATE=1 SCP_FAIL=0 scp "$remote:cachette/runs/learn/alpha.npz" \
    "$work/truncated.npz"
if [ "$(wc -c < "$work/truncated.npz")" = "4" ]; then
    report "the stand-in delivers a partial file, so the case below is real" 0
else
    report "the stand-in delivers a partial file, so the case below is real" 1
fi

# ------------------------------------- case: a connection that dies part way

status=0
SCP_TRUNCATE=1 SCP_FAIL=1 fetch_checkpoints > "$work/broken.out" 2>&1 || status=$?
if [ "$status" = "0" ]; then
    report "a failed fetch does not end the run" 0
else
    report "a failed fetch does not end the run" 1
fi

if [ "$(cat "$out_dir/learn/alpha.npz")" = "$before" ]; then
    report "a partial copy does not overwrite a good weight file" 0
else
    printf '  the file now holds: %s\n' "$(cat "$out_dir/learn/alpha.npz")"
    report "a partial copy does not overwrite a good weight file" 1
fi

if [ "$(cat "$out_dir/learn/report.json")" = "$before_report" ]; then
    report "a partial copy does not overwrite a good report" 0
else
    printf '  the file now holds: %s\n' "$(cat "$out_dir/learn/report.json")"
    report "a partial copy does not overwrite a good report" 1
fi

if compgen -G "$out_dir/learn/*.part" > /dev/null \
    || compgen -G "$out_dir/learn.part" > /dev/null; then
    report "a failed fetch leaves no partial file behind" 1
else
    report "a failed fetch leaves no partial file behind" 0
fi

# -------------------------- case: the weights fail and the other files arrive

# **The fetch line counted every file together.** A round whose weight copy
# failed still fetched the report and the status, so it said that it fetched
# two files and moved the stamp. A reader then took stale weights for new
# ones. This case fails the weight copy alone, and the count of the other
# files in the output proves that the rest of the round went through.
printf 'the stamp of the last round that brought weights\n' > "$out_dir/last-fetch"
stamp_before="$(cat "$out_dir/last-fetch")"
weightless="$(SCP_FAIL_MATCH='*.npz' fetch_checkpoints 2>&1)"
if grep -q 'no weights arrived this round, and 2 other files did' <<<"$weightless" \
    && ! grep -q 'fetched' <<<"$weightless"; then
    report "a round with no weights says that no weights arrived" 0
else
    printf '  the output was: %s\n' "$weightless"
    report "a round with no weights says that no weights arrived" 1
fi

if [ "$(cat "$out_dir/last-fetch")" = "$stamp_before" ]; then
    report "a round with no weights leaves the stamp of the weights alone" 0
else
    report "a round with no weights leaves the stamp of the weights alone" 1
fi

# ------------------------------------ case: the run arguments name an output

# The follower fetches from the directory the launcher names. An `--out` in
# the run arguments sent the weights elsewhere, and every fetch then found
# nothing. The trainer reads an abbreviation of an option as the option.
refused=""
for arguments in "--generations 3 --out /tmp/elsewhere" "--out=/tmp/elsewhere" \
    "--seeds 2 --ou /tmp/elsewhere" "--ou=/tmp/elsewhere"; do
    if refuse_out_argument "$arguments" 2>/dev/null; then
        refused="$refused [$arguments]"
    fi
done
if [ -z "$refused" ]; then
    report "the launcher refuses an output directory in the run arguments" 0
else
    printf '  it accepted:%s\n' "$refused"
    report "the launcher refuses an output directory in the run arguments" 1
fi

if refuse_out_argument "--generations 3 --only alpha --pool 4" 2>/dev/null; then
    report "the launcher accepts run arguments that name no output" 0
else
    report "the launcher accepts run arguments that name no output" 1
fi

# **A refusal that runs after the rental refuses nothing that matters.** The
# launcher is one script that runs from the top, so the order of its lines is
# the order of its work.
refuse_line="$(grep -nF "refuse_out_argument \"\$train_args\"" "$launcher" | head -1 | cut -d: -f1)"
rent_line="$(grep -n 'run-instances' "$launcher" | head -1 | cut -d: -f1)"
if [ -n "$refuse_line" ] && [ -n "$rent_line" ] && [ "$refuse_line" -lt "$rent_line" ]; then
    report "the launcher refuses an output directory before it rents" 0
else
    printf '  the refusal is on line %s and the rental on line %s\n' \
        "${refuse_line:-none}" "${rent_line:-none}"
    report "the launcher refuses an output directory before it rents" 1
fi

# ------------------------------------------ case: the start file of a run

# **A run that starts from a file takes it on the first attempt only.** A
# start or a resume in the run arguments would reach every attempt, so the
# launcher refuses both before it rents. The trainer reads an abbreviation of
# an option as the option, so the launcher refuses each prefix as well.
start_file="$work/wonder-latest.npz"
printf 'a centre\n' > "$start_file"
accepted=""
for arguments in "--resume" "--res" "--generations 3 --resume" \
    "--start-from /x.npz" "--sta=/x.npz"; do
    if refuse_start_from "$arguments" "$start_file" 2>/dev/null; then
        accepted="$accepted [$arguments]"
    fi
done
if [ -z "$accepted" ]; then
    report "the launcher refuses a start or a resume beside a start file" 0
else
    printf '  it accepted:%s\n' "$accepted"
    report "the launcher refuses a start or a resume beside a start file" 1
fi

# `--styles` shares its first four characters with `--start-from`, so a rule
# that refused every short prefix would refuse a style run.
if refuse_start_from "--generations 3 --only alpha --styles bold" "$start_file" \
    2>/dev/null; then
    report "the launcher accepts run arguments beside a start file" 0
else
    report "the launcher accepts run arguments beside a start file" 1
fi

if refuse_start_from "--resume" "" 2>/dev/null; then
    report "the launcher refuses nothing when no start file is set" 0
else
    report "the launcher refuses nothing when no start file is set" 1
fi

if refuse_start_from "--generations 3" "$work/missing-latest.npz" 2>/dev/null; then
    report "the launcher refuses a start file that is not there" 1
else
    report "the launcher refuses a start file that is not there" 0
fi

printf 'a centre\n' > "$work/a centre.npz"
if refuse_start_from "--generations 3" "$work/a centre.npz" 2>/dev/null; then
    report "the launcher refuses a start file name that a command line splits" 1
else
    report "the launcher refuses a start file name that a command line splits" 0
fi

start_line="$(grep -nF "refuse_start_from \"\$train_args\"" "$launcher" | head -1 | cut -d: -f1)"
if [ -n "$start_line" ] && [ -n "$rent_line" ] && [ "$start_line" -lt "$rent_line" ]; then
    report "the launcher refuses a start file before it rents" 0
else
    printf '  the refusal is on line %s and the rental on line %s\n' \
        "${start_line:-none}" "${rent_line:-none}"
    report "the launcher refuses a start file before it rents" 1
fi

# ------------------------------------ case: the instance never answers again

# This is the reclaimed run. Nothing on the far side answers, and the weights
# the follower fetched are the whole of what survives.
local_report="$(SCP_FAIL=1 report_local 2>&1)"
if grep -q 'alpha-latest.npz' <<<"$local_report" \
    && grep -q 'Weights on this machine' <<<"$local_report"; then
    report "a reclaimed run reports the weights that are already here" 0
else
    printf '  the output was: %s\n' "$local_report"
    report "a reclaimed run reports the weights that are already here" 1
fi

collect_report="$(SCP_FAIL=1 collect 2>&1)"
if [ "$(cat "$out_dir/learn/alpha.npz")" = "$before" ]; then
    report "a collect from a dead instance keeps the fetched weights" 0
else
    report "a collect from a dead instance keeps the fetched weights" 1
fi
if grep -q 'gave nothing back' <<<"$collect_report" \
    && grep -q 'alpha.npz' <<<"$collect_report"; then
    report "a collect from a dead instance says what is here instead" 0
else
    printf '  the output was: %s\n' "$collect_report"
    report "a collect from a dead instance says what is here instead" 1
fi

# ------------------------------------------- case: a collect that does answer

collect_report="$(collect 2>&1)"
if [ -f "$out_dir/console.log" ] && [ -f "$out_dir/learn/train.log" ]; then
    report "a collect from a live instance takes the log and the console" 0
else
    printf '  the output was: %s\n' "$collect_report"
    report "a collect from a live instance takes the log and the console" 1
fi

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
