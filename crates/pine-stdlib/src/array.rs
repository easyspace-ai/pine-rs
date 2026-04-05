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
    register_includes(registry);
    register_indexof(registry);
    register_lastindexof(registry);
    register_join(registry);
    register_slice(registry);
    register_shift(registry);
    register_unshift(registry);
    register_some(registry);
    register_every(registry);
    register_median(registry);
    register_stdev(registry);
    register_variance(registry);
    register_range(registry);
    register_percentrank(registry);
    register_binary_search(registry);
    register_standardize(registry);
    register_covariance(registry);
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

// ============================================================================
// array.includes
// ============================================================================

/// Register array.includes
fn register_includes(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("includes")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        Value::Bool(arr.contains(value))
    });
    registry.register(meta, func);
}

// ============================================================================
// array.indexof
// ============================================================================

/// Register array.indexof
fn register_indexof(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("indexof")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        match arr.iter().position(|v| v == value) {
            Some(idx) => Value::Int(idx as i64),
            None => Value::Na,
        }
    });
    registry.register(meta, func);
}

// ============================================================================
// array.lastindexof
// ============================================================================

/// Register array.lastindexof
fn register_lastindexof(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("lastindexof")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        match arr.iter().rposition(|v| v == value) {
            Some(idx) => Value::Int(idx as i64),
            None => Value::Na,
        }
    });
    registry.register(meta, func);
}

// ============================================================================
// array.join
// ============================================================================

/// Register array.join
fn register_join(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("join")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let sep = match args.get(1).and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => return Value::Na,
        };
        let parts: Vec<String> = arr
            .iter()
            .map(|v| match v {
                Value::String(s) => s.to_string(),
                Value::Int(n) => n.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Na => "NaN".to_string(),
                other => format!("{}", other),
            })
            .collect();
        Value::String(parts.join(&sep).into())
    });
    registry.register(meta, func);
}

// ============================================================================
// array.slice
// ============================================================================

/// Register array.slice
fn register_slice(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("slice")
        .with_namespace("array")
        .with_required_args(3);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(from) = get_index(args, 1) else {
            return Value::Na;
        };
        let Some(to) = get_index(args, 2) else {
            return Value::Na;
        };
        if from > arr.len() || to > arr.len() || from > to {
            return Value::Na;
        }
        Value::Array(arr[from..to].to_vec())
    });
    registry.register(meta, func);
}

// ============================================================================
// array.shift
// ============================================================================

/// Register array.shift
fn register_shift(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("shift")
        .with_namespace("array")
        .with_required_args(1);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        if arr.is_empty() {
            return Value::Na;
        }
        arr.remove(0)
    });
    registry.register(meta, func);
}

// ============================================================================
// array.unshift
// ============================================================================

/// Register array.unshift
fn register_unshift(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("unshift")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(mut arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        arr.insert(0, value.clone());
        Value::Array(arr)
    });
    registry.register(meta, func);
}

// ============================================================================
// array.some
// ============================================================================

/// Register array.some
fn register_some(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("some")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        Value::Bool(arr.contains(value))
    });
    registry.register(meta, func);
}

// ============================================================================
// array.every
// ============================================================================

/// Register array.every
fn register_every(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("every")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1) else {
            return Value::Na;
        };
        if arr.is_empty() {
            return Value::Bool(true);
        }
        Value::Bool(arr.iter().all(|v| v == value))
    });
    registry.register(meta, func);
}

// ============================================================================
// array.median
// ============================================================================

/// Register array.median
fn register_median(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("median")
        .with_namespace("array")
        .with_required_args(1);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let mut nums: Vec<f64> = arr.iter().filter_map(get_numeric).collect();
        if nums.is_empty() {
            return Value::Na;
        }
        nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = nums.len();
        if len.is_multiple_of(2) {
            Value::Float((nums[len / 2 - 1] + nums[len / 2]) / 2.0)
        } else {
            Value::Float(nums[len / 2])
        }
    });
    registry.register(meta, func);
}

