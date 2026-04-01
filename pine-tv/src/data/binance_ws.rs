//! Binance WebSocket client for real-time K-line data
//! Subscribes to live K-line streams and handles forming/closed bars.

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::data::OhlcvBar;

/// Binance WebSocket K-line message
#[derive(Debug, Deserialize)]
pub struct WsKlineMessage {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: WsKlineData,
}

/// Binance WebSocket K-line data
#[derive(Debug, Deserialize)]
pub struct WsKlineData {
    #[serde(rename = "t")]
    pub start_time: u64,
    #[serde(rename = "T")]
    pub end_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "x")]
    pub is_final: bool, // true = closed bar, false = forming bar
}

/// Real-time bar update event
#[derive(Debug, Clone)]
pub enum BarUpdate {
    /// Forming bar update (current bar is changing)
    Forming {
        bar: OhlcvBar,
        is_new: bool, // true if this is the first update for this bar
    },
    /// Bar closed (finalized, new bar starting)
    Closed {
        bar: OhlcvBar,
    },
}

/// Binance WebSocket client for real-time K-lines
pub struct BinanceWsClient {
    tx: broadcast::Sender<BarUpdate>,
}

impl BinanceWsClient {
    /// Create a new Binance WebSocket client
    pub fn new(symbol: String, interval: String) -> Self {
        let (tx, _rx) = broadcast::channel(100);

        // Spawn the WebSocket connection task
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            Self::start(symbol, interval, tx_clone).await;
        });

        Self { tx }
    }

    /// Subscribe to updates
    pub fn subscribe(&self) -> broadcast::Receiver<BarUpdate> {
        self.tx.subscribe()
    }

    /// Start the WebSocket connection and begin streaming
    async fn start(symbol: String, interval: String, tx: broadcast::Sender<BarUpdate>) {
        let symbol_lower = symbol.to_lowercase();
        let ws_url = format!(
            "wss://stream.binance.com:9443/ws/{}@kline_{}",
            symbol_lower, interval
        );

        tracing::info!("Connecting to Binance WebSocket: {}", ws_url);

        loop {
            match connect_async(&ws_url).await {
                Ok((ws_stream, _response)) => {
                    tracing::info!("Connected to Binance WebSocket");
                    Self::handle_stream(ws_stream, tx.clone()).await;
                }
                Err(e) => {
                    tracing::error!("WebSocket connection error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
            tracing::warn!("Reconnecting to Binance WebSocket...");
        }
    }

    /// Handle the WebSocket stream
    async fn handle_stream(ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tx: broadcast::Sender<BarUpdate>) {
        let (mut write, mut read) = ws_stream.split();
        let mut last_bar_time = None;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(kline_msg) = serde_json::from_str::<WsKlineMessage>(&text) {
                        let bar = Self::kline_to_bar(&kline_msg.kline);
                        let bar_time = bar.time;

                        let is_new_bar = last_bar_time.map(|t| t != bar_time).unwrap_or(true);
                        last_bar_time = Some(bar_time);

                        if kline_msg.kline.is_final {
                            // Bar is closed
                            let _ = tx.send(BarUpdate::Closed { bar });
                        } else {
                            // Forming bar update
                            let _ = tx.send(BarUpdate::Forming { bar, is_new: is_new_bar });
                        }
                    }
                }
                Ok(Message::Ping(_)) => {
                    let _ = write.send(Message::Pong(Vec::new())).await;
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket read error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    /// Convert Binance K-line to OhlcvBar
    fn kline_to_bar(kline: &WsKlineData) -> OhlcvBar {
        let time = (kline.start_time / 1000) as i64;
        let open = kline.open.parse().unwrap_or(0.0);
        let high = kline.high.parse().unwrap_or(0.0);
        let low = kline.low.parse().unwrap_or(0.0);
        let close = kline.close.parse().unwrap_or(0.0);
        let volume = kline.volume.parse().unwrap_or(0.0);

        OhlcvBar::new(time, open, high, low, close, volume)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kline_deserialize() {
        let json = r#"{
            "s": "BTCUSDT",
            "k": {
                "t": 1672531200000,
                "T": 1672534800000,
                "s": "BTCUSDT",
                "i": "1h",
                "o": "16500.00",
                "h": "16600.00",
                "l": "16400.00",
                "c": "16550.00",
                "v": "1000.0",
                "x": false
            }
        }"#;

        let msg: WsKlineMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.symbol, "BTCUSDT");
        assert_eq!(msg.kline.interval, "1h");
        assert!(!msg.kline.is_final);
    }
}
