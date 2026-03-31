//! Math functions (math.*)
//!
//! This module provides Pine Script v6 compatible mathematical functions.
//! All functions follow TradingView's exact semantics including NA handling.

use crate::registry::{FunctionMeta, FunctionRegistry};
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
    register_to_string(registry);
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

/// Register math.abs - Absolute value
fn register_abs(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("abs")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| match args.first() {
        Some(Value::Float(f)) => Value::Float(f.abs()),
        Some(Value::Int(n)) => Value::Int(n.abs()),
        _ => Value::Na,
    });

    registry.register(meta, func);
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

/// Register math.sum - Sum of all values
fn register_sum(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sum")
        .with_namespace("math")
        .with_required_args(1)
        .with_variadic();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let mut sum = 0.0;
        let mut found = false;

        for arg in args {
            if let Some(v) = get_float(arg) {
                sum += v;
                found = true;
            }
        }

        if found {
            Value::Float(sum)
        } else {
            Value::Na
        }
    });

    registry.register(meta, func);
}

/// Register math.avg - Average of all values
fn register_avg(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("avg")
        .with_namespace("math")
        .with_required_args(1)
        .with_variadic();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let mut sum = 0.0;
        let mut count = 0;

        for arg in args {
            if let Some(v) = get_float(arg) {
                sum += v;
                count += 1;
            }
        }

        if count > 0 {
            Value::Float(sum / count as f64)
        } else {
            Value::Na
        }
    });

    registry.register(meta, func);
}

// ============================================================================
// Power and Roots
// ============================================================================

/// Register math.sqrt - Square root
fn register_sqrt(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sqrt")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| match args.first() {
        Some(Value::Float(f)) => Value::Float(f.sqrt()),
        Some(Value::Int(n)) => Value::Float((*n as f64).sqrt()),
        _ => Value::Na,
    });

    registry.register(meta, func);
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

/// Register math.exp - Exponential function (e^x)
fn register_exp(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("exp")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.exp()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.log - Natural logarithm
fn register_log(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("log")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) if v > 0.0 => Value::Float(v.ln()),
            _ => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.log10 - Base-10 logarithm
fn register_log10(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("log10")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) if v > 0.0 => Value::Float(v.log10()),
            _ => Value::Na,
        });

    registry.register(meta, func);
}

// ============================================================================
// Trigonometric Functions
// ============================================================================

/// Register math.sin - Sine
fn register_sin(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sin")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.sin()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.cos - Cosine
fn register_cos(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("cos")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.cos()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.tan - Tangent
fn register_tan(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("tan")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.tan()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.asin - Arc sine
fn register_asin(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("asin")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) if (-1.0..=1.0).contains(&v) => Value::Float(v.asin()),
            _ => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.acos - Arc cosine
fn register_acos(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("acos")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) if (-1.0..=1.0).contains(&v) => Value::Float(v.acos()),
            _ => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.atan - Arc tangent
fn register_atan(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("atan")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.atan()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

// ============================================================================
// Hyperbolic Functions
// ============================================================================

/// Register math.sinh - Hyperbolic sine
fn register_sinh(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sinh")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.sinh()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.cosh - Hyperbolic cosine
fn register_cosh(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("cosh")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.cosh()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.tanh - Hyperbolic tangent
fn register_tanh(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("tanh")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Float(v.tanh()),
            None => Value::Na,
        });

    registry.register(meta, func);
}

// ============================================================================
// Rounding Functions
// ============================================================================

/// Register math.round - Round to nearest integer
fn register_round(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("round")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Int(v.round() as i64),
            None => Value::Na,
        });

    registry.register(meta, func);
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

/// Register math.ceil - Ceiling (round up)
fn register_ceil(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("ceil")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Int(v.ceil() as i64),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.floor - Floor (round down)
fn register_floor(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("floor")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Int(v.floor() as i64),
            None => Value::Na,
        });

    registry.register(meta, func);
}

/// Register math.trunc - Truncate (remove fractional part)
fn register_trunc(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("trunc")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) => Value::Int(v.trunc() as i64),
            None => Value::Na,
        });

    registry.register(meta, func);
}

// ============================================================================
// Sign and Comparison
// ============================================================================

/// Register math.sign - Sign of number (-1, 0, or 1)
fn register_sign(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sign")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| match args.first().and_then(get_float) {
            Some(v) if v > 0.0 => Value::Int(1),
            Some(v) if v < 0.0 => Value::Int(-1),
            Some(_) => Value::Int(0),
            None => Value::Na,
        });

    registry.register(meta, func);
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

/// Register math.isna - Check if value is NA
fn register_isna(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("isna")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn =
        Arc::new(|args| Value::Bool(matches!(args.first(), Some(Value::Na) | None)));

    registry.register(meta, func);
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

/// Register math.tostring - Convert value to string
fn register_to_string(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("tostring")
        .with_namespace("math")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
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
    });

    registry.register(meta, func);
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

        assert_eq!(
            registry.dispatch("math.pow", &[Value::Float(2.0), Value::Float(3.0)]),
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

        assert_eq!(
            registry.dispatch("math.nz", &[Value::Na]),
            Some(Value::Int(0))
        );
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Na, Value::Float(-1.0)]),
            Some(Value::Float(-1.0))
        );
        assert_eq!(
            registry.dispatch("math.nz", &[Value::Float(5.0)]),
            Some(Value::Float(5.0))
        );
    }
}
