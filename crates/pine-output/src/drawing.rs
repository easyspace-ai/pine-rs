//! Drawing functions (label, box, table)

use crate::{DrawingId, Result, TextAlign, TextSize};
use pine_runtime::value::Color;
use std::collections::HashMap;

/// Label style types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelStyle {
    /// Label pointing up
    LabelUp,
    /// Label pointing down
    LabelDown,
    /// Label pointing left
    LabelLeft,
    /// Label pointing right
    LabelRight,
    /// Label with no arrow
    LabelNone,
    /// X-cross shape
    XCross,
    /// Plus/cross shape
    Cross,
    /// Triangle pointing up
    TriangleUp,
    /// Triangle pointing down
    TriangleDown,
    /// Diamond shape
    Diamond,
    /// Circle shape
    Circle,
    /// Square shape
    Square,
    /// Flag shape
    Flag,
    /// Arrow pointing up
    ArrowUp,
    /// Arrow pointing down
    ArrowDown,
    /// Bar up (candlestick)
    BarUp,
    /// Bar down (candlestick)
    BarDown,
}

impl std::str::FromStr for LabelStyle {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "label.style_labelup" => Ok(Self::LabelUp),
            "label.style_labeldown" => Ok(Self::LabelDown),
            "label.style_labelleft" => Ok(Self::LabelLeft),
            "label.style_labelright" => Ok(Self::LabelRight),
            "label.style_labelnone" => Ok(Self::LabelNone),
            "label.style_xcross" => Ok(Self::XCross),
            "label.style_cross" => Ok(Self::Cross),
            "label.style_triangleup" => Ok(Self::TriangleUp),
            "label.style_triangledown" => Ok(Self::TriangleDown),
            "label.style_diamond" => Ok(Self::Diamond),
            "label.style_circle" => Ok(Self::Circle),
            "label.style_square" => Ok(Self::Square),
            "label.style_flag" => Ok(Self::Flag),
            "label.style_arrowup" => Ok(Self::ArrowUp),
            "label.style_arrowdown" => Ok(Self::ArrowDown),
            "label.style_barup" => Ok(Self::BarUp),
            "label.style_bardown" => Ok(Self::BarDown),
            _ => Err(()),
        }
    }
}

/// A label drawing
#[derive(Debug, Clone)]
pub struct Label {
    /// Label ID
    pub id: DrawingId,
    /// X position (bar index)
    pub x: i64,
    /// Y position (price level)
    pub y: f64,
    /// Text content
    pub text: String,
    /// Text color
    pub textcolor: Color,
    /// Background color
    pub color: Color,
    /// Border color
    pub border_color: Color,
    /// Text size
    pub size: TextSize,
    /// Text alignment
    pub textalign: TextAlign,
    /// Label style
    pub style: LabelStyle,
    /// Whether label is visible
    pub visible: bool,
    /// Tooltip text (optional)
    pub tooltip: Option<String>,
    /// Text wrapping width (0 = no wrap)
    pub textwrap: i32,
    /// Text rotation angle
    pub textrotation: i32,
}

impl Label {
    /// Create a new label
    pub fn new(id: DrawingId, x: i64, y: f64, text: impl Into<String>) -> Self {
        Self {
            id,
            x,
            y,
            text: text.into(),
            textcolor: Color::new(255, 255, 255), // White
            color: Color::new(0, 0, 255),         // Blue
            border_color: Color::new(0, 0, 0),    // Black
            size: TextSize::Normal,
            textalign: TextAlign::Center,
            style: LabelStyle::LabelUp,
            visible: true,
            tooltip: None,
            textwrap: 0,
            textrotation: 0,
        }
    }

    /// Set the text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Set the position
    pub fn set_xy(&mut self, x: i64, y: f64) {
        self.x = x;
        self.y = y;
    }

    /// Set the text color
    pub fn set_textcolor(&mut self, color: Color) {
        self.textcolor = color;
    }

    /// Set the background color
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Set the border color
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }

    /// Set the text size
    pub fn set_size(&mut self, size: TextSize) {
        self.size = size;
    }

    /// Set the text alignment
    pub fn set_textalign(&mut self, align: TextAlign) {
        self.textalign = align;
    }

    /// Set the style
    pub fn set_style(&mut self, style: LabelStyle) {
        self.style = style;
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Set tooltip
    pub fn set_tooltip(&mut self, tooltip: impl Into<String>) {
        self.tooltip = Some(tooltip.into());
    }

    /// Set text wrap width
    pub fn set_textwrap(&mut self, width: i32) {
        self.textwrap = width;
    }

    /// Set text rotation
    pub fn set_textrotation(&mut self, angle: i32) {
        self.textrotation = angle;
    }
}

