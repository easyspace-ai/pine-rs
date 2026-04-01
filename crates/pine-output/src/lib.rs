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

impl std::str::FromStr for Position {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "position.abovebar" => Ok(Self::AboveBar),
            "position.belowbar" => Ok(Self::BelowBar),
            "position.top" => Ok(Self::Top),
            "position.bottom" => Ok(Self::Bottom),
            _ => Err(()),
        }
    }
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum TextAlign {
    /// Left alignment
    Left,
    /// Center alignment
    #[default]
    Center,
    /// Right alignment
    Right,
}

impl std::str::FromStr for TextAlign {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "text.align_left" => Ok(Self::Left),
            "text.align_center" => Ok(Self::Center),
            "text.align_right" => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

/// Text size options
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum TextSize {
    /// Tiny text
    Tiny,
    /// Small text
    Small,
    /// Normal text
    #[default]
    Normal,
    /// Large text
    Large,
    /// Huge text
    Huge,
}

impl std::str::FromStr for TextSize {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "size.tiny" => Ok(Self::Tiny),
            "size.small" => Ok(Self::Small),
            "size.normal" => Ok(Self::Normal),
            "size.large" => Ok(Self::Large),
            "size.huge" => Ok(Self::Huge),
            _ => Err(()),
        }
    }
}

/// Horizontal line style
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum HLineStyle {
    /// Solid line
    #[default]
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
}

impl std::str::FromStr for HLineStyle {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "hline.style_solid" => Ok(Self::Solid),
            "hline.style_dashed" => Ok(Self::Dashed),
            "hline.style_dotted" => Ok(Self::Dotted),
            _ => Err(()),
        }
    }
}

/// Horizontal line output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HLineOutput {
    /// Line price level
    pub price: f64,
    /// Line color
    pub color: pine_runtime::value::Color,
    /// Line style
    pub style: HLineStyle,
    /// Line width
    pub width: u32,
    /// Title/legend
    pub title: Option<String>,
}

/// Fill between two plots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillOutput {
    /// First plot index
    pub plot1_index: usize,
    /// Second plot index
    pub plot2_index: usize,
    /// Fill color
    pub color: pine_runtime::value::Color,
    /// Transparency (0-100)
    pub transp: u8,
    /// Title
    pub title: Option<String>,
}

/// Complete script output container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptOutput {
    /// Plots
    pub plots: Vec<PlotOutput>,
    /// Horizontal lines
    pub hlines: Vec<HLineOutput>,
    /// Fill areas
    pub fills: Vec<FillOutput>,
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
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Direction {
    /// Long position
    Long,
    /// Short position
    Short,
    /// Close position
    Close,
    /// No direction
    #[default]
    None,
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
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum AlertFreq {
    /// Once per bar
    #[default]
    OncePerBar,
    /// Once per bar close
    OncePerBarClose,
    /// All occurrences
    All,
}

/// Alert condition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// Condition name
    pub name: String,
    /// Condition expression result (per bar)
    pub triggered: Vec<bool>,
    /// Alert message template
    pub message: String,
    /// Alert frequency
    pub freq: AlertFreq,
}

impl AlertCondition {
    /// Create a new alert condition
    pub fn new(name: impl Into<String>, message: impl Into<String>, freq: AlertFreq) -> Self {
        Self {
            name: name.into(),
            triggered: Vec::new(),
            message: message.into(),
            freq,
        }
    }

    /// Add a trigger state for the current bar
    pub fn push(&mut self, triggered: bool) {
        self.triggered.push(triggered);
    }

    /// Get triggered bars
    pub fn triggered_bars(&self) -> Vec<usize> {
        self.triggered
            .iter()
            .enumerate()
            .filter(|(_, &t)| t)
            .map(|(i, _)| i)
            .collect()
    }
}

/// Alert condition manager
#[derive(Debug, Default)]
pub struct AlertManager {
    /// All alert conditions
    conditions: Vec<AlertCondition>,
    /// Maximum number of alerts
    max_alerts: usize,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            max_alerts: 100,
        }
    }

    /// Create with custom limit
    pub fn with_max_alerts(max_alerts: usize) -> Self {
        Self {
            conditions: Vec::new(),
            max_alerts,
        }
    }

    /// Add an alert condition
    pub fn add_condition(&mut self, condition: AlertCondition) -> Result<usize> {
        if self.conditions.len() >= self.max_alerts {
            return Err(OutputError::DrawingLimitExceeded {
                max: self.max_alerts,
            });
        }
        let index = self.conditions.len();
        self.conditions.push(condition);
        Ok(index)
    }

    /// Get a condition by index
    pub fn get(&self, index: usize) -> Option<&AlertCondition> {
        self.conditions.get(index)
    }

    /// Get mutable condition
    pub fn get_mut(&mut self, index: usize) -> Option<&mut AlertCondition> {
        self.conditions.get_mut(index)
    }

    /// Get all conditions
    pub fn all_conditions(&self) -> &[AlertCondition] {
        &self.conditions
    }

    /// Clear all conditions
    pub fn clear(&mut self) {
        self.conditions.clear();
    }

    /// Get count
    pub fn count(&self) -> usize {
        self.conditions.len()
    }
}

