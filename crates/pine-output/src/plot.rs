//! Plot functions (plot, plotshape, plotchar, plotarrow)

use crate::Result;
use pine_runtime::value::Color;

/// Shape types for plotshape
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// X shape
    XCross,
    /// Plus shape
    Cross,
    /// Circle
    Circle,
    /// Triangle up
    TriangleUp,
    /// Triangle down
    TriangleDown,
    /// Diamond
    Diamond,
    /// Square
    Square,
    /// Label up (arrow pointing up with label)
    LabelUp,
    /// Label down (arrow pointing down with label)
    LabelDown,
    /// Arrow up
    ArrowUp,
    /// Arrow down
    ArrowDown,
    /// Flag
    Flag,
}

impl Shape {
    /// Parse shape from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "shape.xcross" => Some(Self::XCross),
            "shape.cross" => Some(Self::Cross),
            "shape.circle" => Some(Self::Circle),
            "shape.triangleup" => Some(Self::TriangleUp),
            "shape.triangledown" => Some(Self::TriangleDown),
            "shape.diamond" => Some(Self::Diamond),
            "shape.square" => Some(Self::Square),
            "shape.labelup" => Some(Self::LabelUp),
            "shape.labeldown" => Some(Self::LabelDown),
            "shape.arrowup" => Some(Self::ArrowUp),
            "shape.arrowdown" => Some(Self::ArrowDown),
            "shape.flag" => Some(Self::Flag),
            _ => None,
        }
    }
}

/// Character to plot (for plotchar)
pub type PlotChar = char;

/// Location for plotshape/plotchar
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Location {
    /// Above the bar
    AboveBar,
    /// Below the bar
    BelowBar,
    /// At the top of the chart
    Top,
    /// At the bottom of the chart
    Bottom,
    /// At absolute price level
    Absolute,
}
impl Location {
    /// Parse location from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "location.abovebar" => Some(Self::AboveBar),
            "location.belowbar" => Some(Self::BelowBar),
            "location.top" => Some(Self::Top),
            "location.bottom" => Some(Self::Bottom),
            "location.absolute" => Some(Self::Absolute),
            _ => None,
        }
    }
}

/// Size for plot markers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// Tiny size
    Tiny,
    /// Small size
    Small,
    /// Normal size (default)
    Normal,
    /// Large size
    Large,
    /// Huge size
    Huge,
}

impl Size {
    /// Parse size from string
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

/// A plot output
#[derive(Debug, Clone, PartialEq)]
pub struct Plot {
    /// Plot title/legend
    pub title: String,
    /// Series values
    pub values: Vec<Option<f64>>,
    /// Plot color (per bar or fixed)
    pub color: PlotColor,
    /// Line style
    pub style: LineStyle,
    /// Line width
    pub linewidth: u32,
    /// Transparency (0-100)
    pub transp: u8,
    /// Whether to display in the legend
    pub show_legend: bool,
    /// Whether to display the plot
    pub display: PlotDisplay,
}

/// Color specification for plots
#[derive(Debug, Clone, PartialEq)]
pub enum PlotColor {
    /// Fixed color
    Fixed(Color),
    /// Per-bar colors
    Series(Vec<Option<Color>>),
}

/// Line style for plots
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    /// Solid line (default)
    Solid,
    /// Dotted line
    Dotted,
    /// Dashed line
    Dashed,
}

/// Display mode for plots
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlotDisplay {
    /// Display all
    All,
    /// Don't display
    None,
}

impl Default for Plot {
    fn default() -> Self {
        Self {
            title: String::new(),
            values: Vec::new(),
            color: PlotColor::Fixed(Color::new(0, 0, 255)), // Blue default
            style: LineStyle::Solid,
            linewidth: 1,
            transp: 0,
            show_legend: true,
            display: PlotDisplay::All,
        }
    }
}

impl Plot {
    /// Create a new plot with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Add a value to the plot
    pub fn push(&mut self, value: Option<f64>) {
        self.values.push(value);
    }

    /// Set the plot color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = PlotColor::Fixed(color);
        self
    }

    /// Set the line style
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the line width
    pub fn with_linewidth(mut self, width: u32) -> Self {
        self.linewidth = width;
        self
    }

    /// Set transparency
    pub fn with_transp(mut self, transp: u8) -> Self {
        self.transp = transp.clamp(0, 100);
        self
    }
}

