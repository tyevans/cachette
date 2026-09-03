# The single interface to every gate in this project.
# Run `just` with no arguments to list the targets.

set shell := ["bash", "-euo", "pipefail", "-c"]

# List the targets.
default:
    @just --list --unsorted

# Build the extension into the local environment.
setup:
    uv sync

# Format the Rust code and the Python code.
fmt:
    cargo fmt --all
    uv run ruff format python tests

# Check the formatting without changing a file.
fmt-check:
    cargo fmt --all -- --check
    uv run ruff format --check python tests

# Lint the Rust code, the Python code, and the types.
lint: lint-rust lint-python invariants

# Lint the Rust code.
lint-rust:
    cargo clippy --workspace --all-targets -- -D warnings

# Lint the Python code and check the types.
lint-python:
    uv run ruff check python tests
    uv run mypy

# Check the float ban of ADR-0002 D2 and the crate split of ADR-0041.
invariants:
    ./scripts/check-float-ban.sh
    ./scripts/check-crate-split.sh

# Run the fast tests on both sides.
test: test-rust census probe test-python smoke

# Run the Rust tests. They go through the public crate API.
test-rust:
    cargo test --workspace

# Prove that building a world visits no tile of the value field.
#
# The switch makes the tile value field count the tiles it generates. The
# test asserts that a build generates none, and that a copy of the whole
# column generates one for each tile, so the same test proves the counter
# counts. A counter wired to nothing would read zero for ever.
census:
    cargo test --package cachette-core --features census-generated-tiles --test build_visits_no_tile -- --test-threads=1

# Run the Python tests. They import the installed package.
test-python:
    uv sync
    uv run pytest

# The window draws cards, which hold what changes moment to moment. Hold tab
# to name the colours. Run `just inspect` for every number it does not show.
#
# Open the window and watch the world run. Needs a display.
watch:
    cargo run --release --package cachette-view

# This is where the panel went. It holds every section, at a height that never
# cuts, and a person reads it without opening a window.
#
# Write every number the window does not show, as an image. Needs no display.
inspect out="target/panel.ppm":
    @case "{{out}}" in *.ppm) ;; *) echo "the output path must end in .ppm, and '{{out}}' does not"; exit 1;; esac
    cargo run --release --package cachette-view --example panel_shot -- {{out}}

# The seed and the extent choose the world. The soldier count may be zero,
# which shows the ground with no disc over it.
#
# Write the map as the window draws it, as an image. Needs no display.
map seed="0" extent="128" out="target/world.ppm" soldiers="600":
    @case "{{out}}" in *.ppm) ;; *) echo "the output path must end in .ppm, and '{{out}}' does not"; exit 1;; esac
    cargo run --release --package cachette-view --example picture -- {{seed}} {{extent}} {{out}} {{soldiers}}

# Exercise the installed package the way continuous integration does.
smoke:
    uv sync
    uv run python scripts/smoke.py

# Run the two determinism tests of ADR-0001 D4 on their own.
determinism:
    cargo test --package cachette-core --test thread_equivalence
    cargo test --package cachette-core --test golden_state_hash

# Prove that the determinism tests and the key-field tests can fail.
#
# The perturbed build reverses every slot reduction, drops the row from the
# terrain lattice key, from the resource address key and from the founding
# candidate key, and removes the sort from both admission and the gather
# resolve. The reversal reaches the join of the consumption draw as well, and
# it reaches the scan of the death plane, which then ends the marked units in
# the order the output slots joined rather than in ascending slot order. It
# also scans the choice options from the top of the set, so a tie goes to the
# highest option index rather than the lowest, and it scans the six directions
# of the exit field the same way, so a tie between two equal neighbouring cells
# goes to the highest direction index.
# Each test binary below must then fail, and the probe binary, which asserts
# that every perturbation is visible, must pass. Both determinism tests of
# ADR-0001 D4 are in the list, because ADR-0001 D5 asks both to be able to
# fail.
probe:
    ! cargo test --package cachette-core --features probe-nondeterminism --test thread_equivalence
    ! cargo test --package cachette-core --features probe-nondeterminism --test golden_state_hash
    ! cargo test --package cachette-core --features probe-nondeterminism --test slot_reduction
    ! cargo test --package cachette-core --features probe-nondeterminism --test terrain
    ! cargo test --package cachette-core --features probe-nondeterminism --test resource
    ! cargo test --package cachette-core --features probe-nondeterminism --test founding
    ! cargo test --package cachette-core --features probe-nondeterminism --test consumption
    ! cargo test --package cachette-core --features probe-nondeterminism --test choice
    ! cargo test --package cachette-core --features probe-nondeterminism --test exit_field
    ! cargo test --package cachette-core --features probe-nondeterminism --test starvation
    ! cargo test --package cachette-core --features probe-nondeterminism --test influence
    cargo test --package cachette-core --features probe-nondeterminism --test determinism_probe

