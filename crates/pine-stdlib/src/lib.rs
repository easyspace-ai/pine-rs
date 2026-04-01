//! Pine Script v6 Standard Library
//!
//! This crate provides the built-in functions for Pine Script,
//! including technical analysis (ta.*), math (math.*), string (str.*),
//! array operations, and more.

#![warn(missing_docs)]

pub mod array;
pub mod color;
pub mod input;
pub mod map;
pub mod math;
pub mod registry;
pub mod str;
pub mod ta;

use miette::Diagnostic;
use thiserror::Error;

/// Standard library errors
#[derive(Debug, Error, Diagnostic)]
pub enum StdlibError {
    /// Placeholder error
    #[error("stdlib function not yet implemented")]
    NotImplemented,
}

/// Result type for stdlib operations
pub type Result<T> = std::result::Result<T, StdlibError>;

/// Initialize the standard library function registry
pub fn init(registry: &mut registry::FunctionRegistry) {
    ta::register_functions(registry);
    math::register_functions(registry);
    array::register_functions(registry);
    map::register_functions(registry);
    str::register_functions(registry);
    color::register_functions(registry);
    input::register_functions(registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_registers_functions() {
        let mut registry = registry::FunctionRegistry::new();
        init(&mut registry);

        // Check that ta functions are registered
        assert!(registry.contains("ta.sma"));
        assert!(registry.contains("ta.ema"));
        assert!(registry.contains("ta.rsi"));

        // Check that math functions are registered
        assert!(registry.contains("math.abs"));
        assert!(registry.contains("math.max"));
        assert!(registry.contains("math.sqrt"));

        // Check that array functions are registered
        assert!(registry.contains("array.new_int"));
        assert!(registry.contains("array.size"));
        assert!(registry.contains("array.push"));

        // Check that map functions are registered
        assert!(registry.contains("map.new"));
        assert!(registry.contains("map.get"));
        assert!(registry.contains("map.put"));

        // Check that str functions are registered
        assert!(registry.contains("str.length"));
        assert!(registry.contains("str.substring"));
        assert!(registry.contains("str.concat"));

        // Check that color functions are registered
        assert!(registry.contains("color.rgb"));
        assert!(registry.contains("color.rgba"));
        assert!(registry.contains("color.from_hex"));

        // Check that input functions are registered
        assert!(registry.contains("input.int"));
        assert!(registry.contains("input.float"));
        assert!(registry.contains("input.bool"));
        assert!(registry.contains("input.string"));
        assert!(registry.contains("input.source"));

        // Check total function count
        assert!(registry.len() >= 85); // At least 85 functions should be registered
    }
}
