//! Strategy functions for Pine Script v6
//!
//! This module provides strategy() and related functions for creating
//! trading strategies with entry/exit signals.

use crate::registry::{FunctionMeta, FunctionRegistry};
use pine_runtime::value::Value;
use std::sync::Arc;

/// Register all strategy functions with the registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    register_strategy(registry);
    register_strategy_entry(registry);
    register_strategy_close(registry);
    register_strategy_exit(registry);
    register_strategy_long(registry);
    register_strategy_short(registry);
}

/// Register strategy() function
///
/// Signature: strategy(title, shorttitle, overlay, format, precision,
///                     initial_capital, default_qty_type, default_qty_value,
///                     max_lines_count, max_labels_count, max_boxes_count,
///                     max_bars_back, max_bars_back_bars, calc_on_order_fills,
///                     calc_on_every_tick, process_orders_on_close,
///                     close_entries_rule, currency, pyramiding,
///                     use_bar_magnifier, backtest_fill_limits_assumption)
fn register_strategy(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("strategy")
        .with_required_args(1) // title is required
        .with_optional_args(20); // many optional parameters

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // First argument is the strategy title (required)
        let title = match args.first() {
            Some(Value::String(s)) => s.to_string(),
            Some(other) => other.to_string(),
            None => "Strategy".to_string(),
        };

        // Extract optional parameters with defaults
        let _short_title = extract_string_arg(args, 1, "");
        let _overlay = extract_bool_arg(args, 2, true);
        let _format = extract_string_arg(args, 3, "inherit");
        let _precision = extract_int_arg(args, 4, 4);
        let initial_capital = extract_float_arg(args, 5, 100000.0);
        let _default_qty_type = extract_string_arg(args, 6, "fixed");
        let default_qty_value = extract_float_arg(args, 7, 1.0);
        let _max_lines_count = extract_int_arg(args, 8, 50);
        let _max_labels_count = extract_int_arg(args, 9, 50);
        let _max_boxes_count = extract_int_arg(args, 10, 50);

        // Return a strategy configuration value
        // This is a special marker that the runtime recognizes
        Value::Array(vec![
            Value::String("__strategy__".into()),
            Value::String(title.into()),
            Value::Float(initial_capital),
            Value::Float(default_qty_value),
        ])
    });

    registry.register(meta, func);
}

/// Helper to extract string argument with default
fn extract_string_arg(args: &[Value], index: usize, default: &str) -> String {
    args.get(index)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}

/// Helper to extract int argument with default
fn extract_int_arg(args: &[Value], index: usize, default: i64) -> i64 {
    args.get(index).and_then(|v| v.as_int()).unwrap_or(default)
}

/// Helper to extract float argument with default
fn extract_float_arg(args: &[Value], index: usize, default: f64) -> f64 {
    args.get(index)
        .and_then(|v| v.as_float())
        .or_else(|| args.get(index).and_then(|v| v.as_int()).map(|i| i as f64))
        .unwrap_or(default)
}

/// Helper to extract bool argument with default
fn extract_bool_arg(args: &[Value], index: usize, default: bool) -> bool {
    args.get(index).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Register strategy.entry function
///
/// Signature: strategy.entry(id, direction, qty, limit, stop, oca_name, oca_type, comment)
fn register_strategy_entry(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("entry")
        .with_namespace("strategy")
        .with_required_args(2) // id and direction are required
        .with_optional_args(6);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let id = match args.first() {
            Some(Value::String(s)) => s.to_string(),
            Some(other) => other.to_string(),
            None => "Entry".to_string(),
        };

        let direction = match args.get(1) {
            Some(Value::String(s)) => s.to_string(),
            Some(other) => other.to_string(),
            None => "long".to_string(),
        };

        let qty = extract_float_arg(args, 2, 1.0);

        // Return entry signal marker
        Value::Array(vec![
            Value::String("__entry__".into()),
            Value::String(id.into()),
            Value::String(direction.into()),
            Value::Float(qty),
        ])
    });

    registry.register(meta, func);
}

