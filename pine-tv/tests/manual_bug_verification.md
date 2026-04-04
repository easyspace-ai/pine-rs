# Manual Bug Verification - Multi-Pane Display Issue

## Purpose

This document provides step-by-step instructions to manually verify the multi-pane display bug in pine-tv.

## Prerequisites

1. Build pine-tv:
```bash
cargo build --release --bin pine-tv
```

2. Start the server:
```bash
cargo run --release --bin pine-tv
```

3. Open browser to: http://localhost:7070

## Test Case 1: Two Panes (pane=1, pane=2)

### Pine Script

Copy and paste this script into the pine-tv editor:

```pine
//@version=6
indicator("Two Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(rsi_val, "RSI", color=color.orange, pane=2)
```

### Expected Behavior (After Fix)

- Two indicator panes should be visible below the main chart
- Pane 1 should show the SMA line (blue)
- Pane 2 should show the RSI line (orange)
- Each pane should have height >= 80px

### Actual Behavior (Unfixed Code)

**Observation**:
- [ ] Only one pane is visible (likely pane 2 with RSI)
- [ ] Pane 1 (SMA) is missing or has zero height
- [ ] Only the highest pane index is displayed

**DOM Inspection** (Open DevTools → Elements):
1. Find the chart container: `<div id="chart-container">`
2. Look for pane elements (Lightweight Charts creates divs for each pane)
3. Measure heights using DevTools

**Counterexample**:
```
Expected: 2 visible panes (pane 1 and pane 2)
Actual: 1 visible pane (only pane 2)
Pane 1 height: ___px (should be >= 80px)
Pane 2 height: ___px (should be >= 80px)
```

## Test Case 2: Three Panes (pane=1, pane=2, pane=3)

### Pine Script

```pine
//@version=6
indicator("Three Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
macd_val = ta.macd(close, 12, 26, 9)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(rsi_val, "RSI", color=color.orange, pane=2)
plot(macd_val[0], "MACD", color=color.green, pane=3)
```

### Expected Behavior (After Fix)

- Three indicator panes should be visible
- Each pane should display its respective indicator
- Heights should be distributed appropriately

### Actual Behavior (Unfixed Code)

**Observation**:
- [ ] Only one pane is visible (likely pane 3 with MACD)
- [ ] Panes 1 and 2 are missing or have zero height

**Counterexample**:
```
Expected: 3 visible panes
Actual: 1 visible pane (only pane 3)
Pane 1 height: ___px
Pane 2 height: ___px
Pane 3 height: ___px
```

## Test Case 3: Non-Contiguous Panes (pane=1, pane=3)

### Pine Script

```pine
//@version=6
indicator("Non-Contiguous Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
macd_val = ta.macd(close, 12, 26, 9)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(macd_val[0], "MACD", color=color.green, pane=3)
```

### Expected Behavior (After Fix)

- Pane 1 and pane 3 should both be visible
- Pane 2 may be empty but should not prevent pane 1 from displaying

### Actual Behavior (Unfixed Code)

**Observation**:
- [ ] Only pane 3 is visible
- [ ] Pane 1 is missing

**Counterexample**:
```
Expected: 2 visible panes (pane 1 and pane 3)
Actual: 1 visible pane (only pane 3)
```

## Root Cause Analysis

Based on the bugfix.md and design.md, the issue is in `pine-tv/static/chart.js`, function `applyResult` (lines 273-295):

```javascript
// Current buggy code (lines 289-293):
if (maxPane > 0 && chart.panes().length > maxPane && chartContainer) {
    const sub = chart.panes()[maxPane];
    const h = Math.max(100, Math.floor(chartContainer.clientHeight * 0.28));
    sub.setHeight(h);
}
```

**Problem**: Only sets height for `maxPane`, doesn't iterate through all unique pane indices.

**Expected Fix**: Should collect all unique pane indices and set height for each one.

## Verification Checklist

After running the manual tests, confirm:

- [ ] Bug exists: Only the highest pane index is visible
- [ ] Intermediate panes have zero or minimal height
- [ ] Root cause is in `applyResult` function (only sets height for maxPane)
- [ ] Counterexamples documented above
- [ ] Ready to proceed with implementing the fix

## Next Steps

1. Document counterexamples found in this verification
2. Proceed to Task 2: Write preservation property tests
3. Proceed to Task 3: Implement the fix in chart.js
4. Re-run these manual tests to verify the fix works
