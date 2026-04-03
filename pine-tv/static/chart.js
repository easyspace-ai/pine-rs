// pine-tv chart module
// Wrapper for lightweight-charts with real-time WebSocket support

// Load lightweight-charts from CDN
const LIGHTWEIGHT_CHARTS_VERSION = '4.1.0';

let chart = null;
let priceSeries = null;
let plotSeries = new Map();
let currentSymbol = 'BTCUSDT';
let currentTf = '1h';
let ws = null;
let currentCode = '';
let isRealtime = false;

// Initialize
export function init() {
    loadLightweightCharts().then(() => {
        initChart();
        initEventListeners();
        initWebSocket();
        console.log('[chart] initialized with real-time support');
    });
}

// Load lightweight-charts library
async function loadLightweightCharts() {
    if (window.LightweightCharts) {
        return;
    }

    return new Promise((resolve, reject) => {
        const script = document.createElement('script');
        script.src = `https://unpkg.com/lightweight-charts@${LIGHTWEIGHT_CHARTS_VERSION}/dist/lightweight-charts.standalone.production.js`;
        script.onload = resolve;
        script.onerror = reject;
        document.head.appendChild(script);
    });
}

// Create chart
function initChart() {
    const container = document.getElementById('chart-container');
    if (!container) return;

    chart = window.LightweightCharts.createChart(container, {
        layout: {
            background: { type: 'solid', color: '#131722' },
            textColor: '#d1d4dc',
        },
        grid: {
            vertLines: { color: '#2B2B43' },
            horzLines: { color: '#363A4E' },
        },
        crosshair: {
            mode: window.LightweightCharts.CrosshairMode.Normal,
        },
        rightPriceScale: {
            borderColor: '#485c7b',
        },
        timeScale: {
            borderColor: '#485c7b',
            timeVisible: true,
            secondsVisible: false,
        },
    });

    // Add candlestick series for price
    priceSeries = chart.addCandlestickSeries({
        upColor: '#26a69a',
        downColor: '#ef5350',
        borderDownColor: '#ef5350',
        borderUpColor: '#26a69a',
        wickDownColor: '#ef5350',
        wickUpColor: '#26a69a',
    });

    // Handle resize
    window.addEventListener('pine:resize', () => {
        if (chart) {
            chart.applyOptions({ width: container.clientWidth });
        }
    });

    // Initial size
    chart.applyOptions({ width: container.clientWidth });
}

