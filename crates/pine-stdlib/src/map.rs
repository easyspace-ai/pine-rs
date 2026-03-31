//! Map/dictionary functions for Pine Script v6
//!
//! This module provides functions for working with maps/dictionaries
//! with string keys and any Value as values.

use pine_runtime::value::{Map, Value};
use std::sync::Arc;

use crate::registry::{BuiltinFn, FunctionMeta, FunctionRegistry};

//==========================================================================
// Map constructor functions
//==========================================================================

/// Create a new empty map
fn map_new(_args: &[Value]) -> Value {
    Value::new_map()
}

/// Create a new map with a single key-value pair
fn map_new_from_pair(args: &[Value]) -> Value {
    let key = match args.get(0).and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return Value::Na,
    };
    let value = args.get(1).cloned().unwrap_or(Value::Na);

    let mut map = Map::new();
    map.set(key, value);
    Value::Map(Arc::new(map))
}

//==========================================================================
// Map accessor functions
//==========================================================================

/// Get a value from a map by key
fn map_get(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Na,
    };
    let key = match args.get(1).and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return Value::Na,
    };

    if let Some(map) = map_arg.as_map() {
        map.get(key).cloned().unwrap_or(Value::Na)
    } else {
        Value::Na
    }
}

/// Set a value in a map by key (mutates the map)
fn map_put(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Na,
    };
    let key = match args.get(1).and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return Value::Na,
    };
    let value = args.get(2).cloned().unwrap_or(Value::Na);

    // Clone the map and modify it (since we have Arc)
    let mut new_map = match map_arg.as_map() {
        Some(m) => (*m).clone(),
        None => return Value::Na,
    };
    new_map.set(key, value);
    Value::Map(Arc::new(new_map))
}

/// Remove a key from a map (mutates the map)
fn map_remove(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Na,
    };
    let key = match args.get(1).and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return Value::Na,
    };

    let mut new_map = match map_arg.as_map() {
        Some(m) => (*m).clone(),
        None => return Value::Na,
    };
    new_map.remove(key);
    Value::Map(Arc::new(new_map))
}

/// Check if a map contains a key
fn map_contains(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Bool(false),
    };
    let key = match args.get(1).and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return Value::Bool(false),
    };

    if let Some(map) = map_arg.as_map() {
        Value::Bool(map.contains_key(key))
    } else {
        Value::Bool(false)
    }
}

/// Get the size of a map
fn map_size(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Int(0),
    };

    if let Some(map) = map_arg.as_map() {
        Value::Int(map.len() as i64)
    } else {
        Value::Int(0)
    }
}

/// Check if a map is empty
fn map_is_empty(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Bool(true),
    };

    if let Some(map) = map_arg.as_map() {
        Value::Bool(map.is_empty())
    } else {
        Value::Bool(true)
    }
}

/// Clear all entries from a map (mutates the map)
fn map_clear(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Na,
    };

    let mut new_map = match map_arg.as_map() {
        Some(m) => (*m).clone(),
        None => return Value::Na,
    };
    new_map.clear();
    Value::Map(Arc::new(new_map))
}

/// Get all keys from a map as an array
fn map_keys(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Array(Vec::new()),
    };

    if let Some(map) = map_arg.as_map() {
        let keys: Vec<Value> = map.keys().map(|k| Value::String(k.clone().into())).collect();
        Value::Array(keys)
    } else {
        Value::Array(Vec::new())
    }
}

/// Get all values from a map as an array
fn map_values(args: &[Value]) -> Value {
    let map_arg = match args.first() {
        Some(m) if m.is_map() => m,
        _ => return Value::Array(Vec::new()),
    };

    if let Some(map) = map_arg.as_map() {
        let values: Vec<Value> = map.values().cloned().collect();
        Value::Array(values)
    } else {
        Value::Array(Vec::new())
    }
}

//==========================================================================
// Registration
//==========================================================================

