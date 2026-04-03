//! Math functions (math.*)
//!
//! This module provides Pine Script v6 compatible mathematical functions.
//! All functions follow TradingView's exact semantics including NA handling.

use crate::registry::{FunctionMeta, FunctionRegistry};
use pine_builtin_macro::pine_builtin;
use pine_runtime::value::Value;
use std::sync::Arc;

/// Register all math.* functions with the registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    // Basic arithmetic
    register_abs(registry);
    register_max(registry);
    register_min(registry);
    register_sum(registry);
    register_avg(registry);

    // Power and roots
    register_sqrt(registry);
    register_pow(registry);
    register_exp(registry);
    register_log(registry);
    register_log10(registry);

    // Trigonometric functions
    register_sin(registry);
    register_cos(registry);
    register_tan(registry);
    register_asin(registry);
    register_acos(registry);
    register_atan(registry);

    // Hyperbolic functions
    register_sinh(registry);
    register_cosh(registry);
    register_tanh(registry);

    // Rounding functions
    register_round(registry);
    register_round_to_nearest(registry);
    register_ceil(registry);
    register_floor(registry);
    register_trunc(registry);

    // Sign and comparison
    register_sign(registry);
    register_copysign(registry);

    // Other functions
    register_isna(registry);
    register_nz(registry);
    register_tostring(registry);
}

// ============================================================================
// Helper functions
// ============================================================================

/// Get float value from Value
fn get_float(value: &Value) -> Option<f64> {
    match value {
        Value::Float(f) => Some(*f),
        Value::Int(n) => Some(*n as f64),
        _ => None,
    }
}

// ============================================================================
// Basic Arithmetic
// ============================================================================

#[pine_builtin(name = "abs", namespace = "math", required_args = 1)]
fn builtin_math_abs(args: &[Value]) -> Value {
    match args.first() {
        Some(Value::Float(f)) => Value::Float(f.abs()),
        Some(Value::Int(n)) => Value::Int(n.abs()),
        _ => Value::Na,
    }
}

/// Register math.max - Maximum of two or more values
fn register_max(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("max")
        .with_namespace("math")
        .with_required_args(2)
        .with_variadic();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let mut max_val: f64 = f64::NEG_INFINITY;
        let mut found = false;

        for arg in args {
            if let Some(v) = get_float(arg) {
                if !found || v > max_val {
                    max_val = v;
                    found = true;
                }
            }
        }

        if found {
            Value::Float(max_val)
        } else {
            Value::Na
        }
    });

    registry.register(meta, func);
}

/// Register math.min - Minimum of two or more values
fn register_min(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("min")
        .with_namespace("math")
        .with_required_args(2)
        .with_variadic();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let mut min_val: f64 = f64::INFINITY;
        let mut found = false;

        for arg in args {
            if let Some(v) = get_float(arg) {
                if !found || v < min_val {
                    min_val = v;
                    found = true;
                }
            }
        }

        if found {
            Value::Float(min_val)
        } else {
            Value::Na
        }
    });

    registry.register(meta, func);
}

/// Register math.sum - Rolling sum of series values
///
/// Pine Script: `math.sum(source, length)`
/// Returns the rolling sum of `source` over `length` bars.
/// First (length-1) bars return na.
fn register_sum(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sum")
        .with_namespace("math")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // Extract series
        let series = match args.first() {
            Some(Value::Array(arr)) => arr.as_slice(),
            _ => return Value::Na,
        };

        // Extract length parameter
        let length = match args.get(1) {
            Some(Value::Int(n)) => (*n).max(1) as usize,
            Some(Value::Float(f)) => (*f as i64).max(1) as usize,
            _ => return Value::Na,
        };

        // Need at least `length` values to compute sum
        if series.len() < length {
            return Value::Na;
        }

        // Sum the trailing window
        let window = &series[series.len() - length..];
        let sum: f64 = window.iter().filter_map(get_float).sum();

        // If no valid values found, return Na
        if window.iter().filter_map(get_float).count() == 0 {
            return Value::Na;
        }

        Value::Float(sum)
    });

    registry.register(meta, func);
}