// Initialize WebSocket connection
function initWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/ws`;

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        console.log('[chart] WebSocket connected');
        isRealtime = true;
        updateRealtimeStatus();
    };

    ws.onmessage = (event) => {
        try {
            const msg = JSON.parse(event.data);
            handleWebSocketMessage(msg);
        } catch (e) {
            console.error('[chart] Failed to parse WebSocket message:', e);
        }
    };

    ws.onclose = () => {
        console.log('[chart] WebSocket disconnected');
        isRealtime = false;
        updateRealtimeStatus();
        // Try to reconnect after 5 seconds
        setTimeout(() => {
            console.log('[chart] Reconnecting...');
            initWebSocket();
        }, 5000);
    };

    ws.onerror = (error) => {
        console.error('[chart] WebSocket error:', error);
    };
}

// Update real-time status indicator
function updateRealtimeStatus() {
    // Could add a visual indicator in the UI
}

// Handle WebSocket messages
function handleWebSocketMessage(msg) {
    switch (msg.type) {
        case 'snapshot':
            handleSnapshot(msg);
            break;
        case 'forming_update':
            handleFormingUpdate(msg);
            break;
        case 'new_bar':
            handleNewBar(msg);
            break;
        case 'result':
            applyResult(msg.result);
            break;
        case 'error':
            console.error('[chart] WebSocket error:', msg.errors);
            break;
    }
}

// Handle full data snapshot
function handleSnapshot(msg) {
    renderCandlesticks(msg.bars);
    // If we have code, run it on the snapshot
    if (currentCode) {
        runScriptOnRealtimeData();
    }
}

// Handle forming bar update
function handleFormingUpdate(msg) {
    updateLastBar(msg.bar);
    // Recalculate indicators on each update (TradingView-like behavior)
    if (currentCode) {
        runScriptOnRealtimeData();
    }
}

// Handle new bar
function handleNewBar(msg) {
    addNewBar(msg.new_bar);
    // Recalculate indicators
    if (currentCode) {
        runScriptOnRealtimeData();
    }
}

// Load data from API (fallback if WebSocket not available)
async function loadDataAndRender() {
    try {
        const response = await fetch(`/api/data/${currentSymbol}/${currentTf}`);
        const result = await response.json();

        if (result.ok && result.data) {
            renderCandlesticks(result.data);
        }
    } catch (e) {
        console.error('[chart] Failed to load data:', e);
    }
}

// Render candlesticks from bars data
function renderCandlesticks(bars) {
    if (!priceSeries || !bars) return;

    const candleData = bars.map(bar => ({
        time: bar.time,
        open: bar.open,
        high: bar.high,
        low: bar.low,
        close: bar.close,
    }));

    priceSeries.setData(candleData);

    // Fit content
    if (chart && chart.timeScale()) {
        chart.timeScale().fitContent();
    }
}

// Update the last bar (forming bar)
function updateLastBar(bar) {
    if (!priceSeries) return;

    // Get current data and update the last one
    const lastData = {
        time: bar.time,
        open: bar.open,
        high: bar.high,
        low: bar.low,
        close: bar.close,
    };

    priceSeries.update(lastData);
}

// Add a new bar
function addNewBar(bar) {
    if (!priceSeries) return;

    const newData = {
        time: bar.time,
        open: bar.open,
        high: bar.high,
        low: bar.low,
        close: bar.close,
    };

    priceSeries.update(newData);
}

// Run script on real-time data via WebSocket
function runScriptOnRealtimeData() {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const message = {
        action: 'run',
        code: currentCode,
    };

    ws.send(JSON.stringify(message));
}

// Event listeners
function initEventListeners() {
    // Listen for execution results from REST API
    window.addEventListener('pine:result', (e) => {
        applyResult(e.detail);
    });

    // Listen for code changes
    window.addEventListener('pine:code-change', (e) => {
        currentCode = e.detail.code;
        // If we're in realtime mode, run immediately
        if (isRealtime && currentCode) {
            runScriptOnRealtimeData();
        }
    });

    // Listen for symbol changes
    window.addEventListener('pine:symbol-change', (e) => {
        currentSymbol = e.detail.symbol;
        currentTf = e.detail.tf;

        // Clear existing plot series
        for (const [id, series] of plotSeries) {
            chart.removeSeries(series);
        }
        plotSeries.clear();

        // Note: full symbol switching would require reconnecting WebSocket
        // For now, just load via REST as fallback
        if (!isRealtime) {
            loadDataAndRender();
        }
    });
}

// Apply execution result to chart
function applyResult(result) {
    if (!result.ok) return;

    const plots = result.plots || [];

    // Clear existing plot series
    for (const [id, series] of plotSeries) {
        chart.removeSeries(series);
    }
    plotSeries.clear();

    // Add each plot
    for (const plot of plots) {
        if (plot.pane === 0) {
            // Overlay on price chart
            addLineSeries(plot);
        }
        // TODO: Handle sub-panes
    }
}

// Add a line series
function addLineSeries(plot) {
    const series = chart.addLineSeries({
        color: plot.color || '#2196F3',
        lineWidth: plot.linewidth || 1,
        title: plot.title,
    });

    const data = plot.data.map(d => ({
        time: d.time,
        value: d.value,
    })).filter(d => d.value !== null && d.value !== undefined);

    series.setData(data);
    plotSeries.set(plot.id, series);
}

// Initialize on load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
