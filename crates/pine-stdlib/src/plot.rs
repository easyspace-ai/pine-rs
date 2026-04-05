//! Plot output functions
//!
//! This module provides plot() and related display functions for outputting values
//! that can be captured and displayed on charts.
//!
//! Display functions include:
//! - `plot()` — Line/area/histogram plots
//! - `hline()` — Horizontal price lines
//! - `bgcolor()` — Background coloring
//! - `fill()` — Fill between two plots
//! - `plotshape()` — Shape markers on chart
//! - `plotchar()` — Character markers on chart
//! - `plotarrow()` — Arrow indicators

use crate::registry::{FunctionMeta, FunctionRegistry};
use pine_runtime::value::Value;
use std::sync::Arc;

/// Register plot functions
pub fn register_functions(registry: &mut FunctionRegistry) {
    register_plot(registry);
    register_hline(registry);
    register_bgcolor(registry);
    register_fill(registry);
    register_plotshape(registry);
    register_plotchar(registry);
    register_plotarrow(registry);
}

/// Register plot() function
fn register_plot(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("plot")
        .with_required_args(1)
        .with_optional_args(7);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let value = args.first().cloned().unwrap_or(Value::Na);
        let title = if let Some(arg) = args.get(1) {
            match arg {
                Value::String(s) => s.to_string(),
                _ => "plot".to_string(),
            }
        } else {
            "plot".to_string()
        };

        let plot_value = match value {
            Value::Float(f) => Some(f),
            Value::Int(i) => Some(i as f64),
            _ => None,
        };

        Value::Array(vec![
            Value::String("__plot__".into()),
            Value::String(title.into()),
            match plot_value {
                Some(f) => Value::Float(f),
                None => Value::Na,
            },
        ])
    });

    registry.register(meta, func);
}

/// Register hline() function
fn register_hline(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("hline")
        .with_required_args(1)
        .with_optional_args(4);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let _price = args.first().cloned().unwrap_or(Value::Na);
        // hline is handled specially in eval; registry stub returns Na
        Value::Na
    });

    registry.register(meta, func);
}

/// Register bgcolor() function
fn register_bgcolor(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("bgcolor")
        .with_required_args(1)
        .with_optional_args(3);

    let func: crate::registry::BuiltinFn = Arc::new(|_args| {
        // bgcolor is display-only, handled in eval
        Value::Na
    });

    registry.register(meta, func);
}

/// Register fill() function
fn register_fill(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("fill")
        .with_required_args(2)
        .with_optional_args(3);

    let func: crate::registry::BuiltinFn = Arc::new(|_args| {
        // fill is display-only, handled in eval
        Value::Na
    });

    registry.register(meta, func);
}

/// Register plotshape() function
fn register_plotshape(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("plotshape")
        .with_required_args(1)
        .with_optional_args(7);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let value = args.first().cloned().unwrap_or(Value::Na);
        let plot_value = match value {
            Value::Float(f) => Some(f),
            Value::Int(i) => Some(i as f64),
            Value::Bool(true) => Some(1.0),
            Value::Bool(false) => Some(0.0),
            _ => None,
        };
        Value::Array(vec![
            Value::String("__plotshape__".into()),
            match plot_value {
                Some(f) => Value::Float(f),
                None => Value::Na,
            },
        ])
    });

    registry.register(meta, func);
}

/// Register plotchar() function
fn register_plotchar(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("plotchar")
        .with_required_args(1)
        .with_optional_args(6);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let value = args.first().cloned().unwrap_or(Value::Na);
        let plot_value = match value {
            Value::Float(f) => Some(f),
            Value::Int(i) => Some(i as f64),
            Value::Bool(true) => Some(1.0),
            Value::Bool(false) => Some(0.0),
            _ => None,
        };
        Value::Array(vec![
            Value::String("__plotchar__".into()),
            match plot_value {
                Some(f) => Value::Float(f),
                None => Value::Na,
            },
        ])
    });

    registry.register(meta, func);
}

/// Register plotarrow() function
fn register_plotarrow(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("plotarrow")
        .with_required_args(1)
        .with_optional_args(5);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let value = args.first().cloned().unwrap_or(Value::Na);
        let plot_value = match value {
            Value::Float(f) => Some(f),
            Value::Int(i) => Some(i as f64),
            _ => None,
        };
        Value::Array(vec![
            Value::String("__plotarrow__".into()),
            match plot_value {
                Some(f) => Value::Float(f),
                None => Value::Na,
            },
        ])
    });

    registry.register(meta, func);
}
