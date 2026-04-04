# Pine-TV Browser Tests

## Multi-Pane Display Bug Exploration Test

This directory contains browser-based tests for the multi-pane display bug in pine-tv.

### Test Overview

**File**: `multi_pane_bug_exploration.test.js`

**Purpose**: Verify that Pine Scripts with multiple indicator panes (pane=1, pane=2, etc.) are correctly displayed in the browser with all panes visible and properly sized.

**Property Tested**: Bug Condition - Multiple Indicator Panes Not Displayed (Requirements 1.1, 1.2, 1.3)

### Expected Behavior

**On UNFIXED code** (current state):
- ❌ Test FAILS - this is CORRECT and confirms the bug exists
- Only one pane (typically the highest index) is visible
- Intermediate panes have zero or minimal height
- Counterexamples are documented in test output

**After FIX is implemented**:
- ✅ Test PASSES - confirms the bug is fixed
- All unique pane indices result in visible panes
- Each pane has height >= 80px

### Setup

1. Install dependencies:
```bash
cd pine-tv/tests
npm install
```

2. Install Playwright browsers (first time only):
```bash
npx playwright install chromium
```

### Running the Tests

**Run the bug exploration test**:
```bash
cd pine-tv/tests
npm test
```

**Run with debug output**:
```bash
npm run test:debug
```

### Test Cases

The test includes four scenarios:

1. **Two Panes** (pane=1, pane=2)
   - SMA in pane 1, RSI in pane 2
   - Both panes should be visible

2. **Three Panes** (pane=1, pane=2, pane=3)
   - SMA in pane 1, RSI in pane 2, MACD in pane 3
   - All three panes should be visible

3. **Non-Contiguous Panes** (pane=1, pane=3)
   - SMA in pane 1, MACD in pane 3 (skipping pane 2)
   - Both panes should be visible

4. **Mixed Overlay and Panes** (pane=0, pane=1, pane=2)
   - SMA overlay on main chart, RSI in pane 1, MACD in pane 2
   - Both indicator panes should be visible

### How It Works

1. **Server Startup**: The test automatically starts the pine-tv server on port 3456
2. **Browser Automation**: Uses Playwright to control a headless Chromium browser
3. **Script Execution**: Loads each test Pine Script into the editor and runs it
4. **DOM Inspection**: Queries the browser DOM to find pane elements and measure their heights
5. **Assertions**: Verifies that all expected panes exist, are visible, and have adequate height (>= 80px)

### Interpreting Results

**If tests FAIL** (expected on unfixed code):
- Check the console output for pane information
- Look for which panes are missing or have insufficient height
- This confirms the bug exists and provides counterexamples

**If tests PASS** (expected after fix):
- All panes are correctly created and visible
- Heights are properly distributed
- The bug is fixed

### Troubleshooting

**Server fails to start**:
- Ensure pine-tv builds successfully: `cargo build --release`
- Check if port 3456 is already in use
- Review server logs in test output

**Browser tests timeout**:
- Increase `TEST_TIMEOUT` in the test file
- Run with `--detectOpenHandles` to find hanging processes
- Check if chart container loads properly

**Pane detection fails**:
- Verify the DOM selector matches Lightweight Charts structure
- Check browser console for JavaScript errors
- Ensure the chart.js module is loaded correctly

### Integration with Bugfix Workflow

This test is **Task 1** in the bugfix workflow:

1. ✅ **Task 1**: Write bug condition exploration test (this test)
   - Run on UNFIXED code
   - Expected: FAILS (confirms bug exists)
   - Document counterexamples

2. **Task 2**: Write preservation property tests
   - Test overlay plots, single pane, empty plots
   - Expected: PASSES (confirms baseline behavior)

3. **Task 3**: Implement the fix in chart.js
   - Modify applyResult function
   - Re-run Task 1 test - should now PASS
   - Re-run Task 2 tests - should still PASS

### Notes

- This test uses **Playwright** for browser automation (more reliable than Puppeteer for modern web apps)
- Tests run in **headless mode** by default (no visible browser window)
- The test automatically cleans up the server process after completion
- Each test case runs independently with a fresh page load
