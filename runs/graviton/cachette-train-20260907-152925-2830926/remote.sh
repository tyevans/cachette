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

# --------------------------------------------------- the throughput figure
#
# **This is the first throughput measurement this project owns on the
# target.** Every other figure it holds comes from a development machine on
# x86-64. It runs before the training, because it takes about a minute and
# because a run that is interrupted later still brings this back.
mark measuring
uv run python scripts/train_throughput.py \
    --workers "1,$((cores / 4)),$((cores / 2)),$cores" \
    --decisions 20 --price "${PRICE:-0}" --out /tmp/throughput.txt \
    2>&1 | tee -a /tmp/throughput-console.txt
cat /tmp/throughput.txt

if [ "${PROBE_ONLY:-0}" = "1" ]; then
    exit 0
fi

# ------------------------------------------------------------- the training
mark running
mkdir -p runs/learn
# The worker count is the core count. The batch step takes it, and the
# throughput rows above say what each worker contributed.
uv run python -m cachette.learn --out runs/learn --workers "$cores" \
    $TRAIN_ARGS 2>&1 | tee runs/learn/train.log
