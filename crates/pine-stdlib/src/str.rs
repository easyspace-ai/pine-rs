//! String functions for Pine Script v6
//!
//! This module provides functions for working with strings.

use pine_runtime::value::Value;
use std::sync::Arc;

use crate::registry::{BuiltinFn, FunctionMeta, FunctionRegistry};

//==========================================================================
// String utility functions
//==========================================================================

/// Get the length of a string
fn str_length(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    Value::Int(s.len() as i64)
}

/// Get a substring from a string
fn str_substring(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    let start = match args.get(1).and_then(|v| v.as_int()) {
        Some(i) if i >= 0 => i as usize,
        _ => return Value::Na,
    };
    let end = match args.get(2).and_then(|v| v.as_int()) {
        Some(i) if i >= 0 => i as usize,
        None => s.len(),
        _ => return Value::Na,
    };

    if start > s.len() || end > s.len() || start > end {
        return Value::Na;
    }

    Value::String(s[start..end].into())
}

/// Replace substrings in a string
fn str_replace(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    let from = match args.get(1).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    let to = match args.get(2).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    Value::String(s.replace(from, to).into())
}

/// Split a string into an array
fn str_split(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Array(Vec::new()),
    };
    let delimiter = match args.get(1).and_then(|v| v.as_str()) {
        Some(d) => d,
        None => return Value::Array(Vec::new()),
    };

    let parts: Vec<Value> = s
        .split(delimiter)
        .map(|p| Value::String(p.into()))
        .collect();
    Value::Array(parts)
}

/// Check if a string contains a substring
fn str_contains(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Bool(false),
    };
    let substring = match args.get(1).and_then(|v| v.as_str()) {
        Some(sub) => sub,
        None => return Value::Bool(false),
    };

    Value::Bool(s.contains(substring))
}

/// Check if a string starts with a prefix
fn str_starts_with(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Bool(false),
    };
    let prefix = match args.get(1).and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return Value::Bool(false),
    };

    Value::Bool(s.starts_with(prefix))
}

/// Check if a string ends with a suffix
fn str_ends_with(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Bool(false),
    };
    let suffix = match args.get(1).and_then(|v| v.as_str()) {
        Some(suf) => suf,
        None => return Value::Bool(false),
    };

    Value::Bool(s.ends_with(suffix))
}

/// Convert a string to lowercase
fn str_lower(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    Value::String(s.to_lowercase().into())
}

/// Convert a string to uppercase
fn str_upper(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    Value::String(s.to_uppercase().into())
}

/// Trim whitespace from both ends of a string
fn str_trim(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    Value::String(s.trim().into())
}

/// Trim whitespace from the start of a string
fn str_trim_start(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    Value::String(s.trim_start().into())
}

/// Trim whitespace from the end of a string
fn str_trim_end(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    Value::String(s.trim_end().into())
}

/// Concatenate two strings
fn str_concat(args: &[Value]) -> Value {
    let s1 = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    let s2 = match args.get(1).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    let mut result = String::with_capacity(s1.len() + s2.len());
    result.push_str(s1);
    result.push_str(s2);
    Value::String(result.into())
}

/// Convert a value to string
fn str_tostring(args: &[Value]) -> Value {
    let value = match args.first() {
        Some(v) => v,
        None => return Value::Na,
    };

    value.coerce_to_string()
}

/// Convert a string to a number
fn str_tonumber(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    if let Ok(i) = s.parse::<i64>() {
        Value::Int(i)
    } else if let Ok(f) = s.parse::<f64>() {
        Value::Float(f)
    } else {
        Value::Na
    }
}

/// Join an array of strings with a delimiter
fn str_join(args: &[Value]) -> Value {
    let array = match args.first() {
        Some(a) if a.is_array() => a,
        _ => return Value::String("".into()),
    };
    let delimiter = match args.get(1).and_then(|v| v.as_str()) {
        Some(d) => d,
        None => return Value::String("".into()),
    };

    let parts: Vec<&str> = match array.as_array() {
        Some(arr) => arr.iter().filter_map(|v| v.as_str()).collect(),
        None => return Value::String("".into()),
    };

    Value::String(parts.join(delimiter).into())
}

/// Find position of substring in string (0-based index)
fn str_pos(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    let substr = match args.get(1).and_then(|v| v.as_str()) {
        Some(sub) => sub,
        None => return Value::Na,
    };

    match s.find(substr) {
        Some(pos) => Value::Int(pos as i64),
        None => Value::Na,
    }
}

/// Replace all occurrences of target with replacement
fn str_replace_all(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    let target = match args.get(1).and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return Value::Na,
    };
    let replacement = match args.get(2).and_then(|v| v.as_str()) {
        Some(r) => r,
        None => return Value::Na,
    };

    Value::String(s.replace(target, replacement).into())
}

