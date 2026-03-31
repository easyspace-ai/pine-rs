//! Runtime value types for Pine Script v6
//!
//! This module defines the core Value enum and NA propagation rules.
//! All arithmetic and comparison operations must go through `na_ops` module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// A color value in Pine Script (RGBA)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// A user-defined type (UDT) object
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    /// Type name
    pub type_name: String,
    /// Field values
    pub fields: HashMap<String, Value>,
}

impl Object {
    /// Create a new object of the given type
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
            fields: HashMap::new(),
        }
    }

    /// Create a new object with pre-populated fields
    pub fn with_fields(
        type_name: impl Into<String>,
        fields: impl IntoIterator<Item = (String, Value)>,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            fields: fields.into_iter().collect(),
        }
    }

    /// Get a field value
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.fields.get(field)
    }

    /// Set a field value
    pub fn set(&mut self, field: impl Into<String>, value: Value) {
        self.fields.insert(field.into(), value);
    }
}

/// A map (dictionary) with string keys
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Map {
    /// Key-value pairs
    pub entries: HashMap<String, Value>,
}

impl Map {
    /// Create a new empty map
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Create a new map with pre-populated entries
    pub fn with_entries(entries: impl IntoIterator<Item = (String, Value)>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries.get(key)
    }

    /// Set a key-value pair
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.entries.insert(key.into(), value);
    }

    /// Remove a key and return its value
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.entries.remove(key)
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get an iterator over keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Get an iterator over values
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.entries.values()
    }

    /// Get an iterator over entries
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.entries.iter()
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
    /// Array value (mutable vector for dynamic operations)
    Array(Vec<Value>),
    /// Matrix value (2D vector for matrix operations)
    Matrix {
        /// Number of rows
        rows: usize,
        /// Number of columns
        cols: usize,
        /// Flat data storage (row-major order)
        data: Vec<Value>,
    },
    /// Tuple value (for multiple return values)
    Tuple(Box<[Value]>),
    /// User-defined function
    Closure(Arc<Closure>),
    /// User-defined type (UDT) object
    Object(Arc<Object>),
    /// Map/dictionary with string keys
    Map(Arc<Map>),
}

impl Value {
    //==========================================================================
    // Constructors
    //==========================================================================

    /// Create a new matrix with the given dimensions and default value
    pub fn new_matrix(rows: usize, cols: usize, default: Value) -> Self {
        let data = vec![default; rows * cols];
        Value::Matrix { rows, cols, data }
    }

    /// Create a new empty map
    pub fn new_map() -> Self {
        Value::Map(Arc::new(Map::new()))
    }

    /// Create a new map with pre-populated entries
    pub fn new_map_with_entries(entries: impl IntoIterator<Item = (String, Value)>) -> Self {
        Value::Map(Arc::new(Map::with_entries(entries)))
    }

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

