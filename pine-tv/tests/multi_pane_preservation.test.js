/**
 * Preservation Property Tests - Multi-Pane Display Fix
 * 
 * **Validates: Requirements 3.1, 3.2, 3.3, 3.4**
 * 
 * **Property 2: Preservation** - Existing Pane Behaviors Unchanged
 * 
 * **CRITICAL**: These tests MUST PASS on UNFIXED code - they establish baseline behavior
 * 
 * This test suite follows the observation-first methodology:
 * 1. Observe behavior on UNFIXED code for non-buggy inputs
 * 2. Write property-based tests capturing observed behavior patterns
 * 3. Run tests on UNFIXED code - EXPECTED OUTCOME: Tests PASS
 * 4. After fix is implemented, these tests must still PASS (preservation guarantee)
 * 
 * **Preservation Requirements**:
 * - Plots with pane=0 (overlay on main chart) must continue to display correctly
 * - Single indicator pane (only pane=1) must continue to work as before
 * - Empty plot results must clear all series without errors
 * - Symbol/timeframe switching must clear and re-render correctly
 */

const { chromium } = require('playwright');
const { spawn } = require('child_process');
const path = require('path');

// Test configuration
const SERVER_PORT = 7070;
const SERVER_URL = `http://localhost:${SERVER_PORT}`;
const SERVER_STARTUP_TIMEOUT = 15000;
const TEST_TIMEOUT = 30000;

/**
 * Pine Script test cases for preservation testing
 */
const PRESERVATION_SCRIPTS = {
  overlayOnly: {
    name: 'Overlay Plots (pane=0)',
    code: `//@version=6
indicator("Overlay Test", overlay=true)
sma_val = ta.sma(close, 14)
ema_val = ta.ema(close, 20)
plot(sma_val, "SMA 14", color=color.blue, pane=0)
plot(ema_val, "EMA 20", color=color.orange, pane=0)
`,
    expectedBehavior: {
      overlayPlotsVisible: true,
      indicatorPaneCount: 0,
      description: 'Overlay plots should appear on main chart, no indicator panes'
    }
  },
  
  singlePane: {
    name: 'Single Indicator Pane (pane=1 only)',
    code: `//@version=6
indicator("Single Pane Test", overlay=false)
rsi_val = ta.rsi(close, 14)
plot(rsi_val, "RSI", color=color.purple, pane=1)
`,
    expectedBehavior: {
      indicatorPaneCount: 1,
      minPaneHeight: 80,
      description: 'Single indicator pane should display with height ~28% of container'
    }
  },
  
  emptyPlots: {
    name: 'Empty Plot Results',
    code: `//@version=6
indicator("Empty Test", overlay=false)
// No plot calls - should clear all series
`,
    expectedBehavior: {
      indicatorPaneCount: 0,
      noErrors: true,
      description: 'Empty plot results should clear all previous series without errors'
    }
  },
  
  overlayWithSinglePane: {
    name: 'Overlay + Single Pane (pane=0 and pane=1)',
    code: `//@version=6
indicator("Overlay + Single Pane", overlay=true)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
plot(sma_val, "SMA Overlay", color=color.blue, pane=0)
plot(rsi_val, "RSI", color=color.orange, pane=1)
`,
    expectedBehavior: {
      overlayPlotsVisible: true,
      indicatorPaneCount: 1,
      minPaneHeight: 80,
      description: 'Overlay plots on main chart + single indicator pane'
    }
  }
};

/**
 * Start pine-tv server
 */
