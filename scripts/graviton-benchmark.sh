#!/usr/bin/env bash
# Measures the cost of a frame on the target platform.
#
# The project targets AWS Graviton, and the primary target triple is
# aarch64-unknown-linux-gnu. Every cost figure in this project is derived
# rather than measured, because no measurement exists on the target. This
# script takes one. It launches a Graviton instance, copies the working tree
# to it, builds the benchmark, runs it, brings the output back, and destroys
# everything it made.
#
# Read `docs/reference/graviton-costs.md` for the figures a run produced, and
# `crates/cachette-core/benches/target_cost.rs` for what each row measures.
#
# Usage:
#   scripts/graviton-benchmark.sh              run the full sweep
#   scripts/graviton-benchmark.sh quick        run the small sweep
#   scripts/graviton-benchmark.sh --orphans    list what a run left behind
#
# Settings, as environment variables:
#   CACHETTE_BENCH_INSTANCE   the instance type. Default t4g.medium
#   CACHETTE_BENCH_REGION     the region. Default us-west-2
#   CACHETTE_BENCH_OUT        the output file. Default a file under /tmp
#   CACHETTE_BENCH_KEEP       set to 1 to keep the instance. It then bills
#                             until you terminate it yourself
#   CACHETTE_BENCH_EXTENTS    the world sizes to sweep, as `WIDTHxHEIGHT`
#                             words. Default: the profile
#   CACHETTE_BENCH_THREADS    the thread counts to sweep. Default: the profile
#   CACHETTE_BENCH_UNITS      the unit counts to sweep. Default: the profile
#   CACHETTE_BENCH_MAX_MINUTES  the deadline after which the instance destroys
#                             itself. Default 360
#
# Every axis is a parameter, so a run at another size, another thread count or
# another instance is a setting and not a change to a file. A run at the full
# target scale is the same command with a larger instance and a longer list.
#
# What it needs: the AWS command line tool, authenticated, with permission to
# run an instance, and `ssh`, `scp`, `tar` and `git` on this machine.
#
# What it costs: one instance for the length of one run, plus a root volume
# that lives and dies with it. The default instance costs a few cents for each
# hour on demand. Read the price for the region before a long run, because a
# price is a figure and this script is not a register. Nothing else here bills:
# it creates no gateway, no load balancer, and no volume beyond that root.
#
# The teardown runs from a trap, so it also runs when the build fails, when
# the connection drops, and when a person interrupts the script. The instance
# also shuts itself down after a deadline, and a shutdown from inside
# terminates it, so a run that loses the machine that started it still ends.
set -euo pipefail

readonly REGION="${CACHETTE_BENCH_REGION:-us-west-2}"
# The default is a non-burstable Graviton instance, and the reason is the
# measurement rather than the price. A `t4g` instance earns CPU credits and
# falls back to a fraction of one core when they run out. This benchmark
# saturates the processor for hours, so it would exhaust the credits and every
# row after that point would measure the throttle. In the mode that `t4g`
# takes by default the credits are not exhausted, they are billed, and the
# surplus charge for a run of this length is larger than the whole cost of the
# instance below.
readonly INSTANCE_TYPE="${CACHETTE_BENCH_INSTANCE:-c7g.large}"
readonly ROOT_VOLUME_GB=20
readonly TAG_PROJECT="cachette"
readonly TAG_PURPOSE="blk-007-target-benchmark"
# The instance destroys itself after this many minutes, whatever else happens.
# The teardown below is the ordinary path. This is the net under it: it holds
# when the machine that started the run loses power, and it is the reason an
# abandoned run cannot bill for ever.
readonly SELF_DESTRUCT_MINUTES="${CACHETTE_BENCH_MAX_MINUTES:-360}"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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

if [ "${1:-}" = "--orphans" ]; then
    list_orphans
    exit 0
fi