/// Register all map functions with the function registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    // Constructors
    registry.register(
        FunctionMeta::new("new")
            .with_namespace("map")
            .with_required_args(0),
        Arc::new(map_new) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("new_from_pair")
            .with_namespace("map")
            .with_required_args(2),
        Arc::new(map_new_from_pair) as BuiltinFn,
    );

    // Accessors
    registry.register(
        FunctionMeta::new("get")
            .with_namespace("map")
            .with_required_args(2),
        Arc::new(map_get) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("put")
            .with_namespace("map")
            .with_required_args(3),
        Arc::new(map_put) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("remove")
            .with_namespace("map")
            .with_required_args(2),
        Arc::new(map_remove) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("contains")
            .with_namespace("map")
            .with_required_args(2),
        Arc::new(map_contains) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("size")
            .with_namespace("map")
            .with_required_args(1),
        Arc::new(map_size) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("is_empty")
            .with_namespace("map")
            .with_required_args(1),
        Arc::new(map_is_empty) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("clear")
            .with_namespace("map")
            .with_required_args(1),
        Arc::new(map_clear) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("keys")
            .with_namespace("map")
            .with_required_args(1),
        Arc::new(map_keys) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("values")
            .with_namespace("map")
            .with_required_args(1),
        Arc::new(map_values) as BuiltinFn,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::FunctionRegistry;

    #[test]
    fn test_map_functions_registered() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("map.new"));
        assert!(registry.contains("map.get"));
        assert!(registry.contains("map.put"));
        assert!(registry.contains("map.remove"));
        assert!(registry.contains("map.contains"));
        assert!(registry.contains("map.size"));
        assert!(registry.contains("map.is_empty"));
        assert!(registry.contains("map.clear"));
        assert!(registry.contains("map.keys"));
        assert!(registry.contains("map.values"));
    }

    #[test]
    fn test_map_new() {
        let result = map_new(&[]);
        assert!(result.is_map());
        if let Some(map) = result.as_map() {
            assert!(map.is_empty());
        }
    }

    #[test]
    fn test_map_get_put() {
        // Create a map
        let map = map_new(&[]);

        // Put a value
        let map = map_put(&[map, Value::String("key".into()), Value::Int(42)]);
        assert!(!map.is_na());

        // Get the value
        let result = map_get(&[map.clone(), Value::String("key".into())]);
        assert_eq!(result, Value::Int(42));

        // Get non-existent key
        let result = map_get(&[map, Value::String("nonexistent".into())]);
        assert!(result.is_na());
    }

    #[test]
    fn test_map_contains_size() {
        let map = map_new(&[]);
        let map = map_put(&[map, Value::String("a".into()), Value::Int(1)]);
        let map = map_put(&[map, Value::String("b".into()), Value::Int(2)]);

        let contains_a = map_contains(&[map.clone(), Value::String("a".into())]);
        assert_eq!(contains_a, Value::Bool(true));

        let contains_c = map_contains(&[map.clone(), Value::String("c".into())]);
        assert_eq!(contains_c, Value::Bool(false));

        let size = map_size(&[map]);
        assert_eq!(size, Value::Int(2));
    }

    #[test]
    fn test_map_remove() {
        let map = map_new(&[]);
        let map = map_put(&[map, Value::String("key".into()), Value::Int(42)]);

        let contains_before = map_contains(&[map.clone(), Value::String("key".into())]);
        assert_eq!(contains_before, Value::Bool(true));

        let map = map_remove(&[map, Value::String("key".into())]);

        let contains_after = map_contains(&[map, Value::String("key".into())]);
        assert_eq!(contains_after, Value::Bool(false));
    }

    #[test]
    fn test_map_clear() {
        let map = map_new(&[]);
        let map = map_put(&[map, Value::String("a".into()), Value::Int(1)]);
        let map = map_put(&[map, Value::String("b".into()), Value::Int(2)]);

        let size_before = map_size(&[map.clone()]);
        assert_eq!(size_before, Value::Int(2));

        let map = map_clear(&[map]);

        let size_after = map_size(&[map]);
        assert_eq!(size_after, Value::Int(0));
    }

    #[test]
    fn test_map_keys_values() {
        let map = map_new(&[]);
        let map = map_put(&[map, Value::String("a".into()), Value::Int(1)]);
        let map = map_put(&[map, Value::String("b".into()), Value::Int(2)]);

        let keys = map_keys(&[map.clone()]);
        assert!(keys.is_array());
        if let Some(arr) = keys.as_array() {
            assert_eq!(arr.len(), 2);
        }

        let values = map_values(&[map]);
        assert!(values.is_array());
        if let Some(arr) = values.as_array() {
            assert_eq!(arr.len(), 2);
        }
    }
}
