#!/usr/bin/env bash
# Proves that the toolchain pins a dated nightly compiler.
#
# ADR-0097 D2 requires that the toolchain file names a dated nightly build.
# It must never name a floating channel such as bare nightly, stable, or beta.
#
# A floating channel makes the compiler an unversioned input that changes
# without a commit. Two contributors on one commit would build with two
# different compilers.
#
# This check inspects rust-toolchain.toml and fails when the channel lacks
# a valid date.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-$root/rust-toolchain.toml}"
pattern='^nightly-[0-9]{4}-[0-9]{2}-[0-9]{2}$'

check_file() {
    local file="$1"
    local display_path
    display_path="${file#"$root"/}"

    if [ ! -f "$file" ]; then
        printf 'toolchain pin: file not found: %s\n' "$display_path" >&2
        return 1
    fi

    local channel
    channel="$(awk '
        /^[[:space:]]*\[toolchain\]/ { in_toolchain=1; next }
        /^[[:space:]]*\[/ { in_toolchain=0 }
        in_toolchain && /^[[:space:]]*channel[[:space:]]*=/ {
            line=$0
            sub(/^[[:space:]]*channel[[:space:]]*=[[:space:]]*/, "", line)
            sub(/[[:space:]]*#.*$/, "", line)
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)
            print line
            exit
        }
    ' "$file")"

    # Strip surrounding quotes if present
    channel="${channel#\"}"
    channel="${channel%\"}"
    channel="${channel#\'}"
    channel="${channel%\'}"

    if [ -z "$channel" ]; then
        printf 'toolchain pin: %s contains no channel in [toolchain]\n' "$display_path" >&2
        printf '\nADR-0097 D2 requires that the toolchain pins a dated nightly (nightly-YYYY-MM-DD).\n' >&2
        printf 'A floating channel makes the compiler an unversioned input that changes without a commit.\n' >&2
        return 1
    fi

    if [[ ! "$channel" =~ $pattern ]]; then
        printf 'toolchain pin: %s: invalid channel "%s"\n' "$display_path" "$channel" >&2
        printf '\nADR-0097 D2 requires that the toolchain pins a dated nightly (nightly-YYYY-MM-DD), never a floating channel.\n' >&2
        printf 'A floating channel makes the compiler an unversioned input that changes without a commit.\n' >&2
        return 1
    fi

    printf 'toolchain pin: %s pins %s.\n' "$display_path" "$channel"
    return 0
}

status=0
if [ -d "$target" ]; then
    count=0
    for f in "$target"/*.toml; do
        [ -e "$f" ] || continue
        count=$((count + 1))
        check_file "$f" || status=1
    done
    if [ "$count" -eq 0 ]; then
        printf 'toolchain pin: no TOML files found in directory: %s\n' "$target" >&2
        exit 1
    fi
else
    check_file "$target" || status=1
fi

exit "$status"
