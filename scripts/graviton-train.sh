#!/usr/bin/env bash
# Trains the learner seat on the target platform, and reports what it costs.
#
# The project targets AWS Graviton, and the primary target triple is
# aarch64-unknown-linux-gnu. This script rents one Graviton machine, builds
# the engine on it, measures how fast it runs, trains the learner, follows
# the run while it happens, brings the weights back, and destroys everything
# it made.
#
# It is the sibling of the benchmark launcher, and it keeps that script's
# safety: the same teardown trap, the same orphan check, and the same
# self-destruct deadline on the instance.[^1] It adds three things a long
# job needs that a benchmark sweep does not.
#
# 1. **Nothing spends money before a person sees the number.** The script
#    prices the zones, picks the cheapest, prints the instance type, the
#    zone, the price an hour and the most the run can cost, and stops for a
#    confirmation.
# 2. **A progress feed a person can act on.** While the run goes, the script
#    pulls the log and renders a dashboard: the held-out bar the policy must
#    beat, the spread that says whether the search is still searching, the
#    dollars spent so far and for each generation, and the time left.
# 3. **A way to stop and keep the work.** `--stop` ends a run, brings back
#    everything the instance has written, and terminates the machine.
#
# Usage:
#   scripts/graviton-train.sh                 price it, ask, then run
#   scripts/graviton-train.sh --dry-run       price it and print, launch nothing
#   scripts/graviton-train.sh --attach DIR    follow a run this script started
#   scripts/graviton-train.sh --stop DIR      end that run, keep what it has
#   scripts/graviton-train.sh --orphans       list what a run left behind
#
# Settings, as environment variables:
#   CACHETTE_TRAIN_INSTANCE   the instance type. Default c7g.16xlarge
#   CACHETTE_TRAIN_REGION     the region. Default us-west-2
#   CACHETTE_TRAIN_OUT        where the results land. Default a directory
#                             under `runs/graviton`
#   CACHETTE_TRAIN_ARGS       the arguments the trainer receives
#   CACHETTE_TRAIN_MAX_MINUTES  the wall clock cap. The instance destroys
#                             itself after this many minutes whatever else
#                             happens. Default 360
#   CACHETTE_TRAIN_ON_DEMAND  set to 1 to buy on demand rather than spot.
#                             Spot is about a third of the price and can be
#                             taken back, and the trainer resumes
#   CACHETTE_TRAIN_CONFIRM    set to `yes` to skip the prompt. Use it only
#                             from something that already asked a person
#   CACHETTE_TRAIN_STOP_ON_COLLAPSE  set to 0 to keep paying after the
#                             search stops. Default 1, which ends the run
#   CACHETTE_TRAIN_PROBE_ONLY set to 1 to measure the throughput and train
#                             nothing. This is the cheap way to get a
#                             ticks-a-second figure for the costs register
#
# What it needs: the AWS command line tool, authenticated, with permission
# to run an instance, and `ssh`, `scp`, `tar`, `git` and `python3` here.
#
# What it costs: one instance for the length of the run, plus a root volume
# that lives and dies with it. The confirmation prints both. Nothing else
# bills: it creates no gateway, no load balancer, and no volume beyond that
# root.
#
# References
#   [^1]: The benchmark launcher. `scripts/graviton-benchmark.sh`
set -euo pipefail

# Not readonly: the stop path sources instance.env, which sets this
# again for the run it names, and a readonly name makes that fail.
REGION="${CACHETTE_TRAIN_REGION:-us-west-2}"
# A 64 core Graviton. The evolution strategy plays one episode for each pair
# of a candidate and a seed, and the episodes share nothing, so the work
# divides over the cores. The throughput probe below measures whether it
# actually does, rather than assuming it.
# Not readonly, for the same reason REGION is not: the stop path sources
# instance.env, and that file sets this again for the run it names. A readonly
# name makes the source fail and the stop path then cannot end the run.
#
# **Every name instance.env writes must be writable here.** REGION was fixed
# alone and this one was left, so the stop path broke a second time on the same
# cause. The names the file writes are listed where it is written.
INSTANCE_TYPE="${CACHETTE_TRAIN_INSTANCE:-c7g.16xlarge}"
readonly ROOT_VOLUME_GB=30
readonly TAG_PROJECT="cachette"
readonly TAG_PURPOSE="learner-training-run"
# The instance destroys itself after this many minutes, whatever else
# happens. The teardown below is the ordinary path. This is the net under
# it: it holds when this machine loses power, when the connection dies for
# good, and when the trainer hangs. A spot instance that hangs is the
# expensive failure, and this is what bounds it.
readonly MAX_MINUTES="${CACHETTE_TRAIN_MAX_MINUTES:-360}"
readonly STOP_ON_COLLAPSE="${CACHETTE_TRAIN_STOP_ON_COLLAPSE:-1}"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# **The compiled extension is a function of the sources that build it, and of
# nothing else in the tree.** A run that changes only a document rebuilds the
# same bytes, and the build costs about a quarter of an hour on the target
# before the first episode runs. The key below names the build inputs, so a
# later run with the same inputs installs the wheel the earlier run made.
readonly WHEEL_CACHE="${CACHETTE_WHEEL_CACHE:-$HOME/.cache/cachette-wheels}"

# **The controller baseline is a function of the engine build too.** A run
# plays the built-in controller over the whole held-out seed set, to set the
# bar the trained policy must beat, and that pass took over nine minutes of a
# rented machine. The trainer keeps each measurement under a key that holds
# the engine build, the world, the seeds and the objective. This directory
# travels with the run, so a later run with the same inputs reads the number
# instead of playing for it again.
readonly BASELINE_CACHE="${CACHETTE_BASELINE_CACHE:-$HOME/.cache/cachette-baselines}"

