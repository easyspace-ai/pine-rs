//! Runtime value types

use serde::{Deserialize, Serialize};

/// Pine Script runtime value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// String value
    String(String),
    /// NA (not available) value
    Na,
}

impl Value {
    /// Check if this value is NA
    pub fn is_na(&self) -> bool {
        matches!(self, Value::Na)
    }

    /// Get the value as float, or None if NA
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
}