async function startServer() {
  return new Promise((resolve, reject) => {
    const serverProcess = spawn('cargo', ['run', '--release', '--bin', 'pine-tv'], {
      cwd: path.join(__dirname, '../..'),
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let serverReady = false;
    const timeout = setTimeout(() => {
      if (!serverReady) {
        serverProcess.kill();
        reject(new Error(`Server failed to start within ${SERVER_STARTUP_TIMEOUT}ms`));
      }
    }, SERVER_STARTUP_TIMEOUT);

    serverProcess.stdout.on('data', (data) => {
      const output = data.toString();
      console.log(`[server] ${output}`);
      if (output.includes('listening') || output.includes('pine-tv listening')) {
        serverReady = true;
        clearTimeout(timeout);
        setTimeout(() => resolve(serverProcess), 2000);
      }
    });

    serverProcess.stderr.on('data', (data) => {
      const output = data.toString();
      console.error(`[server stderr] ${output}`);
      if (output.includes('listening') || output.includes('pine-tv listening')) {
        serverReady = true;
        clearTimeout(timeout);
        setTimeout(() => resolve(serverProcess), 2000);
      }
    });

    serverProcess.on('error', (error) => {
      clearTimeout(timeout);
      reject(error);
    });

    serverProcess.on('exit', (code) => {
      if (!serverReady) {
        clearTimeout(timeout);
        reject(new Error(`Server exited with code ${code} before becoming ready`));
      }
    });
  });
}

/**
 * Get pane information from the browser DOM
 */
async function getPaneInfo(page) {
  return await page.evaluate(() => {
    const chartContainer = document.getElementById('chart-container');
    if (!chartContainer) {
      return { error: 'Chart container not found' };
    }

    const paneElements = chartContainer.querySelectorAll('[data-name="pane"]');
    
    const panes = [];
    paneElements.forEach((paneEl, index) => {
      const rect = paneEl.getBoundingClientRect();
      panes.push({
        index: index,
        height: rect.height,
        width: rect.width,
        visible: rect.height > 0 && rect.width > 0,
        top: rect.top,
        bottom: rect.bottom
      });
    });

    return {
      totalPanes: panes.length,
      panes: panes,
      containerHeight: chartContainer.clientHeight,
      containerWidth: chartContainer.clientWidth
    };
  });
}

/**
 * Check for JavaScript errors in the console
 */
async function getConsoleErrors(page) {
  const errors = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      errors.push(msg.text());
    }
  });
  page.on('pageerror', (error) => {
    errors.push(error.message);
  });
  return errors;
}

/**
 * Run a Pine Script and wait for results
 */
async function runScript(page, code) {
  await page.evaluate((scriptCode) => {
    const editor = window.editor;
    if (editor && editor.setValue) {
      editor.setValue(scriptCode);
    }
  }, code);

  await page.click('#run-btn');
  await page.waitForTimeout(2000);
}

/**
 * Get plot series information from the chart
 */
async function getPlotSeriesInfo(page) {
  return await page.evaluate(() => {
    // Access plotSeries Map from chart module
    if (typeof plotSeries !== 'undefined') {
      return {
        seriesCount: plotSeries.size,
        seriesIds: Array.from(plotSeries.keys())
      };
    }
    return { seriesCount: 0, seriesIds: [] };
  });
}

/**
 * Main preservation test suite
 */
