#!/bin/sh
# Prove that the documentation job fails. ADR-0107 D4.
#
# The job that builds the site must fail when the import of the compiled
# extension module does not happen. A guard that nobody has seen fire has not
# been shown to exist, and the project rule on testing says so.
#
# This script breaks the job in the two ways the record names, runs it, and
# requires it to fail each time. It repairs what it broke afterwards, whether
# it passed or not.
#
# Case 1. The compiled module is not in the build environment. The job must
#         stop before it builds the site.
# Case 2. The configuration turns module inspection off. The site build then
#         succeeds, the reference falls back to the type stub, and the prose of
#         every method disappears. The job must fail on the built site.
#
# Case 2 is the case that matters, because it is the one that reports nothing
# on its own. Research report 19 section 4.2 measured that page.
#
# Exit 0 when both cases fail the job. Exit 1 otherwise.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

FIXTURE="tests/fixtures/docs-inspection-off/mkdocs.yml"
# The builder refuses a source directory outside the directory that holds the
# configuration file, so the probe reads the fixture from a copy in the
# repository root. The copy is temporary and this script removes it.
PROBE_CONFIG="$ROOT/mkdocs-inspection-off.yml"
MODULE=""
MOVED=""

restore() {
    if [ -n "$MOVED" ] && [ -f "$MOVED" ]; then
        mv "$MOVED" "$MODULE"
        echo "restored $MODULE"
    fi
    rm -f "$PROBE_CONFIG"
}
trap restore EXIT

echo "==> build the extension and install it beside the documentation tools"
uv sync --group docs

echo
echo "==> case 1: take the compiled module out of the build environment"
MODULE="$(uv run python -c 'import cachette._core as c; print(c.__file__)')"
MOVED="$MODULE.probe"
mv "$MODULE" "$MOVED"
echo "moved $MODULE"

if CACHETTE_DOCS_SKIP_SYNC=1 ./scripts/build-docs.sh; then
    echo "PROBE FAILED: the job passed with no compiled module in the environment"
    exit 1
fi
echo "the job failed, as it must"

mv "$MOVED" "$MODULE"
MOVED=""
echo "restored $MODULE"

echo
echo "==> case 2: turn module inspection off"
# The builder resolves every path in a configuration against the directory
# that holds it, so the fixture only builds from the repository root.
cp "$FIXTURE" "$PROBE_CONFIG"

if CACHETTE_DOCS_SKIP_SYNC=1 ./scripts/build-docs.sh "$PROBE_CONFIG"; then
    echo "PROBE FAILED: the job passed with module inspection off"
    exit 1
fi
echo "the job failed, as it must"

echo
echo "both cases failed the job"