readonly PROFILE="${1:-full}"
case "$PROFILE" in
    full | quick) ;;
    one | stages | placement | memory-placement | collapse)
        # These take one configuration rather than a sweep, and the caller
        # names it. `CACHETTE_BENCH_POINT` holds the whole argument list that
        # the benchmark receives, for example `stages 4096x4096 1000000 12`.
        if [ -z "${CACHETTE_BENCH_POINT:-}" ]; then
            printf 'The `%s` profile needs CACHETTE_BENCH_POINT.\n' "$PROFILE" >&2
            printf 'For example: CACHETTE_BENCH_POINT="stages 4096x4096 1000000 12"\n' >&2
            exit 2
        fi
        ;;
    *)
        printf 'The profile must be `full`, `quick`, `one`, `stages` or `placement`. Got: %s\n' "$PROFILE" >&2
        exit 2
        ;;
esac

# ------------------------------------------------------------------- teardown

run_id="cachette-bench-$(date -u +%Y%m%d-%H%M%S)-$$"
workdir="$(mktemp -d)"
instance_id=""
group_id=""
key_name=""

# Destroys everything this run created. It runs on every exit path, and each
# step reports rather than failing, so one failure cannot strand the rest.
teardown() {
    local status=$?
    set +e
    printf '\n--- teardown ---\n' >&2

    if [ -n "$instance_id" ]; then
        if [ "${CACHETTE_BENCH_KEEP:-0}" = "1" ]; then
            printf 'KEEP is set. Instance %s is still running and still billing.\n' \
                "$instance_id" >&2
            printf 'Terminate it with: aws ec2 terminate-instances --region %s --instance-ids %s\n' \
                "$REGION" "$instance_id" >&2
        else
            printf 'Terminating %s\n' "$instance_id" >&2
            aws ec2 terminate-instances --region "$REGION" \
                --instance-ids "$instance_id" --output text >/dev/null
            # The security group cannot be deleted until the network interface
            # of the instance is gone, so this wait is not optional.
            aws ec2 wait instance-terminated --region "$REGION" \
                --instance-ids "$instance_id"
            printf 'Terminated %s\n' "$instance_id" >&2
        fi
    fi

    if [ -n "$group_id" ] && [ "${CACHETTE_BENCH_KEEP:-0}" != "1" ]; then
        aws ec2 delete-security-group --region "$REGION" --group-id "$group_id" \
            && printf 'Deleted security group %s\n' "$group_id" >&2
    fi

    if [ -n "$key_name" ] && [ "${CACHETTE_BENCH_KEEP:-0}" != "1" ]; then
        aws ec2 delete-key-pair --region "$REGION" --key-name "$key_name" \
            && printf 'Deleted key pair %s\n' "$key_name" >&2
    fi

    rm -rf "$workdir"

    if [ "${CACHETTE_BENCH_KEEP:-0}" != "1" ]; then
        printf 'Checking that nothing this script creates is still running.\n' >&2
        list_orphans >&2
    fi
    exit "$status"
}
trap teardown EXIT INT TERM

say() { printf '=== %s\n' "$1" >&2; }

# --------------------------------------------------------------------- launch

say "Run $run_id, profile $PROFILE, $INSTANCE_TYPE in $REGION"
say "Extents: ${CACHETTE_BENCH_EXTENTS:-the profile default}"
say "Threads: ${CACHETTE_BENCH_THREADS:-the profile default}"
say "Units:   ${CACHETTE_BENCH_UNITS:-the profile default}"
say "The instance destroys itself after $SELF_DESTRUCT_MINUTES minutes"

commit="$(git -C "$root" rev-parse HEAD)"
dirty="clean"
if [ -n "$(git -C "$root" status --porcelain)" ]; then
    dirty="modified"
fi
say "Commit $commit, working tree $dirty"

# Amazon Linux 2023 for arm64. The public parameter always names the current
# image, so this script pins no image identifier of its own.
ami="$(aws ssm get-parameters --region "$REGION" \
    --names /aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64 \
    --query 'Parameters[0].Value' --output text)"