/// Manager for label lifecycle
#[derive(Debug, Default)]
pub struct LabelManager {
    /// All labels by ID
    labels: HashMap<DrawingId, Label>,
    /// Next label ID
    next_id: u64,
    /// Maximum number of labels allowed
    max_labels: usize,
}

impl LabelManager {
    /// Create a new label manager
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            next_id: 1,
            max_labels: 500, // Default limit
        }
    }

    /// Create a new label manager with custom limit
    pub fn with_max_labels(max_labels: usize) -> Self {
        Self {
            labels: HashMap::new(),
            next_id: 1,
            max_labels,
        }
    }

    /// Create a new label
    pub fn new_label(&mut self, x: i64, y: f64, text: impl Into<String>) -> Result<DrawingId> {
        if self.labels.len() >= self.max_labels {
            return Err(crate::OutputError::DrawingLimitExceeded {
                max: self.max_labels,
            });
        }

        let id = DrawingId::new(self.next_id);
        self.next_id += 1;

        let label = Label::new(id, x, y, text);
        self.labels.insert(id, label);

        Ok(id)
    }

    /// Get a label by ID
    pub fn get(&self, id: DrawingId) -> Option<&Label> {
        self.labels.get(&id)
    }

    /// Get a mutable label by ID
    pub fn get_mut(&mut self, id: DrawingId) -> Option<&mut Label> {
        self.labels.get_mut(&id)
    }

    /// Delete a label
    pub fn delete(&mut self, id: DrawingId) -> Result<()> {
        self.labels.remove(&id);
        Ok(())
    }

    /// Delete all labels
    pub fn delete_all(&mut self) {
        self.labels.clear();
    }

    /// Get all labels
    pub fn all_labels(&self) -> &HashMap<DrawingId, Label> {
        &self.labels
    }

    /// Get mutable access to all labels
    pub fn all_labels_mut(&mut self) -> &mut HashMap<DrawingId, Label> {
        &mut self.labels
    }

    /// Get the count of labels
    pub fn count(&self) -> usize {
        self.labels.len()
    }

    /// Set the maximum number of labels
    pub fn set_max_labels(&mut self, max: usize) {
        self.max_labels = max;
        // Clean up excess labels if needed
        while self.labels.len() > self.max_labels {
            // Remove oldest label (lowest ID)
            if let Some(oldest_id) = self.labels.keys().min_by_key(|k| k.0).copied() {
                self.labels.remove(&oldest_id);
            }
        }
    }
}

/// Label drawing functions
pub mod label {
    use super::*;

    /// Create a new label
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        manager: &mut LabelManager,
        x: i64,
        y: f64,
        text: impl Into<String>,
        style: Option<LabelStyle>,
        color: Option<Color>,
        textcolor: Option<Color>,
        size: Option<TextSize>,
    ) -> Result<DrawingId> {
        let id = manager.new_label(x, y, text)?;

        if let Some(label) = manager.get_mut(id) {
            if let Some(s) = style {
                label.set_style(s);
            }
            if let Some(c) = color {
                label.set_color(c);
            }
            if let Some(tc) = textcolor {
                label.set_textcolor(tc);
            }
            if let Some(sz) = size {
                label.set_size(sz);
            }
        }

        Ok(id)
    }

    /// Set label text
    pub fn set_text(
        manager: &mut LabelManager,
        id: DrawingId,
        text: impl Into<String>,
    ) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_text(text);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label position
    pub fn set_xy(manager: &mut LabelManager, id: DrawingId, x: i64, y: f64) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_xy(x, y);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label color (background)
    pub fn set_color(manager: &mut LabelManager, id: DrawingId, color: Color) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_color(color);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label text color
    pub fn set_textcolor(manager: &mut LabelManager, id: DrawingId, color: Color) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_textcolor(color);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label size
    pub fn set_size(manager: &mut LabelManager, id: DrawingId, size: TextSize) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_size(size);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label style
    pub fn set_style(manager: &mut LabelManager, id: DrawingId, style: LabelStyle) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_style(style);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label visibility
    pub fn set_visible(manager: &mut LabelManager, id: DrawingId, visible: bool) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_visible(visible);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label tooltip
    pub fn set_tooltip(
        manager: &mut LabelManager,
        id: DrawingId,
        tooltip: impl Into<String>,
    ) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_tooltip(tooltip);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Set label text alignment
    pub fn set_textalign(
        manager: &mut LabelManager,
        id: DrawingId,
        align: TextAlign,
    ) -> Result<()> {
        if let Some(label) = manager.get_mut(id) {
            label.set_textalign(align);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Label {:?} not found",
                id
            )))
        }
    }

    /// Delete a label
    pub fn delete(manager: &mut LabelManager, id: DrawingId) -> Result<()> {
        manager.delete(id)
    }

    /// Delete all labels
    pub fn delete_all(manager: &mut LabelManager) {
        manager.delete_all();
    }
}