/// A shape plot output
#[derive(Debug, Clone, PartialEq)]
pub struct ShapePlot {
    /// Plot title
    pub title: String,
    /// Shape to plot
    pub shape: Shape,
    /// Series values (where to plot)
    pub values: Vec<Option<f64>>,
    /// Shape color
    pub color: PlotColor,
    /// Location on the chart
    pub location: Location,
    /// Size of the shape
    pub size: Size,
    /// Text to display with the shape
    pub text: Option<String>,
    /// Text color
    pub textcolor: Option<Color>,
    /// Transparency (0-100)
    pub transp: u8,
}

impl Default for ShapePlot {
    fn default() -> Self {
        Self {
            title: String::new(),
            shape: Shape::Circle,
            values: Vec::new(),
            color: PlotColor::Fixed(Color::new(0, 0, 255)),
            location: Location::AboveBar,
            size: Size::Normal,
            text: None,
            textcolor: None,
            transp: 0,
        }
    }
}

impl ShapePlot {
    /// Create a new shape plot
    pub fn new(title: impl Into<String>, shape: Shape) -> Self {
        Self {
            title: title.into(),
            shape,
            ..Default::default()
        }
    }

    /// Add a value to the plot
    pub fn push(&mut self, value: Option<f64>) {
        self.values.push(value);
    }
}

/// A character plot output
#[derive(Debug, Clone, PartialEq)]
pub struct CharPlot {
    /// Plot title
    pub title: String,
    /// Character to plot
    pub char: PlotChar,
    /// Series values (where to plot)
    pub values: Vec<Option<f64>>,
    /// Character color
    pub color: PlotColor,
    /// Location on the chart
    pub location: Location,
    /// Size of the character
    pub size: Size,
    /// Transparency (0-100)
    pub transp: u8,
}

impl Default for CharPlot {
    fn default() -> Self {
        Self {
            title: String::new(),
            char: '●',
            values: Vec::new(),
            color: PlotColor::Fixed(Color::new(0, 0, 255)),
            location: Location::AboveBar,
            size: Size::Normal,
            transp: 0,
        }
    }
}

impl CharPlot {
    /// Create a new character plot
    pub fn new(title: impl Into<String>, char: PlotChar) -> Self {
        Self {
            title: title.into(),
            char,
            ..Default::default()
        }
    }

    /// Add a value to the plot
    pub fn push(&mut self, value: Option<f64>) {
        self.values.push(value);
    }
}

/// An arrow plot output
#[derive(Debug, Clone, PartialEq)]
pub struct ArrowPlot {
    /// Plot title
    pub title: String,
    /// Series values (negative = down arrow, positive = up arrow)
    pub values: Vec<Option<f64>>,
    /// Arrow color
    pub color: PlotColor,
    /// Color for up arrows
    pub colorup: Option<Color>,
    /// Color for down arrows
    pub colordown: Option<Color>,
    /// Offset from bar
    pub offset: i32,
    /// Minimum height difference to show arrow
    pub minheight: Option<f64>,
    /// Whether to fill the arrow
    pub fillcolor: Option<Color>,
    /// Transparency (0-100)
    pub transp: u8,
}

impl Default for ArrowPlot {
    fn default() -> Self {
        Self {
            title: String::new(),
            values: Vec::new(),
            color: PlotColor::Fixed(Color::new(0, 0, 255)),
            colorup: None,
            colordown: None,
            offset: 0,
            minheight: None,
            fillcolor: None,
            transp: 0,
        }
    }
}

impl ArrowPlot {
    /// Create a new arrow plot
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Add a value to the plot
    pub fn push(&mut self, value: Option<f64>) {
        self.values.push(value);
    }
}

/// Plot a series on the chart
pub fn plot(
    series: &[Option<f64>],
    title: impl Into<String>,
    color: Option<Color>,
    linewidth: Option<u32>,
    style: Option<LineStyle>,
    transp: Option<u8>,
) -> Result<Plot> {
    let mut plot = Plot::new(title);

    if let Some(c) = color {
        plot.color = PlotColor::Fixed(c);
    }
    if let Some(lw) = linewidth {
        plot.linewidth = lw;
    }
    if let Some(s) = style {
        plot.style = s;
    }
    if let Some(t) = transp {
        plot.transp = t.clamp(0, 100);
    }

    plot.values = series.to_vec();

    Ok(plot)
}

