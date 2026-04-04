# Multi-Pane Display Fix Design

## Overview

The pine-tv playground fails to display multiple indicator panes when Pine Script outputs plots with different pane indices (e.g., pane=1, pane=2). The root cause is in the `applyResult` function in `pine-tv/static/chart.js`, which only calls `setHeight` on the maximum pane index without ensuring all intermediate panes are properly created and visible. This fix will iterate through all unique pane indices and configure each pane's height and visibility appropriately.

## Glossary

- **Bug_Condition (C)**: The condition that triggers the bug - when multiple plots have different pane indices greater than 0, only one indicator pane is displayed
- **Property (P)**: The desired behavior - all panes with unique indices should be created, visible, and properly sized
- **Preservation**: Existing behavior for pane 0 (overlay), single indicator panes, and plot clearing must remain unchanged
- **applyResult**: The function in `pine-tv/static/chart.js` that processes plot results and renders them on the chart
- **maxPane**: The highest pane index found in the plots array
- **chart.panes()**: Lightweight Charts v5 API method that returns an array of pane objects

## Bug Details

### Bug Condition

The bug manifests when a Pine Script outputs multiple plots with different pane indices greater than 0. The `applyResult` function calculates the maximum pane index but only sets the height for that specific pane, without ensuring all intermediate panes (1 through maxPane-1) are properly created and visible.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type { plots: Array<{pane: number}> }
  OUTPUT: boolean
  
  uniquePanes := UNIQUE(input.plots.map(p => p.pane).filter(p => p > 0))
  
  RETURN uniquePanes.length >= 2
         AND EXISTS paneIdx IN uniquePanes WHERE paneIdx > 1
         AND NOT allPanesVisible(uniquePanes)
