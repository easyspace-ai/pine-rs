//! Runtime configuration

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum bars to look back (default: 500)
    pub max_bars_back: usize,
    /// Calculation precision (default: f64)
    pub precision: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_bars_back: 500,
            precision: 64,
        }
    }
}

impl RuntimeConfig {
    /// Create a new runtime config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max_bars_back
    pub fn with_max_bars_back(mut self, max_bars_back: usize) -> Self {
        self.max_bars_back = max_bars_back;
        self
    }
}