/// A box drawing (rectangle on chart)
#[derive(Debug, Clone)]
pub struct Box {
    /// Box ID
    pub id: DrawingId,
    /// Left bar index
    pub left: i64,
    /// Top price level
    pub top: f64,
    /// Right bar index
    pub right: i64,
    /// Bottom price level
    pub bottom: f64,
    /// Border color
    pub border_color: Color,
    /// Border width
    pub border_width: i32,
    /// Background color
    pub bgcolor: Color,
    /// Whether box is visible
    pub visible: bool,
    /// Text inside the box
    pub text: String,
    /// Text color
    pub text_color: Color,
    /// Text size
    pub text_size: TextSize,
    /// Text alignment
    pub text_halign: TextAlign,
    /// Text vertical alignment
    pub text_valign: TextAlign,
}

impl Box {
    /// Create a new box
    pub fn new(id: DrawingId, left: i64, top: f64, right: i64, bottom: f64) -> Self {
        Self {
            id,
            left,
            top,
            right,
            bottom,
            border_color: Color::new(128, 128, 128),
            border_width: 1,
            bgcolor: Color::new(0, 0, 255),
            visible: true,
            text: String::new(),
            text_color: Color::new(255, 255, 255),
            text_size: TextSize::Normal,
            text_halign: TextAlign::Center,
            text_valign: TextAlign::Center,
        }
    }

    /// Set the left-top corner
    pub fn set_lefttop(&mut self, left: i64, top: f64) {
        self.left = left;
        self.top = top;
    }

    /// Set the right-bottom corner
    pub fn set_rightbottom(&mut self, right: i64, bottom: f64) {
        self.right = right;
        self.bottom = bottom;
    }

    /// Set border color
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }

    /// Set border width
    pub fn set_border_width(&mut self, width: i32) {
        self.border_width = width;
    }

    /// Set background color
    pub fn set_bgcolor(&mut self, color: Color) {
        self.bgcolor = color;
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Set text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Set text color
    pub fn set_text_color(&mut self, color: Color) {
        self.text_color = color;
    }

    /// Set text size
    pub fn set_text_size(&mut self, size: TextSize) {
        self.text_size = size;
    }
}

/// Manager for box lifecycle
#[derive(Debug, Default)]
pub struct BoxManager {
    /// All boxes by ID
    boxes: HashMap<DrawingId, Box>,
    /// Next box ID
    next_id: u64,
    /// Maximum number of boxes allowed
    max_boxes: usize,
}

impl BoxManager {
    /// Create a new box manager
    pub fn new() -> Self {
        Self {
            boxes: HashMap::new(),
            next_id: 1,
            max_boxes: 500,
        }
    }

    /// Create a new box manager with custom limit
    pub fn with_max_boxes(max_boxes: usize) -> Self {
        Self {
            boxes: HashMap::new(),
            next_id: 1,
            max_boxes,
        }
    }

    /// Create a new box
    pub fn new_box(&mut self, left: i64, top: f64, right: i64, bottom: f64) -> Result<DrawingId> {
        if self.boxes.len() >= self.max_boxes {
            return Err(crate::OutputError::DrawingLimitExceeded {
                max: self.max_boxes,
            });
        }

        let id = DrawingId::new(self.next_id);
        self.next_id += 1;

        let bx = Box::new(id, left, top, right, bottom);
        self.boxes.insert(id, bx);

        Ok(id)
    }

    /// Get a box by ID
    pub fn get(&self, id: DrawingId) -> Option<&Box> {
        self.boxes.get(&id)
    }

    /// Get a mutable box by ID
    pub fn get_mut(&mut self, id: DrawingId) -> Option<&mut Box> {
        self.boxes.get_mut(&id)
    }

