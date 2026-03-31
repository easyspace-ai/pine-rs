//! Type system for Pine Script semantic analysis

use serde::{Deserialize, Serialize};

/// Pine Script types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PineType {
    /// Integer type
    Int,
    /// Float type
    Float,
    /// Boolean type
    Bool,
    /// String type
    String,
    /// Color type
    Color,
    /// Series of some type
    Series(Box<PineType>),
    /// Array of some type
    Array(Box<PineType>),
    /// Matrix of some type
    Matrix(Box<PineType>),
    /// Map with key and value types
    Map(Box<PineType>, Box<PineType>),
    /// User-defined type
    Udt(String),
    /// Unknown type (for inference)
    Unknown,
    /// Error type
    Error,
}

impl PineType {
    /// Check if this type is a series type
    pub fn is_series(&self) -> bool {
        matches!(self, PineType::Series(_))
    }

    /// Get the inner type of a series
    pub fn series_inner(&self) -> Option<&PineType> {
        match self {
            PineType::Series(inner) => Some(inner),
            _ => None,
        }
    }
}
