//! Input functions (input.*)
//!
//! This module provides user input functions for Pine Script v6.
//! These functions allow scripts to accept user-configurable parameters
//! through the TradingView UI.

use crate::registry::{FunctionMeta, FunctionRegistry};
use pine_runtime::value::Value;
use std::sync::Arc;

/// Register all input.* functions with the registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    register_input_int(registry);
    register_input_float(registry);
    register_input_bool(registry);
    register_input_string(registry);
    register_input_source(registry);
    register_input_timeframe(registry);
    register_input_symbol(registry);
    register_input_color(registry);
}

/// Extract a named argument from args
///
/// # Arguments
/// * `args` - The argument slice
/// * `name` - The argument name to look for
/// * `index` - The positional index (fallback if not named)
fn extract_named_arg(args: &[Value], _name: &str, index: usize) -> Option<Value> {
    // For now, we use positional extraction
    // In full implementation, this would check for named arguments
    args.get(index).cloned()
}

/// Extract integer value from argument
fn extract_int(arg: Option<Value>, default: i64) -> i64 {
    match arg {
        Some(Value::Int(n)) => n,
        Some(Value::Float(f)) => f as i64,
        _ => default,
    }
}

/// Extract float value from argument
fn extract_float(arg: Option<Value>, default: f64) -> f64 {
    match arg {
        Some(Value::Float(f)) => f,
        Some(Value::Int(n)) => n as f64,
        _ => default,
    }
}

/// Extract bool value from argument
fn extract_bool(arg: Option<Value>, default: bool) -> bool {
    match arg {
        Some(Value::Bool(b)) => b,
        _ => default,
    }
}

/// Extract string value from argument
fn extract_string(arg: Option<Value>, default: &str) -> String {
    match arg {
        Some(Value::String(s)) => s.to_string(),
        _ => default.to_string(),
    }
}

/// Register input.int - Integer input
///
/// Signature: input.int(defval, minval, maxval, step, title, tooltip, inline, group)
fn register_input_int(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("int")
        .with_namespace("input")
        .with_required_args(1)
        .with_optional_args(7);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval (first positional arg or named)
        let defval = extract_int(extract_named_arg(args, "defval", 0), 0);

        // minval constraint
        let minval = extract_int(extract_named_arg(args, "minval", 1), i64::MIN);

        // maxval constraint
        let maxval = extract_int(extract_named_arg(args, "maxval", 2), i64::MAX);

        // Apply constraints
        let value = defval.clamp(minval, maxval);

        Value::Int(value)
    });

    registry.register(meta, func);
}

/// Register input.float - Float input
///
/// Signature: input.float(defval, minval, maxval, step, title, tooltip, inline, group)
fn register_input_float(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("float")
        .with_namespace("input")
        .with_required_args(1)
        .with_optional_args(7);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval
        let defval = extract_float(extract_named_arg(args, "defval", 0), 0.0);

        // minval constraint
        let minval = extract_float(extract_named_arg(args, "minval", 1), f64::NEG_INFINITY);

        // maxval constraint
        let maxval = extract_float(extract_named_arg(args, "maxval", 2), f64::INFINITY);

        // Apply constraints
        let value = if defval < minval {
            minval
        } else if defval > maxval {
            maxval
        } else {
            defval
        };

        Value::Float(value)
    });

    registry.register(meta, func);
}

/// Register input.bool - Boolean input
///
/// Signature: input.bool(defval, title, tooltip, inline, group)
fn register_input_bool(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("bool")
        .with_namespace("input")
        .with_required_args(1)
        .with_optional_args(4);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval
        let defval = extract_bool(extract_named_arg(args, "defval", 0), false);

        Value::Bool(defval)
    });

    registry.register(meta, func);
}

/// Register input.string - String input
///
/// Signature: input.string(defval, options, title, tooltip, inline, group, confirm)
fn register_input_string(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("string")
        .with_namespace("input")
        .with_required_args(1)
        .with_optional_args(6);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval
        let defval = extract_string(extract_named_arg(args, "defval", 0), "");

        Value::String(defval.into())
    });

    registry.register(meta, func);
}

/// Register input.source - Source input (price source)
///
/// Signature: input.source(defval, title, tooltip, inline, group, options)
fn register_input_source(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("source")
        .with_namespace("input")
        .with_required_args(1)
        .with_optional_args(5);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval should be a series (close, open, high, low, hl2, hlc3, ohlc4)
        // For now, return the defval as-is (it will be the current bar's value)
        if let Some(arg) = args.first() {
            arg.clone()
        } else {
            Value::Na
        }
    });

    registry.register(meta, func);
}

/// Register input.timeframe - Timeframe input
///
/// Signature: input.timeframe(defval, title, tooltip, inline, group, options, confirm)
fn register_input_timeframe(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("timeframe")
        .with_namespace("input")
        .with_required_args(0)
        .with_optional_args(6);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval (default timeframe like "D", "W", "M", etc.)
        let defval = extract_string(extract_named_arg(args, "defval", 0), "D");

        Value::String(defval.into())
    });

    registry.register(meta, func);
}

/// Register input.symbol - Symbol input
///
/// Signature: input.symbol(defval, title, tooltip, inline, group, options, confirm)
fn register_input_symbol(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("symbol")
        .with_namespace("input")
        .with_required_args(0)
        .with_optional_args(6);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval (default symbol like "BTCUSDT", "AAPL", etc.)
        let defval = extract_string(extract_named_arg(args, "defval", 0), "");

        Value::String(defval.into())
    });

    registry.register(meta, func);
}

/// Register input.color - Color input
///
/// Signature: input.color(defval, title, tooltip, inline, group, options, confirm)
fn register_input_color(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("color")
        .with_namespace("input")
        .with_required_args(1)
        .with_optional_args(6);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // defval should be a color value
        // For now, return the defval as-is, or blue as default
        if let Some(arg) = args.first() {
            arg.clone()
        } else {
            // Return default blue color
            Value::Color(pine_runtime::value::Color::new(0, 120, 255))
        }
    });

    registry.register(meta, func);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_int() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        // Test basic input.int
        let result = registry.dispatch("input.int", &[Value::Int(5)]);
        assert_eq!(result, Some(Value::Int(5)));

        // Test with minval constraint
        let result = registry.dispatch("input.int", &[Value::Int(5), Value::Int(10)]);
        // 5 < minval(10), so should return 10
        assert_eq!(result, Some(Value::Int(10)));

        // Test with maxval constraint
        let result = registry.dispatch("input.int", &[Value::Int(100), Value::Int(0), Value::Int(50)]);
        // 100 > maxval(50), so should return 50
        assert_eq!(result, Some(Value::Int(50)));
    }

    #[test]
    fn test_input_float() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let result = registry.dispatch("input.float", &[Value::Float(3.14)]);
        assert_eq!(result, Some(Value::Float(3.14)));
    }

    #[test]
    fn test_input_bool() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let result = registry.dispatch("input.bool", &[Value::Bool(true)]);
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_input_string() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let result = registry.dispatch("input.string", &[Value::String("test".into())]);
        assert_eq!(result, Some(Value::String("test".into())));
    }
}
