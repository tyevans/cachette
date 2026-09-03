#!/usr/bin/env bash
# Prove that the moved-path rule of the merge-defect check still works.
#
# The other record checks are probed with a broken fixture directory and a
# `!`, because each reads a directory and returns one verdict.[^1] This check
# cannot be probed that way. Its input is a change, not a directory, so a
# fixture must be a git history. Each case below builds one, in a temporary
# repository, and runs the real command over it.
#
# **A `!` would prove one direction, and the wrong one.** The rule has failed
# twice, and both failures were the check staying quiet: once when a bare
# number matched 14571 lines of ordinary text, and once when the repair
# silenced every dotless name, so that a move of `justfile` reported
# nothing.[^2] A probe that only proves the check can fail would have passed
# through both. Each case here therefore states the verdict it demands, and a
# case that demands silence is as load-bearing as a case that demands a
# failure.
#
# Add a case when the rule changes. A case is cheap and the rule is not.
#
# References
#
# [^1]: The gate recipes. `justfile`
# [^2]: Findings register, FND-257. `docs/FINDINGS.md`
# [^3]: The check under test. `scripts/check_merge_defects.py`
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check="$root/scripts/check-merge-defects.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

passed=0
failed=0

# Build a repository, run the check over the change, and compare the verdict.
#
# $1 the name of the case
# $2 the verdict it demands: "reports" or "is quiet"
# $3 the path the change moves away from
# $4 the text a document holds after the move
case_runs() {
    name="$1"
    demands="$2"
    moved="$3"
    text="$4"

    repo="$work/$name"
    mkdir -p "$repo"
    git -C "$repo" init -q
    git -C "$repo" config user.email probe@example.com
    git -C "$repo" config user.name Probe

    mkdir -p "$repo/$(dirname "$moved")"
    printf 'the file that moves\n' > "$repo/$moved"
    mkdir -p "$repo/docs"
    printf 'a document\n' > "$repo/docs/README.md"
    git -C "$repo" add -A
    git -C "$repo" commit -qm "the state before the move"

    mkdir -p "$repo/moved"
    git -C "$repo" mv "$moved" "moved/$(basename "$moved")"
    printf '%s\n' "$text" > "$repo/docs/README.md"

    out="$("$check" --root "$repo" --since HEAD 2>&1)"
    status="$?"

    if [ "$demands" = "reports" ]; then
        want=1
        did="the check reports the move"
        must="report"
    else
        want=0
        did="the check passes over the move"
        must="pass over"
    fi

    if [ "$status" -eq "$want" ]; then
        printf 'ok   %s: %s\n' "$name" "$did"
        passed=$((passed + 1))
    else
        printf 'FAIL %s: the check must %s the move of %s, and it did not\n' \
            "$name" "$must" "$moved"
        printf '%s\n' "$out" | sed 's/^/       /'
        failed=$((failed + 1))
    fi
}

# The rule fires. Without this case the rule can be silenced and nothing
# notices, which is how the second failure reached a merge.
case_runs a-cited-move-is-reported reports \
    docs/a.md 'the register still names docs/a.md in this line'

# A move nothing cites costs nothing, and must not stop a merge.
case_runs an-uncited-move-is-quiet 'is quiet' \
    docs/a.md 'this document names no path at all'

# The first failure. A bare number is a substring of ordinary prose.
case_runs a-numeric-name-is-not-searched 'is quiet' \
    0 'a level 1 cell equals the exact sum of its level 0 tiles'

# A bare number one directory down is the same defect.
case_runs a-numeric-name-below-the-root-is-not-searched 'is quiet' \
    docs/0 'level 0 holds individual tiles, and level 2 summarises level 1'

# The second failure. A dotless word is a name, and documents cite this one.
case_runs a-dotless-word-is-searched reports \
    justfile 'the gate suite is defined in justfile and nowhere else'

# A match inside a longer name is not a reference to the moved file.
case_runs a-longer-name-is-not-a-match 'is quiet' \
    docs/a.md 'this document names docs/a.mdx and nothing else'

# Prose that ends a sentence with a bare path still names the path.
case_runs a-full-stop-ends-the-name reports \
    docs/a.md 'the detail is in docs/a.md.'

# A full stop that a path character follows continues the name.
case_runs a-suffix-after-the-stop-is-a-longer-name 'is quiet' \
    docs/a.md 'the backup is docs/a.md.bak and nothing names the original'

printf '\nchecked %d cases of the moved-path rule: %d failures\n' \
    "$((passed + failed))" "$failed"

if [ "$failed" -ne 0 ]; then
    exit 1
fi