/// Create an alert condition
pub fn alertcondition(
    manager: &mut AlertManager,
    name: impl Into<String>,
    message: impl Into<String>,
    freq: AlertFreq,
    triggered: &[bool],
) -> Result<usize> {
    let mut condition = AlertCondition::new(name, message, freq);
    for &t in triggered {
        condition.push(t);
    }
    manager.add_condition(condition)
}

/// JSON Output container for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    /// Schema version
    pub version: String,
    /// Execution timestamp
    pub timestamp: String,
    /// Script information
    pub script_info: Option<ScriptInfo>,
    /// Plot outputs
    pub plots: Vec<PlotOutput>,
    /// Horizontal lines
    pub hlines: Vec<HLineOutput>,
    /// Fill areas
    pub fills: Vec<FillOutput>,
    /// Drawings
    pub drawings: DrawingOutput,
    /// Strategy signals (optional)
    pub strategy: Option<StrategyOutput>,
    /// Alert conditions
    pub alerts: Vec<AlertCondition>,
    /// Background colors per bar
    pub bgcolor: Vec<Option<pine_runtime::value::Color>>,
}

impl Default for JsonOutput {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            script_info: None,
            plots: Vec::new(),
            hlines: Vec::new(),
            fills: Vec::new(),
            drawings: DrawingOutput::default(),
            strategy: None,
            alerts: Vec::new(),
            bgcolor: Vec::new(),
        }
    }
}

/// Script information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    /// Script name
    pub name: String,
    /// Script description
    pub description: Option<String>,
}