# The key of the engine build. **One script derives it and three things read
# it**: the wheel cache above, the baseline cache beside it, and the trainer on
# the instance. A second derivation would serve a stale wheel or a stale
# baseline after an engine change, and nothing would fail.
build_key() {
    bash "$root/scripts/build-key.sh"
}

say() { printf '=== %s\n' "$1" >&2; }
die() { printf '%s\n' "$1" >&2; exit 1; }

# ---------------------------------------------------------------- orphan mode

# Lists every instance that a run of this script created and left running.
list_orphans() {
    printf 'Instances tagged Purpose=%s that are not terminated, in %s:\n' \
        "$TAG_PURPOSE" "$REGION"
    aws ec2 describe-instances \
        --region "$REGION" \
        --filters "Name=tag:Purpose,Values=$TAG_PURPOSE" \
        "Name=instance-state-name,Values=pending,running,stopping,stopped" \
        --query 'Reservations[].Instances[].[InstanceId,InstanceType,State.Name,LaunchTime]' \
        --output text
    printf 'Key pairs and security groups tagged Purpose=%s:\n' "$TAG_PURPOSE"
    aws ec2 describe-key-pairs --region "$REGION" \
        --filters "Name=tag:Purpose,Values=$TAG_PURPOSE" \
        --query 'KeyPairs[].KeyName' --output text
    aws ec2 describe-security-groups --region "$REGION" \
        --filters "Name=tag:Purpose,Values=$TAG_PURPOSE" \
        --query 'SecurityGroups[].GroupId' --output text
    printf 'An empty list under a heading means nothing is left.\n'
}

# ------------------------------------------------------------------ the modes

mode="run"
state_dir=""
case "${1:-}" in
    --orphans) list_orphans; exit 0 ;;
    --dry-run) mode="dry" ;;
    --attach) mode="attach"; state_dir="${2:?--attach needs the run directory}" ;;
    --stop) mode="stop"; state_dir="${2:?--stop needs the run directory}" ;;
    "") ;;
    *) die "Unknown option: $1. Read the top of this file for the options." ;;
esac

# ------------------------------------------------------------------- the plan

# What the trainer runs. Every axis is a parameter, so a longer or a wider
# run is a setting here and not a change to a file.
#
# The worker count matches the cores of the instance, because the batch step
# takes it and the throughput probe reports what each worker contributed.
# **The validation seeds are the only figure that compares across
# generations**, so they carry the resolution of the whole run. A win moves
# the return by the win weight divided by the seed count, so eight seeds can
# only report a whole eighth and a policy that improves inside an eighth
# looks flat.
#
# The seed count also sets the error of that figure, and the error decides
# whether a gain is visible at all. The yardstick gives the learner seat back
# to the built-in controller, so every faction of that game is the same
# controller and its win share is one third. A policy that reaches the
# yardstick has reached chance. Over thirty-two worlds the standard error of
# a win share near one third is 0.083, which is larger than the advantage the
# yardstick shows over chance, so thirty-two worlds cannot separate the two.
# Over one hundred and twenty-eight worlds the error is near 0.04.
#
# They are nearly free. Validation plays one policy, so it runs as many
# worlds as it has seeds, and one hundred and twenty-eight worlds on
# sixty-four workers takes about two minutes against the ten minutes of a
# generation.
default_args="--generations 20 --population 24 --seeds 6 --holdout 256 \
--sigma 1.5 --learning-rate 0.3 --validation 128 --validate-every 2"
train_args="${CACHETTE_TRAIN_ARGS:-$default_args}"

# How many generations the whole run takes, read out of the arguments and
# multiplied by the strategies the trainer will train. The progress feed
# needs it to say how many are left. A run that names no strategy trains
# every one the trainer declares.
generations="$(printf '%s' "$train_args" | sed -n 's/.*--generations \([0-9]*\).*/\1/p')"
generations="${generations:-20}"

# How many worlds one generation holds. The trainer plays every candidate on
# every seed, and it puts the whole set in one batch, so this product is the
# batch the throughput probe must measure. **The probe once gave each worker
# one world, and the trainer never runs that shape.** A figure taken that way
# describes the probe and not a training run.
population="$(printf '%s' "$train_args" | sed -n 's/.*--population \([0-9]*\).*/\1/p')"
population="${population:-24}"
probe_seeds="$(printf '%s' "$train_args" | sed -n 's/.*--seeds \([0-9]*\).*/\1/p')"
probe_seeds="${probe_seeds:-6}"
probe_worlds=$((population * probe_seeds))
# **The trainer answers how many strategies it would train.** It reads
# `--only` and it reads the play styles, and a style run holds a table the
# built-in weightings do not. A count taken any other way here is a second
# declaration of the list, and this script held two of them: one that counted
# the rows of the trainer's source with a regular expression, and one that
# read the table before the styles replaced it. Both were wrong for a style
# run, and the run failed after it paid for the instance.
strategies="$(cd "$root" && uv run python -m cachette.learn \
    --print-strategies $train_args 2>/dev/null | wc -w)"
[ "${strategies:-0}" -ge 1 ] || strategies=1
total_generations=$((generations * strategies))

# **The trainer answers the world as well.** The extent, the faction count and
# the tick limit all reach the cost of a run: a tick of a larger world costs
# more, and a longer game holds more ticks. The preview below states them, so
# the person who approves the price sees the world the price is for. This
# script derives none of them.
world_preview="$(cd "$root" && uv run python -m cachette.learn \
    --print-world $train_args 2>/dev/null \
    | awk -F'\t' '{ held[$1] = $2 }
        END { printf "%sx%s, %s factions, tick limit %s, interval %s",
              held["width"], held["height"], held["factions"],
              held["tick_limit"], held["decision_interval"] }')"