END FUNCTION
```

### Examples

- **Example 1**: Script outputs MACD in pane=1 and RSI in pane=2
  - Expected: Two indicator panes displayed below the main chart
  - Actual: Only one indicator pane is visible (typically pane 2)

- **Example 2**: Script outputs three indicators in pane=1, pane=2, pane=3
  - Expected: Three indicator panes displayed with appropriate heights
  - Actual: Only pane 3 is visible and sized

- **Example 3**: Script outputs indicators in pane=1 and pane=3 (skipping pane=2)
  - Expected: Panes 1 and 3 are visible (pane 2 may be empty but should exist)
  - Actual: Only pane 3 is visible

- **Edge case**: Script outputs single indicator in pane=1
  - Expected: Single indicator pane displayed correctly (this currently works)

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Plots with pane index 0 (overlay on main chart) must continue to display correctly on the price chart
- Single indicator pane (only pane=1) must continue to work as before
- Clearing of plot series when no plots are returned must continue to work
- Symbol/timeframe switching and plot series cleanup must remain unchanged

**Scope:**
All inputs that do NOT involve multiple distinct pane indices greater than 0 should be completely unaffected by this fix. This includes:
- Overlay plots (pane=0)
- Single indicator pane scenarios
- Empty plot results
- Chart initialization and WebSocket handling

## Hypothesized Root Cause

Based on the bug description and code analysis, the issue is in the `applyResult` function (lines 273-295):

1. **Incomplete Pane Initialization**: The function only calls `setHeight` on `chart.panes()[maxPane]` without iterating through all panes between 1 and maxPane
   - Current code: `if (maxPane > 0 && chart.panes().length > maxPane && chartContainer)`
   - This assumes all intermediate panes are automatically created and visible

2. **Missing Pane Visibility Configuration**: Lightweight Charts v5 may require explicit configuration for each pane to be visible
   - The code doesn't verify that panes 1 through maxPane-1 exist and are properly sized

3. **Implicit Pane Creation Assumption**: The code assumes that calling `chart.addSeries(LWC.LineSeries, options, paneIndex)` automatically creates and makes visible all necessary panes
   - This may not be the case when pane indices are non-contiguous or when multiple panes need different heights

4. **Single Height Assignment**: Only the maximum pane gets a height assignment, leaving other panes potentially with default (possibly zero or minimal) heights

## Correctness Properties

Property 1: Bug Condition - Multiple Indicator Panes Display

_For any_ plot result where multiple unique pane indices greater than 0 exist, the fixed applyResult function SHALL create and display all panes with those indices, each properly sized and visible.

**Validates: Requirements 2.1, 2.2, 2.3**

Property 2: Preservation - Existing Pane Behaviors

_For any_ plot result where pane indices are 0 (overlay) or only a single pane index greater than 0 exists, the fixed code SHALL produce exactly the same visual result as the original code, preserving overlay display, single indicator pane display, and plot clearing behavior.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4**

## Fix Implementation

### Changes Required

Assuming our root cause analysis is correct:

**File**: `pine-tv/static/chart.js`

**Function**: `applyResult`

**Specific Changes**:

1. **Collect Unique Pane Indices**: After calculating `maxPane`, collect all unique pane indices from the plots array
   - Create a Set of unique pane indices greater than 0
   - This handles non-contiguous pane indices (e.g., pane=1 and pane=3)

2. **Iterate Through All Panes**: Replace the single `setHeight` call with a loop that processes each unique pane
   - For each pane index in the unique set, verify the pane exists in `chart.panes()`
   - Set appropriate height for each pane

3. **Calculate Appropriate Heights**: Distribute available height among multiple panes
   - Current: 28% of container height for the single maxPane
   - Proposed: Divide available indicator space (e.g., 40% of container) among all indicator panes
   - Ensure minimum height per pane (e.g., 80-100px)

4. **Verify Pane Existence**: Add defensive check to ensure pane exists before calling `setHeight`
   - Check `chart.panes().length > paneIdx` before accessing `chart.panes()[paneIdx]`

5. **Handle Edge Cases**: Maintain backward compatibility for single pane and overlay scenarios
   - If only one unique pane index > 0, use existing 28% height logic
   - If all plots are pane=0, skip pane sizing logic entirely

### Proposed Code Structure

```javascript
function applyResult(result) {
    if (!result.ok) return;

    const plots = result.plots || [];
    const LWC = getLwc();
    if (!chart || !LWC) return;

    // Clear existing series
    for (const [, series] of plotSeries) {
        chart.removeSeries(series);
    }
    plotSeries.clear();

    // Collect unique pane indices
    const uniquePanes = new Set();
    let maxPane = 0;
    
    for (const plot of plots) {
        const paneIdx = Number(plot.pane) || 0;
        if (paneIdx > 0) {
            uniquePanes.add(paneIdx);
        }
        if (paneIdx > maxPane) {
            maxPane = paneIdx;
        }
        addLineSeriesToPane(plot, paneIdx, LWC);
    }

    // Configure heights for all indicator panes
    if (uniquePanes.size > 0 && chartContainer) {
        const indicatorPanes = Array.from(uniquePanes).sort((a, b) => a - b);
        const totalIndicatorHeight = Math.floor(chartContainer.clientHeight * 0.4);
        const heightPerPane = Math.max(100, Math.floor(totalIndicatorHeight / indicatorPanes.length));
        
        for (const paneIdx of indicatorPanes) {
            if (chart.panes().length > paneIdx) {
                const pane = chart.panes()[paneIdx];
                pane.setHeight(heightPerPane);
            }
        }
    }
}
```

## Testing Strategy

### Validation Approach

The testing strategy follows a two-phase approach: first, surface counterexamples that demonstrate the bug on unfixed code, then verify the fix works correctly and preserves existing behavior.

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug BEFORE implementing the fix. Confirm or refute the root cause analysis. If we refute, we will need to re-hypothesize.

**Test Plan**: Create Pine Script test cases that output multiple indicators with different pane indices. Run these through the unfixed `pine-tv` and observe which panes are visible in the browser. Use browser DevTools to inspect the DOM and pane heights.

**Test Cases**:
1. **Two Panes Test**: Script with `plot(sma(close, 14), "SMA", pane=1)` and `plot(rsi(close, 14), "RSI", pane=2)` (will fail on unfixed code - only one pane visible)
2. **Three Panes Test**: Script with indicators in pane=1, pane=2, pane=3 (will fail on unfixed code - only pane 3 visible)
3. **Non-Contiguous Panes Test**: Script with indicators in pane=1 and pane=3, skipping pane=2 (will fail on unfixed code)
4. **Mixed Overlay and Panes Test**: Script with overlay (pane=0) and two indicator panes (pane=1, pane=2) (will fail on unfixed code for indicator panes)

**Expected Counterexamples**:
- Only the highest pane index is visible in the chart
- Intermediate panes have zero or minimal height and are not visible
- Browser console may show no errors, indicating silent failure
- Possible causes: missing height assignment, panes not created, panes created but not visible

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds, the fixed function produces the expected behavior.

**Pseudocode:**
```
FOR ALL plotResult WHERE isBugCondition(plotResult) DO
  applyResult_fixed(plotResult)
  visiblePanes := getVisiblePanes()
  uniquePaneIndices := getUniquePaneIndices(plotResult.plots)
  ASSERT visiblePanes.length >= uniquePaneIndices.length
  ASSERT ALL paneIdx IN uniquePaneIndices: paneIsVisible(paneIdx)
  ASSERT ALL paneIdx IN uniquePaneIndices: paneHeight(paneIdx) >= 80
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold, the fixed function produces the same result as the original function.

