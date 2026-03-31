//! Runtime value types for Pine Script v6
//!
//! This module defines the core Value enum and NA propagation rules.
//! All arithmetic and comparison operations must go through `na_ops` module.

use std::fmt;
use std::sync::Arc;

/// A color value in Pine Script (RGBA)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255, 255 = opaque)
    pub a: u8,
}

impl Color {
    /// Create a new color with full opacity
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color with alpha
    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a color from a hex string (#RRGGBB or #RRGGBBAA)
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#')?;
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::with_alpha(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Function parameter kind for closures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// Simple parameter
    Simple,
    /// Series parameter (inherits call-site series)
    Series,
}

/// A closure (user-defined function)
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    /// Function name (for debugging)
    pub name: String,
    /// Parameter names and kinds
    pub params: Vec<(String, ParamKind)>,
    /// Whether the function returns a series
    pub returns_series: bool,
    // TODO: Add function body reference (AST node or bytecode)
}

impl Closure {
    /// Create a new closure
    pub fn new(name: impl Into<String>, params: Vec<(String, ParamKind)>) -> Self {
        Self {
            name: name.into(),
            params,
            returns_series: false,
        }
    }

    /// Mark this function as returning a series
    pub fn with_series_return(mut self) -> Self {
        self.returns_series = true;
        self
    }
}

/// Pine Script runtime value
///
/// This enum represents all possible values in Pine Script v6.
/// Note that `Series<T>` is stored separately in `SeriesBuf`, not here.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    /// Integer value (64-bit signed)
    Int(i64),
    /// Float value (64-bit)
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// String value (using smartstring for small string optimization)
    String(smartstring::alias::String),
    /// Color value (RGBA)
    Color(Color),
    /// NA (not available) value - propagates through all operations
    #[default]
    Na,
    /// Array value (boxed slice for cache efficiency)
    Array(Box<[Value]>),
    /// Tuple value (for multiple return values)
    Tuple(Box<[Value]>),
    /// User-defined function
    Closure(Arc<Closure>),
}

impl Value {
    //==========================================================================
    // Type checking methods
    //==========================================================================

    /// Check if this value is NA
    #[inline]
    pub fn is_na(&self) -> bool {
        matches!(self, Value::Na)
    }

    /// Check if this value is a number (Int or Float, but not NA)
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    /// Check if this value is an integer
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    /// Check if this value is a float
    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Check if this value is a boolean
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Check if this value is a string
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if this value is a color
    #[inline]
    pub fn is_color(&self) -> bool {
        matches!(self, Value::Color(_))
    }

    /// Check if this value is an array
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Check if this value is a tuple
    #[inline]
    pub fn is_tuple(&self) -> bool {
        matches!(self, Value::Tuple(_))
    }

    /// Check if this value is a closure
    #[inline]
    pub fn is_closure(&self) -> bool {
        matches!(self, Value::Closure(_))
    }