world_preview="${world_preview:-unknown}"

# **The trainer does not write its weights after every generation.** It
# writes them when a validation pass finds a centre better than the best it
# has seen, and it validates every few generations.
#
# The trainer once wrote nothing until a whole strategy ended, so an early stop
# threw the strategy away, and this launcher refused a run with no validation
# seeds for that reason. That defect is fixed: the trainer now writes a resume
# point every generation, unconditionally, beside the best validated centre.[^2]
# An interruption costs one generation.
#
# The refusal is gone. Validation still decides which centre is kept as the
# best, so a run with none keeps only its resume point, and the preview says so.
#
# References
#   [^2]: Findings register, FND-625. `docs/FINDINGS.md`
validation="$(printf '%s' "$train_args" | sed -n 's/.*--validation \([0-9]*\).*/\1/p')"
validation="${validation:-6}"
validate_every="$(printf '%s' "$train_args" \
    | sed -n 's/.*--validate-every \([0-9]*\).*/\1/p')"
validate_every="${validate_every:-3}"


# ------------------------------------------------------------------ the price

# Reads the spot price in every zone and names the cheapest. The price
# varies about a fifth between the zones of one region, so the zone is worth
# choosing rather than letting the account pick one.
pick_zone() {
    aws ec2 describe-spot-price-history \
        --region "$REGION" \
        --instance-types "$INSTANCE_TYPE" \
        --product-descriptions "Linux/UNIX" \
        --start-time "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --query 'SpotPriceHistory[].[SpotPrice,AvailabilityZone]' \
        --output text | sort -n | head -1
}

on_demand_price() {
    # The pricing interface lives in one region and prices every region, and
    # it names a region by its long name rather than its code.
    aws pricing get-products --region us-east-1 --service-code AmazonEC2 \
        --filters "Type=TERM_MATCH,Field=instanceType,Value=$INSTANCE_TYPE" \
        "Type=TERM_MATCH,Field=regionCode,Value=$REGION" \
        "Type=TERM_MATCH,Field=operatingSystem,Value=Linux" \
        "Type=TERM_MATCH,Field=tenancy,Value=Shared" \
        "Type=TERM_MATCH,Field=preInstalledSw,Value=NA" \
        "Type=TERM_MATCH,Field=capacitystatus,Value=Used" \
        --max-items 1 --query 'PriceList[0]' --output text 2>/dev/null \
        | python3 -c 'import json,sys
try:
    row = json.loads(sys.stdin.read())
    terms = row["terms"]["OnDemand"]
    offer = next(iter(terms.values()))
    price = next(iter(offer["priceDimensions"].values()))
    print(price["pricePerUnit"]["USD"])
except Exception:
    print("")' 2>/dev/null || printf ''
}

# --------------------------------------------------------- the run directory

run_id="cachette-train-$(date -u +%Y%m%d-%H%M%S)-$$"
if [ -n "$state_dir" ]; then
    out_dir="$state_dir"
    [ -f "$out_dir/instance.env" ] || die "No run in $out_dir. It holds no instance.env"
    # shellcheck disable=SC1091
    . "$out_dir/instance.env"
    run_id="$RUN_ID"
else
    out_dir="${CACHETTE_TRAIN_OUT:-$root/runs/graviton/$run_id}"
fi
mkdir -p "$out_dir"

# ------------------------------------------------------------------- teardown

instance_id=""
group_id=""
key_name=""
key_file="$out_dir/key.pem"
keep_instance=0

# Destroys everything this run created. It runs on every exit path, and each
# step reports rather than failing, so one failure cannot strand the rest.
teardown() {
    local status=$?
    set +e
    printf '\n--- teardown ---\n' >&2

    if [ -n "$instance_id" ] && [ "$keep_instance" != "1" ]; then
        printf 'Terminating %s\n' "$instance_id" >&2
        aws ec2 terminate-instances --region "$REGION" \
            --instance-ids "$instance_id" --output text >/dev/null
        # The security group cannot be deleted until the network interface of
        # the instance is gone, so this wait is not optional.
        aws ec2 wait instance-terminated --region "$REGION" \
            --instance-ids "$instance_id"
        printf 'Terminated %s\n' "$instance_id" >&2
    elif [ -n "$instance_id" ]; then
        printf 'Instance %s is still running and still billing.\n' "$instance_id" >&2
        printf 'Follow it with: scripts/graviton-train.sh --attach %s\n' "$out_dir" >&2
        printf 'End it with:    scripts/graviton-train.sh --stop %s\n' "$out_dir" >&2
    fi

    if [ "$keep_instance" != "1" ]; then
        if [ -n "$group_id" ]; then
            aws ec2 delete-security-group --region "$REGION" --group-id "$group_id" \
                && printf 'Deleted security group %s\n' "$group_id" >&2
        fi
        if [ -n "$key_name" ]; then
            aws ec2 delete-key-pair --region "$REGION" --key-name "$key_name" \
                && printf 'Deleted key pair %s\n' "$key_name" >&2
        fi
        # The private key is the one secret this script writes to disk. It is
        # useless once the instance is gone, so it goes with it.
        rm -f "$key_file"
        rm -f "$out_dir/instance.env"
        printf 'Checking that nothing this script creates is still running.\n' >&2
        list_orphans >&2
    fi
    exit "$status"
}

# ----------------------------------------------------------- follow a run

ssh_options=()
remote=""

