# VM ↔ Eval Parity Status

> Last updated: 2026-04-05
>
> This document tracks the exact substitution status of the VM (bytecode) engine
> relative to the eval (tree-walking) engine. Every claim is backed by a
> regression test; no item may be declared "replaceable" without a passing parity
> test entry.

---

## Summary

| Metric | Count |
|--------|-------|
| Golden tests (eval baseline) | 67 |
| VM parity tests (VM = eval) | 59 |
| Eval-only golden tests (known VM gaps) | 8 |
| Entry-layer parity: CLI | 59 scripts |
| Entry-layer parity: WASM (native bridge) | 20 scripts |
| Entry-layer parity: pine-tv API | 59 scripts |

---

## 1. Replaceable (VM ≥ eval) — 59 scripts proven

All scripts listed in `tests/vm_parity_cases.json` have been verified to
produce **bit-identical** numerical output (tolerance 1e-8) via both engines.

### TA indicators (34 scripts)

| Script | Plots |
|--------|-------|
| sma_14 | SMA 14 |
| ema_12 | EMA 12 |
| rsi_14 | RSI 14 |
| macd_12_26_9 | MACD Line, Signal Line, Histogram |
| bbands_20_2 | Basis (SMA 20), Upper Band, Lower Band |
| bbw_20_2 | BBW 20 2 |
| stoch_14_3_3 | %K |
| atr_14 | ATR 14 |
| tr_basic | TR |
| highest_10 | Highest 10 |
| lowest_10 | Lowest 10 |
| highestbars_10 | Highest Bars 10 |
| lowestbars_10 | Lowest Bars 10 |
| mom_10 | Momentum 10 |
| cci_20 | CCI 20 |
| change_2 | Change 2 |
| correlation_10 | Corr 10 |
| cum_volume | Cum Volume |
| dev_10 | Dev 10 |
| dmi_5_5 | DI Plus, DI Minus, ADX |
| linreg_10 | LinReg 10 |
| median_10 | Median 10 |
| mfi_14 | MFI 14 |
| obv_basic | OBV |
| percentrank_10 | PctRank 10 |
| pvt_basic | PVT |
| range_10 | Range 10 |
| rising_falling | Rising 3, Falling 3 |
| rma_14 | RMA 14 |
| roc_10 | ROC 10 |
| stdev_10 | StDev 10 |
| supertrend_3_5 | Supertrend, Direction |
| swma_basic | SWMA |
| tsi_13_25 | TSI |
| variance_10 | Variance 10 |
| vwma_10 | VWMA 10 |
| wma_10 | WMA 10 |
| wpr_14 | WPR 14 |
| cross_events | Cross Up, Cross Down, Bars Since Up |

### TA combo scripts (4 scripts)

| Script | Plots |
|--------|-------|
| ta_combo_rsi_macd | RSI Scaled, MACD Line, Histogram |
| ta_combo_bb_atr | BB Width, ATR, BB ATR Ratio |
| ta_combo_ema_cross | EMA Diff, Cross Up, Cross Down |
| ta_combo_stoch_rsi | Stoch RSI K, Stoch RSI D |

### Language features (14 scripts)

| Script | Feature | Plots |
|--------|---------|-------|
| sma_manual | series + manual loop | Manual SMA, Close |
| for_na_math | for loop + na + math | For Math Result |
| while_loop | while loop | While Avg 5 |
| switch_basic | switch with scrutinee | Switch Result |
| udf_basic | UDF declaration + call | UDF Diff, UDF Scale |
| else_if | else if chain | Result |
| else_if_only | multi-level else if | Result |
| udf_block_simple | UDF block body | Add |
| udf_chain | chained UDF calls | Chained UDF, UDF Diff |
| nested_loop | nested for loops | Nested Result |
| for_loop_variants | for with step + nested | Step Sum, Nested Sum |
| multi_output | 5 plots (SMA/RSI/ATR) | SMA 5, SMA 10, SMA 20, RSI, ATR |
| var_varip | var + varip | Cumulative Close, Bar Close, Varip Count |
| math_ops | math.abs/max/min | Abs Diff, Max CO, Min CO |

### Regression (2 scripts)

| Script | Feature |
|--------|---------|
| issue-for-inclusive | for i = 0 to N inclusive |
| ta_default_params | ta.* with explicit params | EMA, MOM, RSI, SMA |

---

## 2. Not Yet Replaceable — 8 eval-only golden tests

These scripts have golden CSVs (eval runs correctly) but **fail VM parity**.
They are excluded from `vm_parity_cases.json`.

| Script | Failure Reason | Category |
|--------|----------------|----------|
| switch_guard | Guardless switch (no scrutinee) returns 0 instead of evaluating arms | VM compiler |
| switch_series_golden | Guardless switch + series comparison | VM compiler |
| conditional_ternary | Nested ternary with comparison produces wrong values | VM evaluator |
| na_handling | `nz()` returns None instead of 0.0 | VM stdlib |
| series_access | `sma_val[1]` historical access on derived series | VM series |
| udf_series | UDF with `src[i]` historical access inside loop | VM series + UDF |
| udf_series_golden | UDF with default parameter + multiplication | VM UDF |
| udf_default_params | UDF with default parameter values | VM UDF |

