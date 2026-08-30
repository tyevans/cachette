#!/usr/bin/env bash
# Closes the gap that clippy leaves in the float ban.
#
# ADR-0002 D1 forbids floating point in simulated or aggregated state.
# ADR-0002 D2 requires a lint to enforce that boundary.
#
# clippy `disallowed_types` rejects a named `f32` or `f64`. It does not
# reject a float literal whose type the compiler infers. `let x = 1.5;`
# passes clippy and is a float. This script rejects that case.
#
# It also rejects the reassociating operations by name. ADR-0002 D2 names
# `f32::algebraic_add` and its siblings. Those methods do not resolve on the
# pinned toolchain, so a clippy `disallowed_methods` entry cannot name them.
# The name check covers them until they resolve.
#
# What this script covers:
#   - a float literal, such as `1.0`, `1e5`, `1.0f32`
# It removes line comments, hexadecimal literals and binary literals first.
#   - a float type name, such as `f32` or `f64`
#   - a reassociating or fused operation by name
#
# What clippy covers and this script does not: a float that reaches the code
# through a dependency type.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-$root/crates/cachette-core}"

pattern='[0-9]\.[0-9]|(^|[^A-Za-z0-9_.])[0-9]+(\.[0-9]+)?[eE][-+]?[0-9]|[0-9](f32|f64)\b|\bf32\b|\bf64\b|algebraic_(add|sub|mul|div|rem)|\bmul_add\b|\bf(32|64)::from_bits\b'
status=0

while IFS= read -r file; do
    # Remove line comments and doc comments before the search.
    # Remove line comments, then hexadecimal and binary literals. A hex
    # literal such as 0x9e37 otherwise reads as an exponent.
    stripped="$(sed -E -e 's;//.*$;;' -e 's/0[xXbB][0-9a-fA-F_]+//g' "$file")"
    if hits="$(printf '%s\n' "$stripped" | grep -nE "$pattern" || true)"; [ -n "$hits" ]; then
        printf 'float ban: %s\n' "$file"
        printf '%s\n' "$hits" | sed 's/^/  /'
        status=1
    fi
done < <(find "$target" -name '*.rs' -type f | sort)

if [ "$status" -ne 0 ]; then
    printf '\nADR-0002 D1 forbids floating point in the simulation core.\n' >&2
    printf 'Route the arithmetic through cachette_core::sim_math.\n' >&2
fi
exit "$status"