# Renders the dashboard from the log the instance has written so far.
render_progress() {
    local log="$out_dir/train.log"
    local report="$out_dir/report.json"
    scp "${ssh_options[@]}" "$remote:cachette/runs/learn/train.log" "$log" \
        2>/dev/null || return 0
    scp "${ssh_options[@]}" "$remote:cachette/runs/learn/report.json" "$report" \
        2>/dev/null || true
    python3 "$root/scripts/train_progress.py" "$log" \
        --price "$PRICE" \
        --generations "$TOTAL_GENERATIONS" \
        --instance-type "$INSTANCE_TYPE" \
        --zone "$ZONE" \
        --run-id "$RUN_ID" \
        ${_report_flag:+--report "$report"} >&2 || true
}

# Brings back everything the instance has written. It runs before the machine
# is destroyed on every path, including an early stop, so the weights of a
# run that ends early survive it.
collect() {
    say "Collecting the results"
    mkdir -p "$out_dir/learn"
    scp -r "${ssh_options[@]}" "$remote:cachette/runs/learn/." "$out_dir/learn/" \
        2>/dev/null || printf 'Nothing to collect yet.\n' >&2
    scp "${ssh_options[@]}" "$remote:run.log" "$out_dir/console.log" 2>/dev/null || true
    scp "${ssh_options[@]}" "$remote:/tmp/throughput.txt" \
        "$out_dir/throughput.txt" 2>/dev/null || true
    if [ -f "$out_dir/learn/train.log" ]; then
        python3 "$root/scripts/train_progress.py" "$out_dir/learn/train.log" \
            --rows --run-id "$RUN_ID" > "$out_dir/generations.jsonl" || true
    fi
    printf 'Results are in %s\n' "$out_dir" >&2
    ls -la "$out_dir/learn" 2>/dev/null >&2 || true
}

# Sets up the connection to an instance this script already made.
connect() {
    ssh_options=(
        -i "$key_file"
        -o StrictHostKeyChecking=accept-new
        -o UserKnownHostsFile="$out_dir/known_hosts"
        -o ConnectTimeout=10
        -o ServerAliveInterval=30
        -o ServerAliveCountMax=20
    )
    remote="ec2-user@$HOST"
}

# Fetch the wheel a build produced, once, into the cache. A run that received
# a wheel already has it, and a run whose build has not finished has nothing to
# fetch yet, so this stays quiet until there is something to take.
fetch_wheel() {
    [ -n "${wheel_key:-}" ] || return 0
    local target="$WHEEL_CACHE/$wheel_key"
    compgen -G "$target/*.whl" >/dev/null && return 0
    mkdir -p "$target.part"
    if scp "${ssh_options[@]}" "$remote:wheelhouse/*.whl" "$target.part/" >/dev/null 2>&1; then
        rm -rf "$target"
        mv "$target.part" "$target"
        say "Kept the wheel for build $wheel_key. The next run with these sources skips the compiler"
    else
        rm -rf "$target.part"
    fi
}

# Fetch every controller baseline the instance measured, into the cache. A run
# that measured nothing new copies files the cache already holds, which costs
# one transfer of a few kilobytes and never a wrong answer: each file names
# every input its number answers for, and a reader compares them all.
fetch_baselines() {
    mkdir -p "$BASELINE_CACHE"
    scp "${ssh_options[@]}" "$remote:.cache/cachette-baselines/*.json" \
        "$BASELINE_CACHE/" >/dev/null 2>&1 || true
}

# The loop that follows a running job. It polls, renders the dashboard, and
# ends when the marker says the run finished, when the search has stopped, or
# when the local deadline passes.
follow() {
    local deadline=$(( $(date +%s) + MAX_MINUTES * 60 ))
    local finished=""
    _report_flag=1
    while [ -z "$finished" ]; do
        sleep 120
        if [ "$(date +%s)" -ge "$deadline" ]; then
            say "The wall clock cap of $MAX_MINUTES minutes passed. Ending the run"
            finished="capped"
            break
        fi
        local state
        state="$(ssh "${ssh_options[@]}" "$remote" 'cat /tmp/marker 2>/dev/null' \
            2>/dev/null || true)"
        render_progress
        fetch_wheel
        fetch_baselines
        case "$state" in
            done*) finished="done" ;;
            failed*)
                say "The run failed on the instance. The last of the log follows"
                ssh "${ssh_options[@]}" "$remote" 'tail -40 run.log' >&2 || true
                finished="failed"
                ;;
            "") printf '   no answer from the instance. Retrying\n' >&2 ;;
        esac
        if [ -z "$finished" ] && [ "$STOP_ON_COLLAPSE" = "1" ] \
            && [ -f "$out_dir/train.log" ]; then
            # The search has stopped when every candidate of the last few
            # generations scored the same. A run in that state learns nothing
            # and keeps billing, so it ends here.
            if python3 "$root/scripts/train_progress.py" "$out_dir/train.log" \
                --collapsed; then :; else
                say "The search stopped. Ending the run and keeping the weights"
                finished="collapsed"
            fi
        fi
    done
    say "The run ended: $finished"
}

# ------------------------------------------------------- attach and stop modes

if [ "$mode" = "attach" ] || [ "$mode" = "stop" ]; then
    instance_id="$INSTANCE_ID"
    group_id="$GROUP_ID"
    key_name="$KEY_NAME"
    connect
    if [ "$mode" = "stop" ]; then
        trap teardown EXIT INT TERM
        say "Ending the run on $instance_id and keeping what it wrote"
        # The trainer writes the weights after a generation it improved on,
        # so what is on the instance now is what survives. Nothing is
        # interrupted before it is copied.
        collect
        exit 0
    fi
    # Attaching must not destroy a run that the follower simply stopped
    # watching, so this path keeps the instance unless the run ends.
    keep_instance=1
    trap teardown EXIT INT TERM
    render_progress
    follow
    keep_instance=0
    collect
    exit 0
