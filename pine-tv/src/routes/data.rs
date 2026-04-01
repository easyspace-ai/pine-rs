//! GET /api/data/{symbol}/{tf} endpoint
//! Returns OHLCV data for a symbol and timeframe.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::data::binance::BinanceClient;
use crate::data::loader::DataLoader;

/// Data response
#[derive(Serialize)]
struct DataResponse<T> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Data handler state
pub struct DataHandler {
    data_loader: Arc<DataLoader>,
}

impl DataHandler {
    /// Create a new DataHandler
    pub fn new(data_loader: Arc<DataLoader>) -> Self {
        Self { data_loader }
    }

    /// Handle GET /api/data/:symbol/:tf
    pub async fn handle(
        State(state): State<Arc<Self>>,
        Path((symbol, tf)): Path<(String, String)>,
    ) -> impl IntoResponse {
        // Try Binance first
        let binance_client = BinanceClient::new();
        match binance_client.fetch_klines(&symbol, &tf, 500).await {
            Ok(bars) if !bars.is_empty() => (
                StatusCode::OK,
                Json(DataResponse {
                    ok: true,
                    data: Some(bars),
                    error: None,
                }),
            ),
            _ => {
                // Fall back to local
                match state.data_loader.load_local(&symbol, &tf) {
                    Ok(bars) => (
                        StatusCode::OK,
                        Json(DataResponse {
                            ok: true,
                            data: Some(bars),
                            error: None,
                        }),
                    ),
                    Err(e) => (
                        StatusCode::OK,
                        Json(DataResponse {
                            ok: false,
                            data: None,
                            error: Some(format!("{}", e)),
                        }),
                    ),
                }
            }
        }
    }
}