/// Register strategy.close function
///
/// Signature: strategy.close(id, comment)
fn register_strategy_close(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("close")
        .with_namespace("strategy")
        .with_required_args(1) // id is required
        .with_optional_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let id = match args.first() {
            Some(Value::String(s)) => s.to_string(),
            Some(other) => other.to_string(),
            None => "Position".to_string(),
        };

        // Return close signal marker
        Value::Array(vec![
            Value::String("__close__".into()),
            Value::String(id.into()),
        ])
    });

    registry.register(meta, func);
}

/// Register strategy.exit function
///
/// Signature: strategy.exit(id, from_entry, qty, limit, stop, oca_name, oca_type, comment)
fn register_strategy_exit(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("exit")
        .with_namespace("strategy")
        .with_required_args(1) // id is required
        .with_optional_args(7);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let id = match args.first() {
            Some(Value::String(s)) => s.to_string(),
            Some(other) => other.to_string(),
            None => "Exit".to_string(),
        };

        let from_entry = extract_string_arg(args, 1, "");

        // Return exit signal marker
        Value::Array(vec![
            Value::String("__exit__".into()),
            Value::String(id.into()),
            Value::String(from_entry.into()),
        ])
    });

    registry.register(meta, func);
}

/// Register strategy.long constant
fn register_strategy_long(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("long")
        .with_namespace("strategy")
        .with_required_args(0);

    let func: crate::registry::BuiltinFn = Arc::new(|_args| Value::String("long".into()));

    registry.register(meta, func);
}

/// Register strategy.short constant
fn register_strategy_short(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("short")
        .with_namespace("strategy")
        .with_required_args(0);

    let func: crate::registry::BuiltinFn = Arc::new(|_args| Value::String("short".into()));

    registry.register(meta, func);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_function_registered() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("strategy"));
    }

    #[test]
    fn test_strategy_basic() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let result = registry.dispatch("strategy", &[Value::String("My Strategy".into())]);
        assert!(result.is_some());

        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr[0], Value::String("__strategy__".into()));
            assert_eq!(arr[1], Value::String("My Strategy".into()));
        } else {
            panic!("Expected strategy array value");
        }
    }

    #[test]
    fn test_strategy_with_capital() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        // strategy("Test", "", true, "inherit", 4, 50000.0)
        let result = registry.dispatch(
            "strategy",
            &[
                Value::String("Test".into()),
                Value::String("".into()),
                Value::Bool(true),
                Value::String("inherit".into()),
                Value::Int(4),
                Value::Float(50000.0),
            ],
        );
        assert!(result.is_some());

        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr[2], Value::Float(50000.0));
        } else {
            panic!("Expected strategy array value");
        }
    }

    #[test]
    fn test_strategy_entry() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("strategy.entry"));

        let result = registry.dispatch(
            "strategy.entry",
            &[Value::String("Long".into()), Value::String("long".into())],
        );
        assert!(result.is_some());

        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr[0], Value::String("__entry__".into()));
            assert_eq!(arr[1], Value::String("Long".into()));
            assert_eq!(arr[2], Value::String("long".into()));
        } else {
            panic!("Expected entry array value");
        }
    }

    #[test]
    fn test_strategy_close() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("strategy.close"));

        let result = registry.dispatch("strategy.close", &[Value::String("Long".into())]);
        assert!(result.is_some());

        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr[0], Value::String("__close__".into()));
            assert_eq!(arr[1], Value::String("Long".into()));
        } else {
            panic!("Expected close array value");
        }
    }

    #[test]
    fn test_strategy_exit() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("strategy.exit"));

        let result = registry.dispatch(
            "strategy.exit",
            &[
                Value::String("StopLoss".into()),
                Value::String("Long".into()),
            ],
        );
        assert!(result.is_some());

        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr[0], Value::String("__exit__".into()));
            assert_eq!(arr[1], Value::String("StopLoss".into()));
        } else {
            panic!("Expected exit array value");
        }
    }

    #[test]
    fn test_strategy_long_short() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("strategy.long"));
        assert!(registry.contains("strategy.short"));

        let long_result = registry.dispatch("strategy.long", &[]);
        assert_eq!(long_result, Some(Value::String("long".into())));

        let short_result = registry.dispatch("strategy.short", &[]);
        assert_eq!(short_result, Some(Value::String("short".into())));
    }
}
