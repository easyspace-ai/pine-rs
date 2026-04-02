#!/usr/bin/env bash
# Compare pine-cli against PineTS-sourced expect (optional; not a golden gate).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SCRIPT="tests/compatibility/cases/pinets_math_abs.pine"
DATA="tests/compatibility/data/BTCUSDC_1d_pinets_abs_window.csv"
EXPECT="tests/compatibility/expect/pinets_math_abs.expect.json"

python3 scripts/compare_compat_expect.py \
  --script "$SCRIPT" --data "$DATA" --expect "$EXPECT" \
  --expect-key abs_native --actual-key _plotchar --tol 1e-8

python3 scripts/compare_compat_expect.py \
  --script "$SCRIPT" --data "$DATA" --expect "$EXPECT" \
  --expect-key abs_open --actual-key _plot_open --tol 1e-8

echo "OK: PineTS abs expect matches pine-cli (_plotchar + _plot_open)"