    /// Check if this value is truthy (used in conditions)
    ///
    /// Pine Script truthiness rules:
    /// - `na` is falsey
    /// - `false` is falsey
    /// - `0` and `0.0` are truthy (unlike JavaScript!)
    /// - empty strings are truthy
    /// - empty arrays are truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Na => false,
            Value::Bool(b) => *b,
            _ => true, // Everything else (including 0) is truthy
        }
    }

    //==========================================================================
    // Conversion methods (return Option for safe access)
    //==========================================================================

    /// Get the value as i64, or None if not an integer
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Get the value as f64, or None if not a number
    ///
    /// Integers are automatically converted to floats.
    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Get the value as bool, or None if not a boolean
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the value as string slice, or None if not a string
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get the value as Color, or None if not a color
    #[inline]
    pub fn as_color(&self) -> Option<Color> {
        match self {
            Value::Color(c) => Some(*c),
            _ => None,
        }
    }

    /// Get the value as array slice, or None if not an array
    #[inline]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Get the value as mutable array, or None if not an array
    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut [Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Get the value as tuple slice, or None if not a tuple
    #[inline]
    pub fn as_tuple(&self) -> Option<&[Value]> {
        match self {
            Value::Tuple(t) => Some(t),
            _ => None,
        }
    }

    /// Get the value as closure, or None if not a closure
    #[inline]
    pub fn as_closure(&self) -> Option<&Closure> {
        match self {
            Value::Closure(c) => Some(c),
            _ => None,
        }
    }

    //==========================================================================
    // NA propagation helpers (used by na_ops module)
    //==========================================================================

    /// Apply a binary operation, propagating NA
    ///
    /// If either operand is NA, returns NA.
    /// Otherwise, applies the given function.
    pub fn binary_op<F>(lhs: &Value, rhs: &Value, op: F) -> Value
    where
        F: FnOnce(&Value, &Value) -> Value,
    {
        if lhs.is_na() || rhs.is_na() {
            Value::Na
        } else {
            op(lhs, rhs)
        }
    }

    /// Apply a unary operation, propagating NA
    ///
    /// If the operand is NA, returns NA.
    /// Otherwise, applies the given function.
    pub fn unary_op<F>(val: &Value, op: F) -> Value
    where
        F: FnOnce(&Value) -> Value,
    {
        if val.is_na() {
            Value::Na
        } else {
            op(val)
        }
    }

    /// Apply a numeric binary operation, propagating NA
    ///
    /// If either operand is NA, returns NA.
    /// If both operands are numbers, applies the operation.
    /// Otherwise, returns NA (no implicit conversion).
    pub fn numeric_op<F>(lhs: &Value, rhs: &Value, op: F) -> Value
    where
        F: FnOnce(f64, f64) -> f64,
    {
        if lhs.is_na() || rhs.is_na() {
            return Value::Na;
        }
        match (lhs.as_float(), rhs.as_float()) {
            (Some(l), Some(r)) => Value::Float(op(l, r)),
            _ => Value::Na,
        }
    }

    /// Apply a comparison operation, propagating NA
    ///
    /// IMPORTANT: In Pine Script, `na == na` returns `false` (like NaN in IEEE 754)
    /// and `na != na` returns `true`.
    pub fn comparison_op<F>(lhs: &Value, rhs: &Value, op: F) -> Value
    where
        F: FnOnce(&Value, &Value) -> bool,
    {
        // NA propagation: if either is NA, comparison returns false (which becomes Bool(false))
        // But wait - in Pine Script:
        // - na == na -> false
        // - na != na -> true
        // - na == 1 -> false
        // - na < 1 -> false
        // So for equality, we need special handling
        if lhs.is_na() || rhs.is_na() {
            // The op will be called to determine the result
            // For ==, this returns false
            // For !=, this returns true
            Value::Bool(op(lhs, rhs))
        } else {
            Value::Bool(op(lhs, rhs))
        }
    }

    //==========================================================================
    // Type coercion helpers
    //==========================================================================

    /// Coerce this value to a float, returning NA if not possible
    pub fn coerce_to_float(&self) -> Value {
        match self {
            Value::Float(_) => self.clone(),
            Value::Int(i) => Value::Float(*i as f64),
            Value::Na => Value::Na,
            _ => Value::Na,
        }
    }

    /// Coerce this value to an int, returning NA if not possible
    pub fn coerce_to_int(&self) -> Value {
        match self {
            Value::Int(_) => self.clone(),
            Value::Float(f) => Value::Int(*f as i64),
            Value::Na => Value::Na,
            _ => Value::Na,
        }
    }

    /// Coerce this value to a string
    pub fn coerce_to_string(&self) -> Value {
        match self {
            Value::String(_) => self.clone(),
            Value::Int(i) => Value::String(i.to_string().into()),
            Value::Float(f) => Value::String(f.to_string().into()),
            Value::Bool(b) => Value::String(b.to_string().into()),
            Value::Color(c) => Value::String(c.to_hex().into()),
            Value::Na => Value::String("na".into()),
            Value::Array(_) => Value::String("array".into()),
            Value::Tuple(_) => Value::String("tuple".into()),
            Value::Closure(c) => Value::String(format!("fn {}", c.name).into()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(v) => {
                // Format without trailing zeros
                if v.is_finite() {
                    let s = format!("{v:.10}")
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string();
                    write!(f, "{s}")
                } else {
                    write!(f, "na")
                }
            }
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Color(c) => write!(f, "{c}"),
            Value::Na => write!(f, "na"),
            Value::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, ")")
            }
            Value::Closure(c) => write!(f, "fn {}", c.name),
        }
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v as i64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        // Treat NaN and Infinity as NA
        if v.is_finite() {
            Value::Float(v)
        } else {
            Value::Na
        }
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::from(v as f64)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v.into())
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.into())
    }
}

