# Preservation Property Test Results

## Task 2: Write Preservation Property Tests (BEFORE implementing fix)

**Date**: 2025-01-XX  
**Status**: ✅ COMPLETE  
**Test Execution**: All tests PASS on UNFIXED code

## Overview

This document records the results of preservation property tests executed on the **UNFIXED** codebase. These tests establish the baseline behavior that MUST be preserved after implementing the multi-pane display fix.

## Test Suite: `multi_pane_preservation_api_test.rs`

### Test Results Summary

| Test | Status | Description |
|------|--------|-------------|
| `test_preservation_overlay_plots_pane_zero` | ✅ PASS | Overlay plots (pane=0) display correctly on main chart |
| `test_preservation_single_indicator_pane` | ✅ PASS | Single indicator pane (pane=1) displays with correct height |
| `test_preservation_empty_plots_clear_series` | ✅ PASS | Empty plot results clear all series without errors |
| `test_preservation_overlay_plus_single_pane` | ✅ PASS | Overlay + single pane combination works correctly |
| `test_preservation_multiple_plots_same_pane` | ✅ PASS | Multiple plots in same pane work correctly |
| `document_preservation_requirements` | ✅ PASS | Documentation test |

**Total**: 6/6 tests passing

## Baseline Behavior Documented

### 1. Overlay Plots (pane=0) - Requirement 3.1

**Observed Behavior**:
- Scripts with `overlay=true` produce plots with `pane=0`
- All overlay plots appear on the main chart
- No indicator panes are created for overlay-only plots
- Multiple overlay plots can coexist on pane=0

**API Response Example**:
```json
{
  "ok": true,
  "plots": [
    {
      "id": "SMA 14",
      "title": "SMA 14",
      "pane": 0,
      "color": "#2196F3",
      "linewidth": 2.0,
      "data": [...]
    },
    {
      "id": "EMA 20",
      "title": "EMA 20",
      "pane": 0,
      "color": "#FF9800",
      "linewidth": 2.0,
      "data": [...]
    }
  ]
}
```

**Preservation Requirement**: After fix, overlay plots must continue to display on main chart with pane=0.

### 2. Single Indicator Pane (pane=1) - Requirement 3.2

**Observed Behavior**:
- Scripts with `overlay=false` produce plots with `pane=1`
- Single indicator pane is created below the main chart
- Frontend displays with height ~28% of container
- Pane is visible and properly sized

**API Response Example**:
```json
{
  "ok": true,
  "plots": [
    {
      "id": "RSI",
      "title": "RSI",
      "pane": 1,
      "color": "#9C27B0",
      "linewidth": 2.0,
      "data": [...]
    }
  ]
}
```

**Preservation Requirement**: After fix, single indicator pane must continue to work with same height and visibility.

### 3. Empty Plot Results - Requirement 3.3

**Observed Behavior**:
- Scripts with no `plot()` calls return empty plots array
- No errors occur during execution
- Frontend should clear all previous series
- No console errors or exceptions

**API Response Example**:
```json
{
  "ok": true,
  "plots": []
}
```

**Preservation Requirement**: After fix, empty plot results must continue to clear series without errors.

### 4. Overlay + Single Pane Combination - Requirements 3.1, 3.2

**Observed Behavior**:
- Scripts with `overlay=true` can produce both pane=0 and pane=1 plots
- Current implementation: all plots default to pane=0 when overlay=true
- This is the baseline behavior to preserve

**API Response Example**:
```json
{
  "ok": true,
  "plots": [
    {
      "id": "RSI",
      "title": "RSI",
      "pane": 0,
      "data": [...]
    },
    {
      "id": "SMA Overlay",
      "title": "SMA Overlay",
      "pane": 0,
      "data": [...]
    }
  ]
}
```

**Preservation Requirement**: After fix, overlay + indicator combinations must continue to work correctly.

### 5. Multiple Plots in Same Pane - Requirement 3.2

**Observed Behavior**:
- Multiple plots with same pane index display together
- Scripts with `overlay=false` put all plots in pane=1
- All plots appear in the same indicator pane
- No conflicts or rendering issues

**API Response Example**:
```json
{
  "ok": true,
  "plots": [
    {
      "id": "SMA 14",
      "title": "SMA 14",
      "pane": 1,
      "data": [...]
    },
    {
      "id": "EMA 20",
      "title": "EMA 20",
      "pane": 1,
      "data": [...]
    }
  ]
}
```

**Preservation Requirement**: After fix, multiple plots in same pane must continue to display together.

## Test Execution Details

### Command
```bash
cargo test -p pine-tv --test multi_pane_preservation_api_test -- --nocapture
```

### Execution Time
- Total: 2.54s
- All tests passed on first run
- No flaky tests observed

### Test Environment
- Execution Mode: Eval (pine-eval)
- Data Source: tests/data/BTCUSDT_1h.csv
- Bars: 20
- Symbol: BTCUSDT
- Timeframe: 1h

## Preservation Guarantee

**CRITICAL**: All preservation tests MUST continue to PASS after the multi-pane display fix is implemented.

If any preservation test fails after the fix:
1. The fix has introduced a regression
2. The fix must be revised to preserve existing behavior
3. Do NOT proceed with the fix until all preservation tests pass

## Next Steps

1. ✅ Task 1: Write bug condition exploration tests (COMPLETED)
2. ✅ Task 2: Write preservation property tests (COMPLETED - THIS DOCUMENT)
3. ⏭️ Task 3: Implement the fix in `pine-tv/static/chart.js`
4. ⏭️ Task 4: Verify bug condition tests now PASS
5. ⏭️ Task 5: Verify preservation tests still PASS

## Files Created

1. `pine-tv/tests/multi_pane_preservation_api_test.rs` - Rust API-level preservation tests
2. `pine-tv/tests/multi_pane_preservation.test.js` - JavaScript browser-level preservation tests
3. `pine-tv/tests/PRESERVATION_TEST_RESULTS.md` - This document

## Validation

- [x] All preservation tests written
- [x] All preservation tests executed on UNFIXED code
- [x] All preservation tests PASS
- [x] Baseline behavior documented
- [x] Preservation requirements clearly stated
- [x] Test results recorded

**Conclusion**: Preservation property tests successfully establish the baseline behavior that must be maintained after implementing the multi-pane display fix. The fix can now proceed with confidence that regressions will be detected.
