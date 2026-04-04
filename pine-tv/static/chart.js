// pine-tv chart module
// Wrapper for lightweight-charts v5+ with multi-pane support (overlay vs indicator pane)

const LIGHTWEIGHT_CHARTS_VERSION = '5.0.3';

let chart = null;
let priceSeries = null;
let plotSeries = new Map();
let chartContainer = null;
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
        console.log('[chart] initialized with real-time support (LWC v5 panes)');
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

function getLwc() {
    return window.LightweightCharts;
}

// Create chart
function initChart() {
    chartContainer = document.getElementById('chart-container');
    if (!chartContainer) return;

    const LWC = getLwc();
    chart = LWC.createChart(chartContainer, {
        layout: {
            background: { type: 'solid', color: '#131722' },
            textColor: '#d1d4dc',
            panes: {
                separatorColor: '#363A4E',
                separatorHoverColor: '#485c7b',
                enableResize: true,
            },
        },
        grid: {
            vertLines: { color: '#2B2B43' },
            horzLines: { color: '#363A4E' },
        },
        crosshair: {
            mode: LWC.CrosshairMode.Normal,
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

    priceSeries = chart.addSeries(LWC.CandlestickSeries, {
        upColor: '#26a69a',
        downColor: '#ef5350',
        borderDownColor: '#ef5350',
        borderUpColor: '#26a69a',
        wickDownColor: '#ef5350',
        wickUpColor: '#26a69a',
    });

    window.addEventListener('pine:resize', () => {
        if (chart) {
            chart.applyOptions({ width: chartContainer.clientWidth });
        }
    });

    chart.applyOptions({ width: chartContainer.clientWidth });
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
        setTimeout(() => {
            console.log('[chart] Reconnecting...');
            initWebSocket();
        }, 5000);
    };

    ws.onerror = (error) => {
        console.error('[chart] WebSocket error:', error);
    };
}

function updateRealtimeStatus() {}

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

function handleSnapshot(msg) {
    renderCandlesticks(msg.bars);
    if (currentCode) {
        runScriptOnRealtimeData();
    }
}

function handleFormingUpdate(msg) {
    updateLastBar(msg.bar);
    if (currentCode) {
        runScriptOnRealtimeData();
    }
}

function handleNewBar(msg) {
    addNewBar(msg.new_bar);
    if (currentCode) {
        runScriptOnRealtimeData();
    }
}

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

    if (chart && chart.timeScale()) {
        chart.timeScale().fitContent();
    }
}

function updateLastBar(bar) {
    if (!priceSeries) return;

    priceSeries.update({
        time: bar.time,
        open: bar.open,
        high: bar.high,
        low: bar.low,
        close: bar.close,
    });
}

function addNewBar(bar) {
    if (!priceSeries) return;

    priceSeries.update({
        time: bar.time,
        open: bar.open,
        high: bar.high,
        low: bar.low,
        close: bar.close,
    });
}

function runScriptOnRealtimeData() {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    ws.send(JSON.stringify({
        action: 'run',
        code: currentCode,
    }));
}

function initEventListeners() {
    window.addEventListener('pine:result', (e) => {
        applyResult(e.detail);
    });

    window.addEventListener('pine:code-change', (e) => {
        currentCode = e.detail.code;
        if (isRealtime && currentCode) {
            runScriptOnRealtimeData();
        }
    });

    window.addEventListener('pine:symbol-change', (e) => {
        currentSymbol = e.detail.symbol;
        currentTf = e.detail.tf;

        for (const [, series] of plotSeries) {
            chart.removeSeries(series);
        }
        plotSeries.clear();

        if (!isRealtime) {
            loadDataAndRender();
        }
    });
}

function applyResult(result) {
    if (!result.ok) return;

    const plots = result.plots || [];
    const LWC = getLwc();
    if (!chart || !LWC) return;

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
        
        // For single pane: maintain existing 28% height logic (backward compatibility)
        // For multiple panes: use ~40% of container height total, divide by number of panes
        const heightPerPane = indicatorPanes.length === 1
            ? Math.max(100, Math.floor(chartContainer.clientHeight * 0.28))
            : Math.max(80, Math.floor(chartContainer.clientHeight * 0.4 / indicatorPanes.length));
        
        for (const paneIdx of indicatorPanes) {
            // Defensive check: verify chart.panes().length > paneIdx before calling setHeight
            if (chart.panes().length > paneIdx) {
                const pane = chart.panes()[paneIdx];
                pane.setHeight(heightPerPane);
            }
        }
    }
}

function addLineSeriesToPane(plot, paneIndex, LWC) {
    const options = {
        color: plot.color || '#2196F3',
        lineWidth: plot.linewidth || 1,
        title: plot.title,
    };
    const series = paneIndex === 0
        ? chart.addSeries(LWC.LineSeries, options)
        : chart.addSeries(LWC.LineSeries, options, paneIndex);

    const data = plot.data.map(d => ({
        time: d.time,
        value: d.value,
    })).filter(d => d.value !== null && d.value !== undefined);

    series.setData(data);
    plotSeries.set(plot.id, series);
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