fi

# ------------------------------------------------------- price and confirm

say "Pricing $INSTANCE_TYPE in $REGION"
price=""
zone=""
market="spot"
if [ "${CACHETTE_TRAIN_ON_DEMAND:-0}" = "1" ]; then
    market="on-demand"
    price="$(on_demand_price)"
    [ -n "$price" ] || die "Could not read the on-demand price. It will not guess."
    zone=""
else
    read -r price zone <<< "$(pick_zone)"
    [ -n "$price" ] || die "Could not read a spot price for $INSTANCE_TYPE in $REGION."
fi

cores="$(aws ec2 describe-instance-types --region "$REGION" \
    --instance-types "$INSTANCE_TYPE" \
    --query 'InstanceTypes[0].VCpuInfo.DefaultVCpus' --output text)"

# The cap is the honest bound. Whatever the run does, the instance destroys
# itself after it, so this multiplication is the most the run can cost.
max_cost="$(python3 -c "print(f'{$price * $MAX_MINUTES / 60:.2f}')")"
# The root volume is charged for the time it exists. A gp3 volume costs about
# eight cents a gigabyte a month, which is a fraction of a cent an hour here.
volume_cost="$(python3 -c "print(f'{$ROOT_VOLUME_GB * 0.08 / 730 * $MAX_MINUTES / 60:.3f}')")"

cat >&2 <<PLAN

  =============== nothing has been created yet ===============

  instance      $INSTANCE_TYPE, $cores vCPU
  market        $market${zone:+, zone $zone}
  region        $REGION
  price         \$$price an hour
  wall cap      $MAX_MINUTES minutes, after which the instance destroys itself
  MOST IT COSTS \$$max_cost for the instance, plus \$$volume_cost for the disk

  trainer       $train_args
  world         $world_preview
  generations   $total_generations across $strategies strategies
  results       $out_dir

  The cap is the bound, not the estimate. The run stops when the trainer
  finishes, when the search stops, or at the cap, whichever comes first.

  Stopping early keeps the weights. The trainer writes a resume point every
  generation, so an interruption costs one generation and never a strategy.
  It validates every $validate_every generations and keeps the best centre
  beside that. A spot instance can be taken back at any time, and the same
  rule holds.

PLAN

if [ "$mode" = "dry" ]; then
    say "Dry run. Nothing was created."
    exit 0
fi

if [ "${CACHETTE_TRAIN_CONFIRM:-}" = "yes" ]; then
    say "Confirmed by CACHETTE_TRAIN_CONFIRM"
elif [ -t 0 ]; then
    printf '  Type yes to spend up to $%s: ' "$max_cost" >&2
    read -r answer
    [ "$answer" = "yes" ] || die "Not confirmed. Nothing was created."
else
    die "There is no terminal to ask, and CACHETTE_TRAIN_CONFIRM is not set.
Nothing was created. A run nobody authorised is worse than no run."
fi

# --------------------------------------------------------------------- launch

trap teardown EXIT INT TERM

commit="$(git -C "$root" rev-parse HEAD)"
dirty="clean"
if [ -n "$(git -C "$root" status --porcelain)" ]; then dirty="modified"; fi
say "Run $run_id, commit $commit, working tree $dirty"

# Amazon Linux 2023 for arm64. The public parameter always names the current
# image, so this script pins no image identifier of its own.
ami="$(aws ssm get-parameters --region "$REGION" \
    --names /aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64 \
    --query 'Parameters[0].Value' --output text)"
say "Image $ami"

# The security group admits this machine and nothing else. A run that cannot
# learn its own address stops, because the alternative is an open port.
my_ip="$(curl --silent --show-error --fail --max-time 10 https://checkip.amazonaws.com || true)"
[ -n "$my_ip" ] || die "Could not read the public address of this machine.
The script stops rather than opening port 22 to everybody."
say "Admitting $my_ip only"

tags="ResourceType=%s,Tags=[{Key=Name,Value=$run_id},{Key=Project,Value=$TAG_PROJECT},{Key=Purpose,Value=$TAG_PURPOSE}]"

vpc_id="$(aws ec2 describe-vpcs --region "$REGION" \
    --filters Name=isDefault,Values=true --query 'Vpcs[0].VpcId' --output text)"

group_id="$(aws ec2 create-security-group --region "$REGION" \
    --group-name "$run_id" --vpc-id "$vpc_id" \
    --description "Temporary group for a Cachette learner training run" \
    --tag-specifications "$(printf "$tags" security-group)" \
    --query 'GroupId' --output text)"
aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$group_id" --protocol tcp --port 22 --cidr "${my_ip}/32" >/dev/null
say "Security group $group_id"

key_name="$run_id"
aws ec2 create-key-pair --region "$REGION" --key-name "$key_name" \
    --tag-specifications "$(printf "$tags" key-pair)" \
    --query 'KeyMaterial' --output text > "$key_file"
chmod 600 "$key_file"
say "Key pair $key_name"

# The instance shuts itself down after the cap, and a shutdown from inside
# terminates it. A run that loses the machine that started it still ends.
cat > "$out_dir/user-data.sh" <<USERDATA
#!/bin/bash
shutdown -h +$MAX_MINUTES
USERDATA

