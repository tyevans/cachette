#!/usr/bin/env bash
# Prove that the toolchain pin check fails on invalid configurations.
#
# ADR-0097 D2 requires a dated nightly compiler. The check must reject
# floating channels and undated channels.
#
# This probe runs the check against rust-toolchain.toml and against every
# broken fixture in tests/fixtures/toolchain-broken/. It asserts that the real
# configuration passes and that each broken fixture fails.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check="$root/scripts/check-toolchain-pin.sh"
fixtures="$root/tests/fixtures/toolchain-broken"

status=0

# 1. Assert that the real toolchain passes.
if ! "$check" "$root/rust-toolchain.toml" > /dev/null 2>&1; then
    printf 'toolchain probe: rust-toolchain.toml failed the check, but it should pass\n' >&2
    status=1
fi

# 2. Assert that each broken fixture fails.
found=0
for fixture in "$fixtures"/*.toml; do
    [ -e "$fixture" ] || continue
    found=$((found + 1))
    if "$check" "$fixture" > /dev/null 2>&1; then
        printf 'toolchain probe: %s passed the check, but it should fail\n' "${fixture#"$root"/}" >&2
        status=1
    fi
done

if [ "$found" -eq 0 ]; then
    printf 'toolchain probe: no broken fixtures found in %s\n' "${fixtures#"$root"/}" >&2
    status=1
fi

if [ "$status" -eq 0 ]; then
    printf 'toolchain probe: check passed on rust-toolchain.toml and rejected all %d broken fixtures.\n' "$found"
fi

exit "$status"
