//! Runtime configuration for Pine Script execution
//!
//! This module defines `RuntimeConfig` which controls all execution parameters
//! such as max_bars_back, precision, and Pine Script version compatibility.

/// Default maximum bars to look back (TradingView default)
pub const DEFAULT_MAX_BARS_BACK: usize = 500;

/// Default calculation precision bits
pub const DEFAULT_PRECISION: usize = 64;

/// Runtime configuration for Pine Script execution
///
/// This struct controls how the Pine Script engine behaves during execution.
/// It includes limits like max_bars_back which affects how much historical
/// data series can access.
///
/// # Examples
///
/// ```
/// use pine_runtime::config::RuntimeConfig;
///
/// let config = RuntimeConfig::new()
///     .with_max_bars_back(1000)
///     .with_pine_version(6);
///
/// assert_eq!(config.max_bars_back, 1000);
/// assert_eq!(config.pine_version, 6);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Maximum bars to look back for series history (default: 500)
    ///
    /// This controls how much historical data is kept in SeriesBuf.
    /// Once this limit is reached, older bars are overwritten in the circular buffer.
    pub max_bars_back: usize,

    /// Pine Script language version (4, 5, or 6)
    ///
    /// Version 6 is the default and recommended version.
    pub pine_version: u8,

    /// Enable strict mode (additional validation)
    ///
    /// When enabled, certain deprecated or ambiguous constructs will
    /// produce errors instead of warnings.
    pub strict_mode: bool,

    /// Maximum recursion depth for user-defined functions
    ///
    /// This prevents stack overflow from infinite recursion.
    /// Default: 100
    pub max_recursion_depth: usize,

    /// Maximum array size
    ///
    /// Prevents excessive memory usage from large arrays.
    /// Default: 100,000
    pub max_array_size: usize,

    /// Enable runtime bounds checking
    ///
    /// When enabled, series/array access beyond bounds returns na.
    /// When disabled, may panic (for performance in trusted contexts).
    pub bounds_check: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_bars_back: DEFAULT_MAX_BARS_BACK,
            pine_version: 6,
            strict_mode: false,
            max_recursion_depth: 100,
            max_array_size: 100_000,
            bounds_check: true,
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

    /// Set Pine Script version
    pub fn with_pine_version(mut self, version: u8) -> Self {
        self.pine_version = version;
        self
    }

    /// Enable strict mode
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Set maximum recursion depth
    pub fn with_max_recursion_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Set maximum array size
    pub fn with_max_array_size(mut self, size: usize) -> Self {
        self.max_array_size = size;
        self
    }

    /// Enable/disable bounds checking
    pub fn with_bounds_check(mut self, check: bool) -> Self {
        self.bounds_check = check;
        self
    }

    /// Validate the configuration
    ///
    /// Returns an error string if the configuration is invalid.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.max_bars_back == 0 {
            return Err("max_bars_back must be greater than 0");
        }
        if self.max_bars_back > 50_000 {
            return Err("max_bars_back cannot exceed 50000");
        }
        if ![4, 5, 6].contains(&self.pine_version) {
            return Err("pine_version must be 4, 5, or 6");
        }
        if self.max_recursion_depth == 0 {
            return Err("max_recursion_depth must be greater than 0");
        }
        if self.max_array_size == 0 {
            return Err("max_array_size must be greater than 0");
        }
        Ok(())
    }
}

/// Data feed configuration
///
/// Controls how external data (OHLCV) is loaded and processed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFeedConfig {
    /// Symbol being traded (e.g., "BTCUSDT")
    pub symbol: String,
    /// Timeframe (e.g., "1h", "1D")
    pub timeframe: String,
    /// Exchange identifier
    pub exchange: Option<String>,
    /// Currency for calculations
    pub currency: Option<String>,
}

impl DataFeedConfig {
    /// Create a new data feed configuration
    pub fn new(symbol: impl Into<String>, timeframe: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            timeframe: timeframe.into(),
            exchange: None,
            currency: None,
        }
    }

    /// Set the exchange
    pub fn with_exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = Some(exchange.into());
        self
    }

    /// Set the currency
    pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert_eq!(config.max_bars_back, DEFAULT_MAX_BARS_BACK);
        assert_eq!(config.pine_version, 6);
        assert!(!config.strict_mode);
        assert_eq!(config.max_recursion_depth, 100);
        assert_eq!(config.max_array_size, 100_000);
        assert!(config.bounds_check);
    }

    #[test]
    fn test_builder_pattern() {
        let config = RuntimeConfig::new()
            .with_max_bars_back(1000)
            .with_pine_version(5)
            .with_strict_mode(true)
            .with_max_recursion_depth(50)
            .with_max_array_size(10_000)
            .with_bounds_check(false);

        assert_eq!(config.max_bars_back, 1000);
        assert_eq!(config.pine_version, 5);
        assert!(config.strict_mode);
        assert_eq!(config.max_recursion_depth, 50);
        assert_eq!(config.max_array_size, 10_000);
        assert!(!config.bounds_check);
    }

    #[test]
    fn test_validate() {
        assert!(RuntimeConfig::default().validate().is_ok());

        let config = RuntimeConfig::new().with_max_bars_back(0);
        assert!(config.validate().is_err());

        let config = RuntimeConfig::new().with_max_bars_back(100_000);
        assert!(config.validate().is_err());

        let config = RuntimeConfig::new().with_pine_version(3);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_data_feed_config() {
        let config = DataFeedConfig::new("BTCUSDT", "1h")
            .with_exchange("Binance")
            .with_currency("USD");

        assert_eq!(config.symbol, "BTCUSDT");
        assert_eq!(config.timeframe, "1h");
        assert_eq!(config.exchange, Some("Binance".to_string()));
        assert_eq!(config.currency, Some("USD".to_string()));
    }
}
