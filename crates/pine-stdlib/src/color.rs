//! Color functions for Pine Script v6
//!
//! This module provides functions for working with colors.

use pine_runtime::value::{Color, Value};
use std::sync::Arc;

use crate::registry::{BuiltinFn, FunctionMeta, FunctionRegistry};

//==========================================================================
// Color utility functions
//==========================================================================

/// Create a color from RGB values (0-255)
fn color_rgb(args: &[Value]) -> Value {
    let r = match args.first().and_then(|v| v.as_int()) {
        Some(i) if (0..=255).contains(&i) => i as u8,
        _ => return Value::Na,
    };
    let g = match args.get(1).and_then(|v| v.as_int()) {
        Some(i) if (0..=255).contains(&i) => i as u8,
        _ => return Value::Na,
    };
    let b = match args.get(2).and_then(|v| v.as_int()) {
        Some(i) if (0..=255).contains(&i) => i as u8,
        _ => return Value::Na,
    };

    Value::Color(Color::new(r, g, b))
}

/// Create a color from RGBA values (0-255)
fn color_rgba(args: &[Value]) -> Value {
    let r = match args.first().and_then(|v| v.as_int()) {
        Some(i) if (0..=255).contains(&i) => i as u8,
        _ => return Value::Na,
    };
    let g = match args.get(1).and_then(|v| v.as_int()) {
        Some(i) if (0..=255).contains(&i) => i as u8,
        _ => return Value::Na,
    };
    let b = match args.get(2).and_then(|v| v.as_int()) {
        Some(i) if (0..=255).contains(&i) => i as u8,
        _ => return Value::Na,
    };
    let a = match args.get(3).and_then(|v| v.as_int()) {
        Some(i) if (0..=255).contains(&i) => i as u8,
        _ => return Value::Na,
    };

    Value::Color(Color::with_alpha(r, g, b, a))
}

/// Create a color from a hex string
fn color_from_hex(args: &[Value]) -> Value {
    let hex = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Value::Na,
    };

    match Color::from_hex(hex) {
        Some(c) => Value::Color(c),
        None => Value::Na,
    }
}

/// Get the red component of a color (0-255)
fn color_r(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };

    Value::Int(color.r as i64)
}

/// Get the green component of a color (0-255)
fn color_g(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };

    Value::Int(color.g as i64)
}

/// Get the blue component of a color (0-255)
fn color_b(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };

    Value::Int(color.b as i64)
}

/// Get the alpha component of a color (0-255)
fn color_a(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };

    Value::Int(color.a as i64)
}

/// Get the transparency of a color (0-100, where 100 is fully transparent)
fn color_transparency(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };

    let transparency = 100 - (color.a as f64 / 255.0 * 100.0) as i64;
    Value::Int(transparency.clamp(0, 100))
}

/// Set the transparency of a color (0-100)
fn color_new_transparency(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };
    let transparency = match args.get(1).and_then(|v| v.as_int()) {
        Some(i) if (0..=100).contains(&i) => i,
        _ => return Value::Na,
    };

    let alpha = 255 - (transparency as f64 / 100.0 * 255.0) as u8;
    Value::Color(Color::with_alpha(color.r, color.g, color.b, alpha))
}

/// Mix two colors with a given weight
fn color_mix(args: &[Value]) -> Value {
    let color1 = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };
    let color2 = match args.get(1).and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };
    let weight = match args.get(2).and_then(|v| v.as_float()) {
        Some(w) if (0.0..=1.0).contains(&w) => w,
        None => 0.5,
        _ => return Value::Na,
    };

    let r = (color1.r as f64 * (1.0 - weight) + color2.r as f64 * weight) as u8;
    let g = (color1.g as f64 * (1.0 - weight) + color2.g as f64 * weight) as u8;
    let b = (color1.b as f64 * (1.0 - weight) + color2.b as f64 * weight) as u8;
    let a = (color1.a as f64 * (1.0 - weight) + color2.a as f64 * weight) as u8;

    Value::Color(Color::with_alpha(r, g, b, a))
}

/// Lighten a color by a percentage (0-100)
fn color_lighten(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };
    let amount = match args.get(1).and_then(|v| v.as_int()) {
        Some(i) if (0..=100).contains(&i) => i as f64 / 100.0,
        _ => return Value::Na,
    };

    let r = (color.r as f64 + (255.0 - color.r as f64) * amount) as u8;
    let g = (color.g as f64 + (255.0 - color.g as f64) * amount) as u8;
    let b = (color.b as f64 + (255.0 - color.b as f64) * amount) as u8;

    Value::Color(Color::with_alpha(r, g, b, color.a))
}

/// Darken a color by a percentage (0-100)
fn color_darken(args: &[Value]) -> Value {
    let color = match args.first().and_then(|v| v.as_color()) {
        Some(c) => c,
        None => return Value::Na,
    };
    let amount = match args.get(1).and_then(|v| v.as_int()) {
        Some(i) if (0..=100).contains(&i) => i as f64 / 100.0,
        _ => return Value::Na,
    };

    let r = (color.r as f64 * (1.0 - amount)) as u8;
    let g = (color.g as f64 * (1.0 - amount)) as u8;
    let b = (color.b as f64 * (1.0 - amount)) as u8;

    Value::Color(Color::with_alpha(r, g, b, color.a))
}

//==========================================================================
// Registration
//==========================================================================

