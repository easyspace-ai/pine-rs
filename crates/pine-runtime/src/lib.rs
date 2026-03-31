//! Pine Script v6 Runtime
//!
//! This crate provides the core runtime types and operations for Pine Script,
//! including Value types, Series buffers, NA propagation, and execution context.

#![warn(missing_docs)]

pub mod config;
pub mod context;
pub mod module;
pub mod na_ops;
pub mod series;
pub mod value;

use miette::Diagnostic;
use thiserror::Error;

/// Runtime errors
#[derive(Debug, Error, Diagnostic)]
pub enum RuntimeError {
    /// Placeholder error
    #[error("runtime not yet implemented")]
    NotImplemented,
}

/// Result type for runtime operations
pub type Result<T> = std::result::Result<T, RuntimeError>;

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