/// Register math.avg - Rolling average of series values
///
/// Pine Script: `math.avg(source, length)`
/// Returns the rolling average of `source` over `length` bars.
/// First (length-1) bars return na.
fn register_avg(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("avg")
        .with_namespace("math")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // Extract series
        let series = match args.first() {
            Some(Value::Array(arr)) => arr.as_slice(),
            _ => return Value::Na,
        };

        // Extract length parameter
        let length = match args.get(1) {
            Some(Value::Int(n)) => (*n).max(1) as usize,
            Some(Value::Float(f)) => (*f as i64).max(1) as usize,
            _ => return Value::Na,
        };

        // Need at least `length` values to compute average
        if series.len() < length {
            return Value::Na;
        }

        // Average the trailing window
        let window = &series[series.len() - length..];
        let valid_values: Vec<f64> = window.iter().filter_map(get_float).collect();

        if valid_values.is_empty() {
            return Value::Na;
        }

        let sum: f64 = valid_values.iter().sum();
        Value::Float(sum / valid_values.len() as f64)
    });

    registry.register(meta, func);
}

// ============================================================================
// Power and Roots
// ============================================================================

#[pine_builtin(name = "sqrt", namespace = "math", required_args = 1)]
fn builtin_math_sqrt(args: &[Value]) -> Value {
    match args.first() {
        Some(Value::Float(f)) => Value::Float(f.sqrt()),
        Some(Value::Int(n)) => Value::Float((*n as f64).sqrt()),
        _ => Value::Na,
    }
}

/// Register math.pow - Power function
fn register_pow(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("pow")
        .with_namespace("math")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let base = args.first().and_then(get_float);
        let exp = args.get(1).and_then(get_float);

        match (base, exp) {
            (Some(b), Some(e)) => Value::Float(b.powf(e)),
            _ => Value::Na,
        }
    });

    registry.register(meta, func);
}

#[pine_builtin(name = "exp", namespace = "math", required_args = 1)]
fn builtin_math_exp(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.exp()),
        None => Value::Na,
    }
}

#[pine_builtin(name = "log", namespace = "math", required_args = 1)]
fn builtin_math_log(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) if v > 0.0 => Value::Float(v.ln()),
        _ => Value::Na,
    }
}

#[pine_builtin(name = "log10", namespace = "math", required_args = 1)]
fn builtin_math_log10(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) if v > 0.0 => Value::Float(v.log10()),
        _ => Value::Na,
    }
}

// ============================================================================
// Trigonometric Functions
// ============================================================================

#[pine_builtin(name = "sin", namespace = "math", required_args = 1)]
fn builtin_math_sin(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.sin()),
        None => Value::Na,
    }
}

#[pine_builtin(name = "cos", namespace = "math", required_args = 1)]
fn builtin_math_cos(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.cos()),
        None => Value::Na,
    }
}

#[pine_builtin(name = "tan", namespace = "math", required_args = 1)]
fn builtin_math_tan(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.tan()),
        None => Value::Na,
    }
}

#[pine_builtin(name = "asin", namespace = "math", required_args = 1)]
fn builtin_math_asin(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) if (-1.0..=1.0).contains(&v) => Value::Float(v.asin()),
        _ => Value::Na,
    }
}

#[pine_builtin(name = "acos", namespace = "math", required_args = 1)]
fn builtin_math_acos(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) if (-1.0..=1.0).contains(&v) => Value::Float(v.acos()),
        _ => Value::Na,
    }
}

#[pine_builtin(name = "atan", namespace = "math", required_args = 1)]
fn builtin_math_atan(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.atan()),
        None => Value::Na,
    }
}

// ============================================================================
// Hyperbolic Functions
// ============================================================================

#[pine_builtin(name = "sinh", namespace = "math", required_args = 1)]
fn builtin_math_sinh(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.sinh()),
        None => Value::Na,
    }
}

