#!/usr/bin/env bash
# Smoke-run compatibility fixtures (non-fatal vs golden; extend to JSON diff as needed).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CASE="${1:-tests/compatibility/cases/plot_close.pine}"
DATA="${2:-tests/data/BTCUSDT_1h.csv}"

echo "==> compat smoke: check $CASE"
cargo run -p pine-cli --quiet -- check "$CASE"

echo "==> compat smoke: run $CASE on $DATA (first lines of JSON only)"
cargo run -p pine-cli --quiet -- run "$CASE" --data "$DATA" | head -c 400 || true
echo
echo "OK: compat smoke finished"