launch=(
    aws ec2 run-instances --region "$REGION"
    --image-id "$ami" --instance-type "$INSTANCE_TYPE"
    --key-name "$key_name" --security-group-ids "$group_id"
    --associate-public-ip-address
    --block-device-mappings "DeviceName=/dev/xvda,Ebs={VolumeSize=$ROOT_VOLUME_GB,VolumeType=gp3,DeleteOnTermination=true}"
    --instance-initiated-shutdown-behavior terminate
    --user-data "file://$out_dir/user-data.sh"
    --tag-specifications "$(printf "$tags" instance)" "$(printf "$tags" volume)"
    --query 'Instances[0].InstanceId' --output text
)
if [ "$market" = "spot" ]; then
    # A one-time request that terminates on interruption. It never restarts
    # itself, so an interrupted run cannot come back without a person, and a
    # forgotten request cannot launch a second machine.
    launch+=(--instance-market-options
        'MarketType=spot,SpotOptions={SpotInstanceType=one-time,InstanceInterruptionBehavior=terminate}')
    # The zone was chosen by price, so the subnet must be the one in it.
    subnet="$(aws ec2 describe-subnets --region "$REGION" \
        --filters "Name=vpc-id,Values=$vpc_id" \
        "Name=availability-zone,Values=$zone" \
        --query 'Subnets[0].SubnetId' --output text)"
    if [ -n "$subnet" ] && [ "$subnet" != "None" ]; then
        launch+=(--subnet-id "$subnet")
    else
        say "No default subnet in $zone. The account picks the zone instead"
        zone=""
    fi
fi

instance_id="$("${launch[@]}")"
say "Instance $instance_id"

aws ec2 wait instance-running --region "$REGION" --instance-ids "$instance_id"
host="$(aws ec2 describe-instances --region "$REGION" --instance-ids "$instance_id" \
    --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)"
say "Address $host"

# The state file is what `--attach` and `--stop` read. It is written as soon
# as the instance exists, so a person can always find and end the machine
# even when this process dies in the next second.
cat > "$out_dir/instance.env" <<STATE
RUN_ID=$run_id
INSTANCE_ID=$instance_id
GROUP_ID=$group_id
KEY_NAME=$key_name
HOST=$host
ZONE=$zone
PRICE=$price
REGION=$REGION
TOTAL_GENERATIONS=$total_generations
INSTANCE_TYPE=$INSTANCE_TYPE
STATE
RUN_ID="$run_id"; HOST="$host"; ZONE="$zone"; PRICE="$price"
TOTAL_GENERATIONS="$total_generations"
say "Wrote $out_dir/instance.env. Stop the run with: $0 --stop $out_dir"

connect

say "Waiting for the instance to accept a connection"
for _ in $(seq 1 60); do
    if ssh "${ssh_options[@]}" "$remote" true 2>/dev/null; then break; fi
    sleep 10
done
ssh "${ssh_options[@]}" "$remote" true

# --------------------------------------------------------------------- source

# The archive holds the tracked files, with the content the working tree has
# now. An edit that is not committed still reaches the instance. A file that
# git does not track does not, so a run trains the project and not whatever
# else the directory holds.
say "Copying the tracked files"
( cd "$root" \
    && git ls-files -c -z \
    | tar --null --files-from=- --create --gzip --file "$out_dir/tree.tgz" )
scp "${ssh_options[@]}" "$out_dir/tree.tgz" "$remote:tree.tgz" >/dev/null
rm -f "$out_dir/tree.tgz"

