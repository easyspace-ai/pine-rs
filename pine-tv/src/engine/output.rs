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
            errors: None,
        }
    }

    /// Create an error response
    pub fn error(errors: Vec<ApiError>) -> Self {
        Self {
            ok: false,
            exec_ms: None,
            plots: None,
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
