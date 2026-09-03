#!/bin/sh
# Build the documentation site. ADR-0107 D4.
#
# The documentation build is a compile step. It is not a text step. The
# reference takes its prose from an import of the compiled extension module, so
# this job builds the extension, installs it, and only then builds the site.
#
# The order below is the decision. A job that installs a documentation tool and
# points it at the source tree publishes a reference with signatures and no
# prose, and it reports no error. The two checks around the site build are what
# turns that silence into a failure.
#
# Give a configuration file as the first argument to build another site. Set
# CACHETTE_DOCS_SKIP_SYNC to 1 to build the site against the environment as it
# is. The `docs-probe` recipe of the justfile uses both, to prove that this job
# can fail.
#
# The site goes to the output directory that the configuration names.
set -eu

CONFIG="${1:-mkdocs.yml}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [ "${CACHETTE_DOCS_SKIP_SYNC:-0}" = "1" ]; then
    echo "==> skip the build of the extension, because the caller asked for it"
else
    echo "==> build the extension and install it beside the documentation tools"
    uv sync --group docs
fi

echo "==> check that the build environment imports the compiled module"
uv run python scripts/check_reference.py --import-only

echo "==> build the site from $CONFIG"
uv run zensical build --clean --strict --config-file "$CONFIG"

# The output directory is read from the configuration, which is its one
# declaration site. The builder resolves it against the directory of the
# configuration file.
SITE_NAME="$(sed -n 's/^site_dir:[[:space:]]*//p' "$CONFIG")"
if [ -z "$SITE_NAME" ]; then
    echo "the configuration $CONFIG names no site_dir"
    exit 1
fi
SITE="$(cd "$(dirname "$CONFIG")/$SITE_NAME" && pwd)"

echo "==> check that the built site carries the prose of the compiled module"
uv run python scripts/check_reference.py "$SITE"

echo "the site is in $SITE"