say "Image $ami"

# The security group admits this machine and nothing else. A run that cannot
# learn its own address stops, because the alternative is an open port.
my_ip="$(curl --silent --show-error --fail --max-time 10 https://checkip.amazonaws.com || true)"
if [ -z "$my_ip" ]; then
    printf 'Could not read the public address of this machine.\n' >&2
    printf 'The script stops rather than opening port 22 to everybody.\n' >&2
    exit 1
fi
say "Admitting $my_ip only"

tags="ResourceType=%s,Tags=[{Key=Name,Value=$run_id},{Key=Project,Value=$TAG_PROJECT},{Key=Purpose,Value=$TAG_PURPOSE}]"

vpc_id="$(aws ec2 describe-vpcs --region "$REGION" \
    --filters Name=isDefault,Values=true --query 'Vpcs[0].VpcId' --output text)"

group_id="$(aws ec2 create-security-group --region "$REGION" \
    --group-name "$run_id" --vpc-id "$vpc_id" \
    --description "Temporary group for a Cachette target benchmark run" \
    --tag-specifications "$(printf "$tags" security-group)" \
    --query 'GroupId' --output text)"
aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$group_id" --protocol tcp --port 22 --cidr "${my_ip}/32" >/dev/null
say "Security group $group_id"

key_name="$run_id"
key_file="$workdir/key.pem"
aws ec2 create-key-pair --region "$REGION" --key-name "$key_name" \
    --tag-specifications "$(printf "$tags" key-pair)" \
    --query 'KeyMaterial' --output text > "$key_file"
chmod 600 "$key_file"
say "Key pair $key_name"

# The instance shuts itself down after the deadline, and a shutdown from
# inside terminates it, so a run that nobody watches still ends.
cat > "$workdir/user-data.sh" <<USERDATA
#!/bin/bash
shutdown -h +$SELF_DESTRUCT_MINUTES
USERDATA

instance_id="$(aws ec2 run-instances --region "$REGION" \
    --image-id "$ami" --instance-type "$INSTANCE_TYPE" \
    --key-name "$key_name" --security-group-ids "$group_id" \
    --associate-public-ip-address \
    --block-device-mappings "DeviceName=/dev/xvda,Ebs={VolumeSize=$ROOT_VOLUME_GB,VolumeType=gp3,DeleteOnTermination=true}" \
    --instance-initiated-shutdown-behavior terminate \
    --user-data "file://$workdir/user-data.sh" \
    --tag-specifications "$(printf "$tags" instance)" "$(printf "$tags" volume)" \
    --query 'Instances[0].InstanceId' --output text)"
say "Instance $instance_id"

aws ec2 wait instance-running --region "$REGION" --instance-ids "$instance_id"
host="$(aws ec2 describe-instances --region "$REGION" --instance-ids "$instance_id" \
    --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)"
say "Address $host"

# The instance was created seconds ago in this account, and its key pair was
# created by this script, so the host key is accepted on first sight. The file
# belongs to this run, so nothing is written to the key store of the user.
ssh_options=(
    -i "$key_file"
    -o StrictHostKeyChecking=accept-new
    -o UserKnownHostsFile="$workdir/known_hosts"
    -o ConnectTimeout=10
    -o ServerAliveInterval=30
    -o ServerAliveCountMax=20
)
remote="ec2-user@$host"

say "Waiting for the instance to accept a connection"
for _ in $(seq 1 60); do
    if ssh "${ssh_options[@]}" "$remote" true 2>/dev/null; then
        break
    fi
    sleep 10
done
ssh "${ssh_options[@]}" "$remote" true

# --------------------------------------------------------------------- source

