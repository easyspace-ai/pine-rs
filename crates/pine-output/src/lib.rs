//! Pine Script v6 Output Layer
//!
//! This crate provides output handling for Pine Script,
//! including plots, drawings (labels, boxes, tables), and strategy signals.

#![warn(missing_docs)]

pub mod drawing;
pub mod plot;
pub mod strategy;

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Output errors
#[derive(Debug, Error, Diagnostic)]
pub enum OutputError {
    /// Placeholder error
    #[error("output not yet implemented")]
    NotImplemented,

    /// Invalid plot value
    #[error("invalid plot value: {0}")]
    InvalidValue(String),

    /// Invalid drawing id
    #[error("invalid drawing id: {0}")]
    InvalidDrawingId(String),

    /// Drawing limit exceeded
    #[error("drawing limit exceeded: max {max}")]
    DrawingLimitExceeded {
        /// Maximum allowed
        max: usize,
    },
}

/// Result type for output operations
pub type Result<T> = std::result::Result<T, OutputError>;

/// Unique identifier for drawings (labels, boxes, tables)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrawingId(pub u64);

impl DrawingId {
    /// Create a new drawing id
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Position for drawing elements
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Position {
    /// Position above the bar
    AboveBar,
    /// Position below the bar
    BelowBar,
    /// Position at top of chart
    Top,
    /// Position at bottom of chart
    Bottom,
    /// Absolute price position
    Price(f64),
}

impl Position {
    /// Parse position from string value
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "position.abovebar" => Some(Self::AboveBar),
            "position.belowbar" => Some(Self::BelowBar),
            "position.top" => Some(Self::Top),
            "position.bottom" => Some(Self::Bottom),
            _ => None,
        }
    }
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    /// Left alignment
    Left,
    /// Center alignment
    Center,
    /// Right alignment
    Right,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Center
    }
}

impl TextAlign {
    /// Parse text alignment from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text.align_left" => Some(Self::Left),
            "text.align_center" => Some(Self::Center),
            "text.align_right" => Some(Self::Right),
            _ => None,
        }
    }
}

/// Text size options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextSize {
    /// Tiny text
    Tiny,
    /// Small text
    Small,
    /// Normal text
    Normal,
    /// Large text
    Large,
    /// Huge text
    Huge,
}

impl Default for TextSize {
    fn default() -> Self {
        Self::Normal
    }
}

impl TextSize {
    /// Parse text size from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "size.tiny" => Some(Self::Tiny),
            "size.small" => Some(Self::Small),
            "size.normal" => Some(Self::Normal),
            "size.large" => Some(Self::Large),
            "size.huge" => Some(Self::Huge),
            _ => None,
        }
    }
}

/// Complete script output container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptOutput {
    /// Plots
    pub plots: Vec<PlotOutput>,
    /// Drawings
    pub drawings: DrawingOutput,
    /// Strategy signals
    pub strategy: Option<StrategyOutput>,
    /// Alert conditions
    pub alerts: Vec<AlertOutput>,
    /// Background colors per bar
    pub bgcolor: Vec<Option<pine_runtime::value::Color>>,
}

/// Individual plot output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotOutput {
    /// Plot name
    pub name: String,
    /// Plot type
    pub plot_type: PlotType,
    /// Series values
    pub values: Vec<Option<f64>>,
}

/// Type of plot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlotType {
    /// Line plot
    Line {
        /// Line color
        color: pine_runtime::value::Color,
        /// Line width
        width: u32,
        /// Line style
        style: String,
    },
    /// Shape plot
    Shape {
        /// Shape type
        shape: String,
        /// Shape color
        color: pine_runtime::value::Color,
        /// Shape size
        size: String,
        /// Location
        location: String,
    },
    /// Character plot
    Char {
        /// Character
        char: char,
        /// Character color
        color: pine_runtime::value::Color,
        /// Character size
        size: String,
        /// Location
        location: String,
    },
    /// Arrow plot
    Arrow {
        /// Up arrow color
        colorup: pine_runtime::value::Color,
        /// Down arrow color
        colordown: pine_runtime::value::Color,
        /// Offset
        offset: i32,
    },
}

/// Drawing outputs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DrawingOutput {
    /// Labels
    pub labels: Vec<LabelOutput>,
    /// Boxes
    pub boxes: Vec<BoxOutput>,
    /// Tables
    pub tables: Vec<TableOutput>,
}

