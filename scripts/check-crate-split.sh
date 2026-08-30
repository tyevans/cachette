#!/usr/bin/env bash
# Proves that the simulation core does not depend on PyO3.
#
# ADR-0041 puts the simulation in a core crate that has no PyO3
# dependency at all. That turns a convention into a compile error: no type
# in the core can name a Python object, and no function in it can take an
# interpreter token. The split also lets Miri run over the unsafe storage
# code, because Miri cannot run the interpreter.
#
# The check reads the resolved dependency tree of the core crate, for normal
# and for build dependencies, and fails when a forbidden crate appears.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

forbidden='^(pyo3|pyo3-ffi|pyo3-macros|pyo3-macros-backend|pyo3-build-config|numpy|cpython|rustpython)( |$)'
status=0

for kinds in "normal" "build" "dev"; do
    tree="$(cargo tree --quiet --package cachette-core --edges "$kinds" --prefix none --no-dedupe)"
    if hits="$(printf '%s\n' "$tree" | sed 's/^ *//' | grep -E "$forbidden" || true)"; [ -n "$hits" ]; then
        printf 'crate split: cachette-core has a %s dependency on the interpreter bindings:\n' "$kinds" >&2
        printf '%s\n' "$hits" | sort -u | sed 's/^/  /' >&2
        status=1
    fi
done

if [ "$status" -eq 0 ]; then
    printf 'crate split: cachette-core is free of PyO3.\n'
else
    printf '\nADR-0041 requires that the core crate never links the interpreter.\n' >&2
fi
exit "$status"
