# Compatibility-style checks (PineTS-inspired)

This directory holds **optional** fixtures for a PineTS-like workflow: `.pine` + fixed bars + expected series JSON.

- **Authoritative correctness** for the repo remains golden tests (`tests/run_golden.sh`) and `AGENTS.md`.
- Use these fixtures to prototype stricter per-script expectations before promoting cases into golden CSV/JSON.

## Layout

- `cases/` — small scripts (e.g. smoke indicators).
- `expect/` — expected outputs (JSON with plot/series names → arrays of numbers or `"na"`).
- `data/` — CSV slices aligned with specific `expect` files (optional).

### Real PineTS fixture: `math.abs`

Copied from `three/PineTS/tests/compatibility/namespace/math/methods/data/abs.expect.json` into `expect/pinets_math_abs.expect.json`. The CSV window `data/BTCUSDC_1d_pinets_abs_window.csv` is **51 daily bars** (UTC) from `three/PineTS/tests/compatibility/_data/BTCUSDC-1d-1704067200000-1763683199000.json`, matching PineTS’s filtered range **2025-10-01 through 2025-11-20** (same `openTime`s as the `plotchar_data` / `plot_data` strings in that expect file).

PineTS is AGPL-licensed; this fixture is for local cross-checking only—**not** a substitute for the repo’s golden tests.

```bash
bash tests/compatibility/run_pinets_abs_compare.sh
```

## Running

```bash
bash scripts/run_compat_smoke.sh
```

The script runs `pine-cli` where applicable and can be extended to diff against `expect/*.json` using the same tolerances as `tests/compare_golden.py`.

## PineTS-style `expect.json` vs `pine-cli` JSON

For a PineTS-shaped file (`results.<key>` arrays of numbers or `"na"`), compare against one series key from `pine-cli --format json` (`outputs.<plot_key>`):

```bash
python3 scripts/compare_compat_expect.py \
  --script tests/compatibility/cases/plot_close.pine \
  --data tests/data/BTCUSDT_1h.csv \
  --expect path/to/pinets_case.expect.json \
  --expect-key close_or_other_series \
  --actual-key plot \
  --tol 1e-8
```

`expect/plot_close.expect.json` in this repo is a **shape stub** only; PineTS-style files should expose `results: { "<key>": [ number | "na", ... ] }`.

This is optional scaffolding; golden tests remain the authoritative pass/fail gate for the kernel.
