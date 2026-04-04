/**
 * Bug Condition Exploration Test - Multi-Pane Display Issue
 * 
 * **Validates: Requirements 1.1, 1.2, 1.3**
 * 
 * **CRITICAL**: This test MUST FAIL on unfixed code - failure confirms the bug exists
 * 
 * **Property 1: Bug Condition** - Multiple Indicator Panes Not Displayed
 * 
 * This test verifies that when a Pine Script outputs multiple plots with different
 * pane indices (e.g., pane=1, pane=2), all panes should be created, visible, and
 * properly sized in the browser DOM.
 * 
 * **EXPECTED OUTCOME ON UNFIXED CODE**: Test FAILS
 * - Only one pane (typically the highest index) is visible
 * - Intermediate panes have zero or minimal height
 * - This failure confirms the bug exists
 * 
 * **EXPECTED OUTCOME AFTER FIX**: Test PASSES
 * - All unique pane indices result in visible panes
 * - Each pane has height >= 80px
 * - This confirms the bug is fixed
 */

const { chromium } = require('playwright');
const { spawn } = require('child_process');
const path = require('path');

// Test configuration
const SERVER_PORT = 7070; // Default pine-tv port
const SERVER_URL = `http://localhost:${SERVER_PORT}`;
const SERVER_STARTUP_TIMEOUT = 15000;
const TEST_TIMEOUT = 30000;

/**
 * Pine Script test cases with multiple pane indices
 */
const TEST_SCRIPTS = {
  twoPanes: {
    name: 'Two Panes (pane=1, pane=2)',
    code: `//@version=6
indicator("Two Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(rsi_val, "RSI", color=color.orange, pane=2)
`,
    expectedPanes: [1, 2]
  },
  
  threePanes: {
    name: 'Three Panes (pane=1, pane=2, pane=3)',
    code: `//@version=6
indicator("Three Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
macd_val = ta.macd(close, 12, 26, 9)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(rsi_val, "RSI", color=color.orange, pane=2)
plot(macd_val[0], "MACD", color=color.green, pane=3)
`,
    expectedPanes: [1, 2, 3]
  },
  
  nonContiguousPanes: {
    name: 'Non-Contiguous Panes (pane=1, pane=3)',
    code: `//@version=6
indicator("Non-Contiguous Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
macd_val = ta.macd(close, 12, 26, 9)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(macd_val[0], "MACD", color=color.green, pane=3)
`,
    expectedPanes: [1, 3]
  },
  
  mixedOverlayAndPanes: {
    name: 'Mixed Overlay and Panes (pane=0, pane=1, pane=2)',
    code: `//@version=6
indicator("Mixed Overlay Test", overlay=true)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
macd_val = ta.macd(close, 12, 26, 9)
plot(sma_val, "SMA Overlay", color=color.blue, pane=0)
plot(rsi_val, "RSI", color=color.orange, pane=1)
plot(macd_val[0], "MACD", color=color.green, pane=2)
`,
    expectedPanes: [1, 2] // pane=0 is overlay, not counted as indicator pane
  }
};

/**
 * Start pine-tv server
 */