/// Label output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelOutput {
    /// Label ID
    pub id: DrawingId,
    /// X position (bar index)
    pub x: i64,
    /// Y position (price)
    pub y: f64,
    /// Text content
    pub text: String,
    /// Text color
    pub textcolor: pine_runtime::value::Color,
    /// Background color
    pub color: pine_runtime::value::Color,
    /// Text size
    pub size: TextSize,
    /// Text alignment
    pub textalign: TextAlign,
    /// Style (e.g., label.style_label_up)
    pub style: String,
}

/// Box output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxOutput {
    /// Box ID
    pub id: DrawingId,
    /// Left x coordinate
    pub left: i64,
    /// Top y coordinate
    pub top: f64,
    /// Right x coordinate
    pub right: i64,
    /// Bottom y coordinate
    pub bottom: f64,
    /// Border color
    pub border_color: pine_runtime::value::Color,
    /// Background color
    pub bgcolor: pine_runtime::value::Color,
    /// Border width
    pub border_width: i32,
}

/// Table output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOutput {
    /// Table ID
    pub id: DrawingId,
    /// Position (e.g., position.top_right)
    pub position: String,
    /// Table columns
    pub columns: usize,
    /// Table rows
    pub rows: usize,
    /// Cell contents
    pub cells: Vec<TableCell>,
}

/// Table cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Column index
    pub column: usize,
    /// Row index
    pub row: usize,
    /// Cell text
    pub text: String,
    /// Text color
    pub text_color: pine_runtime::value::Color,
    /// Background color
    pub bg_color: pine_runtime::value::Color,
    /// Text size
    pub text_size: TextSize,
}

/// Strategy output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyOutput {
    /// Strategy name
    pub name: String,
    /// Entry signals
    pub entries: Vec<TradeSignal>,
    /// Exit signals
    pub exits: Vec<TradeSignal>,
    /// Position size
    pub position_size: f64,
    /// Position direction
    pub position_direction: Direction,
}

/// Trade signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    /// Bar index
    pub bar_index: i64,
    /// Signal direction
    pub direction: Direction,
    /// Quantity
    pub qty: f64,
    /// Price (optional)
    pub price: Option<f64>,
    /// Signal comment
    pub comment: Option<String>,
}

/// Trade direction
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    /// Long position
    Long,
    /// Short position
    Short,
    /// Close position
    Close,
    /// No direction
    None,
}

impl Default for Direction {
    fn default() -> Self {
        Self::None
    }
}

/// Alert output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertOutput {
    /// Alert name
    pub name: String,
    /// Alert message
    pub message: String,
    /// Alert frequency
    pub freq: AlertFreq,
}

/// Alert frequency
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlertFreq {
    /// Once per bar
    OncePerBar,
    /// Once per bar close
    OncePerBarClose,
    /// All occurrences
    All,
}

impl Default for AlertFreq {
    fn default() -> Self {
        Self::OncePerBar
    }
}

/// Collect and format script output
pub fn collect() -> Result<()> {
    // TODO: Implement output collection
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawing_id() {
        let id = DrawingId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_position_parsing() {
        assert_eq!(Position::from_str("position.abovebar"), Some(Position::AboveBar));
        assert_eq!(Position::from_str("position.belowbar"), Some(Position::BelowBar));
        assert_eq!(Position::from_str("position.top"), Some(Position::Top));
        assert_eq!(Position::from_str("position.bottom"), Some(Position::Bottom));
        assert_eq!(Position::from_str("invalid"), None);
    }

    #[test]
    fn test_text_align_parsing() {
        assert_eq!(TextAlign::from_str("text.align_left"), Some(TextAlign::Left));
        assert_eq!(TextAlign::from_str("text.align_center"), Some(TextAlign::Center));
        assert_eq!(TextAlign::from_str("text.align_right"), Some(TextAlign::Right));
        assert_eq!(TextAlign::from_str("invalid"), None);
    }

    #[test]
    fn test_text_size_parsing() {
        assert_eq!(TextSize::from_str("size.tiny"), Some(TextSize::Tiny));
        assert_eq!(TextSize::from_str("size.normal"), Some(TextSize::Normal));
        assert_eq!(TextSize::from_str("size.huge"), Some(TextSize::Huge));
    }

    #[test]
    fn test_script_output_default() {
        let output = ScriptOutput::default();
        assert!(output.plots.is_empty());
        assert!(output.drawings.labels.is_empty());
        assert!(output.drawings.boxes.is_empty());
        assert!(output.drawings.tables.is_empty());
        assert!(output.strategy.is_none());
        assert!(output.alerts.is_empty());
    }
}
