//! Plot output functions
//!
//! This module provides plot() and related functions for outputting values
//! that can be captured and displayed on charts.

use crate::registry::{FunctionMeta, FunctionRegistry};
use pine_runtime::value::Value;
use std::sync::Arc;

/// Register plot functions
pub fn register_functions(registry: &mut FunctionRegistry) {
    register_plot(registry);
}

/// Register plot() function
fn register_plot(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("plot")
        .with_required_args(1)
        .with_optional_args(5);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        // First argument is the series/value to plot
        let value = args.first().cloned().unwrap_or(Value::Na);

        // Second argument (optional) is the title
        let title = if let Some(arg) = args.get(1) {
            match arg {
                Value::String(s) => s.to_string(),
                _ => "plot".to_string(),
            }
        } else {
            "plot".to_string()
        };

        // Convert value to Option<f64> for plotting
        let plot_value = match value {
            Value::Float(f) => Some(f),
            Value::Int(i) => Some(i as f64),
            _ => None, // NA or other types = no value
        };

        // Return a special plot value that the runtime can capture
        // For now, we return the value with metadata attached
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
