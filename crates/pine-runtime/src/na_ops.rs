//! NA propagation operations
//!
//! All arithmetic and comparison operations must go through this module
//! to ensure proper NA propagation semantics.

use crate::value::Value;

/// Add two values with NA propagation
pub fn add(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a + *b as f64),
        _ => Value::Na,
    }
}

/// Subtract two values with NA propagation
pub fn sub(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 - b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a - *b as f64),
        _ => Value::Na,
    }
}

/// Multiply two values with NA propagation
pub fn mul(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 * b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a * *b as f64),
        _ => Value::Na,
    }
}

/// Divide two values with NA propagation
pub fn div(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (_, Value::Int(0)) => Value::Na,
        (_, Value::Float(b)) if *b == 0.0 => Value::Na,
        (Value::Int(a), Value::Int(b)) => Value::Float(*a as f64 / *b as f64),
        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 / b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a / *b as f64),
        _ => Value::Na,
    }
}

/// Check if a value is NA
pub fn is_na(value: &Value) -> bool {
    value.is_na()
}

/// Replace NA with a default value
pub fn nz(value: &Value, default: &Value) -> Value {
    if value.is_na() {
        default.clone()
    } else {
        value.clone()
    }
}