    /// Delete a box
    pub fn delete(&mut self, id: DrawingId) -> Result<()> {
        self.boxes.remove(&id);
        Ok(())
    }

    /// Delete all boxes
    pub fn delete_all(&mut self) {
        self.boxes.clear();
    }

    /// Get count of boxes
    pub fn count(&self) -> usize {
        self.boxes.len()
    }
}

/// Box drawing functions
pub mod r#box {
    use super::*;

    /// Create a new box
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        manager: &mut BoxManager,
        left: i64,
        top: f64,
        right: i64,
        bottom: f64,
        border_color: Option<Color>,
        border_width: Option<i32>,
        bgcolor: Option<Color>,
    ) -> Result<DrawingId> {
        let id = manager.new_box(left, top, right, bottom)?;

        if let Some(bx) = manager.get_mut(id) {
            if let Some(c) = border_color {
                bx.set_border_color(c);
            }
            if let Some(w) = border_width {
                bx.set_border_width(w);
            }
            if let Some(c) = bgcolor {
                bx.set_bgcolor(c);
            }
        }

        Ok(id)
    }

    /// Set box left-top corner
    pub fn set_lefttop(manager: &mut BoxManager, id: DrawingId, left: i64, top: f64) -> Result<()> {
        if let Some(bx) = manager.get_mut(id) {
            bx.set_lefttop(left, top);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Box {:?} not found",
                id
            )))
        }
    }

    /// Set box right-bottom corner
    pub fn set_rightbottom(
        manager: &mut BoxManager,
        id: DrawingId,
        right: i64,
        bottom: f64,
    ) -> Result<()> {
        if let Some(bx) = manager.get_mut(id) {
            bx.set_rightbottom(right, bottom);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Box {:?} not found",
                id
            )))
        }
    }

    /// Set border color
    pub fn set_border_color(manager: &mut BoxManager, id: DrawingId, color: Color) -> Result<()> {
        if let Some(bx) = manager.get_mut(id) {
            bx.set_border_color(color);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Box {:?} not found",
                id
            )))
        }
    }

    /// Set border width
    pub fn set_border_width(manager: &mut BoxManager, id: DrawingId, width: i32) -> Result<()> {
        if let Some(bx) = manager.get_mut(id) {
            bx.set_border_width(width);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Box {:?} not found",
                id
            )))
        }
    }

    /// Set background color
    pub fn set_bgcolor(manager: &mut BoxManager, id: DrawingId, color: Color) -> Result<()> {
        if let Some(bx) = manager.get_mut(id) {
            bx.set_bgcolor(color);
            Ok(())
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Box {:?} not found",
                id
            )))
        }
    }

    /// Delete a box
    pub fn delete(manager: &mut BoxManager, id: DrawingId) -> Result<()> {
        manager.delete(id)
    }

    /// Delete all boxes
    pub fn delete_all(manager: &mut BoxManager) {
        manager.delete_all();
    }
}

/// Table position on chart
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TablePosition {
    /// Top left
    TopLeft,
    /// Top center
    TopCenter,
    /// Top right
    #[default]
    TopRight,
    /// Middle left
    MiddleLeft,
    /// Middle center
    MiddleCenter,
    /// Middle right
    MiddleRight,
    /// Bottom left
    BottomLeft,
    /// Bottom center
    BottomCenter,
    /// Bottom right
    BottomRight,
}

impl std::str::FromStr for TablePosition {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "position.top_left" => Ok(Self::TopLeft),
            "position.top_center" => Ok(Self::TopCenter),
            "position.top_right" => Ok(Self::TopRight),
            "position.middle_left" => Ok(Self::MiddleLeft),
            "position.middle_center" => Ok(Self::MiddleCenter),
            "position.middle_right" => Ok(Self::MiddleRight),
            "position.bottom_left" => Ok(Self::BottomLeft),
            "position.bottom_center" => Ok(Self::BottomCenter),
            "position.bottom_right" => Ok(Self::BottomRight),
            _ => Err(()),
        }
    }
}

/// A table cell
#[derive(Debug, Clone)]
pub struct TableCell {
    /// Column index
    pub column: usize,
    /// Row index
    pub row: usize,
    /// Cell text
    pub text: String,
    /// Text color
    pub text_color: Color,
    /// Background color
    pub bg_color: Color,
    /// Text size
    pub text_size: TextSize,
    /// Text horizontal alignment
    pub text_halign: TextAlign,
    /// Text vertical alignment
    pub text_valign: TextAlign,
}