**Pseudocode:**
```
FOR ALL plotResult WHERE NOT isBugCondition(plotResult) DO
  originalVisiblePanes := applyResult_original(plotResult)
  fixedVisiblePanes := applyResult_fixed(plotResult)
  ASSERT originalVisiblePanes = fixedVisiblePanes
  ASSERT overlayPlotsStillWork(plotResult)
  ASSERT singlePanePlotsStillWork(plotResult)
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation checking because:
- It generates many test cases automatically across the input domain
- It catches edge cases that manual unit tests might miss
- It provides strong guarantees that behavior is unchanged for all non-buggy inputs

**Test Plan**: Observe behavior on UNFIXED code first for overlay plots and single-pane indicators, then write property-based tests capturing that behavior.

**Test Cases**:
1. **Overlay Preservation**: Verify that plots with pane=0 continue to display on the main chart after fix
2. **Single Pane Preservation**: Verify that scripts with only pane=1 continue to work with same height and visibility
3. **Empty Plots Preservation**: Verify that empty plot results continue to clear all series without errors
4. **Symbol Switch Preservation**: Verify that switching symbols/timeframes continues to clear and re-render correctly

### Unit Tests

- Test `applyResult` with two distinct pane indices (pane=1, pane=2)
- Test `applyResult` with three distinct pane indices (pane=1, pane=2, pane=3)
- Test `applyResult` with non-contiguous pane indices (pane=1, pane=3)
- Test edge case: single pane (pane=1 only) - should preserve existing behavior
- Test edge case: overlay only (pane=0) - should preserve existing behavior
- Test edge case: empty plots array - should clear all series

### Property-Based Tests

- Generate random plot configurations with varying numbers of panes (1-5) and verify all panes are visible
- Generate random pane index combinations (including non-contiguous) and verify correct pane creation
- Generate random plot data with mixed overlay and indicator panes and verify correct rendering
- Test that total allocated height for indicator panes doesn't exceed container height

### Integration Tests

- Load pine-tv in browser with multi-pane script and verify all panes are visible
- Switch between single-pane and multi-pane scripts and verify correct rendering
- Test real-time updates with multi-pane indicators and verify panes remain visible
- Test with different container sizes and verify responsive pane heights
