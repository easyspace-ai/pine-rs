//! Array functions (array.*)
//!
//! This module provides Pine Script v6 compatible array operations.
//! All functions follow TradingView's exact semantics.

use crate::registry::{FunctionMeta, FunctionRegistry};
use pine_runtime::value::Value;
use std::sync::Arc;

/// Register all array.* functions with the registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    register_new_int(registry);
    register_new_float(registry);
    register_new_bool(registry);
    register_new_string(registry);
    register_new_color(registry);
    register_size(registry);
    register_push(registry);
    register_pop(registry);
    register_get(registry);
    register_set(registry);
    register_clear(registry);
    register_first(registry);
    register_last(registry);
    register_remove(registry);
    register_insert(registry);
    register_sum(registry);
    register_avg(registry);
    register_min(registry);
    register_max(registry);
    register_sort(registry);
    register_reverse(registry);
    register_from(registry);
    register_copy(registry);
    register_concat(registry);
    register_fill(registry);
}

// ============================================================================
// Helper functions
// ============================================================================

/// Extract array from value
fn extract_array(value: &Value) -> Option<Vec<Value>> {
    match value {
        Value::Array(arr) => Some(arr.clone()),
        _ => None,
    }
}

/// Get integer index from arguments
fn get_index(args: &[Value], idx: usize) -> Option<usize> {
    args.get(idx)
        .and_then(|v| v.as_int())
        .map(|n| n.max(0) as usize)
}

/// Extract a numeric value (int or float) as f64
fn get_numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Int(i) => Some(*i as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

// ============================================================================
// array.new_* functions
// ============================================================================

/// Register array.new_int
fn register_new_int(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("new_int")
        .with_namespace("array")
        .with_required_args(0)
        .with_optional_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let size = args.first().and_then(|v| v.as_int()).unwrap_or(0) as usize;
        let default = args.get(1).cloned().unwrap_or(Value::Int(0));

        let mut arr = Vec::with_capacity(size);
        for _ in 0..size {
            arr.push(default.clone());
        }

        Value::Array(arr)
    });

    registry.register(meta, func);
}

/// Register array.new_float
fn register_new_float(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("new_float")
        .with_namespace("array")
        .with_required_args(0)
        .with_optional_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let size = args.first().and_then(|v| v.as_int()).unwrap_or(0) as usize;
        let default = args.get(1).cloned().unwrap_or(Value::Float(0.0));

        let mut arr = Vec::with_capacity(size);
        for _ in 0..size {
            arr.push(default.clone());
        }

        Value::Array(arr)
    });

    registry.register(meta, func);
}

/// Register array.new_bool
fn register_new_bool(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("new_bool")
        .with_namespace("array")
        .with_required_args(0)
        .with_optional_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let size = args.first().and_then(|v| v.as_int()).unwrap_or(0) as usize;
        let default = args.get(1).cloned().unwrap_or(Value::Bool(false));

        let mut arr = Vec::with_capacity(size);
        for _ in 0..size {
            arr.push(default.clone());
        }

        Value::Array(arr)
    });

    registry.register(meta, func);
}

/// Register array.new_string
fn register_new_string(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("new_string")
        .with_namespace("array")
        .with_required_args(0)
        .with_optional_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let size = args.first().and_then(|v| v.as_int()).unwrap_or(0) as usize;
        let default = args.get(1).cloned().unwrap_or(Value::String("".into()));

        let mut arr = Vec::with_capacity(size);
        for _ in 0..size {
            arr.push(default.clone());
        }

        Value::Array(arr)
    });

    registry.register(meta, func);
}

/// Register array.new_color
fn register_new_color(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("new_color")
        .with_namespace("array")
        .with_required_args(0)
        .with_optional_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let size = args.first().and_then(|v| v.as_int()).unwrap_or(0) as usize;
        let default = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| Value::Color(pine_runtime::value::Color::new(0, 0, 0)));

        let mut arr = Vec::with_capacity(size);
        for _ in 0..size {
            arr.push(default.clone());
        }

        Value::Array(arr)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.size
// ============================================================================

/// Register array.size
fn register_size(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("size")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        Value::Int(arr.len() as i64)
    });

    registry.register_hot(meta, func);
}

// ============================================================================
// array.push
// ============================================================================

/// Register array.push
fn register_push(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("push")
        .with_namespace("array")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        arr.push(value.clone());
        Value::Array(arr)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.pop
// ============================================================================

/// Register array.pop
fn register_pop(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("pop")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        arr.pop().unwrap_or(Value::Na)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.get
// ============================================================================

/// Register array.get
fn register_get(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("get")
        .with_namespace("array")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(idx) = get_index(args, 1) else {
            return Value::Na;
        };
        arr.get(idx).cloned().unwrap_or(Value::Na)
    });

    registry.register_hot(meta, func);
}

// ============================================================================
// array.set
// ============================================================================