/// Plot a shape on the chart
pub fn plotshape(
    series: &[Option<f64>],
    title: impl Into<String>,
    shape: Shape,
    location: Location,
    color: Option<Color>,
    size: Option<Size>,
    text: Option<String>,
) -> Result<ShapePlot> {
    let mut plot = ShapePlot::new(title, shape);
    plot.location = location;
    plot.values = series.to_vec();

    if let Some(c) = color {
        plot.color = PlotColor::Fixed(c);
    }
    if let Some(s) = size {
        plot.size = s;
    }
    if let Some(t) = text {
        plot.text = Some(t);
    }

    Ok(plot)
}

/// Plot a character on the chart
pub fn plotchar(
    series: &[Option<f64>],
    title: impl Into<String>,
    char: PlotChar,
    location: Location,
    color: Option<Color>,
    size: Option<Size>,
) -> Result<CharPlot> {
    let mut plot = CharPlot::new(title, char);
    plot.location = location;
    plot.values = series.to_vec();

    if let Some(c) = color {
        plot.color = PlotColor::Fixed(c);
    }
    if let Some(s) = size {
        plot.size = s;
    }

    Ok(plot)
}

/// Plot an arrow on the chart
pub fn plotarrow(
    series: &[Option<f64>],
    title: impl Into<String>,
    colorup: Option<Color>,
    colordown: Option<Color>,
    offset: Option<i32>,
) -> Result<ArrowPlot> {
    let mut plot = ArrowPlot::new(title);
    plot.values = series.to_vec();
    plot.colorup = colorup;
    plot.colordown = colordown;
    if let Some(o) = offset {
        plot.offset = o;
    }

    Ok(plot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plot_creation() {
        let values = vec![Some(100.0), Some(101.0), Some(102.0)];
        let plot = plot(
            &values,
            "Close",
            Some(Color::new(255, 0, 0)),
            Some(2),
            Some(LineStyle::Solid),
            Some(50),
        )
        .unwrap();

        assert_eq!(plot.title, "Close");
        assert_eq!(plot.values.len(), 3);
        assert_eq!(plot.linewidth, 2);
        assert_eq!(plot.transp, 50);
    }

    #[test]
    fn test_shape_plot() {
        let values = vec![Some(100.0), None, Some(102.0)];
        let plot = plotshape(
            &values,
            "Buy Signal",
            Shape::TriangleUp,
            Location::BelowBar,
            Some(Color::new(0, 255, 0)),
            Some(Size::Large),
            Some("Buy".to_string()),
        )
        .unwrap();

        assert_eq!(plot.shape, Shape::TriangleUp);
        assert_eq!(plot.location, Location::BelowBar);
        assert_eq!(plot.text, Some("Buy".to_string()));
    }

    #[test]
    fn test_char_plot() {
        let values = vec![Some(100.0), Some(101.0)];
        let plot = plotchar(
            &values,
            "Char Plot",
            '★',
            Location::AboveBar,
            Some(Color::new(255, 215, 0)),
            Some(Size::Normal),
        )
        .unwrap();

        assert_eq!(plot.char, '★');
        assert_eq!(plot.size, Size::Normal);
    }

    #[test]
    fn test_arrow_plot() {
        let values = vec![Some(1.0), Some(-1.0), Some(0.0)];
        let plot = plotarrow(
            &values,
            "Arrows",
            Some(Color::new(0, 255, 0)),
            Some(Color::new(255, 0, 0)),
            Some(1),
        )
        .unwrap();

        assert_eq!(plot.values.len(), 3);
        assert_eq!(plot.offset, 1);
        assert!(plot.colorup.is_some());
        assert!(plot.colordown.is_some());
    }

    #[test]
    fn test_shape_parsing() {
        assert_eq!(Shape::from_str("shape.circle"), Some(Shape::Circle));
        assert_eq!(Shape::from_str("shape.triangleup"), Some(Shape::TriangleUp));
        assert_eq!(Shape::from_str("shape.arrowdown"), Some(Shape::ArrowDown));
        assert_eq!(Shape::from_str("invalid"), None);
    }

    #[test]
    fn test_location_parsing() {
        assert_eq!(Location::from_str("location.abovebar"), Some(Location::AboveBar));
        assert_eq!(Location::from_str("location.belowbar"), Some(Location::BelowBar));
        assert_eq!(Location::from_str("location.top"), Some(Location::Top));
    }

    #[test]
    fn test_transparency_clamping() {
        let plot = Plot::default().with_transp(150);
        assert_eq!(plot.transp, 100);

        let plot = Plot::default().with_transp(50);
        assert_eq!(plot.transp, 50);
    }
}
