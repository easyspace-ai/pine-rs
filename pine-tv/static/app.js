// pine-tv main app module — three-column layout: chart | editor | run output

import './chart.js';
import './editor.js';

const state = {
    symbol: 'BTCUSDT',
    timeframe: '1h',
    code: '',
    isRunning: false,
    lastResult: null,
    lastErrors: [],
    chartW: 0,
    editorW: 0,
    examples: null,
};

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
    side: $('side-panel'),
};

const resizer1 = $('resizer-1');
const resizer2 = $('resizer-2');
const statusEl = $('editor-status');
const sideContentEl = $('side-content');

export function init() {
    initLayoutSizes();
    initResizers();
    initToolbar();
    loadExampleList();
    initEventListeners();
    window.addEventListener('resize', () => {
        const main = $('main-container');
        if (main) {
            applyLayoutSizes(main.getBoundingClientRect().width, 8);
        }
    });
    console.log('[app] pine-tv initialized (3-column)');
}

function initLayoutSizes() {
    const main = $('main-container');
    if (!main || !panels.chart || !panels.editor || !panels.side) return;

    const total = main.getBoundingClientRect().width;
    const gap = 8;
    if (!state.chartW || !state.editorW) {
        state.chartW = Math.round(total * 0.38);
        state.editorW = Math.round(total * 0.34);
    }
    applyLayoutSizes(total, gap);
}

function applyLayoutSizes(total, gap) {
    const minChart = Math.max(180, total * 0.16);
    const minEditor = 260;
    const minSide = 160;

    let cw = state.chartW;
    let ew = state.editorW;
    let sw = total - cw - ew - gap;

    if (sw < minSide) {
        const deficit = minSide - sw;
        ew = Math.max(minEditor, ew - deficit);
        sw = total - cw - ew - gap;
    }
    if (sw < minSide) {
        cw = Math.max(minChart, cw - (minSide - sw));
        sw = total - cw - ew - gap;
    }

    cw = Math.max(minChart, Math.min(cw, total - minEditor - minSide - gap));
    ew = Math.max(minEditor, Math.min(ew, total - cw - minSide - gap));
    sw = Math.max(minSide, total - cw - ew - gap);

    state.chartW = cw;
    state.editorW = ew;

    panels.chart.style.width = `${cw}px`;
    panels.editor.style.width = `${ew}px`;
    panels.side.style.width = `${sw}px`;

    window.dispatchEvent(new CustomEvent('pine:resize'));
}

function initResizers() {
    const main = $('main-container');
    if (!main || !resizer1 || !resizer2) return;

    let active = null;

    const onMove = (e) => {
        if (!active) return;
        const rect = main.getBoundingClientRect();
        const total = rect.width;
        const gap = 8;
        const minChart = Math.max(180, total * 0.16);
        const minEditor = 260;
        const minSide = 160;
        const x = e.clientX - rect.left;

        if (active === 'chart') {
            state.chartW = Math.max(minChart, Math.min(x, total - minEditor - minSide - gap));
        } else if (active === 'editor') {
            state.editorW = x - state.chartW - 4;
        }
        applyLayoutSizes(total, gap);
    };

    const onUp = () => {
        active = null;
        document.removeEventListener('mousemove', onMove);
        document.removeEventListener('mouseup', onUp);
    };

    resizer1.addEventListener('mousedown', (e) => {
        active = 'chart';
        document.addEventListener('mousemove', onMove);
        document.addEventListener('mouseup', onUp);
        e.preventDefault();
    });

    resizer2.addEventListener('mousedown', (e) => {
        active = 'editor';
        document.addEventListener('mousemove', onMove);
        document.addEventListener('mouseup', onUp);
        e.preventDefault();
    });
}

function initToolbar() {
    toolbar.symbolSelect.value = state.symbol;
    toolbar.tfSelect.value = state.timeframe;
}

async function loadExampleList() {
    try {
        const response = await fetch('/api/examples');
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        const categories = await response.json();
        state.examples = categories;
        populateExampleSelect(categories);
    } catch (e) {
        console.error('[app] Failed to load examples:', e);
        toolbar.exampleSelect.innerHTML = '<option value="">Load Error</option>';
    }
}

function populateExampleSelect(categories) {
    const select = toolbar.exampleSelect;
    select.innerHTML = '<option value="">Examples...</option>';

    for (const category of categories) {
        const optgroup = document.createElement('optgroup');
        optgroup.label = `${category.name} - ${category.description}`;

        for (const example of category.examples) {
            const option = document.createElement('option');
            option.value = example.id;
            option.textContent = example.name;
            optgroup.appendChild(option);
        }

        select.appendChild(optgroup);
    }
}

function initEventListeners() {
    toolbar.symbolSelect.addEventListener('change', () => {
        state.symbol = toolbar.symbolSelect.value;
        window.dispatchEvent(new CustomEvent('pine:symbol-change', {
            detail: { symbol: state.symbol, tf: state.timeframe },
        }));
    });

    toolbar.tfSelect.addEventListener('change', () => {
        state.timeframe = toolbar.tfSelect.value;
        window.dispatchEvent(new CustomEvent('pine:symbol-change', {
            detail: { symbol: state.symbol, tf: state.timeframe },
        }));
    });

    toolbar.exampleSelect.addEventListener('change', async () => {
        const exampleId = toolbar.exampleSelect.value;
        if (exampleId) {
            await loadExample(exampleId);
            toolbar.exampleSelect.value = '';
        }
    });

    toolbar.runBtn.addEventListener('click', runScript);

    window.addEventListener('pine:code-change', (e) => {
        state.code = e.detail.code;
    });

    window.addEventListener('pine:result', (e) => {
        if (sideContentEl) {
            try {
                sideContentEl.textContent = JSON.stringify(e.detail, null, 2);
            } catch {
                sideContentEl.textContent = String(e.detail);
            }
        }
    });
}

async function loadExample(exampleId) {
    try {
        setStatus('Loading example...');
        const response = await fetch(`/api/examples/${exampleId}`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        const example = await response.json();

        window.dispatchEvent(new CustomEvent('pine:editor-set-code', {
            detail: { code: example.code },
        }));
        setStatus(`Loaded: ${example.name}`);
    } catch (e) {
        console.error('[app] Failed to load example:', e);
        setStatus(`Error loading example: ${e.message}`);
    }
}

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
            if (sideContentEl) {
                sideContentEl.textContent = JSON.stringify(result, null, 2);
            }
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

export function setStatus(text) {
    if (statusEl) statusEl.textContent = text;
}

export function getState() {
    return state;
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