#[pine_builtin(name = "cosh", namespace = "math", required_args = 1)]
fn builtin_math_cosh(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.cosh()),
        None => Value::Na,
    }
}

#[pine_builtin(name = "tanh", namespace = "math", required_args = 1)]
fn builtin_math_tanh(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Float(v.tanh()),
        None => Value::Na,
    }
}

// ============================================================================
// Rounding Functions
// ============================================================================

#[pine_builtin(name = "round", namespace = "math", required_args = 1)]
fn builtin_math_round(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Int(v.round() as i64),
        None => Value::Na,
    }
}

/// Register math.round_to_nearest - Round to nearest multiple
fn register_round_to_nearest(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("round_to_nearest")
        .with_namespace("math")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let value = args.first().and_then(get_float);
        let precision = args.get(1).and_then(get_float);

        match (value, precision) {
            (Some(v), Some(p)) if p != 0.0 => Value::Float((v / p).round() * p),
            _ => Value::Na,
        }
    });

    registry.register(meta, func);
}

#[pine_builtin(name = "ceil", namespace = "math", required_args = 1)]
fn builtin_math_ceil(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Int(v.ceil() as i64),
        None => Value::Na,
    }
}

#[pine_builtin(name = "floor", namespace = "math", required_args = 1)]
fn builtin_math_floor(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Int(v.floor() as i64),
        None => Value::Na,
    }
}

#[pine_builtin(name = "trunc", namespace = "math", required_args = 1)]
fn builtin_math_trunc(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) => Value::Int(v.trunc() as i64),
        None => Value::Na,
    }
}

// ============================================================================
// Sign and Comparison
// ============================================================================

#[pine_builtin(name = "sign", namespace = "math", required_args = 1)]
fn builtin_math_sign(args: &[Value]) -> Value {
    match args.first().and_then(get_float) {
        Some(v) if v > 0.0 => Value::Int(1),
        Some(v) if v < 0.0 => Value::Int(-1),
        Some(_) => Value::Int(0),
        None => Value::Na,
    }
}

/// Register math.copysign - Copy sign from one value to another
fn register_copysign(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("copysign")
        .with_namespace("math")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let magnitude = args.first().and_then(get_float);
        let sign = args.get(1).and_then(get_float);

        match (magnitude, sign) {
            (Some(m), Some(s)) => Value::Float(if s >= 0.0 { m.abs() } else { -m.abs() }),
            _ => Value::Na,
        }
    });

    registry.register(meta, func);
}

// ============================================================================
// Utility Functions
// ============================================================================

#[pine_builtin(name = "isna", namespace = "math", required_args = 1)]
fn builtin_math_isna(args: &[Value]) -> Value {
    Value::Bool(matches!(args.first(), Some(Value::Na) | None))
}

/// Register math.nz - Not zero (returns replacement if value is 0 or NA)
fn register_nz(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("nz")
        .with_namespace("math")
        .with_required_args(1)
        .with_optional_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let value = args.first();
        let replacement = args.get(1).cloned().unwrap_or(Value::Int(0));

        match value {
            Some(Value::Na) => replacement,
            Some(v) => v.clone(),
            None => replacement,
        }
    });

    registry.register(meta, func);
}

