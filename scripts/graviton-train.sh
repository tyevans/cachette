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
default_args="--generations 20 --population 24 --seeds 6 --holdout 24 \
--hidden 24 --sigma 1.5 --learning-rate 0.3 --validation 6 --validate-every 2"
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
if printf '%s' "$train_args" | grep -q -- '--only'; then
    strategies="$(printf '%s' "$train_args" \
        | sed -n 's/.*--only \([^ ]*\).*/\1/p' | tr ',' '\n' | grep -c .)"
else
    strategies="$(grep -c '^    "[a-z-]*": ($' "$root/python/cachette/learn/__main__.py" \
        2>/dev/null || echo 5)"
fi
total_generations=$((generations * strategies))

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
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain none >/dev/null
. "$HOME/.cargo/env"
curl -LsSf https://astral.sh/uv/install.sh | sh >/dev/null
. "$HOME/.local/bin/env" 2>/dev/null || export PATH="$HOME/.local/bin:$PATH"
cd cachette
# The toolchain manifest at the root pins the channel, so rustup installs the
# version the project states and this script names none.
rustup show active-toolchain

# The extension is a compiled module, so the learner needs a build and not
# only an install. This is the step that fails first if the target platform
# cannot build it, and it fails before anything long has run.
uv sync --frozen 2>&1 | tail -5 || uv sync 2>&1 | tail -5
uv run maturin develop --release 2>&1 | tail -5
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
names="$(uv run python -c \
    'from cachette.learn.__main__ import STRATEGIES; print(" ".join(STRATEGIES))')"
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
# One row, in the shape this run trains in. It costs about half a minute and
# it says what the machine reached, which is what the costs register wants.
# A sweep of other shapes belongs in a probe-only run, not here, because
# every second it takes is a second the training does not get.
uv run python scripts/train_throughput.py \
    --workers "$each" \
    --worlds "${PROBE_WORLDS:-144}" \
    --decisions 20 --price "${PRICE:-0}" --out /tmp/throughput.txt \
    2>&1 | tee -a /tmp/throughput-console.txt
cat /tmp/throughput.txt

if [ "${PROBE_ONLY:-0}" = "1" ]; then
    exit 0
fi

# ------------------------------------------------------------- the training
mark running
mkdir -p runs/learn

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
        uv run python -u -m cachette.learn --only "$name" \
            --out "runs/learn/$name" --workers "$each" $TRAIN_ARGS 2>&1 \
            | tee -a runs/learn/train.log > "runs/learn/$name.log"
        printf '%s exited %s\n' "$name" "${PIPESTATUS[0]}" >> runs/learn/status
    ) &
done
wait
printf '=== how each strategy ended ===\n'
cat runs/learn/status 2>/dev/null
# The run failed if any strategy failed. A marker that says done over a
# dead strategy is worse than no marker.
! grep -qv 'exited 0$' runs/learn/status
REMOTE

say "Building and measuring on the instance. It runs detached"
scp "${ssh_options[@]}" "$out_dir/remote.sh" "$remote:remote.sh" >/dev/null
ssh "${ssh_options[@]}" "$remote" \
    "TRAIN_ARGS='$train_args' PRICE='$price' \
     PROBE_WORLDS='$probe_worlds' \
     PROBE_ONLY='${CACHETTE_TRAIN_PROBE_ONLY:-0}' \
     nohup setsid bash remote.sh > run.log 2>&1 < /dev/null & echo started"

say "Following the run. The dashboard prints every two minutes"
follow
collect

say "Done. Read $out_dir/throughput.txt for the ticks a second on the target"