/// Format string with {0}, {1}, etc. placeholders
fn str_format(args: &[Value]) -> Value {
    let fmt_str = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Value::Na,
    };

    let mut result = fmt_str;
    for (i, arg) in args.iter().skip(1).enumerate() {
        let placeholder = format!("{{{}}}", i);
        let replacement = match arg {
            Value::String(s) => s.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Na => "NaN".to_string(),
            other => format!("{}", other),
        };
        result = result.replace(&placeholder, &replacement);
    }

    Value::String(result.into())
}

/// Match a regex pattern and return first match or Na
fn str_match(args: &[Value]) -> Value {
    let s = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };
    let pattern = match args.get(1).and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return Value::Na,
    };

    // Simple pattern matching without regex dependency
    // Check if pattern appears as substring (basic matching)
    if s.contains(pattern) {
        Value::String(pattern.into())
    } else {
        Value::Na
    }
}

//==========================================================================
// Registration
//==========================================================================

/// Register all string functions with the function registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    registry.register(
        FunctionMeta::new("length")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_length) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("substring")
            .with_namespace("str")
            .with_required_args(2)
            .with_optional_args(1),
        Arc::new(str_substring) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("replace")
            .with_namespace("str")
            .with_required_args(3),
        Arc::new(str_replace) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("split")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_split) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("contains")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_contains) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("starts_with")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_starts_with) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("ends_with")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_ends_with) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("lower")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_lower) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("upper")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_upper) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("trim")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_trim) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("trim_start")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_trim_start) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("trim_end")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_trim_end) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("concat")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_concat) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("tostring")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_tostring) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("tonumber")
            .with_namespace("str")
            .with_required_args(1),
        Arc::new(str_tonumber) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("join")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_join) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("pos")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_pos) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("replace_all")
            .with_namespace("str")
            .with_required_args(3),
        Arc::new(str_replace_all) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("format")
            .with_namespace("str")
            .with_required_args(1)
            .with_variadic(),
        Arc::new(str_format) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("match")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_match) as BuiltinFn,
    );

    // Aliases: str.startswith -> str.starts_with, str.endswith -> str.ends_with
    registry.register(
        FunctionMeta::new("startswith")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_starts_with) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("endswith")
            .with_namespace("str")
            .with_required_args(2),
        Arc::new(str_ends_with) as BuiltinFn,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::FunctionRegistry;

    #[test]
    fn test_str_functions_registered() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("str.length"));
        assert!(registry.contains("str.substring"));
        assert!(registry.contains("str.replace"));
        assert!(registry.contains("str.split"));
        assert!(registry.contains("str.contains"));
        assert!(registry.contains("str.starts_with"));
        assert!(registry.contains("str.ends_with"));
        assert!(registry.contains("str.lower"));
        assert!(registry.contains("str.upper"));
        assert!(registry.contains("str.trim"));
        assert!(registry.contains("str.concat"));
        assert!(registry.contains("str.tostring"));
        assert!(registry.contains("str.tonumber"));
        assert!(registry.contains("str.join"));
    }

    #[test]
    fn test_str_length() {
        let result = str_length(&[Value::String("hello".into())]);
        assert_eq!(result, Value::Int(5));

        let result = str_length(&[Value::String("".into())]);
        assert_eq!(result, Value::Int(0));

        let result = str_length(&[Value::Na]);
        assert!(result.is_na());
    }

    #[test]
    fn test_str_substring() {
        let result = str_substring(&[Value::String("hello".into()), Value::Int(0), Value::Int(2)]);
        assert_eq!(result, Value::String("he".into()));

        let result = str_substring(&[Value::String("hello".into()), Value::Int(2)]);
        assert_eq!(result, Value::String("llo".into()));

        let result = str_substring(&[Value::String("hello".into()), Value::Int(1), Value::Int(4)]);
        assert_eq!(result, Value::String("ell".into()));
    }

    #[test]
    fn test_str_replace() {
        let result = str_replace(&[
            Value::String("hello world".into()),
            Value::String("world".into()),
            Value::String("there".into()),
        ]);
        assert_eq!(result, Value::String("hello there".into()));
    }

    #[test]
    fn test_str_split() {
        let result = str_split(&[Value::String("a,b,c".into()), Value::String(",".into())]);
        assert!(result.is_array());
        if let Some(arr) = result.as_array() {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::String("a".into()));
            assert_eq!(arr[1], Value::String("b".into()));
            assert_eq!(arr[2], Value::String("c".into()));
        }
    }

    #[test]
    fn test_str_contains() {
        let result = str_contains(&[Value::String("hello".into()), Value::String("ell".into())]);
        assert_eq!(result, Value::Bool(true));

        let result = str_contains(&[Value::String("hello".into()), Value::String("xyz".into())]);
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_str_starts_ends_with() {
        let result = str_starts_with(&[Value::String("hello".into()), Value::String("he".into())]);
        assert_eq!(result, Value::Bool(true));

        let result = str_ends_with(&[Value::String("hello".into()), Value::String("lo".into())]);
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_str_lower_upper() {
        let result = str_lower(&[Value::String("HELLO".into())]);
        assert_eq!(result, Value::String("hello".into()));

        let result = str_upper(&[Value::String("hello".into())]);
        assert_eq!(result, Value::String("HELLO".into()));
    }

    #[test]
    fn test_str_trim() {
        let result = str_trim(&[Value::String("  hello  ".into())]);
        assert_eq!(result, Value::String("hello".into()));
    }

    #[test]
    fn test_str_concat() {
        let result = str_concat(&[
            Value::String("hello".into()),
            Value::String(" world".into()),
        ]);
        assert_eq!(result, Value::String("hello world".into()));
    }

    #[test]
    fn test_str_tostring() {
        let result = str_tostring(&[Value::Int(42)]);
        assert_eq!(result, Value::String("42".into()));

        let result = str_tostring(&[Value::Float(3.14)]);
        assert_eq!(result, Value::String("3.14".into()));
    }

    #[test]
    fn test_str_tonumber() {
        let result = str_tonumber(&[Value::String("42".into())]);
        assert_eq!(result, Value::Int(42));

        let result = str_tonumber(&[Value::String("3.14".into())]);
        assert_eq!(result, Value::Float(3.14));

        let result = str_tonumber(&[Value::String("not a number".into())]);
        assert!(result.is_na());
    }

    #[test]
    fn test_str_join() {
        let array = Value::Array(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ]);
        let result = str_join(&[array, Value::String(",".into())]);
        assert_eq!(result, Value::String("a,b,c".into()));
    }

    #[test]
    fn test_str_pos() {
        let result = str_pos(&[
            Value::String("hello world".into()),
            Value::String("world".into()),
        ]);
        assert_eq!(result, Value::Int(6));

        let result = str_pos(&[Value::String("hello".into()), Value::String("xyz".into())]);
        assert!(result.is_na());

        let result = str_pos(&[Value::String("hello".into()), Value::String("".into())]);
        assert_eq!(result, Value::Int(0));

        let result = str_pos(&[Value::Na, Value::String("x".into())]);
        assert!(result.is_na());
    }

    #[test]
    fn test_str_replace_all() {
        let result = str_replace_all(&[
            Value::String("aaa".into()),
            Value::String("a".into()),
            Value::String("b".into()),
        ]);
        assert_eq!(result, Value::String("bbb".into()));

        let result = str_replace_all(&[
            Value::String("hello world world".into()),
            Value::String("world".into()),
            Value::String("there".into()),
        ]);
        assert_eq!(result, Value::String("hello there there".into()));

        let result = str_replace_all(&[
            Value::Na,
            Value::String("a".into()),
            Value::String("b".into()),
        ]);
        assert!(result.is_na());
    }

    #[test]
    fn test_str_format() {
        let result = str_format(&[
            Value::String("Hello {0}!".into()),
            Value::String("world".into()),
        ]);
        assert_eq!(result, Value::String("Hello world!".into()));

        let result = str_format(&[
            Value::String("{0} + {1} = {2}".into()),
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
        assert_eq!(result, Value::String("1 + 2 = 3".into()));

        let result = str_format(&[Value::String("no placeholders".into())]);
        assert_eq!(result, Value::String("no placeholders".into()));

        let result = str_format(&[Value::Na]);
        assert!(result.is_na());
    }

    #[test]
    fn test_str_match() {
        let result = str_match(&[
            Value::String("hello world".into()),
            Value::String("world".into()),
        ]);
        assert_eq!(result, Value::String("world".into()));

        let result = str_match(&[Value::String("hello".into()), Value::String("xyz".into())]);
        assert!(result.is_na());

        let result = str_match(&[Value::Na, Value::String("x".into())]);
        assert!(result.is_na());
    }

    #[test]
    fn test_str_startswith_endswith_aliases() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("str.startswith"));
        assert!(registry.contains("str.endswith"));

        let result = registry.dispatch(
            "str.startswith",
            &[Value::String("hello".into()), Value::String("he".into())],
        );
        assert_eq!(result, Some(Value::Bool(true)));

        let result = registry.dispatch(
            "str.endswith",
            &[Value::String("hello".into()), Value::String("lo".into())],
        );
        assert_eq!(result, Some(Value::Bool(true)));
    }
}