#[pine_builtin(name = "tostring", namespace = "math", required_args = 1)]
fn builtin_math_tostring(args: &[Value]) -> Value {
    let s = match args.first() {
        Some(Value::Int(n)) => n.to_string(),
        Some(Value::Float(f)) => f.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::String(s)) => s.to_string(),
        Some(Value::Color(c)) => c.to_hex(),
        Some(Value::Na) => "na".to_string(),
        _ => return Value::Na,
    };
    Value::String(s.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);
        registry
    }

    #[test]
    fn test_abs() {
        let registry = test_registry();

        assert_eq!(
            registry.dispatch("math.abs", &[Value::Float(-5.5)]),
            Some(Value::Float(5.5))
        );
        assert_eq!(
            registry.dispatch("math.abs", &[Value::Int(-10)]),
            Some(Value::Int(10))
        );
    }

    #[test]
    fn test_max() {
        let registry = test_registry();

        assert_eq!(
            registry.dispatch("math.max", &[Value::Float(3.0), Value::Float(5.0)]),
            Some(Value::Float(5.0))
        );
        assert_eq!(
            registry.dispatch(
                "math.max",
                &[Value::Float(1.0), Value::Float(2.0), Value::Float(3.0)]
            ),
            Some(Value::Float(3.0))
        );
    }

    #[test]
    fn test_min() {
        let registry = test_registry();

        assert_eq!(
            registry.dispatch("math.min", &[Value::Float(3.0), Value::Float(5.0)]),
            Some(Value::Float(3.0))
        );
    }

    #[test]
    fn test_sqrt() {
        let registry = test_registry();

        assert_eq!(
            registry.dispatch("math.sqrt", &[Value::Float(16.0)]),
            Some(Value::Float(4.0))
        );
    }

    #[test]
    fn test_pow() {
        let registry = test_registry();

        // Basic cases
        assert_eq!(
            registry.dispatch("math.pow", &[Value::Float(2.0), Value::Float(3.0)]),
            Some(Value::Float(8.0))
        );

        // pow(10, 0) = 1
        assert_eq!(
            registry.dispatch("math.pow", &[Value::Float(10.0), Value::Float(0.0)]),
            Some(Value::Float(1.0))
        );

        // pow(4, 0.5) = 2 (square root)
        assert_eq!(
            registry.dispatch("math.pow", &[Value::Float(4.0), Value::Float(0.5)]),
            Some(Value::Float(2.0))
        );

        // pow(2, -1) = 0.5
        assert_eq!(
            registry.dispatch("math.pow", &[Value::Float(2.0), Value::Float(-1.0)]),
            Some(Value::Float(0.5))
        );

        // pow with NA returns NA
        assert_eq!(
            registry.dispatch("math.pow", &[Value::Na, Value::Float(2.0)]),
            Some(Value::Na)
        );
        assert_eq!(
            registry.dispatch("math.pow", &[Value::Float(2.0), Value::Na]),
            Some(Value::Na)
        );

        // pow with integers (should be converted to float)
        assert_eq!(
            registry.dispatch("math.pow", &[Value::Int(2), Value::Int(3)]),
            Some(Value::Float(8.0))
        );
    }

    #[test]
    fn test_round() {
        let registry = test_registry();

        assert_eq!(
            registry.dispatch("math.round", &[Value::Float(3.7)]),
            Some(Value::Int(4))
        );
        assert_eq!(
            registry.dispatch("math.round", &[Value::Float(3.2)]),
            Some(Value::Int(3))
        );
    }

    #[test]
    fn test_isna() {
        let registry = test_registry();

        assert_eq!(
            registry.dispatch("math.isna", &[Value::Na]),
            Some(Value::Bool(true))
        );
        assert_eq!(
            registry.dispatch("math.isna", &[Value::Float(5.0)]),
            Some(Value::Bool(false))
        );
    }

    #[test]
    fn test_nz() {
        let registry = test_registry();

        // nz(na) returns 0 (default replacement)
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Na]),
            Some(Value::Int(0))
        );

        // nz(na, replacement) returns replacement
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Na, Value::Float(-1.0)]),
            Some(Value::Float(-1.0))
        );

        // nz(value) returns value when not na
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Float(5.0)]),
            Some(Value::Float(5.0))
        );

        // nz with integer
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Int(42)]),
            Some(Value::Int(42))
        );

        // nz with zero (zero is not na)
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Float(0.0)]),
            Some(Value::Float(0.0))
        );

        // nz with boolean
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Bool(true)]),
            Some(Value::Bool(true))
        );

        // nz with string
        assert_eq!(
            registry.dispatch("math.nz", &[Value::String("hello".into())]),
            Some(Value::String("hello".into()))
        );

        // nz(na, 0) explicitly
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Na, Value::Int(0)]),
            Some(Value::Int(0))
        );
    }

    #[test]
    fn test_sum_rolling() {
        let registry = test_registry();

        // math.sum(series, length) - rolling sum
        let series = Value::Array(vec![
            Value::Float(10.0),
            Value::Float(20.0),
            Value::Float(30.0),
            Value::Float(40.0),
        ]);

        // Sum of last 3 values: 20 + 30 + 40 = 90
        let result = registry.dispatch("math.sum", &[series, Value::Int(3)]);
        assert_eq!(result, Some(Value::Float(90.0)));
    }

    #[test]
    fn test_sum_returns_na_when_insufficient_data() {
        let registry = test_registry();

        // Only 2 values but requesting sum of 3
        let series = Value::Array(vec![Value::Float(10.0), Value::Float(20.0)]);
        let result = registry.dispatch("math.sum", &[series, Value::Int(3)]);
        assert_eq!(result, Some(Value::Na));
    }

    #[test]
    fn test_sum_single_value() {
        let registry = test_registry();

        let series = Value::Array(vec![Value::Float(42.0)]);
        let result = registry.dispatch("math.sum", &[series, Value::Int(1)]);
        assert_eq!(result, Some(Value::Float(42.0)));
    }

    #[test]
    fn test_sum_with_na_values() {
        let registry = test_registry();

        // math.sum should skip NA values
        let series = Value::Array(vec![
            Value::Na,
            Value::Float(20.0),
            Value::Float(30.0),
            Value::Float(40.0),
        ]);

        // Sum of last 3: 20 + 30 + 40 = 90 (NA is skipped)
        let result = registry.dispatch("math.sum", &[series, Value::Int(3)]);
        assert_eq!(result, Some(Value::Float(90.0)));
    }

    #[test]
    fn test_avg_rolling() {
        let registry = test_registry();

        // math.avg(series, length) - rolling average
        let series = Value::Array(vec![
            Value::Float(10.0),
            Value::Float(20.0),
            Value::Float(30.0),
            Value::Float(40.0),
        ]);

        // Average of last 3 values: (20 + 30 + 40) / 3 = 30
        let result = registry.dispatch("math.avg", &[series, Value::Int(3)]);
        assert_eq!(result, Some(Value::Float(30.0)));
    }

    #[test]
    fn test_avg_returns_na_when_insufficient_data() {
        let registry = test_registry();

        // Only 2 values but requesting average of 3
        let series = Value::Array(vec![Value::Float(10.0), Value::Float(20.0)]);
        let result = registry.dispatch("math.avg", &[series, Value::Int(3)]);
        assert_eq!(result, Some(Value::Na));
    }

    #[test]
    fn test_avg_single_value() {
        let registry = test_registry();

        let series = Value::Array(vec![Value::Float(42.0)]);
        let result = registry.dispatch("math.avg", &[series, Value::Int(1)]);
        assert_eq!(result, Some(Value::Float(42.0)));
    }

    #[test]
    fn test_avg_with_na_values() {
        let registry = test_registry();

        // math.avg should skip NA values
        let series = Value::Array(vec![
            Value::Na,
            Value::Float(20.0),
            Value::Float(30.0),
            Value::Float(40.0),
        ]);

        // Average of last 3: (20 + 30 + 40) / 3 = 30 (NA is skipped)
        let result = registry.dispatch("math.avg", &[series, Value::Int(3)]);
        assert_eq!(result, Some(Value::Float(30.0)));
    }

    #[test]
    fn test_avg_relation_to_sum() {
        let registry = test_registry();

        // math.avg(series, length) = math.sum(series, length) / length (when no NA values)
        let series = Value::Array(vec![
            Value::Float(10.0),
            Value::Float(20.0),
            Value::Float(30.0),
            Value::Float(40.0),
        ]);

        let sum_result = registry.dispatch("math.sum", &[series.clone(), Value::Int(4)]);
        let avg_result = registry.dispatch("math.avg", &[series, Value::Int(4)]);

        // avg = sum / 4
        let sum_val = match sum_result {
            Some(Value::Float(v)) => v,
            _ => panic!("sum should return a float"),
        };
        let avg_val = match avg_result {
            Some(Value::Float(v)) => v,
            _ => panic!("avg should return a float"),
        };

        assert!((avg_val - sum_val / 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_copysign_basic() {
        let registry = test_registry();

        // copysign(value, positive_sign) returns positive value
        assert_eq!(
            registry.dispatch("math.copysign", &[Value::Float(-5.0), Value::Float(1.0)]),
            Some(Value::Float(5.0))
        );

        // copysign(value, negative_sign) returns negative value
        assert_eq!(
            registry.dispatch("math.copysign", &[Value::Float(5.0), Value::Float(-1.0)]),
            Some(Value::Float(-5.0))
        );

        // copysign with zero sign (zero is non-negative)
        assert_eq!(
            registry.dispatch("math.copysign", &[Value::Float(-5.0), Value::Float(0.0)]),
            Some(Value::Float(5.0))
        );
    }

    #[test]
    fn test_copysign_with_na() {
        let registry = test_registry();

        // copysign with NA returns NA
        assert_eq!(
            registry.dispatch("math.copysign", &[Value::Na, Value::Float(1.0)]),
            Some(Value::Na)
        );
        assert_eq!(
            registry.dispatch("math.copysign", &[Value::Float(5.0), Value::Na]),
            Some(Value::Na)
        );
    }

    #[test]
    fn test_copysign_with_integers() {
        let registry = test_registry();

        // copysign with integers
        assert_eq!(
            registry.dispatch("math.copysign", &[Value::Int(-10), Value::Int(1)]),
            Some(Value::Float(10.0))
        );
        assert_eq!(
            registry.dispatch("math.copysign", &[Value::Int(10), Value::Int(-1)]),
            Some(Value::Float(-10.0))
        );
    }

    #[test]
    fn test_round_to_nearest_basic() {
        let registry = test_registry();

        // Round to nearest 0.5
        assert_eq!(
            registry.dispatch(
                "math.round_to_nearest",
                &[Value::Float(1.3), Value::Float(0.5)]
            ),
            Some(Value::Float(1.5))
        );

        // Round to nearest 10
        assert_eq!(
            registry.dispatch(
                "math.round_to_nearest",
                &[Value::Float(23.0), Value::Float(10.0)]
            ),
            Some(Value::Float(20.0))
        );
        assert_eq!(
            registry.dispatch(
                "math.round_to_nearest",
                &[Value::Float(27.0), Value::Float(10.0)]
            ),
            Some(Value::Float(30.0))
        );

        // Round to nearest 0.1
        assert_eq!(
            registry.dispatch(
                "math.round_to_nearest",
                &[Value::Float(3.14159), Value::Float(0.1)]
            ),
            Some(Value::Float(3.1))
        );
    }

    #[test]
    fn test_round_to_nearest_with_na() {
        let registry = test_registry();

        // round_to_nearest with NA returns NA
        assert_eq!(
            registry.dispatch("math.round_to_nearest", &[Value::Na, Value::Float(0.5)]),
            Some(Value::Na)
        );
        assert_eq!(
            registry.dispatch("math.round_to_nearest", &[Value::Float(5.0), Value::Na]),
            Some(Value::Na)
        );
    }

    #[test]
    fn test_round_to_nearest_zero_precision() {
        let registry = test_registry();

        // round_to_nearest with zero precision returns NA
        assert_eq!(
            registry.dispatch(
                "math.round_to_nearest",
                &[Value::Float(5.0), Value::Float(0.0)]
            ),
            Some(Value::Na)
        );
    }

    #[test]
    fn test_round_to_nearest_with_integers() {
        let registry = test_registry();

        // round_to_nearest with integers
        assert_eq!(
            registry.dispatch("math.round_to_nearest", &[Value::Int(23), Value::Int(10)]),
            Some(Value::Float(20.0))
        );
    }
}
