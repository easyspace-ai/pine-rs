//! Binance API data loader
//! Fetch real-time and historical K-line data from Binance.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data::OhlcvBar;

/// Binance API client
pub struct BinanceClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for BinanceClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceClient {
    /// Create a new Binance client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.binance.com".to_string(),
        }
    }

    /// Create a new Binance client with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Fetch K-line (candlestick) data from Binance
    ///
    /// # Arguments
    /// * `symbol` - Trading pair (e.g., "BTCUSDT")
    /// * `interval` - Time interval (1m, 5m, 15m, 1h, 4h, 1d, etc.)
    /// * `limit` - Number of candles to fetch (max 1000)
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: usize,
    ) -> Result<Vec<OhlcvBar>, BinanceError> {
        let limit = limit.min(1000); // Binance max is 1000

        let url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&limit={}",
            self.base_url,
            symbol.to_uppercase(),
            interval,
            limit
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(BinanceError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }

        let klines: Vec<Vec<Value>> = response.json().await?;

        let bars = klines
            .into_iter()
            .filter_map(|k| Self::parse_kline(k))
            .collect();

        Ok(bars)
    }

    /// Fetch K-line data with start time
    pub async fn fetch_klines_with_start(
        &self,
        symbol: &str,
        interval: &str,
        start_time: u64,
        limit: usize,
    ) -> Result<Vec<OhlcvBar>, BinanceError> {
        let limit = limit.min(1000);

        let url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&startTime={}&limit={}",
            self.base_url,
            symbol.to_uppercase(),
            interval,
            start_time,
            limit
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(BinanceError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }

        let klines: Vec<Vec<Value>> = response.json().await?;

        let bars = klines
            .into_iter()
            .filter_map(|k| Self::parse_kline(k))
            .collect();

        Ok(bars)
    }

    /// Parse a single Binance kline array into OhlcvBar
    fn parse_kline(kline: Vec<Value>) -> Option<OhlcvBar> {
        if kline.len() < 6 {
            return None;
        }

        let open_time = kline[0].as_i64()?;
        let open = kline[1].as_str()?.parse::<f64>().ok()?;
        let high = kline[2].as_str()?.parse::<f64>().ok()?;
        let low = kline[3].as_str()?.parse::<f64>().ok()?;
        let close = kline[4].as_str()?.parse::<f64>().ok()?;
        let volume = kline[5].as_str()?.parse::<f64>().ok()?;

        // Binance returns timestamp in milliseconds, convert to seconds
        let time = open_time / 1000;

        Some(OhlcvBar::new(time, open, high, low, close, volume))
    }

    /// Get current server time from Binance
    pub async fn get_server_time(&self) -> Result<u64, BinanceError> {
        let url = format!("{}/api/v3/time", self.base_url);
        let response = self.client.get(&url).send().await?;

        #[derive(Deserialize)]
        struct TimeResponse {
            serverTime: u64,
        }

        let time: TimeResponse = response.json().await?;
        Ok(time.serverTime)
    }

    /// Get exchange info (available symbols)
    pub async fn get_exchange_info(&self) -> Result<ExchangeInfo, BinanceError> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);
        let response = self.client.get(&url).send().await?;

        let info: ExchangeInfo = response.json().await?;
        Ok(info)
    }
}

/// Exchange info from Binance
#[derive(Debug, Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<SymbolInfo>,
}

/// Symbol information
#[derive(Debug, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: String,
    pub baseAsset: String,
    pub quoteAsset: String,
}

/// Binance API error
#[derive(Debug, thiserror::Error)]
pub enum BinanceError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_fetch_klines() {
        let client = BinanceClient::new();
        let bars = client.fetch_klines("BTCUSDT", "1h", 10).await.unwrap();
        assert_eq!(bars.len(), 10);
    }
}