impl JsonOutput {
    /// Create a new JSON output
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert from ScriptOutput
    pub fn from_script_output(output: ScriptOutput, alerts: Vec<AlertCondition>) -> Self {
        Self {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            script_info: None,
            plots: output.plots,
            hlines: output.hlines,
            fills: output.fills,
            drawings: output.drawings,
            strategy: output.strategy,
            alerts,
            bgcolor: output.bgcolor,
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| OutputError::InvalidValue(e.to_string()))
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| OutputError::InvalidValue(e.to_string()))
    }

    /// Set script info
    pub fn with_script_info(
        mut self,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Self {
        self.script_info = Some(ScriptInfo {
            name: name.into(),
            description: description.map(|d| d.into()),
        });
        self
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
        use std::str::FromStr;

        assert_eq!(
            Position::from_str("position.abovebar"),
            Ok(Position::AboveBar)
        );
        assert_eq!(
            Position::from_str("position.belowbar"),
            Ok(Position::BelowBar)
        );
        assert_eq!(Position::from_str("position.top"), Ok(Position::Top));
        assert_eq!(Position::from_str("position.bottom"), Ok(Position::Bottom));
        assert_eq!(Position::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_text_align_parsing() {
        use std::str::FromStr;

        assert_eq!(TextAlign::from_str("text.align_left"), Ok(TextAlign::Left));
        assert_eq!(
            TextAlign::from_str("text.align_center"),
            Ok(TextAlign::Center)
        );
        assert_eq!(
            TextAlign::from_str("text.align_right"),
            Ok(TextAlign::Right)
        );
        assert_eq!(TextAlign::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_text_size_parsing() {
        use std::str::FromStr;

        assert_eq!(TextSize::from_str("size.tiny"), Ok(TextSize::Tiny));
        assert_eq!(TextSize::from_str("size.normal"), Ok(TextSize::Normal));
        assert_eq!(TextSize::from_str("size.huge"), Ok(TextSize::Huge));
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

    #[test]
    fn test_alert_condition_creation() {
        let condition = AlertCondition::new(
            "Buy Signal",
            "Price crossed above SMA",
            AlertFreq::OncePerBar,
        );
        assert_eq!(condition.name, "Buy Signal");
        assert_eq!(condition.message, "Price crossed above SMA");
        assert_eq!(condition.freq, AlertFreq::OncePerBar);
        assert!(condition.triggered.is_empty());
    }

    #[test]
    fn test_alert_condition_push_and_triggered_bars() {
        let mut condition = AlertCondition::new("Test", "Test message", AlertFreq::All);
        condition.push(true);
        condition.push(false);
        condition.push(true);
        condition.push(true);

        let triggered = condition.triggered_bars();
        assert_eq!(triggered, vec![0, 2, 3]);
    }

    #[test]
    fn test_alert_manager() {
        let mut manager = AlertManager::new();

        let triggered = vec![true, false, true];
        let index = alertcondition(
            &mut manager,
            "Alert1",
            "Message1",
            AlertFreq::OncePerBar,
            &triggered,
        )
        .unwrap();

        assert_eq!(index, 0);
        assert_eq!(manager.count(), 1);

        let condition = manager.get(0).unwrap();
        assert_eq!(condition.name, "Alert1");
        assert_eq!(condition.triggered, vec![true, false, true]);
    }

    #[test]
    fn test_alert_manager_limit() {
        let mut manager = AlertManager::with_max_alerts(2);

        let triggered = vec![true];
        alertcondition(
            &mut manager,
            "Alert1",
            "Msg1",
            AlertFreq::OncePerBar,
            &triggered,
        )
        .unwrap();
        alertcondition(
            &mut manager,
            "Alert2",
            "Msg2",
            AlertFreq::OncePerBar,
            &triggered,
        )
        .unwrap();

        assert_eq!(manager.count(), 2);

        // Third alert should fail
        let result = alertcondition(
            &mut manager,
            "Alert3",
            "Msg3",
            AlertFreq::OncePerBar,
            &triggered,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_alert_freq_default() {
        assert_eq!(AlertFreq::default(), AlertFreq::OncePerBar);
    }

    #[test]
    fn test_json_output_default() {
        let output = JsonOutput::default();
        assert_eq!(output.version, "1.0.0");
        assert!(output.script_info.is_none());
        assert!(output.plots.is_empty());
        assert!(output.hlines.is_empty());
        assert!(output.fills.is_empty());
        assert!(output.strategy.is_none());
        assert!(output.alerts.is_empty());
        assert!(output.bgcolor.is_empty());
    }

    #[test]
    fn test_json_output_with_script_info() {
        let output = JsonOutput::new().with_script_info("Test Script", Some("A test script"));

        assert_eq!(output.version, "1.0.0");
        assert!(output.script_info.is_some());
        let info = output.script_info.unwrap();
        assert_eq!(info.name, "Test Script");
        assert_eq!(info.description, Some("A test script".to_string()));
    }

    #[test]
    fn test_json_output_serialization() {
        let mut output = JsonOutput::new();
        output.script_info = Some(ScriptInfo {
            name: "SMA Strategy".to_string(),
            description: Some("Simple moving average strategy".to_string()),
        });

        // Add a plot
        output.plots.push(PlotOutput {
            name: "SMA".to_string(),
            plot_type: PlotType::Line {
                color: pine_runtime::value::Color::new(255, 87, 51),
                width: 2,
                style: "solid".to_string(),
            },
            values: vec![Some(100.0), Some(101.0), None, Some(103.0)],
        });

        // Add an alert
        output.alerts.push(AlertCondition::new(
            "Buy Signal",
            "Price crossed above SMA",
            AlertFreq::OncePerBar,
        ));

        // Serialize to JSON
        let json = output.to_json().unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("1.0.0"));
        assert!(json.contains("SMA Strategy"));
        assert!(json.contains("SMA"));
        assert!(json.contains("Buy Signal"));

        // Deserialize back
        let parsed = JsonOutput::from_json(&json).unwrap();
        assert_eq!(parsed.version, "1.0.0");
        assert_eq!(parsed.plots.len(), 1);
        assert_eq!(parsed.plots[0].name, "SMA");
        assert_eq!(parsed.alerts.len(), 1);
        assert_eq!(parsed.alerts[0].name, "Buy Signal");
    }

    #[test]
    fn test_json_output_from_script_output() {
        let script_output = ScriptOutput {
            plots: vec![PlotOutput {
                name: "Price".to_string(),
                plot_type: PlotType::Line {
                    color: pine_runtime::value::Color::new(0, 150, 136),
                    width: 1,
                    style: "solid".to_string(),
                },
                values: vec![Some(50000.0), Some(50100.0)],
            }],
            hlines: vec![HLineOutput {
                price: 50000.0,
                color: pine_runtime::value::Color::new(255, 0, 0),
                style: HLineStyle::Dashed,
                width: 1,
                title: Some("Support".to_string()),
            }],
            fills: vec![],
            drawings: DrawingOutput::default(),
            strategy: None,
            alerts: vec![],
            bgcolor: vec![None, Some(pine_runtime::value::Color::new(255, 255, 0))],
        };

        let alerts = vec![AlertCondition::new(
            "Test Alert",
            "Test message",
            AlertFreq::All,
        )];

        let json_output = JsonOutput::from_script_output(script_output, alerts);

        assert_eq!(json_output.plots.len(), 1);
        assert_eq!(json_output.hlines.len(), 1);
        assert_eq!(json_output.bgcolor.len(), 2);
        assert_eq!(json_output.alerts.len(), 1);
        assert_eq!(json_output.alerts[0].name, "Test Alert");
    }
}
