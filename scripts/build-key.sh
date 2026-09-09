#!/usr/bin/env bash
# Print the key of the engine build, and nothing else.
#
# **The compiled extension is a function of the sources that build it, and of
# nothing else in the tree.** A run that changes only a document builds the
# same bytes. Two things cache on that fact: the wheel a run installs, and the
# controller baseline a run measures. Both would serve a stale answer after an
# engine change if they derived the key differently, so this script is the one
# definition and both read it.
#
# The key holds the sources, the two manifests that pin the dependency
# versions, and the toolchain file that pins the compiler. A change to any one
# of them changes the bytes, and a change to anything else does not.
#
# **The key follows the working tree, not the last commit.** The remote runner
# ships the tracked files with the content the working tree holds now, so an
# uncommitted edit to the engine reaches the machine. A key taken from the
# commit would not move, and the run would install the wheel of the previous
# sources and report a baseline the previous sources measured.
#
# A caller that has no git repository cannot derive the key. This exits
# non-zero and prints nothing, and each caller decides what that means. The
# remote runner has no repository, so the launcher passes the value it
# derived here in the environment instead.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || exit 1

git ls-files -c -z -- crates Cargo.lock Cargo.toml rust-toolchain.toml \
    | sort -z \
    | xargs -0 sha256sum \
    | sha256sum \
    | cut -c1-16