# **A cached wheel skips the compiler, and the compiler is most of the wait.**
# The instance installs the wheel when one arrives and builds one when none
# does. The follower fetches the wheel a build produces, so the next run with
# the same inputs pays nothing for it.
wheel_key="$(build_key)"
cached_wheel="$WHEEL_CACHE/$wheel_key"
if compgen -G "$cached_wheel/*.whl" >/dev/null; then
    say "Sending the cached wheel for build $wheel_key. The instance skips the compiler"
    ssh "${ssh_options[@]}" "$remote" "mkdir -p wheelhouse" >/dev/null
    scp "${ssh_options[@]}" "$cached_wheel"/*.whl "$remote:wheelhouse/" >/dev/null
else
    say "No cached wheel for build $wheel_key. The instance compiles once, and the run keeps the result"
fi

# **The baseline cache must travel, because the instance is new.** A fresh
# machine holds no measurement, so a run that sent nothing would play for the
# bar again whatever the last run paid for it.
if compgen -G "$BASELINE_CACHE/*.json" >/dev/null; then
    say "Sending the stored controller baselines. A run with the same inputs reads them"
    ssh "${ssh_options[@]}" "$remote" \
        "mkdir -p .cache/cachette-baselines" >/dev/null
    scp "${ssh_options[@]}" "$BASELINE_CACHE"/*.json \
        "$remote:.cache/cachette-baselines/" >/dev/null
else
    say "No stored controller baseline. This run measures it once and keeps the result"
fi

# --------------------------------------------------------------------- remote

cat > "$out_dir/remote.sh" <<'REMOTE'
set -euo pipefail
# The marker is what the follower on the other machine reads. A trap writes
# it, so a failure anywhere below reports itself and the follower never waits
# for ever.
mark() { printf '%s\n' "$1" > /tmp/marker; }
finish() {
    local status=$?
    if [ "$status" -eq 0 ]; then mark done; else mark "failed $status"; fi
}
trap finish EXIT
mark building

sudo dnf install -y -q gcc gcc-c++ tar gzip python3.12 python3.12-devel \
    >/dev/null
mkdir -p cachette
tar -xzf tree.tgz -C cachette
# **The compiler is only needed when no wheel arrived.** Installing the Rust
# toolchain and building the extension is most of the time between boot and
# the first episode, and the bytes it produces are a function of the sources
# alone. A run that received a wheel skips both.
if ls "$HOME"/wheelhouse/*.whl >/dev/null 2>&1; then
    printf 'a wheel arrived for this build. Skipping the compiler\n'
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain none >/dev/null
    . "$HOME/.cargo/env"
fi
curl -LsSf https://astral.sh/uv/install.sh | sh >/dev/null
. "$HOME/.local/bin/env" 2>/dev/null || export PATH="$HOME/.local/bin:$PATH"
cd cachette

# **The sync installs the dependencies and never the project itself.** The
# build backend of this package is the Rust builder, so a plain sync compiles
# the extension, and the release build after it then compiles the same sources
# a second time. The project arrives below as one wheel, built once or fetched
# from the cache.
uv sync --frozen --no-install-project 2>&1 | tail -5 \
    || uv sync --no-install-project 2>&1 | tail -5

# The extension is a compiled module, so the learner needs a build and not
# only an install. This is the step that fails first if the target platform
# cannot build it, and it fails before anything long has run.
if ! ls "$HOME"/wheelhouse/*.whl >/dev/null 2>&1; then
    # The toolchain manifest at the root pins the channel, so rustup installs
    # the version the project states and this script names none.
    rustup show active-toolchain
    # **Build a wheel rather than install in place.** The two produce the same
    # module, and only the wheel is a file the follower can fetch and keep for
    # the next run.
    uv run --no-project --with maturin maturin build --release \
        --out "$HOME/wheelhouse" 2>&1 | tail -5
fi
uv pip install --reinstall "$HOME"/wheelhouse/*.whl 2>&1 | tail -3

# **The module must import before anything long runs.** A wheel built for
# another platform or another Python installs without a word and fails at the
# first episode, which is an hour of billing after the mistake.
uv run --no-sync python -c 'import cachette._core; print("the engine module imports")'
# numpy on aarch64 is the one dependency this project does not control the
# build of. The line below states which wheel arrived, so a report can say
# whether it was a native wheel or a build from source.
uv run python -c "import numpy; print('numpy', numpy.__version__, numpy.show_config('dicts')['Machine Information']['host']['cpu'])" \
    2>/dev/null || uv run python -c "import numpy; print('numpy', numpy.__version__)"

cores="$(nproc)"
printf '# cores\t%s\n' "$cores"

# **One thread for each matrix library, because this run holds one process
# for each strategy.** numpy starts a pool of one thread for every core, and
# it does that in every process. Five processes on a machine of sixty four
# cores then hold three hundred and twenty threads, and the machine spends
# its time changing between them. The matrices here are small, so a pool
# wins nothing even in one process.
export OPENBLAS_NUM_THREADS=1
export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1
export NUMEXPR_NUM_THREADS=1

# **The instance holds no repository, so it cannot derive the engine key.**
# The launcher derived it from the tracked sources and passes it in. Without
# it the trainer stores nothing, because a stored baseline that cannot name
# its engine could answer for an engine that never played.
export CACHETTE_ENGINE_KEY="${CACHETTE_ENGINE_KEY:-}"

# **One trainer process for each strategy, all at once.** One process of
# every core does not use them: the batch crosses into the engine once for
# each tick and waits for the slowest world, and one interpreter picks the
# actions for every world between the decisions while the workers wait.
# Several processes keep one batch and one interpreter each, and they run
# together.
#
# The strategy names come from the trainer, so this script declares no list
# of its own. A second list here would go stale the first time a strategy is
# added, and nothing would fail.
# **A run that names its strategies trains those and no others.** The loop
# below gives each process its own `--only`, and the arguments of the run are
# appended after it, so a second `--only` there would win and every process
# would train the same strategy at a fraction of the cores. The names are
# taken here instead, and the argument is removed from what each process
# receives.
#
# The trainer answers the names, for the whole argument set of this run. It
# reads `--only`, and it reads the play styles that replace the table. **This
# script read the table at import instead, which is the table before the
# styles replace it.** A style run then launched one process for each built-in
# weighting, every one of them failed on the first name it looked up, and the
# instance was paid for and torn down without a generation.
names="$(uv run python -m cachette.learn --print-strategies $TRAIN_ARGS)"
TRAIN_ARGS="$(printf '%s' "$TRAIN_ARGS" | sed 's/--only [^ ]*//')"
count="$(printf '%s' "$names" | wc -w)"
each="$((cores / count))"
[ "$each" -ge 1 ] || each=1
printf '# strategies\t%s\n# workers each\t%s\n' "$count" "$each"

# --------------------------------------------------- the throughput figure
#
# **This is the first throughput measurement this project owns on the
# target.** Every other figure it holds comes from a development machine on
# x86-64. It runs before the training, because it takes about a minute and
# because a run that is interrupted later still brings this back.
mark measuring
mkdir -p runs/learn
# One row, in the shape this run trains in. It costs about half a minute and
# it says what the machine reached, which is what the costs register wants.
# A sweep of other shapes belongs in a probe-only run, not here, because
# every second it takes is a second the training does not get.
#
# **The figure must reach the log the follower reads.** It went to two files
# under `/tmp`, and the follower reads neither, so the one tick rate this
# project owns on the target never appeared on the dashboard.

# **The probe must measure the world the run trains in.** A tick of a 48 by
# 48 world is not a tick of a 256 by 256 world, so a probe that took its own
# default extent would report a figure that describes no training run. The
# trainer answers the world, for the whole argument set of this run, through
# one flag that runs the same setup path a run runs. This script therefore
# holds no extent, no faction count and no decision interval of its own.
#
# The launcher held two copies of the strategy list once, and a run failed
# after it had paid for the instance. A world copied here would be the same
# shape.[^3]
#
# References
#   [^3]: Findings register, FND-693. `docs/FINDINGS.md`
world="$(uv run python -m cachette.learn --print-world $TRAIN_ARGS)"
world_field() { printf '%s\n' "$world" | awk -F'\t' -v k="$1" '$1==k{print $2}'; }
probe_width="$(world_field width)"
probe_height="$(world_field height)"
probe_factions="$(world_field factions)"
probe_interval="$(world_field decision_interval)"
printf '# world\t%sx%s, %s factions, interval %s\n' \
    "$probe_width" "$probe_height" "$probe_factions" "$probe_interval"