async function startServer() {
  return new Promise((resolve, reject) => {
    const serverProcess = spawn('cargo', ['run', '--release', '--bin', 'pine-tv'], {
      cwd: path.join(__dirname, '../..'), // Go to workspace root
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
        // Give it a moment to fully initialize
        setTimeout(() => resolve(serverProcess), 2000);
      }
    });

    serverProcess.stderr.on('data', (data) => {
      const output = data.toString();
      console.error(`[server stderr] ${output}`);
      // Some logs might go to stderr, check for listening message there too
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
    // Access the chart instance from the global scope
    const chartContainer = document.getElementById('chart-container');
    if (!chartContainer) {
      return { error: 'Chart container not found' };
    }

    // Get all pane elements - Lightweight Charts creates divs for each pane
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
 * Run a Pine Script and wait for results
 */
async function runScript(page, code) {
  // Set the code in the editor
  await page.evaluate((scriptCode) => {
    const editor = window.editor;
    if (editor && editor.setValue) {
      editor.setValue(scriptCode);
    }
  }, code);

  // Trigger script execution
  await page.click('#run-btn');

  // Wait for results to be applied
  await page.waitForTimeout(2000);
}

/**
 * Main test suite
 */
describe('Bug Condition Exploration: Multi-Pane Display', () => {
  let browser;
  let page;
  let serverProcess;

  beforeAll(async () => {
    console.log('Starting pine-tv server...');
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
    await page.goto(SERVER_URL);
    await page.waitForSelector('#chart-container', { timeout: 5000 });
  });

  afterEach(async () => {
    if (page) {
      await page.close();
    }
  });

  /**
   * Test Case 1: Two Panes (pane=1, pane=2)
   */
  test('Two panes should both be visible with adequate height', async () => {
    const testCase = TEST_SCRIPTS.twoPanes;
    console.log(`\n=== Testing: ${testCase.name} ===`);

    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info:', JSON.stringify(paneInfo, null, 2));

    // Assertions
    expect(paneInfo.error).toBeUndefined();
    
    // Should have at least the expected number of panes (main chart + indicator panes)
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0); // Skip main chart pane
    
    console.log(`Found ${indicatorPanes.length} indicator panes, expected ${testCase.expectedPanes.length}`);
    
    // **CRITICAL ASSERTION**: All expected panes should exist
    expect(indicatorPanes.length).toBeGreaterThanOrEqual(testCase.expectedPanes.length);

    // **CRITICAL ASSERTION**: Each indicator pane should be visible with height >= 80px
    indicatorPanes.forEach((pane, idx) => {
      console.log(`Pane ${idx + 1}: height=${pane.height}px, visible=${pane.visible}`);
      expect(pane.visible).toBe(true);
      expect(pane.height).toBeGreaterThanOrEqual(80);
    });
  }, TEST_TIMEOUT);

  /**
   * Test Case 2: Three Panes (pane=1, pane=2, pane=3)
   */
  test('Three panes should all be visible with adequate height', async () => {
    const testCase = TEST_SCRIPTS.threePanes;
    console.log(`\n=== Testing: ${testCase.name} ===`);

    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info:', JSON.stringify(paneInfo, null, 2));

    expect(paneInfo.error).toBeUndefined();
    
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0);
    
    console.log(`Found ${indicatorPanes.length} indicator panes, expected ${testCase.expectedPanes.length}`);
    
    expect(indicatorPanes.length).toBeGreaterThanOrEqual(testCase.expectedPanes.length);

    indicatorPanes.forEach((pane, idx) => {
      console.log(`Pane ${idx + 1}: height=${pane.height}px, visible=${pane.visible}`);
      expect(pane.visible).toBe(true);
      expect(pane.height).toBeGreaterThanOrEqual(80);
    });
  }, TEST_TIMEOUT);

  /**
   * Test Case 3: Non-Contiguous Panes (pane=1, pane=3)
   */
  test('Non-contiguous panes should both be visible', async () => {
    const testCase = TEST_SCRIPTS.nonContiguousPanes;
    console.log(`\n=== Testing: ${testCase.name} ===`);

    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info:', JSON.stringify(paneInfo, null, 2));

    expect(paneInfo.error).toBeUndefined();
    
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0);
    
    console.log(`Found ${indicatorPanes.length} indicator panes, expected ${testCase.expectedPanes.length}`);
    
    expect(indicatorPanes.length).toBeGreaterThanOrEqual(testCase.expectedPanes.length);

    indicatorPanes.forEach((pane, idx) => {
      console.log(`Pane ${idx + 1}: height=${pane.height}px, visible=${pane.visible}`);
      expect(pane.visible).toBe(true);
      expect(pane.height).toBeGreaterThanOrEqual(80);
    });
  }, TEST_TIMEOUT);

  /**
   * Test Case 4: Mixed Overlay and Indicator Panes
   */
  test('Mixed overlay and indicator panes should display correctly', async () => {
    const testCase = TEST_SCRIPTS.mixedOverlayAndPanes;
    console.log(`\n=== Testing: ${testCase.name} ===`);

    await runScript(page, testCase.code);
    const paneInfo = await getPaneInfo(page);

    console.log('Pane Info:', JSON.stringify(paneInfo, null, 2));

    expect(paneInfo.error).toBeUndefined();
    
    const indicatorPanes = paneInfo.panes.filter((p, idx) => idx > 0);
    
    console.log(`Found ${indicatorPanes.length} indicator panes, expected ${testCase.expectedPanes.length}`);
    
    expect(indicatorPanes.length).toBeGreaterThanOrEqual(testCase.expectedPanes.length);

    indicatorPanes.forEach((pane, idx) => {
      console.log(`Pane ${idx + 1}: height=${pane.height}px, visible=${pane.visible}`);
      expect(pane.visible).toBe(true);
      expect(pane.height).toBeGreaterThanOrEqual(80);
    });
  }, TEST_TIMEOUT);
});