# The archive holds the tracked files, with the content the working tree has
# now. An edit that is not committed still reaches the instance. A file that
# git does not track does not, so a run measures the project and not whatever
# else the directory holds. Add a file to git before you measure it.
say "Copying the tracked files"
( cd "$root" \
    && git ls-files -c -z \
    | tar --null --files-from=- --create --gzip --file "$workdir/tree.tgz" )
scp "${ssh_options[@]}" "$workdir/tree.tgz" "$remote:tree.tgz" >/dev/null

# --------------------------------------------------------------------- remote

cat > "$workdir/remote.sh" <<'REMOTE'
set -euo pipefail
# The marker is what the poll on the other machine reads. A trap writes it, so
# a failure anywhere below reports itself and the poll never waits for ever.
mark() { printf '%s\n' "$1" > /tmp/marker; }
finish() {
    local status=$?
    if [ "$status" -eq 0 ]; then mark done; else mark "failed $status"; fi
}
trap finish EXIT
mark running
sudo dnf install -y -q gcc tar gzip >/dev/null
mkdir -p cachette
tar -xzf tree.tgz -C cachette
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain none >/dev/null
. "$HOME/.cargo/env"
cd cachette
# The toolchain manifest at the root pins the channel, so rustup installs the
# version the project states and this script names none.
rustup show active-toolchain

