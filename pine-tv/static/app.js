// pine-tv main app module
// Handles layout, panel resizing, and cross-panel communication

import './chart.js';
import './editor.js';

// State
const state = {
    symbol: 'BTCUSDT',
    timeframe: '1h',
    code: '',
    isRunning: false,
    lastResult: null,
    lastErrors: [],
};

// DOM elements
const $ = (id) => document.getElementById(id);

const toolbar = {
    symbolSelect: $('symbol-select'),
    tfSelect: $('tf-select'),
    exampleSelect: $('example-select'),
    runBtn: $('run-btn'),
};

const panels = {
    chart: $('chart-panel'),
    editor: $('editor-panel'),
};

const resizer = $('resizer-1');
const statusEl = $('editor-status');

// Initialize
export function init() {
    initResizer();
    initToolbar();
    initEventListeners();
    console.log('[app] pine-tv initialized');
}

// Resizable panels
function initResizer() {
    let isResizing = false;

    const startResize = (e) => {
        isResizing = true;
        document.addEventListener('mousemove', onResize);
        document.addEventListener('mouseup', stopResize);
        e.preventDefault();
    };

    const onResize = (e) => {
        if (!isResizing) return;

        const containerRect = $('main-container').getBoundingClientRect();
        const totalWidth = containerRect.width;
        const x = e.clientX - containerRect.left;

        const minWidth = totalWidth * 0.2;
        const maxWidth = totalWidth * 0.8;
        const chartWidth = Math.max(minWidth, Math.min(maxWidth, x));
        const editorWidth = totalWidth - chartWidth - 4; // 4px for resizer

        panels.chart.style.width = `${chartWidth}px`;
        panels.editor.style.width = `${editorWidth}px`;

        // Notify panels of resize
        window.dispatchEvent(new CustomEvent('pine:resize'));
    };

    const stopResize = () => {
        isResizing = false;
        document.removeEventListener('mousemove', onResize);
        document.removeEventListener('mouseup', stopResize);
    };

    resizer.addEventListener('mousedown', startResize);
}

// Toolbar
function initToolbar() {
    toolbar.symbolSelect.value = state.symbol;
    toolbar.tfSelect.value = state.timeframe;
}

// Event listeners
function initEventListeners() {
    // Symbol change
    toolbar.symbolSelect.addEventListener('change', () => {
        state.symbol = toolbar.symbolSelect.value;
        window.dispatchEvent(new CustomEvent('pine:symbol-change', {
            detail: { symbol: state.symbol, tf: state.timeframe }
        }));
    });

    // Timeframe change
    toolbar.tfSelect.addEventListener('change', () => {
        state.timeframe = toolbar.tfSelect.value;
        window.dispatchEvent(new CustomEvent('pine:symbol-change', {
            detail: { symbol: state.symbol, tf: state.timeframe }
        }));
    });

    // Example selection
    toolbar.exampleSelect.addEventListener('change', async () => {
        const example = toolbar.exampleSelect.value;
        if (example) {
            await loadExample(example);
            toolbar.exampleSelect.value = '';
        }
    });

    // Run button
    toolbar.runBtn.addEventListener('click', runScript);

    // Listen for editor code changes
    window.addEventListener('pine:code-change', (e) => {
        state.code = e.detail.code;
    });
}

// Load example script
async function loadExample(name) {
    const examples = {
        sma: `//@version=6
indicator("SMA Indicator", overlay=true)
len = input.int(20, title="Length")
src = input(close, title="Source")
sma_val = ta.sma(src, len)
plot(sma_val, title="SMA", color=#2196F3, linewidth=2)`,
        rsi: `//@version=6
indicator("RSI Indicator")
len = input.int(14, title="Length")
src = input(close, title="Source")
rsi_val = ta.rsi(src, len)
plot(rsi_val, title="RSI", color=#FF9800, linewidth=2)
hline(70, "Overbought", color=#F44336)
hline(30, "Oversold", color=#4CAF50)`,
        ema_triple: `//@version=6
indicator("EMA Triple - 30/60/120", overlay=true)

// Calculate EMAs
ema30 = ta.ema(close, 30)
ema60 = ta.ema(close, 60)
ema120 = ta.ema(close, 120)

// Plot EMAs
plot(ema30, title="EMA 30", color=#FF9800, linewidth=2)
plot(ema60, title="EMA 60", color=#E91E63, linewidth=2)
plot(ema120, title="EMA 120", color=#9C27B0, linewidth=2)`,
    };

    const code = examples[name];
    if (code) {
        window.dispatchEvent(new CustomEvent('pine:editor-set-code', {
            detail: { code }
        }));
    }
}

// Run script
async function runScript() {
    if (state.isRunning) return;

    state.isRunning = true;
    toolbar.runBtn.disabled = true;
    toolbar.runBtn.innerHTML = '<span class="spinner"></span>Running...';
    setStatus('Running...');

    try {
        const response = await fetch('/api/run', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                code: state.code,
                symbol: state.symbol,
                timeframe: state.timeframe,
                bars: 500,
            }),
        });

        const result = await response.json();
        state.lastResult = result;

        if (result.ok) {
            setStatus(`Executed in ${result.exec_ms}ms`);
            state.lastErrors = [];
            window.dispatchEvent(new CustomEvent('pine:result', { detail: result }));
        } else {
            const errors = result.errors || [];
            state.lastErrors = errors;
            const msg = errors.length > 0 ? errors[0].msg : 'Unknown error';
            setStatus(`Error: ${msg}`);
            window.dispatchEvent(new CustomEvent('pine:error', { detail: { errors } }));
        }
    } catch (e) {
        console.error('[app] Run failed:', e);
        setStatus(`Error: ${e.message}`);
    } finally {
        state.isRunning = false;
        toolbar.runBtn.disabled = false;
        toolbar.runBtn.textContent = 'Run';
    }
}

// Set status
export function setStatus(text) {
    statusEl.textContent = text;
}

// Expose state for other modules
export function getState() {
    return state;
}

// Initialize on load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
