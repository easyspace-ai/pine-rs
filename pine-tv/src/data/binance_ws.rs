//! Binance WebSocket client for real-time K-line data
//! Subscribes to live K-line streams and handles forming/closed bars.

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::data::OhlcvBar;
use std::env;

const BINANCE_FUTURES_WS_HOST: &str = "fstream.binance.com:443";
const DEFAULT_BINANCE_PROXY: &str = "127.0.0.1:15236";

/// Generate a random WebSocket key for the handshake
fn generate_websocket_key() -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut buf = [0u8; 16];
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Simple pseudo-random from timestamp
    for (i, byte) in buf.iter_mut().enumerate() {
        *byte = ((timestamp >> (i * 4)) % 256) as u8;
    }
    STANDARD.encode(buf)
}

/// Binance WebSocket K-line message
#[derive(Debug, Deserialize)]
pub struct WsKlineMessage {
    #[serde(rename = "s")]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub end_time: u64,
    #[serde(rename = "s")]
    #[allow(dead_code)]
    pub symbol: String,
    #[serde(rename = "i")]
    #[allow(dead_code)]
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
    Closed { bar: OhlcvBar },
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
        let ws_path = format!("{}@kline_{}", symbol_lower, interval);
        let ws_url = format!("wss://{}/ws/{}", BINANCE_FUTURES_WS_HOST, ws_path);

        // Check for proxy configuration
        let proxy_addr = env::var("PINE_TV_BINANCE_PROXY")
            .ok()
            .or_else(|| Some(DEFAULT_BINANCE_PROXY.to_string()));

        tracing::info!("Connecting to Binance WebSocket: {}", ws_url);
        if let Some(ref proxy) = proxy_addr {
            tracing::info!("Using proxy: {}", proxy);
        }

        loop {
            let result = if let Some(ref proxy) = proxy_addr {
                Self::connect_with_proxy(&ws_url, proxy).await
            } else {
                Self::connect_direct(&ws_url).await
            };

            match result {
                Ok(ws_stream) => {
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

    /// Connect directly without proxy
    async fn connect_direct(
        ws_url: &str,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let (ws_stream, _response) = tokio_tungstenite::connect_async(ws_url).await?;
        Ok(ws_stream)
    }

    /// Connect via HTTP proxy using CONNECT method
    async fn connect_with_proxy(
        ws_url: &str,
        proxy: &str,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        // Parse proxy address
        let proxy_addr = if proxy.contains("://") {
            // Extract host:port from URL like http://127.0.0.1:15236
            proxy.split("://").nth(1).unwrap_or(proxy).to_string()
        } else {
            proxy.to_string()
        };

        tracing::debug!("Connecting to proxy: {}", proxy_addr);

        // Connect to proxy
        let proxy_stream = TcpStream::connect(&proxy_addr).await.map_err(|e| {
            tracing::error!("Failed to connect to proxy {}: {}", proxy_addr, e);
            e
        })?;

        // Send CONNECT request to establish tunnel to Binance
        const CONNECT_REQ: &str = "CONNECT fstream.binance.com:443 HTTP/1.1\r\n\
             Host: fstream.binance.com:443\r\n\
             Proxy-Connection: keep-alive\r\n\
             \r\n";

        let mut stream = proxy_stream;
        stream.write_all(CONNECT_REQ.as_bytes()).await?;
        stream.flush().await?;

        // Read CONNECT response
        let mut buffer = vec![0u8; 1024];
        let n = stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]);

        if !response.contains("200") {
            return Err(format!("Proxy CONNECT failed: {}", response).into());
        }

        tracing::debug!("Proxy tunnel established");

        // Wrap the stream with TLS using tokio_native_tls
        let tls_connector =
            tokio_native_tls::TlsConnector::from(native_tls::TlsConnector::builder().build()?);
        let tls_stream = tls_connector.connect("fstream.binance.com", stream).await?;

        // Wrap in MaybeTlsStream
        let maybe_tls = tokio_tungstenite::MaybeTlsStream::NativeTls(tls_stream);

        // Complete WebSocket handshake
        let request = tokio_tungstenite::tungstenite::handshake::client::Request::builder()
            .uri(ws_url)
            .header("Host", BINANCE_FUTURES_WS_HOST)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", generate_websocket_key())
            .header("Sec-WebSocket-Version", "13")
            .body(())?;

        let (ws_stream, _) = tokio_tungstenite::client_async(request, maybe_tls).await?;

        Ok(ws_stream)
    }

    /// Handle the WebSocket stream
    async fn handle_stream(
        ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tx: broadcast::Sender<BarUpdate>,
    ) {
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
                            let _ = tx.send(BarUpdate::Forming {
                                bar,
                                is_new: is_new_bar,
                            });
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

    #[test]
    fn futures_ws_url_uses_fstream_host() {
        let ws_path = "btcusdt@kline_1m";
        let ws_url = format!("wss://{}/ws/{}", BINANCE_FUTURES_WS_HOST, ws_path);
        assert!(ws_url.contains("fstream.binance.com"));
        assert!(ws_url.contains("/ws/btcusdt@kline_1m"));
    }
}