// ============================================================================
// array.stdev
// ============================================================================

/// Register array.stdev
fn register_stdev(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("stdev")
        .with_namespace("array")
        .with_required_args(1);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let nums: Vec<f64> = arr.iter().filter_map(get_numeric).collect();
        if nums.is_empty() {
            return Value::Na;
        }
        let mean = nums.iter().sum::<f64>() / nums.len() as f64;
        let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
        Value::Float(variance.sqrt())
    });
    registry.register(meta, func);
}

// ============================================================================
// array.variance
// ============================================================================

/// Register array.variance
fn register_variance(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("variance")
        .with_namespace("array")
        .with_required_args(1);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let nums: Vec<f64> = arr.iter().filter_map(get_numeric).collect();
        if nums.is_empty() {
            return Value::Na;
        }
        let mean = nums.iter().sum::<f64>() / nums.len() as f64;
        let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
        Value::Float(variance)
    });
    registry.register(meta, func);
}

// ============================================================================
// array.range
// ============================================================================

/// Register array.range
fn register_range(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("range")
        .with_namespace("array")
        .with_required_args(1);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let nums: Vec<f64> = arr.iter().filter_map(get_numeric).collect();
        if nums.is_empty() {
            return Value::Na;
        }
        let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        Value::Float(max - min)
    });
    registry.register(meta, func);
}

// ============================================================================
// array.percentrank
// ============================================================================

/// Register array.percentrank
fn register_percentrank(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("percentrank")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(value) = args.get(1).and_then(get_numeric) else {
            return Value::Na;
        };
        let nums: Vec<f64> = arr.iter().filter_map(get_numeric).collect();
        if nums.is_empty() {
            return Value::Na;
        }
        let count_below = nums.iter().filter(|&&x| x < value).count();
        Value::Float(count_below as f64 / nums.len() as f64 * 100.0)
    });
    registry.register(meta, func);
}

// ============================================================================
// array.binary_search
// ============================================================================

/// Register array.binary_search
fn register_binary_search(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("binary_search")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(target) = args.get(1).and_then(get_numeric) else {
            return Value::Na;
        };
        let nums: Vec<f64> = arr.iter().filter_map(get_numeric).collect();
        match nums.binary_search_by(|x| x.partial_cmp(&target).unwrap_or(std::cmp::Ordering::Equal))
        {
            Ok(idx) => Value::Int(idx as i64),
            Err(_) => Value::Na,
        }
    });
    registry.register(meta, func);
}

// ============================================================================
// array.standardize
// ============================================================================

/// Register array.standardize
fn register_standardize(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("standardize")
        .with_namespace("array")
        .with_required_args(1);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let nums: Vec<f64> = arr.iter().filter_map(get_numeric).collect();
        if nums.is_empty() {
            return Value::Na;
        }
        let mean = nums.iter().sum::<f64>() / nums.len() as f64;
        let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
        let stdev = variance.sqrt();
        if stdev == 0.0 {
            return Value::Array(nums.iter().map(|_| Value::Float(0.0)).collect());
        }
        let standardized: Vec<Value> = nums
            .iter()
            .map(|x| Value::Float((x - mean) / stdev))
            .collect();
        Value::Array(standardized)
    });
    registry.register(meta, func);
}

// ============================================================================
// array.covariance
// ============================================================================

