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

impl std::str::FromStr for Shape {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "shape.xcross" => Ok(Self::XCross),
            "shape.cross" => Ok(Self::Cross),
            "shape.circle" => Ok(Self::Circle),
            "shape.triangleup" => Ok(Self::TriangleUp),
            "shape.triangledown" => Ok(Self::TriangleDown),
            "shape.diamond" => Ok(Self::Diamond),
            "shape.square" => Ok(Self::Square),
            "shape.labelup" => Ok(Self::LabelUp),
            "shape.labeldown" => Ok(Self::LabelDown),
            "shape.arrowup" => Ok(Self::ArrowUp),
            "shape.arrowdown" => Ok(Self::ArrowDown),
            "shape.flag" => Ok(Self::Flag),
            _ => Err(()),
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
impl std::str::FromStr for Location {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "location.abovebar" => Ok(Self::AboveBar),
            "location.belowbar" => Ok(Self::BelowBar),
            "location.top" => Ok(Self::Top),
            "location.bottom" => Ok(Self::Bottom),
            "location.absolute" => Ok(Self::Absolute),
            _ => Err(()),
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

impl std::str::FromStr for Size {
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

/// Plot a horizontal line
pub fn hline(
    price: f64,
    title: impl Into<String>,
    color: Option<Color>,
    linestyle: Option<LineStyle>,
    linewidth: Option<u32>,
) -> Result<crate::HLineOutput> {
    use crate::HLineStyle;

    let style = match linestyle {
        Some(LineStyle::Solid) => HLineStyle::Solid,
        Some(LineStyle::Dashed) => HLineStyle::Dashed,
        Some(LineStyle::Dotted) => HLineStyle::Dotted,
        None => HLineStyle::default(),
    };

    Ok(crate::HLineOutput {
        price,
        color: color.unwrap_or_else(|| Color::new(128, 128, 128)),
        style,
        width: linewidth.unwrap_or(1),
        title: Some(title.into()),
    })
}

/// Fill between two plots
pub fn fill(
    plot1_index: usize,
    plot2_index: usize,
    color: Color,
    transp: Option<u8>,
    title: Option<impl Into<String>>,
) -> Result<crate::FillOutput> {
    Ok(crate::FillOutput {
        plot1_index,
        plot2_index,
        color,
        transp: transp.unwrap_or(0).clamp(0, 100),
        title: title.map(|t| t.into()),
    })
}

/// Set background color
pub fn bgcolor(colors: &[Option<Color>], offset: Option<i32>) -> Vec<Option<Color>> {
    let _ = offset; // offset is used for visual positioning, not stored
    colors.to_vec()
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
        use std::str::FromStr;

        assert_eq!(Shape::from_str("shape.circle"), Ok(Shape::Circle));
        assert_eq!(Shape::from_str("shape.triangleup"), Ok(Shape::TriangleUp));
        assert_eq!(Shape::from_str("shape.arrowdown"), Ok(Shape::ArrowDown));
        assert_eq!(Shape::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_location_parsing() {
        use std::str::FromStr;

        assert_eq!(
            Location::from_str("location.abovebar"),
            Ok(Location::AboveBar)
        );
        assert_eq!(
            Location::from_str("location.belowbar"),
            Ok(Location::BelowBar)
        );
        assert_eq!(Location::from_str("location.top"), Ok(Location::Top));
    }

    #[test]
    fn test_transparency_clamping() {
        let plot = Plot::default().with_transp(150);
        assert_eq!(plot.transp, 100);

        let plot = Plot::default().with_transp(50);
        assert_eq!(plot.transp, 50);
    }

    #[test]
    fn test_hline() {
        let hline = hline(
            100.0,
            "Support",
            Some(Color::new(0, 255, 0)),
            Some(LineStyle::Dashed),
            Some(2),
        )
        .unwrap();

        assert_eq!(hline.price, 100.0);
        assert_eq!(hline.color, Color::new(0, 255, 0));
        assert_eq!(hline.style, crate::HLineStyle::Dashed);
        assert_eq!(hline.width, 2);
        assert_eq!(hline.title, Some("Support".to_string()));
    }

    #[test]
    fn test_fill() {
        let fill = fill(0, 1, Color::new(255, 0, 0), Some(50), Some::<&str>("Band")).unwrap();

        assert_eq!(fill.plot1_index, 0);
        assert_eq!(fill.plot2_index, 1);
        assert_eq!(fill.color, Color::new(255, 0, 0));
        assert_eq!(fill.transp, 50);
        assert_eq!(fill.title, Some("Band".to_string()));
    }

    #[test]
    fn test_fill_transparency_clamping() {
        let fill = fill(0, 1, Color::new(255, 0, 0), Some(150), None::<&str>).unwrap();
        assert_eq!(fill.transp, 100);
    }

    #[test]
    fn test_bgcolor() {
        let colors = vec![
            Some(Color::new(255, 0, 0)),
            None,
            Some(Color::new(0, 255, 0)),
        ];
        let result = bgcolor(&colors, Some(0));

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Some(Color::new(255, 0, 0)));
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(Color::new(0, 255, 0)));
    }
}