/// Register all color functions with the function registry
pub fn register_functions(registry: &mut FunctionRegistry) {
    registry.register(
        FunctionMeta::new("rgb")
            .with_namespace("color")
            .with_required_args(3),
        Arc::new(color_rgb) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("rgba")
            .with_namespace("color")
            .with_required_args(4),
        Arc::new(color_rgba) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("from_hex")
            .with_namespace("color")
            .with_required_args(1),
        Arc::new(color_from_hex) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("r")
            .with_namespace("color")
            .with_required_args(1),
        Arc::new(color_r) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("g")
            .with_namespace("color")
            .with_required_args(1),
        Arc::new(color_g) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("b")
            .with_namespace("color")
            .with_required_args(1),
        Arc::new(color_b) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("a")
            .with_namespace("color")
            .with_required_args(1),
        Arc::new(color_a) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("transparency")
            .with_namespace("color")
            .with_required_args(1),
        Arc::new(color_transparency) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("new_transparency")
            .with_namespace("color")
            .with_required_args(2),
        Arc::new(color_new_transparency) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("mix")
            .with_namespace("color")
            .with_required_args(2)
            .with_optional_args(1),
        Arc::new(color_mix) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("lighten")
            .with_namespace("color")
            .with_required_args(2),
        Arc::new(color_lighten) as BuiltinFn,
    );

    registry.register(
        FunctionMeta::new("darken")
            .with_namespace("color")
            .with_required_args(2),
        Arc::new(color_darken) as BuiltinFn,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::FunctionRegistry;

    #[test]
    fn test_color_functions_registered() {
        let mut registry = FunctionRegistry::new();
        register_functions(&mut registry);

        assert!(registry.contains("color.rgb"));
        assert!(registry.contains("color.rgba"));
        assert!(registry.contains("color.from_hex"));
        assert!(registry.contains("color.r"));
        assert!(registry.contains("color.g"));
        assert!(registry.contains("color.b"));
        assert!(registry.contains("color.a"));
        assert!(registry.contains("color.transparency"));
        assert!(registry.contains("color.new_transparency"));
        assert!(registry.contains("color.mix"));
        assert!(registry.contains("color.lighten"));
        assert!(registry.contains("color.darken"));
    }

    #[test]
    fn test_color_rgb() {
        let result = color_rgb(&[Value::Int(255), Value::Int(0), Value::Int(0)]);
        assert_eq!(result, Value::Color(Color::new(255, 0, 0)));

        let result = color_rgb(&[Value::Int(0), Value::Int(255), Value::Int(0)]);
        assert_eq!(result, Value::Color(Color::new(0, 255, 0)));

        let result = color_rgb(&[Value::Int(-1), Value::Int(0), Value::Int(0)]);
        assert!(result.is_na());
    }

    #[test]
    fn test_color_rgba() {
        let result = color_rgba(&[
            Value::Int(255),
            Value::Int(0),
            Value::Int(0),
            Value::Int(128),
        ]);
        assert_eq!(result, Value::Color(Color::with_alpha(255, 0, 0, 128)));
    }

    #[test]
    fn test_color_from_hex() {
        let result = color_from_hex(&[Value::String("#FF0000".into())]);
        assert_eq!(result, Value::Color(Color::new(255, 0, 0)));

        let result = color_from_hex(&[Value::String("#FF000080".into())]);
        assert_eq!(result, Value::Color(Color::with_alpha(255, 0, 0, 128)));

        let result = color_from_hex(&[Value::String("invalid".into())]);
        assert!(result.is_na());
    }

    #[test]
    fn test_color_components() {
        let color = Value::Color(Color::with_alpha(10, 20, 30, 40));

        assert_eq!(color_r(&[color.clone()]), Value::Int(10));
        assert_eq!(color_g(&[color.clone()]), Value::Int(20));
        assert_eq!(color_b(&[color.clone()]), Value::Int(30));
        assert_eq!(color_a(&[color]), Value::Int(40));
    }

    #[test]
    fn test_color_transparency() {
        let color = Value::Color(Color::with_alpha(255, 0, 0, 255));
        assert_eq!(color_transparency(&[color]), Value::Int(0));

        let color = Value::Color(Color::with_alpha(255, 0, 0, 128));
        let transparency = color_transparency(&[color]);
        // Accept either 49 or 50 due to rounding
        assert!(transparency == Value::Int(49) || transparency == Value::Int(50));

        let color = Value::Color(Color::with_alpha(255, 0, 0, 0));
        assert_eq!(color_transparency(&[color]), Value::Int(100));
    }

    #[test]
    fn test_color_new_transparency() {
        let color = Value::Color(Color::new(255, 0, 0));
        let result = color_new_transparency(&[color, Value::Int(50)]);
        if let Value::Color(c) = result {
            assert!(c.a < 255 && c.a > 0);
        } else {
            panic!("Expected color");
        }
    }

    #[test]
    fn test_color_mix() {
        let color1 = Value::Color(Color::new(255, 0, 0));
        let color2 = Value::Color(Color::new(0, 0, 255));

        let result = color_mix(&[color1, color2, Value::Float(0.5)]);
        if let Value::Color(c) = result {
            assert_eq!(c.r, 127);
            assert_eq!(c.b, 127);
        } else {
            panic!("Expected color");
        }
    }

    #[test]
    fn test_color_lighten_darken() {
        let color = Value::Color(Color::new(100, 100, 100));

        let lightened = color_lighten(&[color.clone(), Value::Int(50)]);
        if let Value::Color(c) = lightened {
            assert!(c.r > 100);
        } else {
            panic!("Expected color");
        }

        let darkened = color_darken(&[color, Value::Int(50)]);
        if let Value::Color(c) = darkened {
            assert!(c.r < 100);
        } else {
            panic!("Expected color");
        }
    }
}