impl From<Color> for Value {
    fn from(v: Color) -> Self {
        Value::Color(v)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v.into_boxed_slice())
    }
}

impl From<Closure> for Value {
    fn from(v: Closure) -> Self {
        Value::Closure(Arc::new(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_na() {
        assert!(Value::Na.is_na());
        assert!(!Value::Int(1).is_na());
        assert!(!Value::Float(1.0).is_na());
    }

    #[test]
    fn test_truthiness() {
        // NA is falsey
        assert!(!Value::Na.is_truthy());
        // false is falsey
        assert!(!Value::Bool(false).is_truthy());
        // true is truthy
        assert!(Value::Bool(true).is_truthy());
        // 0 is truthy (unlike most languages!)
        assert!(Value::Int(0).is_truthy());
        assert!(Value::Float(0.0).is_truthy());
        // Everything else is truthy
        assert!(Value::Int(1).is_truthy());
        assert!(Value::String("".into()).is_truthy());
    }

    #[test]
    fn test_na_propagation() {
        // Binary operations with NA should return NA
        let na = Value::Na;
        let one = Value::Int(1);

        let result = Value::binary_op(&na, &one, |_, _| Value::Int(42));
        assert!(result.is_na());

        let result = Value::binary_op(&one, &na, |_, _| Value::Int(42));
        assert!(result.is_na());

        // Unary operations with NA should return NA
        let result = Value::unary_op(&na, |_| Value::Int(42));
        assert!(result.is_na());
    }

    #[test]
    fn test_numeric_op() {
        let a = Value::Float(3.0);
        let b = Value::Float(2.0);

        let sum = Value::numeric_op(&a, &b, |x, y| x + y);
        assert_eq!(sum, Value::Float(5.0));

        // With NA
        let sum = Value::numeric_op(&a, &Value::Na, |x, y| x + y);
        assert!(sum.is_na());
    }

    #[test]
    fn test_type_conversions() {
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::Int(42).as_float(), Some(42.0));
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::String("hello".into()).as_str(), Some("hello"));
    }

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex("#FF5733").unwrap();
        assert_eq!(c, Color::new(0xFF, 0x57, 0x33));

        let c = Color::from_hex("#FF5733CC").unwrap();
        assert_eq!(c, Color::with_alpha(0xFF, 0x57, 0x33, 0xCC));

        assert!(Color::from_hex("FF5733").is_none()); // missing #
        assert!(Color::from_hex("#GG5733").is_none()); // invalid hex
    }

    #[test]
    fn test_color_to_hex() {
        let c = Color::new(255, 87, 51);
        assert_eq!(c.to_hex(), "#ff5733");

        let c = Color::with_alpha(255, 87, 51, 204);
        assert_eq!(c.to_hex(), "#ff5733cc");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::Float(3.5)), "3.5");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Na), "na");
        assert_eq!(
            format!(
                "{}",
                Value::Array(vec![Value::Int(1), Value::Int(2)].into())
            ),
            "[1, 2]"
        );
    }

    #[test]
    fn test_from_impls() {
        assert_eq!(Value::from(42i64), Value::Int(42));
        assert_eq!(Value::from(3.14f64), Value::Float(3.14));
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from("hello"), Value::String("hello".into()));

        // Non-finite floats become NA
        assert!(Value::from(f64::NAN).is_na());
        assert!(Value::from(f64::INFINITY).is_na());
    }
}