    /// Check if this value is a matrix
    #[inline]
    pub fn is_matrix(&self) -> bool {
        matches!(self, Value::Matrix { .. })
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

    /// Check if this value is an object (UDT)
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Check if this value is a map
    #[inline]
    pub fn is_map(&self) -> bool {
        matches!(self, Value::Map(_))
    }

    /// Check if this value is an object of a specific type
    #[inline]
    pub fn is_object_of_type(&self, type_name: &str) -> bool {
        match self {
            Value::Object(obj) => obj.type_name == type_name,
            _ => false,
        }
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
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Get the matrix data, rows, and cols
    #[inline]
    pub fn as_matrix(&self) -> Option<(usize, usize, &[Value])> {
        match self {
            Value::Matrix { rows, cols, data } => Some((*rows, *cols, data)),
            _ => None,
        }
    }

    /// Get mutable access to matrix data
    #[inline]
    pub fn as_matrix_mut(&mut self) -> Option<(usize, usize, &mut Vec<Value>)> {
        match self {
            Value::Matrix { rows, cols, data } => Some((*rows, *cols, data)),
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

    /// Get the value as object, or None if not an object
    #[inline]
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Get the value as mutable object, or None if not an object
    #[inline]
    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match self {
            Value::Object(o) => Some(Arc::make_mut(o)),
            _ => None,
        }
    }

    /// Get the value as map, or None if not a map
    #[inline]
    pub fn as_map(&self) -> Option<&Map> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Get the value as mutable map, or None if not a map
    #[inline]
    pub fn as_map_mut(&mut self) -> Option<&mut Map> {
        match self {
            Value::Map(m) => Some(Arc::make_mut(m)),
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
            Value::Matrix { .. } => Value::String("matrix".into()),
            Value::Tuple(_) => Value::String("tuple".into()),
            Value::Closure(c) => Value::String(format!("fn {}", c.name).into()),
            Value::Object(obj) => Value::String(obj.type_name.clone().into()),
            Value::Map(_) => Value::String("map".into()),
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
            Value::Matrix { rows, cols, data } => {
                write!(f, "matrix({rows}x{cols}) [")?;
                for (i, v) in data.iter().enumerate() {
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
            Value::Object(obj) => {
                write!(f, "{} {{", obj.type_name)?;
                for (i, (name, value)) in obj.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {value}")?;
                }
                write!(f, "}}")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (key, value)) in m.entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                }
                write!(f, "}}")
            }
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
        Value::Array(v)
    }
}

impl From<Closure> for Value {
    fn from(v: Closure) -> Self {
        Value::Closure(Arc::new(v))
    }
}

impl From<Object> for Value {
    fn from(v: Object) -> Self {
        Value::Object(Arc::new(v))
    }
}

impl From<Map> for Value {
    fn from(v: Map) -> Self {
        Value::Map(Arc::new(v))
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
            format!("{}", Value::Array(vec![Value::Int(1), Value::Int(2)])),
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

    #[test]
    fn test_object() {
        // Create an object
        let mut obj = Object::new("Point");
        obj.set("x", Value::Int(10));
        obj.set("y", Value::Int(20));

        // Test getters
        assert_eq!(obj.get("x"), Some(&Value::Int(10)));
        assert_eq!(obj.get("y"), Some(&Value::Int(20)));
        assert_eq!(obj.get("z"), None);

        // Convert to Value
        let value: Value = obj.into();
        assert!(value.is_object());
        assert!(value.is_object_of_type("Point"));
        assert!(!value.is_object_of_type("Line"));

        // Test as_object
        let obj_ref = value.as_object().unwrap();
        assert_eq!(obj_ref.type_name, "Point");
        assert_eq!(obj_ref.get("x"), Some(&Value::Int(10)));

        // Test object display
        let obj = Object::with_fields(
            "Point",
            vec![
                ("x".to_string(), Value::Int(1)),
                ("y".to_string(), Value::Int(2)),
            ],
        );
        let value: Value = obj.into();
        let display_str = format!("{}", value);
        assert!(display_str.starts_with("Point {"));
        assert!(display_str.contains("x: 1"));
        assert!(display_str.contains("y: 2"));
    }

    #[test]
    fn test_matrix() {
        // Create a 2x3 matrix with default value 0
        let matrix = Value::new_matrix(2, 3, Value::Int(0));
        assert!(matrix.is_matrix());

        // Test as_matrix
        let (rows, cols, data) = matrix.as_matrix().unwrap();
        assert_eq!(rows, 2);
        assert_eq!(cols, 3);
        assert_eq!(data.len(), 6);
        assert_eq!(data[0], Value::Int(0));

        // Test matrix display
        let display_str = format!("{}", matrix);
        assert!(display_str.starts_with("matrix(2x3) ["));
    }

    #[test]
    fn test_map() {
        // Create a map
        let mut map = Map::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        // Set entries
        map.set("a", Value::Int(1));
        map.set("b", Value::Float(2.5));
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());

        // Test get
        assert_eq!(map.get("a"), Some(&Value::Int(1)));
        assert_eq!(map.get("b"), Some(&Value::Float(2.5)));
        assert_eq!(map.get("c"), None);

        // Test contains_key
        assert!(map.contains_key("a"));
        assert!(!map.contains_key("c"));

        // Test remove
        assert_eq!(map.remove("a"), Some(Value::Int(1)));
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key("a"));

        // Test clear
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn test_map_value() {
        // Create a map value
        let mut map_value = Value::new_map();
        assert!(map_value.is_map());

        // Modify through as_map_mut
        if let Some(map) = map_value.as_map_mut() {
            map.set("x", Value::Int(10));
            map.set("y", Value::Int(20));
        }

        // Check the modifications
        if let Some(map) = map_value.as_map() {
            assert_eq!(map.get("x"), Some(&Value::Int(10)));
            assert_eq!(map.get("y"), Some(&Value::Int(20)));
        }

        // Test map display
        let display_str = format!("{}", map_value);
        assert!(display_str.starts_with("{"));
        assert!(display_str.contains("x: 10"));
        assert!(display_str.contains("y: 20"));

        // Test with_entries constructor
        let entries = vec![
            ("name".to_string(), Value::String("test".into())),
            ("value".to_string(), Value::Int(42)),
        ];
        let map_value = Value::new_map_with_entries(entries);
        assert!(map_value.is_map());
    }
}
