#!/usr/bin/env bash
# Check the product requirement records. See docs/product/README.md.
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 scripts/check_prds.py "${1:-docs/product}"
