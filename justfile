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
test: test-rust probe test-python smoke

# Run the Rust tests. They go through the public crate API.
test-rust:
    cargo test --workspace

# Run the Python tests. They import the installed package.
test-python:
    uv sync
    uv run pytest

# Open the window and watch the world run. Needs a display.
watch:
    cargo run --release --package cachette-view

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
# The perturbed build reverses every slot reduction and drops the row from the
# terrain lattice key. Three test binaries must then fail, and the probe
# binary, which asserts that both perturbations are visible, must pass.
probe:
    ! cargo test --package cachette-core --features probe-nondeterminism --test thread_equivalence
    ! cargo test --package cachette-core --features probe-nondeterminism --test slot_reduction
    ! cargo test --package cachette-core --features probe-nondeterminism --test terrain
    cargo test --package cachette-core --features probe-nondeterminism --test determinism_probe

# Record the golden state hash files. Read the difference before you commit.
golden:
    CACHETTE_UPDATE_GOLDEN=1 cargo test --package cachette-core --test golden_state_hash

# Run the slower gates: release tests, licence audit, and the target check.
test-slow:
    cargo test --workspace --release
    cargo deny check
    cargo check --workspace --target aarch64-unknown-linux-gnu

# Check that the code compiles for the primary target of ADR-0008.
target-check:
    # The viewer is excluded. It opens a window, so it links a C library that
    # needs a cross-compiler, and a window on a headless server means nothing.
    # ADR-0008 names the primary target for the engine, which is what ships.
    cargo check --package cachette-core --package cachette-py --target aarch64-unknown-linux-gnu

# Run mutation testing over the Rust core. Slow. Not a commit gate.
mutants:
    cargo mutants --no-shuffle

# Check the decision records and the product records.
records:
    ./scripts/check-adrs.sh
    ./scripts/check-prds.sh
    ./scripts/check-citations.sh

# Prove that the record checks can fail. Each must reject its broken fixture.
records-probe:
    ! ./scripts/check-prds.sh tests/fixtures/prd-broken
    ! ./scripts/check-citations.sh tests/fixtures/citations-broken

# Everything a commit must pass.
check: fmt-check lint test records records-probe

# What continuous integration runs.
ci: check test-slow
