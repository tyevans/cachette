#!/usr/bin/env bash
# Runs the reviewer's mutation table over the terrain generator.
#
# Each row changes one constant, runs the whole core test suite, and counts
# the test binaries that fail. A constant that nothing pins gives a count of
# zero, which is the defect this table exists to find.
#
# The last row perturbs the generator mixer instead of the terrain. It is the
# control: it must fail, or the harness itself is not detecting change.
#
# The script restores every file it touched, whatever the outcome.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
terrain="$root/crates/cachette-core/src/terrain.rs"
rng="$root/crates/cachette-core/src/rng.rs"

cp "$terrain" /tmp/terrain.orig
cp "$rng" /tmp/rng.orig
restore() {
    cp /tmp/terrain.orig "$terrain"
    cp /tmp/rng.orig "$rng"
}
trap restore EXIT

run_case() {
    local label="$1"
    local failed
    failed="$(cargo test --package cachette-core --no-fail-fast 2>&1 |
        grep -c '^error: test failed')"
    printf '%-34s %s failing test binaries\n' "$label" "$failed"
    restore
}

sed -i 's/const CONTRAST: Fix32 = Fix32(114_688);/const CONTRAST: Fix32 = Fix32(110_000);/' "$terrain"
run_case "CONTRAST   114688 -> 110000"

sed -i 's/const NORMALISER: Fix32 = Fix32(69_905);/const NORMALISER: Fix32 = Fix32(69_000);/' "$terrain"
run_case "NORMALISER  69905 ->  69000"

sed -i 's/const HEIGHT_HILL: Fix32 = Fix32(51_118);/const HEIGHT_HILL: Fix32 = Fix32(50_000);/' "$terrain"
run_case "HEIGHT_HILL 51118 ->  50000"

sed -i 's/const OCTAVES: u32 = 4;/const OCTAVES: u32 = 3;/' "$terrain"
run_case "OCTAVES         4 ->      3"

sed -i 's/state \^= state >> 30;/state ^= state >> 31;/' "$rng"
run_case "RNG mixer shift (control)"