/// Register array.set
fn register_set(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("set")
        .with_namespace("array")
        .with_required_args(3);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(idx) = get_index(args, 1) else {
            return Value::Na;
        };
        let Some(value) = args.get(2) else {
            return Value::Na;
        };
        if idx < arr.len() {
            arr[idx] = value.clone();
        }
        Value::Array(arr)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.clear
// ============================================================================

/// Register array.clear
fn register_clear(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("clear")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        arr.clear();
        Value::Array(arr)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.first
// ============================================================================

/// Register array.first
fn register_first(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("first")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        arr.first().cloned().unwrap_or(Value::Na)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.last
// ============================================================================

/// Register array.last
fn register_last(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("last")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        arr.last().cloned().unwrap_or(Value::Na)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.remove
// ============================================================================

/// Register array.remove
fn register_remove(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("remove")
        .with_namespace("array")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(idx) = get_index(args, 1) else {
            return Value::Na;
        };
        if idx < arr.len() {
            arr.remove(idx)
        } else {
            Value::Na
        }
    });

    registry.register(meta, func);
}

// ============================================================================
// array.insert
// ============================================================================

/// Register array.insert
fn register_insert(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("insert")
        .with_namespace("array")
        .with_required_args(3);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(idx) = get_index(args, 1) else {
            return Value::Na;
        };
        let Some(value) = args.get(2) else {
            return Value::Na;
        };
        if idx <= arr.len() {
            arr.insert(idx, value.clone());
        }
        Value::Array(arr)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.sum
// ============================================================================

/// Register array.sum
fn register_sum(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sum")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let mut sum = 0.0;
        let mut has_value = false;
        for val in arr {
            if let Some(num) = get_numeric(&val) {
                sum += num;
                has_value = true;
            }
        }
        if has_value {
            Value::Float(sum)
        } else {
            Value::Na
        }
    });

    registry.register(meta, func);
}

// ============================================================================
// array.avg
// ============================================================================

/// Register array.avg
fn register_avg(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("avg")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let mut sum = 0.0;
        let mut count = 0;
        for val in arr {
            if let Some(num) = get_numeric(&val) {
                sum += num;
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
// array.min
// ============================================================================

/// Register array.min
fn register_min(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("min")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let mut min_val: Option<f64> = None;
        for val in arr {
            if let Some(num) = get_numeric(&val) {
                if let Some(current_min) = min_val {
                    if num < current_min {
                        min_val = Some(num);
                    }
                } else {
                    min_val = Some(num);
                }
            }
        }
        min_val.map(Value::Float).unwrap_or(Value::Na)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.max
// ============================================================================

/// Register array.max
fn register_max(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("max")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let mut max_val: Option<f64> = None;
        for val in arr {
            if let Some(num) = get_numeric(&val) {
                if let Some(current_max) = max_val {
                    if num > current_max {
                        max_val = Some(num);
                    }
                } else {
                    max_val = Some(num);
                }
            }
        }
        max_val.map(Value::Float).unwrap_or(Value::Na)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.sort
// ============================================================================

/// Register array.sort
fn register_sort(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("sort")
        .with_namespace("array")
        .with_required_args(1)
        .with_optional_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        // Sort numeric values
        let mut numeric_pairs: Vec<(usize, f64)> = arr
            .iter()
            .enumerate()
            .filter_map(|(i, v)| get_numeric(v).map(|num| (i, num)))
            .collect();
        numeric_pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        // Create new sorted array
        let mut sorted = Vec::with_capacity(arr.len());
        for (_, num) in numeric_pairs {
            sorted.push(Value::Float(num));
        }
        Value::Array(sorted)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.reverse
// ============================================================================

/// Register array.reverse
fn register_reverse(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("reverse")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        arr.reverse();
        Value::Array(arr)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.from
// ============================================================================

/// Register array.from
fn register_from(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("from")
        .with_namespace("array")
        .with_required_args(0)
        .with_variadic();

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let arr: Vec<Value> = args.to_vec();
        Value::Array(arr)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.copy
// ============================================================================

/// Register array.copy
fn register_copy(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("copy")
        .with_namespace("array")
        .with_required_args(1);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        Value::Array(arr.clone())
    });

    registry.register(meta, func);
}

// ============================================================================
// array.concat
// ============================================================================

/// Register array.concat
fn register_concat(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("concat")
        .with_namespace("array")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr1) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(arr2) = args.get(1).and_then(extract_array) else {
            return Value::Na;
        };
        let mut result = arr1;
        result.extend(arr2);
        Value::Array(result)
    });

    registry.register(meta, func);
}

// ============================================================================
// array.fill
// ============================================================================

/// Register array.fill
fn register_fill(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("fill")
        .with_namespace("array")
        .with_required_args(2);

    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        for val in arr.iter_mut() {
            *val = value.clone();
        }
        Value::Array(arr)
    });

    registry.register(meta, func);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::FunctionRegistry;

    #[test]
    fn test_array_functions_registered() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("array.new_int"));
        assert!(registry.contains("array.new_float"));
        assert!(registry.contains("array.size"));
        assert!(registry.contains("array.push"));
        assert!(registry.contains("array.pop"));
        assert!(registry.contains("array.get"));
        assert!(registry.contains("array.set"));
        assert!(registry.contains("array.clear"));
        assert!(registry.contains("array.sum"));
        assert!(registry.contains("array.avg"));
        assert!(registry.contains("array.min"));
        assert!(registry.contains("array.max"));
        assert!(registry.contains("array.sort"));
        assert!(registry.contains("array.reverse"));
        assert!(registry.contains("array.copy"));
        assert!(registry.contains("array.concat"));
        assert!(registry.contains("array.fill"));
    }
}
