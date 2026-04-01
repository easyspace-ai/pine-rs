//! Data loader for pine-tv
//! Loads OHLCV data from CSV files or Binance API.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::binance::BinanceClient;

/// OHLCV bar data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvBar {
    /// Unix timestamp in seconds
    pub time: i64,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume
    pub volume: f64,
}

impl OhlcvBar {
    /// Create a new OHLCV bar
    pub fn new(time: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            time,
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

/// Error type for data loading
#[derive(Debug, thiserror::Error)]
pub enum DataLoadError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("CSV parse error at line {line}: {msg}")]
    CsvParse { line: usize, msg: String },

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("No data found")]
    NoData,

    #[error("Binance API error: {0}")]
    Binance(#[from] crate::data::binance::BinanceError),
}

/// Data loader for OHLCV data
pub struct DataLoader {
    data_dir: PathBuf,
    #[allow(dead_code)]
    binance_client: BinanceClient,
}

impl DataLoader {
    /// Create a new DataLoader with the given data directory
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            binance_client: BinanceClient::new(),
        }
    }

    /// Load OHLCV data for a symbol and timeframe
    #[allow(dead_code)]
    pub async fn load(
        &self,
        symbol: &str,
        timeframe: &str,
    ) -> Result<Vec<OhlcvBar>, DataLoadError> {
        match self.load_from_binance(symbol, timeframe, 500).await {
            Ok(bars) if !bars.is_empty() => Ok(bars),
            _ => self.load_local(symbol, timeframe),
        }
    }

    /// Load data from Binance API
    #[allow(dead_code)]
    pub async fn load_from_binance(
        &self,
        symbol: &str,
        timeframe: &str,
        limit: usize,
    ) -> Result<Vec<OhlcvBar>, DataLoadError> {
        // Map pine-tv timeframe to Binance interval
        let binance_interval = match timeframe {
            "1m" => "1m",
            "5m" => "5m",
            "15m" => "15m",
            "1h" => "1h",
            "4h" => "4h",
            "1d" => "1d",
            "1w" => "1w",
            _ => "1h",
        };

        let bars = self
            .binance_client
            .fetch_klines(symbol, binance_interval, limit)
            .await?;

        Ok(bars)
    }

    /// Load data from local sources (CSV or sample)
    pub fn load_local(
        &self,
        symbol: &str,
        timeframe: &str,
    ) -> Result<Vec<OhlcvBar>, DataLoadError> {
        // Try to find a matching CSV file
        let filename = format!("{}_{}.csv", symbol, timeframe);
        let path = self.data_dir.join(&filename);

        if path.exists() {
            return self.load_csv(&path);
        }

        // Try alternative filename format
        let filename = format!("{}.csv", symbol);
        let path = self.data_dir.join(&filename);

        if path.exists() {
            return self.load_csv(&path);
        }

        // Fall back to test data directory
        let test_data_path = Path::new("tests/data").join(&filename);
        if test_data_path.exists() {
            return self.load_csv(&test_data_path);
        }

        // If no file found, generate sample data
        Ok(self.generate_sample_data(symbol, timeframe))
    }

    /// Load data from a CSV file
    fn load_csv<P: AsRef<Path>>(&self, path: P) -> Result<Vec<OhlcvBar>, DataLoadError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut bars = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Skip header if present
            if line_num == 0 && line.to_lowercase().contains("time") {
                continue;
            }

            let bar = self.parse_csv_line(line, line_num + 1)?;
            bars.push(bar);
        }

        if bars.is_empty() {
            return Err(DataLoadError::NoData);
        }

        // Sort by time
        bars.sort_by_key(|b| b.time);

        Ok(bars)
    }

    /// Parse a single CSV line into an OhlcvBar
    fn parse_csv_line(&self, line: &str, line_num: usize) -> Result<OhlcvBar, DataLoadError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() < 6 {
            return Err(DataLoadError::CsvParse {
                line: line_num,
                msg: format!("expected at least 6 columns, got {}", parts.len()),
            });
        }

        // Parse time - can be Unix timestamp or ISO string
        let time = self.parse_timestamp(parts[0])?;

        let open = parts[1]
            .parse::<f64>()
            .map_err(|_| DataLoadError::InvalidNumber(format!("invalid open: {}", parts[1])))?;

        let high = parts[2]
            .parse::<f64>()
            .map_err(|_| DataLoadError::InvalidNumber(format!("invalid high: {}", parts[2])))?;

        let low = parts[3]
            .parse::<f64>()
            .map_err(|_| DataLoadError::InvalidNumber(format!("invalid low: {}", parts[3])))?;

        let close = parts[4]
            .parse::<f64>()
            .map_err(|_| DataLoadError::InvalidNumber(format!("invalid close: {}", parts[4])))?;

        let volume = parts[5]
            .parse::<f64>()
            .map_err(|_| DataLoadError::InvalidNumber(format!("invalid volume: {}", parts[5])))?;

        Ok(OhlcvBar::new(time, open, high, low, close, volume))
    }

    /// Parse a timestamp from string
    fn parse_timestamp(&self, s: &str) -> Result<i64, DataLoadError> {
        // Try as Unix timestamp first
        if let Ok(ts) = s.parse::<i64>() {
            return Ok(ts);
        }

        // Try as ISO 8601
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.timestamp());
        }

        // Try as other formats
        if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            return Ok(dt.timestamp());
        }

        Err(DataLoadError::InvalidTimestamp(s.to_string()))
    }

    /// Generate sample data for demonstration
    fn generate_sample_data(&self, symbol: &str, timeframe: &str) -> Vec<OhlcvBar> {
        let mut bars = Vec::new();
        let now = Utc::now().timestamp();
        let interval = self.timeframe_to_seconds(timeframe);

        let base_price = match symbol.to_uppercase().as_str() {
            "BTCUSDT" => 42000.0,
            "ETHUSDT" => 2200.0,
            _ => 100.0,
        };

        let mut price = base_price;

        for i in (0..500).rev() {
            let time = now - i * interval;
            let volatility = base_price * 0.02;

            // Simple pseudo-random using a deterministic formula
            let seed = (time as u64).wrapping_mul(1103515245).wrapping_add(12345);
            let r1 = (seed % 10000) as f64 / 10000.0;
            let r2 = ((seed >> 16) % 10000) as f64 / 10000.0;
            let r3 = ((seed >> 32) % 10000) as f64 / 10000.0;

            let change = (r1 - 0.5) * volatility;

            let open = price;
            price += change;
            let close = price;
            let high = f64::max(open, close) + r2 * volatility * 0.5;
            let low = f64::min(open, close) - r3 * volatility * 0.5;
            let volume = (r1 + 0.5) * 1000.0;

            bars.push(OhlcvBar::new(time, open, high, low, close, volume));
        }

        bars
    }

    /// Convert timeframe string to seconds
    fn timeframe_to_seconds(&self, tf: &str) -> i64 {
        match tf {
            "1m" => 60,
            "5m" => 5 * 60,
            "15m" => 15 * 60,
            "1h" => 60 * 60,
            "4h" => 4 * 60 * 60,
            "1d" => 24 * 60 * 60,
            "1w" => 7 * 24 * 60 * 60,
            _ => 60 * 60, // Default to 1h
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sample_data() {
        let loader = DataLoader::new(".");
        let bars = loader.generate_sample_data("BTCUSDT", "1h");
        assert_eq!(bars.len(), 500);
        assert!(bars[0].open > 0.0);
    }
}