impl TableCell {
    /// Create a new table cell
    pub fn new(column: usize, row: usize, text: impl Into<String>) -> Self {
        Self {
            column,
            row,
            text: text.into(),
            text_color: Color::new(255, 255, 255),
            bg_color: Color::new(0, 0, 0),
            text_size: TextSize::Normal,
            text_halign: TextAlign::Center,
            text_valign: TextAlign::Center,
        }
    }
}

/// A table drawing
#[derive(Debug, Clone)]
pub struct Table {
    /// Table ID
    pub id: DrawingId,
    /// Position on chart
    pub position: TablePosition,
    /// Number of columns
    pub columns: usize,
    /// Number of rows
    pub rows: usize,
    /// Cell contents (sparse storage)
    pub cells: Vec<TableCell>,
    /// Background color
    pub bgcolor: Color,
    /// Border color
    pub border_color: Color,
    /// Border width
    pub border_width: i32,
    /// Frame width
    pub frame_width: i32,
    /// Whether table is visible
    pub visible: bool,
}

impl Table {
    /// Create a new table
    pub fn new(id: DrawingId, position: TablePosition, columns: usize, rows: usize) -> Self {
        Self {
            id,
            position,
            columns,
            rows,
            cells: Vec::new(),
            bgcolor: Color::new(0, 0, 0),
            border_color: Color::new(128, 128, 128),
            border_width: 1,
            frame_width: 0,
            visible: true,
        }
    }

    /// Set a cell value
    pub fn set_cell(
        &mut self,
        column: usize,
        row: usize,
        text: impl Into<String>,
        text_color: Option<Color>,
        bg_color: Option<Color>,
        text_size: Option<TextSize>,
    ) -> Result<()> {
        if column >= self.columns || row >= self.rows {
            return Err(crate::OutputError::InvalidValue(format!(
                "Cell ({}, {}) is outside table bounds ({}, {})",
                column, row, self.columns, self.rows
            )));
        }

        // Find existing cell or create new
        if let Some(cell) = self
            .cells
            .iter_mut()
            .find(|c| c.column == column && c.row == row)
        {
            cell.text = text.into();
            if let Some(tc) = text_color {
                cell.text_color = tc;
            }
            if let Some(bc) = bg_color {
                cell.bg_color = bc;
            }
            if let Some(ts) = text_size {
                cell.text_size = ts;
            }
        } else {
            let mut cell = TableCell::new(column, row, text);
            if let Some(tc) = text_color {
                cell.text_color = tc;
            }
            if let Some(bc) = bg_color {
                cell.bg_color = bc;
            }
            if let Some(ts) = text_size {
                cell.text_size = ts;
            }
            self.cells.push(cell);
        }

        Ok(())
    }

    /// Get a cell
    pub fn get_cell(&self, column: usize, row: usize) -> Option<&TableCell> {
        self.cells
            .iter()
            .find(|c| c.column == column && c.row == row)
    }

    /// Set background color
    pub fn set_bgcolor(&mut self, color: Color) {
        self.bgcolor = color;
    }

    /// Set border color
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }

    /// Set border width
    pub fn set_border_width(&mut self, width: i32) {
        self.border_width = width;
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

/// Manager for table lifecycle
#[derive(Debug, Default)]
pub struct TableManager {
    /// All tables by ID
    tables: HashMap<DrawingId, Table>,
    /// Next table ID
    next_id: u64,
    /// Maximum number of tables allowed
    max_tables: usize,
}

impl TableManager {
    /// Create a new table manager
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            next_id: 1,
            max_tables: 100,
        }
    }

    /// Create a new table manager with custom limit
    pub fn with_max_tables(max_tables: usize) -> Self {
        Self {
            tables: HashMap::new(),
            next_id: 1,
            max_tables,
        }
    }

    /// Create a new table
    pub fn new_table(
        &mut self,
        position: TablePosition,
        columns: usize,
        rows: usize,
    ) -> Result<DrawingId> {
        if self.tables.len() >= self.max_tables {
            return Err(crate::OutputError::DrawingLimitExceeded {
                max: self.max_tables,
            });
        }

        let id = DrawingId::new(self.next_id);
        self.next_id += 1;

        let table = Table::new(id, position, columns, rows);
        self.tables.insert(id, table);

        Ok(id)
    }

    /// Get a table by ID
    pub fn get(&self, id: DrawingId) -> Option<&Table> {
        self.tables.get(&id)
    }

    /// Get a mutable table by ID
    pub fn get_mut(&mut self, id: DrawingId) -> Option<&mut Table> {
        self.tables.get_mut(&id)
    }

    /// Delete a table
    pub fn delete(&mut self, id: DrawingId) -> Result<()> {
        self.tables.remove(&id);
        Ok(())
    }

    /// Delete all tables
    pub fn delete_all(&mut self) {
        self.tables.clear();
    }

    /// Get count of tables
    pub fn count(&self) -> usize {
        self.tables.len()
    }
}