# Record the golden state hash files. Read the difference before you commit.
golden:
    CACHETTE_UPDATE_GOLDEN=1 cargo test --package cachette-core --test golden_state_hash

# Run the slower gates: release tests, licence audit, and the target check.
test-slow:
    cargo test --workspace --release
    cargo deny check
    # The target check is the same one `target-check` runs, and it excludes the
    # viewer for the same reason: the viewer links a window library that needs a
    # cross-compiler, and ADR-0008 names the engine as what ships. Checking the
    # whole workspace here asked for a toolchain the project says it does not
    # need, so this recipe could not go green on a developer machine.
    just target-check

# Check that the code compiles for the primary target of ADR-0008.
target-check:
    # The viewer is excluded. It opens a window, so it links a C library that
    # needs a cross-compiler, and a window on a headless server means nothing.
    # ADR-0008 names the primary target for the engine, which is what ships.
    cargo check --package cachette-core --package cachette-py --target aarch64-unknown-linux-gnu

# Measure the cost of a frame. A benchmark does not gate a merge.
#
# The sweep runs the step against the tile count and against the unit count,
# and it reaches the target scale of the project on the `full` profile. The
# `quick` profile checks the apparatus in about a minute and measures nothing
# that the project may cite.
#
# A figure taken here is a figure about this machine, and the target register
# takes no such figure. Run `just graviton-bench` for a figure the project may
# record.
bench profile="quick":
    cargo bench --bench target_cost -- {{profile}}

# Measure the cost of a frame on the target platform, on a Graviton machine.
#
# The script launches an instance, builds the benchmark on it, runs the sweep,
# brings the rows back, and destroys everything it made. It needs the AWS
# command line tool, authenticated. It costs a few cents on the default
# instance.
#
# Every axis is a parameter. Set CACHETTE_BENCH_INSTANCE for the machine,
# and CACHETTE_BENCH_EXTENTS, CACHETTE_BENCH_THREADS and CACHETTE_BENCH_UNITS
# for the sweep. The script header lists them all.
graviton-bench profile="full":
    ./scripts/graviton-benchmark.sh {{profile}}

# List anything a Graviton benchmark run left behind. It should list nothing.
graviton-orphans:
    ./scripts/graviton-benchmark.sh --orphans

# Run mutation testing over the Rust core. Slow. Not a commit gate.
mutants:
    cargo mutants --no-shuffle

# Check the decision records and the product records.
records:
    ./scripts/check-adrs.sh
    ./scripts/check-prds.sh
    ./scripts/check-backlog.sh
    ./scripts/check-registers.sh
    ./scripts/check-priority.sh
    ./scripts/check-citations.sh
    ./scripts/check-conflict-markers.sh
    ./scripts/check-footnotes.sh

# Prove that the record checks can fail. Each must reject its broken fixture.
# Install the pre-commit hook, once per clone. The hooks are versioned.
install-hooks:
    git config core.hooksPath .githooks
    @echo "the pre-commit hook is installed for this clone"

# A merge conflict in a register is resolved by choosing between two sides.
# Each side is a correct file and the merged result is not. This asks what the
# merged file says. The script holds the reasoning and the rules.
#
# The same check runs as a pre-commit hook over the staged change, which is
# where these defects are born. Install it with `just install-hooks`. The hook
# is bypassable and git does not run it for a clean automatic merge, so this
# recipe is the enforcement and the hook is the early warning.
#
# Check the branch for the defects a hand-resolved merge produces.
merge-defects:
    ./scripts/check-merge-defects.sh --branch

records-probe:
    ! ./scripts/check-adrs.sh tests/fixtures/records-broken
    ! ./scripts/check-prds.sh tests/fixtures/prd-broken
    ! ./scripts/check-citations.sh tests/fixtures/citations-broken
    ! ./scripts/check-conflict-markers.sh tests/fixtures/conflict-broken
    ! ./scripts/check-footnotes.sh tests/fixtures/footnotes-broken
    ! CACHETTE_FOOTNOTE_BASELINE=tests/fixtures/footnotes-stale/baseline.txt ./scripts/check-footnotes.sh tests/fixtures/footnotes-stale

# Everything a commit must pass. The wrapper times the run and reports the
# cost against the local budget for this architecture. It reports; it does
# not fail on a figure, because wall clock on a loaded machine is not a gate.
check:
    ./scripts/gate-budget.sh just gates

# The gates themselves. Run `just check` instead, to get the cost report.
gates: fmt-check lint test records records-probe merge-defects

# What continuous integration runs.
ci: check test-slow
