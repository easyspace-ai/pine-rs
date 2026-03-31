//! Pine Script v6 Output Layer
//!
//! This crate provides output handling for Pine Script,
//! including plots, drawings (labels, boxes, tables), and strategy signals.

#![warn(missing_docs)]

pub mod plot;
pub mod drawing;
pub mod strategy;

use miette::Diagnostic;
use thiserror::Error;

/// Output errors
#[derive(Debug, Error, Diagnostic)]
pub enum OutputError {
    /// Placeholder error
    #[error("output not yet implemented")]
    NotImplemented,
}

/// Result type for output operations
pub type Result<T> = std::result::Result<T, OutputError>;

/// Collect and format script output
pub fn collect() -> Result<()> {
    // TODO: Implement output collection
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert!(collect().is_ok());
    }
}