uv run python scripts/train_throughput.py \
    --width "$probe_width" --height "$probe_height" \
    --factions "$probe_factions" \
    --decision-interval "$probe_interval" \
    --workers "$each" \
    --worlds "${PROBE_WORLDS:-144}" \
    --decisions 20 --price "${PRICE:-0}" --out /tmp/throughput.txt \
    2>&1 | tee -a /tmp/throughput-console.txt | tee -a runs/learn/train.log
tee -a runs/learn/train.log < /tmp/throughput.txt

if [ "${PROBE_ONLY:-0}" = "1" ]; then
    exit 0
fi

# ---------------------------------------------- the bar every strategy needs
#
# **One number sets the bar for the whole run, and every process needed it.**
# The trainer reports each policy against the built-in controller playing the
# learner's own seat, and it measures that by playing the controller over the
# whole held-out seed set. One process for each strategy measured it before
# training and again after, so a run of six strategies played the same worlds
# twelve times, each at a sixth of the cores.
#
# This pass measures it once, before any trainer starts, and it may hold every
# core. It writes each number into the cache under a key that holds the engine
# build, the world, the seeds and the objective, so each trainer reads it. Two
# strategies that hold the same objective share one number.
#
# **A failure here must not end the run.** The cache is an optimisation, and
# each trainer measures the number itself when the cache does not hold it.
# Ending the run over a missing optimisation would cost the whole run.
mark baseline
uv run python -u -m cachette.learn --baseline-only \
    --only "$(printf '%s' "$names" | tr ' ' ',')" \
    --out runs/learn/baseline --workers "$cores" $TRAIN_ARGS 2>&1 \
    | tee -a runs/learn/train.log || true

# ------------------------------------------------------------- the training
mark running

# Each process writes its own log, and appends to the one the follower reads.
# A line of the log is short and each process writes whole lines, so the
# combined file stays readable.
for name in $names; do
    (
        # **Every strategy records how it ended, whether it worked or not.**
        # Without this, a strategy that dies leaves no line, the others
        # finish, and the run reports done while a fifth of it is missing.
        # The status comes from the trainer and not from the tee after it.
        set +e
        # **A strategy that dies takes its share of the machine with it.** Two
        # runs have lost a strategy to a fault in native code, once with a
        # segmentation fault and once with a bus error, and each left a machine
        # of sixty four cores at half load for hours while it kept billing.
        # The trainer writes a resume point every generation, so a restart
        # costs one generation and never the run.
        attempt=1
        extra=""
        while :; do
            started="$(date +%s)"
            uv run python -u -m cachette.learn --only "$name" \
                --out "runs/learn/$name" --workers "$each" $TRAIN_ARGS $extra 2>&1 \
                | tee -a runs/learn/train.log >> "runs/learn/$name.log"
            code="${PIPESTATUS[0]}"
            ran=$(( $(date +%s) - started ))
            printf '%s attempt %s exited %s after %ss\n' \
                "$name" "$attempt" "$code" "$ran" >> runs/learn/status
            [ "$code" -eq 0 ] && break
            [ "$attempt" -ge 5 ] && break
            # **A strategy that dies at once dies for a reason a restart cannot
            # fix.** A bad argument or a missing module fails in seconds, and
            # retrying it only fills the log. A fault in a long run is the case
            # a restart is for, so only a run that lasted a while earns one.
            if [ "$ran" -lt 120 ]; then
                printf '%s failed in %ss, which is too fast to be a fault worth retrying\n' \
                    "$name" "$ran" >> runs/learn/status
                break
            fi
            attempt=$(( attempt + 1 ))
            # The resume point carries the centre, so the restart continues the
            # search instead of starting it again.
            extra="--resume"
            printf '  %s restarting from its resume point, attempt %s\n' \
                "$name" "$attempt" | tee -a runs/learn/train.log >> "runs/learn/$name.log"
            sleep 15
        done
    ) &
done
wait
printf '=== how each strategy ended ===\n'
cat runs/learn/status 2>/dev/null
# The run failed if any strategy ended on a failure. A marker that says done
# over a dead strategy is worse than no marker.
#
# **A strategy may hold several attempts, and only its last one decides.** An
# earlier attempt that died and was restarted is a fault the run recovered
# from, so it must not fail the run. The reduction below keeps the last line
# of each strategy and asks whether every one of those ended at zero.
awk '/attempt/ { last[$1] = $5 }
     END { for (n in last) if (last[n] != 0) bad = 1; exit bad }' \
    runs/learn/status
REMOTE

say "Building and measuring on the instance. It runs detached"
scp "${ssh_options[@]}" "$out_dir/remote.sh" "$remote:remote.sh" >/dev/null
ssh "${ssh_options[@]}" "$remote" \
    "TRAIN_ARGS='$train_args' PRICE='$price' \
     PROBE_WORLDS='$probe_worlds' \
     CACHETTE_ENGINE_KEY='$wheel_key' \
     PROBE_ONLY='${CACHETTE_TRAIN_PROBE_ONLY:-0}' \
     nohup setsid bash remote.sh > run.log 2>&1 < /dev/null & echo started"

say "Following the run. The dashboard prints every two minutes"
follow
collect

say "Done. Read $out_dir/throughput.txt for the ticks a second on the target"