/// Table functions
pub mod table {
    use super::*;

    /// Create a new table
    pub fn new(
        manager: &mut TableManager,
        position: TablePosition,
        columns: usize,
        rows: usize,
        bgcolor: Option<Color>,
        border_color: Option<Color>,
        border_width: Option<i32>,
    ) -> Result<DrawingId> {
        let id = manager.new_table(position, columns, rows)?;

        if let Some(t) = manager.get_mut(id) {
            if let Some(c) = bgcolor {
                t.set_bgcolor(c);
            }
            if let Some(c) = border_color {
                t.set_border_color(c);
            }
            if let Some(w) = border_width {
                t.set_border_width(w);
            }
        }

        Ok(id)
    }

    /// Set table cell
    #[allow(clippy::too_many_arguments)]
    pub fn cell(
        manager: &mut TableManager,
        table_id: DrawingId,
        column: usize,
        row: usize,
        text: impl Into<String>,
        text_color: Option<Color>,
        bg_color: Option<Color>,
        text_size: Option<TextSize>,
    ) -> Result<()> {
        if let Some(t) = manager.get_mut(table_id) {
            t.set_cell(column, row, text, text_color, bg_color, text_size)
        } else {
            Err(crate::OutputError::InvalidDrawingId(format!(
                "Table {:?} not found",
                table_id
            )))
        }
    }

    /// Delete a table
    pub fn delete(manager: &mut TableManager, id: DrawingId) -> Result<()> {
        manager.delete(id)
    }

