# Implementation Plan

- [x] 1. Write bug condition exploration test
  - **Property 1: Bug Condition** - Multiple Indicator Panes Not Displayed
  - **CRITICAL**: This test MUST FAIL on unfixed code - failure confirms the bug exists
  - **DO NOT attempt to fix the test or the code when it fails**
  - **NOTE**: This test encodes the expected behavior - it will validate the fix when it passes after implementation
  - **GOAL**: Surface counterexamples that demonstrate the bug exists
  - **Scoped PBT Approach**: Test with concrete failing cases - scripts with pane=1 and pane=2 (or pane=1, pane=2, pane=3)
  - Create Pine Script test cases that output multiple indicators with different pane indices (e.g., MACD in pane=1, RSI in pane=2)
  - Load these scripts in pine-tv and verify which panes are actually visible in the browser
  - Use browser DevTools to inspect DOM and measure pane heights
  - Test assertions should verify: all unique pane indices > 0 result in visible panes with height >= 80px
  - Run test on UNFIXED code
  - **EXPECTED OUTCOME**: Test FAILS (this is correct - it proves the bug exists)
  - Document counterexamples found: which panes are missing, what heights are observed, DOM structure
  - Mark task complete when test is written, run, and failure is documented
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Existing Pane Behaviors Unchanged
  - **IMPORTANT**: Follow observation-first methodology
  - Observe behavior on UNFIXED code for non-buggy inputs:
    - Overlay plots (pane=0) display correctly on main chart
    - Single indicator pane (only pane=1) displays with correct height
    - Empty plot results clear all series without errors
    - Symbol/timeframe switching clears and re-renders correctly
  - Write property-based tests capturing observed behavior patterns:
    - For all plots with pane=0, they appear on the main candlestick chart
    - For scripts with only pane=1, exactly one indicator pane is created with height ~28% of container
    - For empty plot arrays, all previous series are removed
    - For symbol changes, plotSeries Map is cleared before new data loads
  - Property-based testing generates many test cases for stronger guarantees
  - Run tests on UNFIXED code
  - **EXPECTED OUTCOME**: Tests PASS (this confirms baseline behavior to preserve)
  - Mark task complete when tests are written, run, and passing on unfixed code
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 3. Fix for multi-pane display issue

  - [x] 3.1 Implement the fix in applyResult function
    - Collect all unique pane indices from plots array (filter pane > 0, use Set for uniqueness)
    - Replace single maxPane setHeight call with iteration through all unique pane indices
    - Calculate appropriate height distribution: divide available indicator space among all panes
    - For multiple panes: use ~40% of container height total, divide by number of panes, minimum 80-100px per pane
    - For single pane: maintain existing 28% height logic (backward compatibility)
    - Add defensive check: verify chart.panes().length > paneIdx before calling setHeight
    - Handle non-contiguous pane indices (e.g., pane=1 and pane=3)
    - _Bug_Condition: isBugCondition(input) where input.plots contains multiple unique pane indices > 0_
    - _Expected_Behavior: All unique pane indices result in visible panes with appropriate heights (from design expectedBehavior)_
    - _Preservation: Overlay plots (pane=0), single pane scenarios, empty plots, and symbol switching behavior unchanged (from Preservation Requirements in design)_
    - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 3.4_

  - [x] 3.2 Verify bug condition exploration test now passes
    - **Property 1: Expected Behavior** - Multiple Indicator Panes Displayed
    - **IMPORTANT**: Re-run the SAME test from task 1 - do NOT write a new test
    - The test from task 1 encodes the expected behavior
    - When this test passes, it confirms the expected behavior is satisfied
    - Run bug condition exploration test from step 1
    - **EXPECTED OUTCOME**: Test PASSES (confirms bug is fixed)
    - Verify all unique pane indices are now visible with appropriate heights
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 3.3 Verify preservation tests still pass
    - **Property 2: Preservation** - Existing Behaviors Unchanged
    - **IMPORTANT**: Re-run the SAME tests from task 2 - do NOT write new tests
    - Run preservation property tests from step 2
    - **EXPECTED OUTCOME**: Tests PASS (confirms no regressions)
    - Confirm overlay plots, single pane, empty plots, and symbol switching still work correctly
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4. Checkpoint - Ensure all tests pass
  - Verify bug condition test passes (all panes visible)
  - Verify preservation tests pass (no regressions)
  - Test with various Pine Script examples: 2 panes, 3 panes, non-contiguous panes
  - Ensure pine-tv displays all indicator panes correctly in browser
  - Ask the user if questions arise
