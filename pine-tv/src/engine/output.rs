//! Output serialization for pine-tv API
//! Converts pine-rs execution results to JSON format for frontend consumption.

use serde::{Deserialize, Serialize};

/// API response for /api/run and /api/check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Whether the request was successful
    pub ok: bool,
    /// Execution time in milliseconds (only for /api/run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exec_ms: Option<u64>,
    /// Plot data (only for /api/run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plots: Option<Vec<Plot>>,
    /// Strategy signals (only for /api/run when using strategy())
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<StrategyOutput>,
    /// Error information (only when ok = false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ApiError>>,
}

impl ApiResponse {
    /// Create a successful response with plots
    pub fn success(exec_ms: u64, plots: Vec<Plot>) -> Self {
        Self {
            ok: true,
            exec_ms: Some(exec_ms),
            plots: Some(plots),
            strategy: None,
            errors: None,
        }
    }

    /// Create a successful response with plots and strategy signals
    pub fn success_with_strategy(exec_ms: u64, plots: Vec<Plot>, strategy: StrategyOutput) -> Self {
        Self {
            ok: true,
            exec_ms: Some(exec_ms),
            plots: Some(plots),
            strategy: Some(strategy),
            errors: None,
        }
    }

    /// Create an error response
    pub fn error(errors: Vec<ApiError>) -> Self {
        Self {
            ok: false,
            exec_ms: None,
            plots: None,
            strategy: None,
            errors: Some(errors),
        }
    }
}

/// A single plot series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plot {
    /// Unique identifier for this plot
    pub id: String,
    /// Display title
    pub title: String,
    /// Plot type: "line", "histogram", "cross", "area", "columns"
    #[serde(rename = "type")]
    pub plot_type: String,
    /// Color as hex string: "#RRGGBB" or "#RRGGBBAA"
    pub color: String,
    /// Line width (for line plots)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linewidth: Option<f64>,
    /// Pane index: 0 = overlay on price, 1+ = separate pane
    pub pane: i32,
    /// Data points
    pub data: Vec<PlotData>,
}

/// A single data point in a plot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotData {
    /// Unix timestamp in seconds
    pub time: i64,
    /// Value (null = na)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

/// API error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Line number (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Column number (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<usize>,
    /// End column (1-based, inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<usize>,
    /// Error message
    pub msg: String,
}

impl ApiError {
    /// Create a simple error message without location
    pub fn simple(msg: String) -> Self {
        Self {
            line: None,
            col: None,
            end_col: None,
            msg,
        }
    }
}

/// Request body for /api/run
#[derive(Debug, Clone, Deserialize)]
pub struct RunRequest {
    /// Pine Script code
    pub code: String,
    /// Symbol (e.g., "BTCUSDT")
    #[serde(default = "default_symbol")]
    pub symbol: String,
    /// Timeframe (e.g., "1h", "1d")
    #[serde(default = "default_timeframe")]
    pub timeframe: String,
    /// Number of bars to run
    #[serde(default = "default_bars")]
    pub bars: usize,
}

fn default_symbol() -> String {
    "BTCUSDT".to_string()
}

fn default_timeframe() -> String {
    "1h".to_string()
}

fn default_bars() -> usize {
    500
}

/// Request body for /api/check
#[derive(Debug, Clone, Deserialize)]
pub struct CheckRequest {
    /// Pine Script code
    pub code: String,
}

/// Strategy output for trading signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyOutput {
    /// Strategy name/title
    pub name: String,
    /// Entry signals (buy/long)
    pub entries: Vec<TradeSignal>,
    /// Exit signals (sell/close)
    pub exits: Vec<TradeSignal>,
    /// Closed trade list with PnL details
    pub trades: Vec<StrategyTrade>,
    /// Backtest summary report
    pub report: StrategyReport,
    /// Current position size (positive = long, negative = short)
    pub position_size: f64,
    /// Current position direction
    pub position_direction: String,
}