    /// Delete all tables
    pub fn delete_all(manager: &mut TableManager) {
        manager.delete_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_creation() {
        let mut manager = LabelManager::new();
        let id = manager.new_label(10, 100.0, "Test Label").unwrap();

        let label = manager.get(id).unwrap();
        assert_eq!(label.x, 10);
        assert_eq!(label.y, 100.0);
        assert_eq!(label.text, "Test Label");
        assert!(label.visible);
    }

    #[test]
    fn test_label_style_parsing() {
        use std::str::FromStr;

        assert_eq!(
            LabelStyle::from_str("label.style_labelup"),
            Ok(LabelStyle::LabelUp)
        );
        assert_eq!(
            LabelStyle::from_str("label.style_labeldown"),
            Ok(LabelStyle::LabelDown)
        );
        assert_eq!(
            LabelStyle::from_str("label.style_triangleup"),
            Ok(LabelStyle::TriangleUp)
        );
        assert_eq!(
            LabelStyle::from_str("label.style_arrowup"),
            Ok(LabelStyle::ArrowUp)
        );
        assert_eq!(LabelStyle::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_label_setters() {
        let mut manager = LabelManager::new();
        let id = manager.new_label(0, 0.0, "Initial").unwrap();

        // Test set_text
        label::set_text(&mut manager, id, "Updated").unwrap();
        assert_eq!(manager.get(id).unwrap().text, "Updated");

        // Test set_xy
        label::set_xy(&mut manager, id, 5, 50.0).unwrap();
        assert_eq!(manager.get(id).unwrap().x, 5);
        assert_eq!(manager.get(id).unwrap().y, 50.0);

        // Test set_color
        let red = Color::new(255, 0, 0);
        label::set_color(&mut manager, id, red).unwrap();
        assert_eq!(manager.get(id).unwrap().color, red);

        // Test set_textcolor
        let white = Color::new(255, 255, 255);
        label::set_textcolor(&mut manager, id, white).unwrap();
        assert_eq!(manager.get(id).unwrap().textcolor, white);

        // Test set_style
        label::set_style(&mut manager, id, LabelStyle::ArrowDown).unwrap();
        assert_eq!(manager.get(id).unwrap().style, LabelStyle::ArrowDown);

        // Test set_visible
        label::set_visible(&mut manager, id, false).unwrap();
        assert!(!manager.get(id).unwrap().visible);
    }

    #[test]
    fn test_label_delete() {
        let mut manager = LabelManager::new();
        let id = manager.new_label(0, 0.0, "To Delete").unwrap();

        assert!(manager.get(id).is_some());
        label::delete(&mut manager, id).unwrap();
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn test_label_limit() {
        let mut manager = LabelManager::with_max_labels(3);

        // Should be able to create 3 labels
        let _id1 = manager.new_label(0, 0.0, "Label 1").unwrap();
        let _id2 = manager.new_label(1, 1.0, "Label 2").unwrap();
        let _id3 = manager.new_label(2, 2.0, "Label 3").unwrap();

        assert_eq!(manager.count(), 3);

        // Fourth label should fail
        let result = manager.new_label(3, 3.0, "Label 4");
        assert!(result.is_err());
    }

    #[test]
    fn test_label_not_found() {
        let mut manager = LabelManager::new();
        let fake_id = DrawingId::new(999);

        let result = label::set_text(&mut manager, fake_id, "Test");
        assert!(result.is_err());
    }

    #[test]
    fn test_new_label_with_options() {
        let mut manager = LabelManager::new();

        let id = label::new(
            &mut manager,
            10,
            100.0,
            "Styled Label",
            Some(LabelStyle::ArrowUp),
            Some(Color::new(255, 0, 0)),
            Some(Color::new(0, 255, 0)),
            Some(TextSize::Large),
        )
        .unwrap();

        let label = manager.get(id).unwrap();
        assert_eq!(label.style, LabelStyle::ArrowUp);
        assert_eq!(label.color, Color::new(255, 0, 0));
        assert_eq!(label.textcolor, Color::new(0, 255, 0));
        assert_eq!(label.size, TextSize::Large);
    }

    #[test]
    fn test_box_creation() {
        let mut manager = BoxManager::new();
        let id = manager.new_box(0, 100.0, 10, 90.0).unwrap();

        let bx = manager.get(id).unwrap();
        assert_eq!(bx.left, 0);
        assert_eq!(bx.top, 100.0);
        assert_eq!(bx.right, 10);
        assert_eq!(bx.bottom, 90.0);
        assert!(bx.visible);
    }

    #[test]
    fn test_box_setters() {
        let mut manager = BoxManager::new();
        let id = manager.new_box(0, 0.0, 10, 10.0).unwrap();

        // Test set_lefttop
        r#box::set_lefttop(&mut manager, id, 5, 50.0).unwrap();
        assert_eq!(manager.get(id).unwrap().left, 5);
        assert_eq!(manager.get(id).unwrap().top, 50.0);

        // Test set_rightbottom
        r#box::set_rightbottom(&mut manager, id, 15, 40.0).unwrap();
        assert_eq!(manager.get(id).unwrap().right, 15);
        assert_eq!(manager.get(id).unwrap().bottom, 40.0);

        // Test set_border_color
        let red = Color::new(255, 0, 0);
        r#box::set_border_color(&mut manager, id, red).unwrap();
        assert_eq!(manager.get(id).unwrap().border_color, red);

        // Test set_bgcolor
        let blue = Color::new(0, 0, 255);
        r#box::set_bgcolor(&mut manager, id, blue).unwrap();
        assert_eq!(manager.get(id).unwrap().bgcolor, blue);

        // Test set_border_width
        r#box::set_border_width(&mut manager, id, 2).unwrap();
        assert_eq!(manager.get(id).unwrap().border_width, 2);
    }

    #[test]
    fn test_box_delete() {
        let mut manager = BoxManager::new();
        let id = manager.new_box(0, 0.0, 10, 10.0).unwrap();

        assert!(manager.get(id).is_some());
        r#box::delete(&mut manager, id).unwrap();
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn test_box_limit() {
        let mut manager = BoxManager::with_max_boxes(3);

        // Should be able to create 3 boxes
        let _id1 = manager.new_box(0, 0.0, 1, 1.0).unwrap();
        let _id2 = manager.new_box(1, 1.0, 2, 2.0).unwrap();
        let _id3 = manager.new_box(2, 2.0, 3, 3.0).unwrap();

        assert_eq!(manager.count(), 3);

        // Fourth box should fail
        let result = manager.new_box(3, 3.0, 4, 4.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_box_not_found() {
        let mut manager = BoxManager::new();
        let fake_id = DrawingId::new(999);

        let result = r#box::set_lefttop(&mut manager, fake_id, 0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_box_new_with_options() {
        let mut manager = BoxManager::new();

        let id = r#box::new(
            &mut manager,
            0,
            100.0,
            10,
            90.0,
            Some(Color::new(255, 0, 0)),
            Some(2),
            Some(Color::new(0, 255, 0)),
        )
        .unwrap();

        let bx = manager.get(id).unwrap();
        assert_eq!(bx.border_color, Color::new(255, 0, 0));
        assert_eq!(bx.border_width, 2);
        assert_eq!(bx.bgcolor, Color::new(0, 255, 0));
    }

    #[test]
    fn test_table_creation() {
        let mut manager = TableManager::new();
        let id = manager.new_table(TablePosition::TopRight, 3, 2).unwrap();

        let table = manager.get(id).unwrap();
        assert_eq!(table.position, TablePosition::TopRight);
        assert_eq!(table.columns, 3);
        assert_eq!(table.rows, 2);
        assert!(table.visible);
    }

    #[test]
    fn test_table_position_parsing() {
        use std::str::FromStr;

        assert_eq!(
            TablePosition::from_str("position.top_left"),
            Ok(TablePosition::TopLeft)
        );
        assert_eq!(
            TablePosition::from_str("position.top_center"),
            Ok(TablePosition::TopCenter)
        );
        assert_eq!(
            TablePosition::from_str("position.top_right"),
            Ok(TablePosition::TopRight)
        );
        assert_eq!(
            TablePosition::from_str("position.bottom_right"),
            Ok(TablePosition::BottomRight)
        );
        assert_eq!(TablePosition::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_table_cell() {
        let mut manager = TableManager::new();
        let id = manager.new_table(TablePosition::TopRight, 3, 2).unwrap();

        // Set a cell
        table::cell(
            &mut manager,
            id,
            0,
            0,
            "Header",
            Some(Color::new(255, 255, 255)),
            Some(Color::new(0, 0, 255)),
            Some(TextSize::Large),
        )
        .unwrap();

        let table = manager.get(id).unwrap();
        let cell = table.get_cell(0, 0).unwrap();
        assert_eq!(cell.text, "Header");
        assert_eq!(cell.text_color, Color::new(255, 255, 255));
        assert_eq!(cell.bg_color, Color::new(0, 0, 255));
        assert_eq!(cell.text_size, TextSize::Large);
    }

    #[test]
    fn test_table_cell_out_of_bounds() {
        let mut manager = TableManager::new();
        let id = manager.new_table(TablePosition::TopRight, 2, 2).unwrap();

        // Try to set cell outside bounds
        let result = table::cell(&mut manager, id, 5, 5, "Out", None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_delete() {
        let mut manager = TableManager::new();
        let id = manager.new_table(TablePosition::TopRight, 3, 2).unwrap();

        assert!(manager.get(id).is_some());
        table::delete(&mut manager, id).unwrap();
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn test_table_limit() {
        let mut manager = TableManager::with_max_tables(3);

        // Should be able to create 3 tables
        let _id1 = manager.new_table(TablePosition::TopLeft, 2, 2).unwrap();
        let _id2 = manager.new_table(TablePosition::TopCenter, 2, 2).unwrap();
        let _id3 = manager.new_table(TablePosition::TopRight, 2, 2).unwrap();

        assert_eq!(manager.count(), 3);

        // Fourth table should fail
        let result = manager.new_table(TablePosition::BottomLeft, 2, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_not_found() {
        let mut manager = TableManager::new();
        let fake_id = DrawingId::new(999);

        let result = table::cell(&mut manager, fake_id, 0, 0, "Test", None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_new_with_options() {
        let mut manager = TableManager::new();

        let id = table::new(
            &mut manager,
            TablePosition::MiddleCenter,
            4,
            3,
            Some(Color::new(0, 0, 0)),
            Some(Color::new(255, 255, 255)),
            Some(2),
        )
        .unwrap();

        let table = manager.get(id).unwrap();
        assert_eq!(table.position, TablePosition::MiddleCenter);
        assert_eq!(table.columns, 4);
        assert_eq!(table.rows, 3);
        assert_eq!(table.bgcolor, Color::new(0, 0, 0));
        assert_eq!(table.border_color, Color::new(255, 255, 255));
        assert_eq!(table.border_width, 2);
    }
}