# Every axis of the sweep is a parameter. The caller sets these on the machine
# that starts the run, and the launcher forwards them here, so a run at
# another size or another thread count changes no file.
export CACHETTE_BENCH_EXTENTS="${CACHETTE_BENCH_EXTENTS:-}"
export CACHETTE_BENCH_THREADS="${CACHETTE_BENCH_THREADS:-}"
export CACHETTE_BENCH_UNITS="${CACHETTE_BENCH_UNITS:-}"
export CACHETTE_BENCH_POINT="${CACHETTE_BENCH_POINT:-}"
printf '# machine facts\n' > /tmp/facts.txt
{
    printf '# uname\t%s\n' "$(uname -srm)"
    printf '# instance_type\t%s\n' "$(curl -s -H "X-aws-ec2-metadata-token: $(curl -sX PUT http://169.254.169.254/latest/api/token -H 'X-aws-ec2-metadata-token-ttl-seconds: 60')" http://169.254.169.254/latest/meta-data/instance-type)"
    printf '# cpu_implementer\t%s\n' "$(sed -n 's/^CPU implementer[^:]*: *//p' /proc/cpuinfo | head -1)"
    printf '# cpu_part\t%s\n' "$(sed -n 's/^CPU part[^:]*: *//p' /proc/cpuinfo | head -1)"
    printf '# cpu_count\t%s\n' "$(nproc)"
    printf '# cache_line_bytes\t%s\n' "$(cat /sys/devices/system/cpu/cpu0/cache/index0/coherency_line_size)"
    printf '# memory_kb\t%s\n' "$(sed -n 's/^MemTotal: *\([0-9]*\).*/\1/p' /proc/meminfo)"
    printf '# rustc\t%s\n' "$(rustc --version)"
} >> /tmp/facts.txt
cat /tmp/facts.txt
cargo bench --bench target_cost --no-run 2>&1 | tail -3

# A caller may ask for one named point or for the stage split instead of a
# sweep. The extra words are passed straight through to the benchmark.
if [ "$1" = "one" ] || [ "$1" = "stages" ] || [ "$1" = "placement" ] || [ "$1" = "memory-placement" ] || [ "$1" = "collapse" ]; then
    cargo bench --bench target_cost -- $CACHETTE_BENCH_POINT > /tmp/rows.txt
    cat /tmp/facts.txt /tmp/rows.txt > /tmp/result.txt
    exit 0
fi

cargo bench --bench target_cost -- "$1" > /tmp/rows.txt

# The memory sweep starts one process for each point, because a process that
# has already built a large world does not give the allocator back and would
# report the high mark of the run rather than the cost of the world it holds.
cargo bench --bench target_cost -- memory > /tmp/memory.txt

# The timing rows above come from the bench profile, which carries no overflow
# check. Hard invariant 9 says a `u8` tile field summed over the target tile
# count sits inside a `u32` by less than one percent, and that an accumulator
# must not depend on that margin. Nothing above could see a wrap, because a
# release build wraps in silence. This point is built with the check on, so a
# wrap panics. It is a separate build on purpose: the check costs time, and a
# timing row taken under it would measure the check.
printf '# invariant 9, one frame at the target scale, overflow checks on\n' \
    > /tmp/overflow.txt
if RUSTFLAGS="-C overflow-checks=on" CACHETTE_BENCH_EXTENTS="4096x4096" \
    CACHETTE_BENCH_THREADS="2" CACHETTE_BENCH_UNITS="1000000" \
    cargo bench --bench target_cost -- memory >> /tmp/overflow.txt 2>&1; then
    printf '# overflow_checked_target_scale\tpassed\n' >> /tmp/overflow.txt
else
    printf '# overflow_checked_target_scale\tFAILED\n' >> /tmp/overflow.txt
fi

cat /tmp/facts.txt /tmp/rows.txt /tmp/memory.txt /tmp/overflow.txt > /tmp/result.txt
REMOTE

# The run takes hours on a small machine, and a connection held open for
# hours is a connection that drops. The remote work therefore starts detached
# and writes a marker when it ends, and this script polls for the marker. A
# drop then costs one reconnection and no work.
say "Starting the benchmark on the instance. It runs detached"
scp "${ssh_options[@]}" "$workdir/remote.sh" "$remote:remote.sh" >/dev/null
ssh "${ssh_options[@]}" "$remote" \
    "CACHETTE_BENCH_EXTENTS='${CACHETTE_BENCH_EXTENTS:-}' \
     CACHETTE_BENCH_THREADS='${CACHETTE_BENCH_THREADS:-}' \
     CACHETTE_BENCH_UNITS='${CACHETTE_BENCH_UNITS:-}' \
     CACHETTE_BENCH_POINT='${CACHETTE_BENCH_POINT:-}' \
     nohup setsid bash remote.sh $PROFILE > run.log 2>&1 < /dev/null & echo started"

say "Waiting for the run to finish. Progress follows"
finished=""
while [ -z "$finished" ]; do
    sleep 60
    state="$(ssh "${ssh_options[@]}" "$remote" \
        'cat /tmp/marker 2>/dev/null; echo; tail -1 run.log 2>/dev/null' 2>/dev/null || true)"
    if [ -z "$state" ]; then
        printf '.  no answer from the instance. Retrying\n' >&2
        continue
    fi
    printf '   %s\n' "$(printf '%s' "$state" | tr '\n' ' ')" >&2
    case "$state" in
        done*) finished="done" ;;
        failed*)
            printf 'The run failed on the instance. The log follows.\n' >&2
            ssh "${ssh_options[@]}" "$remote" 'tail -50 run.log' >&2 || true
            exit 1
            ;;
    esac
done

out="${CACHETTE_BENCH_OUT:-/tmp/$run_id.txt}"
scp "${ssh_options[@]}" "$remote:/tmp/result.txt" "$out" >/dev/null

{
    printf '# run_id\t%s\n' "$run_id"
    printf '# region\t%s\n' "$REGION"
    printf '# instance_type\t%s\n' "$INSTANCE_TYPE"
    printf '# commit\t%s\n' "$commit"
    printf '# working_tree\t%s\n' "$dirty"
    printf '# taken_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} | cat - "$out" > "$out.tmp" && mv "$out.tmp" "$out"

# The log goes beside the result, under the same name, so the shipper finds
# it without being told and the observability stack holds both.
cp "${CACHETTE_BENCH_LOG:-/dev/null}" "${out%.txt}.log" 2>/dev/null || true

say "Wrote $out"
say "Load it into the local stack with: just obs-load $out"
cat "$out"