/// Equity curve sample captured after a trade closes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEquityPoint {
    /// Trade close time in unix seconds
    pub time: i64,
    /// Equity after applying the trade PnL
    pub equity: f64,
    /// Drawdown from previous closed-equity peak
    pub drawdown: f64,
}

/// Report slice for one side of the strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySideReport {
    /// Number of closed trades for this side
    pub closed_trades: usize,
    /// Number of winning trades
    pub winning_trades: usize,
    /// Net profit for this side
    pub net_profit: f64,
    /// Win rate percentage
    pub win_rate: f64,
}

/// Closed trade detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTrade {
    /// Entry signal id
    pub entry_id: String,
    /// Exit signal id
    pub exit_id: String,
    /// Direction: "long" or "short"
    pub direction: String,
    /// Entry bar index
    pub entry_bar_index: usize,
    /// Exit bar index
    pub exit_bar_index: usize,
    /// Entry time in unix seconds
    pub entry_time: i64,
    /// Exit time in unix seconds
    pub exit_time: i64,
    /// Entry fill price
    pub entry_price: f64,
    /// Exit fill price
    pub exit_price: f64,
    /// Filled quantity
    pub qty: f64,
    /// Profit before trading costs
    pub gross_pnl: f64,
    /// Commission charged to this trade
    pub commission: f64,
    /// Slippage cost charged to this trade
    pub slippage_cost: f64,
    /// Profit or loss in quote currency
    pub pnl: f64,
    /// Profit or loss in percent
    pub pnl_percent: f64,
    /// Number of bars between entry and exit
    pub bars_held: usize,
    /// Optional entry comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_comment: Option<String>,
    /// Optional exit comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_comment: Option<String>,
}

/// Strategy backtest summary report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyReport {
    /// Initial capital declared by the strategy
    pub initial_capital: f64,
    /// Current equity after closed trades
    pub equity: f64,
    /// Net profit after closed trades
    pub net_profit: f64,
    /// Net profit percentage on initial capital
    pub net_profit_percent: f64,
    /// Sum of winning trades
    pub gross_profit: f64,
    /// Sum of losing trades (negative)
    pub gross_loss: f64,
    /// Total commissions charged to closed trades
    pub total_commission: f64,
    /// Total slippage cost charged to closed trades
    pub total_slippage_cost: f64,
    /// Number of closed trades
    pub total_closed_trades: usize,
    /// Number of winning trades
    pub winning_trades: usize,
    /// Number of losing trades
    pub losing_trades: usize,
    /// Win rate percentage
    pub win_rate: f64,
    /// Profit factor if gross loss exists
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor: Option<f64>,
    /// Average profit per closed trade
    pub avg_trade: f64,
    /// Average profit percent per closed trade
    pub avg_trade_percent: f64,
    /// Best closed trade
    pub largest_win: f64,
    /// Worst closed trade
    pub largest_loss: f64,
    /// Maximum closed-equity drawdown
    pub max_drawdown: f64,
    /// Maximum closed-equity drawdown percent
    pub max_drawdown_percent: f64,
    /// Average bars held across closed trades
    pub avg_bars_held: f64,
    /// Number of still-open trade lots
    pub open_trades: usize,
    /// Long-only report slice
    pub long: StrategySideReport,
    /// Short-only report slice
    pub short: StrategySideReport,
    /// Closed-equity curve
    pub equity_curve: Vec<StrategyEquityPoint>,
}

/// Individual trade signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    /// Bar index where signal occurred
    pub bar_index: usize,
    /// Unix timestamp in seconds
    pub time: i64,
    /// Signal type: "entry" or "exit"
    pub signal_type: String,
    /// Signal ID (e.g., "Long", "Short")
    pub id: String,
    /// Direction: "long", "short", or "close"
    pub direction: String,
    /// Quantity (contracts/shares)
    pub qty: f64,
    /// Price (optional, None for market orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// Comment (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}