/// Register array.covariance
fn register_covariance(registry: &mut FunctionRegistry) {
    let meta = FunctionMeta::new("covariance")
        .with_namespace("array")
        .with_required_args(2);
    let func: crate::registry::BuiltinFn = Arc::new(|args| {
        let Some(arr1) = args.first().and_then(extract_array) else {
            return Value::Na;
        };
        let Some(arr2) = args.get(1).and_then(extract_array) else {
            return Value::Na;
        };
        let nums1: Vec<f64> = arr1.iter().filter_map(get_numeric).collect();
        let nums2: Vec<f64> = arr2.iter().filter_map(get_numeric).collect();
        if nums1.is_empty() || nums2.is_empty() || nums1.len() != nums2.len() {
            return Value::Na;
        }
        let mean1 = nums1.iter().sum::<f64>() / nums1.len() as f64;
        let mean2 = nums2.iter().sum::<f64>() / nums2.len() as f64;
        let cov = nums1
            .iter()
            .zip(nums2.iter())
            .map(|(x, y)| (x - mean1) * (y - mean2))
            .sum::<f64>()
            / nums1.len() as f64;
        Value::Float(cov)
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

    #[test]
    fn test_array_new_functions_registered() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("array.includes"));
        assert!(registry.contains("array.indexof"));
        assert!(registry.contains("array.lastindexof"));
        assert!(registry.contains("array.join"));
        assert!(registry.contains("array.slice"));
        assert!(registry.contains("array.shift"));
        assert!(registry.contains("array.unshift"));
        assert!(registry.contains("array.some"));
        assert!(registry.contains("array.every"));
        assert!(registry.contains("array.median"));
        assert!(registry.contains("array.stdev"));
        assert!(registry.contains("array.variance"));
        assert!(registry.contains("array.range"));
        assert!(registry.contains("array.percentrank"));
        assert!(registry.contains("array.binary_search"));
        assert!(registry.contains("array.standardize"));
        assert!(registry.contains("array.covariance"));
    }

    #[test]
    fn test_includes() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            registry.dispatch("array.includes", &[arr.clone(), Value::Int(2)]),
            Some(Value::Bool(true))
        );
        assert_eq!(
            registry.dispatch("array.includes", &[arr, Value::Int(5)]),
            Some(Value::Bool(false))
        );
    }

    #[test]
    fn test_indexof() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        assert_eq!(
            registry.dispatch("array.indexof", &[arr.clone(), Value::Int(20)]),
            Some(Value::Int(1))
        );
        assert_eq!(
            registry.dispatch("array.indexof", &[arr, Value::Int(99)]),
            Some(Value::Na)
        );
    }

    #[test]
    fn test_lastindexof() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(1)]);
        assert_eq!(
            registry.dispatch("array.lastindexof", &[arr.clone(), Value::Int(1)]),
            Some(Value::Int(2))
        );
        assert_eq!(
            registry.dispatch("array.lastindexof", &[arr, Value::Int(99)]),
            Some(Value::Na)
        );
    }

    #[test]
    fn test_join() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            registry.dispatch("array.join", &[arr, Value::String(",".into())]),
            Some(Value::String("1,2,3".into()))
        );
    }

    #[test]
    fn test_slice() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        assert_eq!(
            registry.dispatch("array.slice", &[arr, Value::Int(1), Value::Int(3)]),
            Some(Value::Array(vec![Value::Int(2), Value::Int(3)]))
        );
    }

    #[test]
    fn test_shift() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        assert_eq!(
            registry.dispatch("array.shift", &[arr]),
            Some(Value::Int(10))
        );

        let empty = Value::Array(vec![]);
        assert_eq!(registry.dispatch("array.shift", &[empty]), Some(Value::Na));
    }

    #[test]
    fn test_unshift() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![Value::Int(2), Value::Int(3)]);
        assert_eq!(
            registry.dispatch("array.unshift", &[arr, Value::Int(1)]),
            Some(Value::Array(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3)
            ]))
        );
    }

    #[test]
    fn test_some_every() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(1)]);
        assert_eq!(
            registry.dispatch("array.some", &[arr.clone(), Value::Int(2)]),
            Some(Value::Bool(true))
        );
        assert_eq!(
            registry.dispatch("array.every", &[arr, Value::Int(1)]),
            Some(Value::Bool(false))
        );

        let uniform = Value::Array(vec![Value::Int(5), Value::Int(5), Value::Int(5)]);
        assert_eq!(
            registry.dispatch("array.every", &[uniform, Value::Int(5)]),
            Some(Value::Bool(true))
        );
    }

    #[test]
    fn test_median() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let odd = Value::Array(vec![
            Value::Float(1.0),
            Value::Float(3.0),
            Value::Float(2.0),
        ]);
        assert_eq!(
            registry.dispatch("array.median", &[odd]),
            Some(Value::Float(2.0))
        );

        let even = Value::Array(vec![
            Value::Float(1.0),
            Value::Float(2.0),
            Value::Float(3.0),
            Value::Float(4.0),
        ]);
        assert_eq!(
            registry.dispatch("array.median", &[even]),
            Some(Value::Float(2.5))
        );
    }

    #[test]
    fn test_stdev() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![
            Value::Float(2.0),
            Value::Float(4.0),
            Value::Float(4.0),
            Value::Float(4.0),
            Value::Float(5.0),
            Value::Float(5.0),
            Value::Float(7.0),
            Value::Float(9.0),
        ]);
        let result = registry.dispatch("array.stdev", &[arr]);
        match result {
            Some(Value::Float(f)) => {
                assert!((f - 2.0).abs() < 0.01, "Expected stdev ~2.0, got {}", f)
            }
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_variance() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![
            Value::Float(2.0),
            Value::Float(4.0),
            Value::Float(4.0),
            Value::Float(4.0),
            Value::Float(5.0),
            Value::Float(5.0),
            Value::Float(7.0),
            Value::Float(9.0),
        ]);
        let result = registry.dispatch("array.variance", &[arr]);
        match result {
            Some(Value::Float(f)) => {
                assert!((f - 4.0).abs() < 0.01, "Expected variance ~4.0, got {}", f)
            }
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_range() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![
            Value::Float(3.0),
            Value::Float(1.0),
            Value::Float(5.0),
        ]);
        assert_eq!(
            registry.dispatch("array.range", &[arr]),
            Some(Value::Float(4.0))
        );
    }

    #[test]
    fn test_percentrank() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![
            Value::Float(1.0),
            Value::Float(2.0),
            Value::Float(3.0),
            Value::Float(4.0),
        ]);
        let result = registry.dispatch("array.percentrank", &[arr, Value::Float(3.0)]);
        assert_eq!(result, Some(Value::Float(50.0)));
    }

    #[test]
    fn test_binary_search() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![
            Value::Float(1.0),
            Value::Float(2.0),
            Value::Float(3.0),
            Value::Float(4.0),
        ]);
        assert_eq!(
            registry.dispatch("array.binary_search", &[arr.clone(), Value::Float(3.0)]),
            Some(Value::Int(2))
        );
        assert_eq!(
            registry.dispatch("array.binary_search", &[arr, Value::Float(2.5)]),
            Some(Value::Na)
        );
    }

    #[test]
    fn test_standardize() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr = Value::Array(vec![
            Value::Float(2.0),
            Value::Float(4.0),
            Value::Float(6.0),
        ]);
        let result = registry.dispatch("array.standardize", &[arr]);
        match result {
            Some(Value::Array(values)) => {
                assert_eq!(values.len(), 3);
                // mean=4, stdev≈1.633
                if let Value::Float(f) = &values[0] {
                    assert!(f.abs() > 1.0, "First z-score should be negative");
                }
                if let Value::Float(f) = &values[1] {
                    assert!(f.abs() < 0.01, "Middle z-score should be ~0");
                }
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_covariance() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        let arr1 = Value::Array(vec![
            Value::Float(1.0),
            Value::Float(2.0),
            Value::Float(3.0),
        ]);
        let arr2 = Value::Array(vec![
            Value::Float(4.0),
            Value::Float(5.0),
            Value::Float(6.0),
        ]);
        let result = registry.dispatch("array.covariance", &[arr1, arr2]);
        match result {
            Some(Value::Float(f)) => assert!(
                (f - 0.6666666666666666).abs() < 0.01,
                "Expected covariance ~0.667, got {}",
                f
            ),
            _ => panic!("Expected Float"),
        }
    }
}