---

## 3. Not Yet Replaceable — Functional gaps (no test scripts)

These features work in eval but are **not compiled by the VM** at all.

| Feature | VM Status | Eval Status | Blocking |
|---------|-----------|-------------|----------|
| **strategy() signals** | Not implemented (entry/close/exit capture) | ✅ Full | Hard block |
| **hline()** | Stub (returns Na) | ✅ Special handler | Display only |
| **bgcolor()** | Stub | ✅ Special handler | Display only |
| **fill()** | Stub | ✅ Special handler | Display only |
| **plotshape()** | Stub | ✅ Special handler | Display only |
| **plotchar()** | Stub | ✅ Special handler | Display only |
| **plotarrow()** | Stub | ✅ Special handler | Display only |
| **plot() pane index** | Not tracked | ✅ record_with_pane() | Multi-pane charts |
| **plot() metadata** (color, linewidth) | Not tracked | ✅ record_metadata() | Styling |
| **import / export** | CompileError::Unsupported | ✅ Parsed + eval | Module system |
| **library()** | CompileError::Unsupported | ✅ Parsed + eval | Module system |
| **TypeDef** | CompileError::Unsupported | ✅ Parsed + eval | UDT |
| **EnumDef** | CompileError::Unsupported | ✅ Parsed + eval | UDT |
| **MethodDef** | CompileError::Unsupported | ✅ Parsed + eval | UDT |
| **Array literals** | CompileError: "Expression type" | ✅ Full | Language |
| **Tuple returns** (`[a, b]` from UDF) | CompileError: "Expression type" | ✅ Full | Language |

---

## 4. Acceptable Gaps (documented, not blocking replacement)

These differences exist but do not affect numerical correctness:

| Gap | Impact | Mitigation |
|-----|--------|------------|
| plot() pane index | Single-pane scripts unaffected | VM defaults to auto-pane |
| plot() color/linewidth | Does not affect data values | Styling is display-only |
| Display functions (hline, bgcolor, fill) | Scripts succeed, but visual elements missing | Documented behavior |

---

## 5. Entry-Layer Coverage

| Entry Point | Default Engine | VM Available | Parity Tests | Notes |
|-------------|---------------|-------------|--------------|-------|
| **pine-cli** | Auto (VM → eval fallback) | ✅ | 59 scripts | `--engine vm/eval` flag |
| **pine-tv** (playground) | VM | ✅ | 59 scripts (API integration) | `PINE_TV_MODE=eval` fallback |
| **pine-wasm** | eval | ✅ (new: `runScriptJsonVm`) | 20 scripts (native bridge) | New VM path added |

---

## 6. Eval Deprecation Thresholds

### Level 1 (Current State)
VM is default for indicators; eval auto-fallback on any VM error.
No formal boundary between VM and eval usage.

### Level 2 (Target: eval → diagnostics only for indicators)

All conditions must be met:

| Gate | Condition | Current | Target | Status |
|------|-----------|---------|--------|--------|
| G1 | VM parity scripts | 59 | ≥ 65 | ⬜ |
| G2 | VM↔eval parity all green | 59/59 | 65+/65+ | ⬜ |
| G3 | CLI entry parity | 59 | ≥ 65 | ⬜ |
| G4 | WASM entry VM path | 20 | ≥ 20 | ✅ |
| G5 | pine-tv API parity | 59 | ≥ 59 | ✅ |
| G6 | Gap list documented | This file | Maintained | ✅ |
| G7 | Strategy → eval hardcoded | Auto fallback | Explicit + tested | ✅ |
| G8 | Display stubs don't crash | Stubs return Na | No panics | ✅ |

### Level 3 (Ultimate: eval as pure debug tool)
- All indicator scripts run exclusively on VM
- eval accessible only via `--engine eval` debug flag
- No production path touches eval for non-strategy scripts

### Remaining work to reach Level 2

1. **Fix 8 known VM gaps** (§2) to bring VM parity from 59 → 67:
   - Guardless switch compilation
   - `nz()` function in VM stdlib
   - UDF parameter series access (`param[i]`)
   - UDF default parameter evaluation
   - Nested ternary with comparisons
   - Derived series historical access (`sma_val[1]`)

2. **Verify 6 more scripts** to exceed G1 target of 65.

---

## 7. How to Update This Document

When merging a PR that changes VM or eval behavior:

1. Run `./scripts/dev_verify.sh --full` — must remain green
2. If a new script passes VM parity, add it to `tests/vm_parity_cases.json`
3. Update counts in §Summary and §6 table
4. If fixing a §2 gap, move the entry from §2 to §1
5. If adding a new gap, add to §2 or §3 with failure reason

Manifest file: `tests/vm_parity_cases.json`
Golden test runner: `tests/run_golden.sh`
VM golden test: `crates/pine-vm/tests/vm_golden_test.rs`
CLI parity test: `crates/pine-cli/src/main.rs` (test_execute_script_vm_matches_eval_for_regression_scripts)
WASM parity test: `crates/pine-wasm/tests/wasm_parity_test.rs`
pine-tv parity test: `pine-tv/tests/api_integration_test.rs` (test_api_vm_matches_eval_for_regression_scripts)