describe('Preservation Property Tests: Multi-Pane Display', () => {
  let browser;
  let page;
  let serverProcess;

  beforeAll(async () => {
    console.log('Starting pine-tv server for preservation tests...');
    serverProcess = await startServer();
    console.log('Server started successfully');

    browser = await chromium.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
  }, SERVER_STARTUP_TIMEOUT + 5000);

  afterAll(async () => {
    if (browser) {
      await browser.close();
    }
    if (serverProcess) {
      serverProcess.kill();
    }
  });

  beforeEach(async () => {
    page = await browser.newPage();
    
    // Track console errors
    const consoleErrors = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
    page.on('pageerror', (error) => {
      consoleErrors.push(error.message);
    });
    page.consoleErrors = consoleErrors;
    
    await page.goto(SERVER_URL);
    await page.waitForSelector('#chart-container', { timeout: 5000 });
  });

  afterEach(async () => {
    if (page) {
      await page.close();
    }
  });

  /**
   * Preservation Test 1: Overlay Plots (pane=0)
   * 
   * **Validates: Requirement 3.1**
   * 
   * Plots with pane index 0 must continue to display correctly on the main chart.
   * This is the baseline behavior that must be preserved after the fix.
   */
  test('Overlay plots (pane=0) display correctly on main chart', async () => {
    const testCase = PRESERVATION_SCRIPTS.overlayOnly;
    console.log(`\n=== Preservation Test: ${testCase.name} ===`);
    console.log(`Expected: ${testCase.expectedBehavior.description}`);

    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info:', JSON.stringify(paneInfo, null, 2));

    // Verify no errors occurred
    expect(paneInfo.error).toBeUndefined();
    
    // Main chart pane (index 0) should exist and be visible
    expect(paneInfo.totalPanes).toBeGreaterThanOrEqual(1);
    const mainPane = paneInfo.panes[0];
    expect(mainPane).toBeDefined();
    expect(mainPane.visible).toBe(true);
    expect(mainPane.height).toBeGreaterThan(0);
    
    // No indicator panes should be created for overlay-only plots
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0 && p.visible);
    console.log(`Indicator panes found: ${indicatorPanes.length}, expected: ${testCase.expectedBehavior.indicatorPaneCount}`);
    expect(indicatorPanes.length).toBe(testCase.expectedBehavior.indicatorPaneCount);
    
    // No console errors
    expect(page.consoleErrors.length).toBe(0);
    
    console.log('✓ Overlay plots display correctly on main chart (baseline preserved)');
  }, TEST_TIMEOUT);

  /**
   * Preservation Test 2: Single Indicator Pane (pane=1 only)
   * 
   * **Validates: Requirement 3.2**
   * 
   * Single indicator pane must continue to work as before with correct height (~28% of container).
   * This is existing working behavior that must not regress.
   */
  test('Single indicator pane (pane=1) displays with correct height', async () => {
    const testCase = PRESERVATION_SCRIPTS.singlePane;
    console.log(`\n=== Preservation Test: ${testCase.name} ===`);
    console.log(`Expected: ${testCase.expectedBehavior.description}`);

    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info:', JSON.stringify(paneInfo, null, 2));

    expect(paneInfo.error).toBeUndefined();
    
    // Should have main chart + 1 indicator pane
    expect(paneInfo.totalPanes).toBeGreaterThanOrEqual(2);
    
    // Get indicator panes (skip main chart at index 0)
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0 && p.visible);
    console.log(`Indicator panes found: ${indicatorPanes.length}, expected: ${testCase.expectedBehavior.indicatorPaneCount}`);
    expect(indicatorPanes.length).toBe(testCase.expectedBehavior.indicatorPaneCount);
    
    // Verify the single indicator pane has adequate height
    const indicatorPane = indicatorPanes[0];
    console.log(`Indicator pane height: ${indicatorPane.height}px (min: ${testCase.expectedBehavior.minPaneHeight}px)`);
    expect(indicatorPane.height).toBeGreaterThanOrEqual(testCase.expectedBehavior.minPaneHeight);
    
    // Verify height is approximately 28% of container (with tolerance)
    const expectedHeight = Math.floor(paneInfo.containerHeight * 0.28);
    const tolerance = 50; // Allow 50px tolerance
    console.log(`Expected height: ~${expectedHeight}px (28% of ${paneInfo.containerHeight}px), actual: ${indicatorPane.height}px`);
    expect(Math.abs(indicatorPane.height - expectedHeight)).toBeLessThanOrEqual(tolerance);
    
    // No console errors
    expect(page.consoleErrors.length).toBe(0);
    
    console.log('✓ Single indicator pane displays with correct height (baseline preserved)');
  }, TEST_TIMEOUT);

  /**
   * Preservation Test 3: Empty Plot Results
   * 
   * **Validates: Requirement 3.3**
   * 
   * Clearing of plot series when no plots are returned must continue to work without errors.
   * This ensures the fix doesn't break the cleanup logic.
   */
  test('Empty plot results clear all series without errors', async () => {
    const testCase = PRESERVATION_SCRIPTS.emptyPlots;
    console.log(`\n=== Preservation Test: ${testCase.name} ===`);
    console.log(`Expected: ${testCase.expectedBehavior.description}`);

    // First, run a script with plots to populate series
    const setupScript = `//@version=6
indicator("Setup", overlay=false)
plot(ta.rsi(close, 14), "RSI", pane=1)
`;
    await runScript(page, setupScript);
    await page.waitForTimeout(1000);
    
    // Now run the empty script - should clear all series
    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info after empty script:', JSON.stringify(paneInfo, null, 2));

    expect(paneInfo.error).toBeUndefined();
    
    // Should have main chart pane, but no visible indicator panes
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0 && p.visible);
    console.log(`Indicator panes after clear: ${indicatorPanes.length}, expected: ${testCase.expectedBehavior.indicatorPaneCount}`);
    expect(indicatorPanes.length).toBe(testCase.expectedBehavior.indicatorPaneCount);
    
    // No console errors should occur during clearing
    expect(page.consoleErrors.length).toBe(0);
    
    console.log('✓ Empty plot results clear series without errors (baseline preserved)');
  }, TEST_TIMEOUT);

  /**
   * Preservation Test 4: Symbol/Timeframe Switching
   * 
   * **Validates: Requirement 3.4**
   * 
   * Symbol/timeframe switching must continue to clear and re-render correctly.
   * The plotSeries Map should be cleared before new data loads.
   */
  test('Symbol switching clears and re-renders correctly', async () => {
    console.log('\n=== Preservation Test: Symbol Switching ===');
    console.log('Expected: plotSeries Map cleared before new data loads');

    // Run initial script
    const initialScript = `//@version=6
indicator("Initial", overlay=false)
plot(ta.rsi(close, 14), "RSI", pane=1)
`;
    await runScript(page, initialScript);
    await page.waitForTimeout(1000);
    
    const paneInfoBefore = await getPaneInfo(page);
    console.log('Pane Info before symbol switch:', JSON.stringify(paneInfoBefore, null, 2));
    
    // Simulate symbol change by triggering the pine:symbol-change event
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('pine:symbol-change', {
        detail: { symbol: 'ETHUSDT', tf: '1h' }
      }));
    });
    
    await page.waitForTimeout(2000);
    
    // Run script again with new symbol
    await runScript(page, initialScript);
    const paneInfoAfter = await getPaneInfo(page);
    
    console.log('Pane Info after symbol switch:', JSON.stringify(paneInfoAfter, null, 2));
    
    // Verify no errors occurred during switch
    expect(paneInfoAfter.error).toBeUndefined();
    expect(page.consoleErrors.length).toBe(0);
    
    // Verify panes are still rendered correctly
    const indicatorPanes = paneInfoAfter.panes.filter((p, idx) => idx > 0 && p.visible);
    expect(indicatorPanes.length).toBeGreaterThanOrEqual(1);
    expect(indicatorPanes[0].height).toBeGreaterThanOrEqual(80);
    
    console.log('✓ Symbol switching clears and re-renders correctly (baseline preserved)');
  }, TEST_TIMEOUT);

  /**
   * Preservation Test 5: Overlay + Single Pane Combination
   * 
   * **Validates: Requirements 3.1, 3.2**
   * 
   * Combination of overlay plots and single indicator pane must work correctly.
   * This tests that both preservation requirements work together.
   */
  test('Overlay + single indicator pane combination works correctly', async () => {
    const testCase = PRESERVATION_SCRIPTS.overlayWithSinglePane;
    console.log(`\n=== Preservation Test: ${testCase.name} ===`);
    console.log(`Expected: ${testCase.expectedBehavior.description}`);

    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info:', JSON.stringify(paneInfo, null, 2));

    expect(paneInfo.error).toBeUndefined();
    
    // Main chart should be visible (for overlay plots)
    expect(paneInfo.totalPanes).toBeGreaterThanOrEqual(2);
    const mainPane = paneInfo.panes[0];
    expect(mainPane.visible).toBe(true);
    expect(mainPane.height).toBeGreaterThan(0);
    
    // Should have exactly 1 indicator pane
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0 && p.visible);
    console.log(`Indicator panes found: ${indicatorPanes.length}, expected: ${testCase.expectedBehavior.indicatorPaneCount}`);
    expect(indicatorPanes.length).toBe(testCase.expectedBehavior.indicatorPaneCount);
    
    // Indicator pane should have adequate height
    const indicatorPane = indicatorPanes[0];
    console.log(`Indicator pane height: ${indicatorPane.height}px (min: ${testCase.expectedBehavior.minPaneHeight}px)`);
    expect(indicatorPane.height).toBeGreaterThanOrEqual(testCase.expectedBehavior.minPaneHeight);
    
    // No console errors
    expect(page.consoleErrors.length).toBe(0);
    
    console.log('✓ Overlay + single pane combination works correctly (baseline preserved)');
  }, TEST_TIMEOUT);
});
