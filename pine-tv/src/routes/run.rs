//! POST /api/run endpoint
//! Execute Pine Script code and return results.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::data::binance::BinanceClient;
use crate::data::loader::DataLoader;
use crate::engine::output::{ApiResponse, RunRequest};
use crate::engine::runner::PineEngine;

/// Run handler state
pub struct RunHandler {
    engine: Arc<PineEngine>,
    data_loader: Arc<DataLoader>,
}

impl RunHandler {
    /// Create a new RunHandler
    pub fn new(engine: Arc<PineEngine>, data_loader: Arc<DataLoader>) -> Self {
        Self {
            engine,
            data_loader,
        }
    }

    /// Handle POST /api/run
    pub async fn handle(
        State(state): State<Arc<Self>>,
        Json(request): Json<RunRequest>,
    ) -> impl IntoResponse {
        // Load data - use the async load method
        let binance_client = BinanceClient::new();
        let bars = match binance_client
            .fetch_klines(&request.symbol, &request.timeframe, request.bars)
            .await
        {
            Ok(mut b) => {
                if b.len() > request.bars {
                    let start = b.len() - request.bars;
                    b.drain(..start);
                }
                b
            }
            Err(_) => {
                // Fall back to local/sample data (synchronous)
                match state
                    .data_loader
                    .load_local(&request.symbol, &request.timeframe)
                {
                    Ok(mut b) => {
                        if b.len() > request.bars {
                            let start = b.len() - request.bars;
                            b.drain(..start);
                        }
                        b
                    }
                    Err(e) => {
                        let response =
                            ApiResponse::error(vec![crate::engine::output::ApiError::simple(
                                format!("Data load error: {}", e),
                            )]);
                        return (StatusCode::OK, Json(response));
                    }
                }
            }
        };

        if bars.is_empty() {
            let response = ApiResponse::error(vec![crate::engine::output::ApiError::simple(
                "No data available".to_string(),
            )]);
            return (StatusCode::OK, Json(response));
        }

        // Run the script
        match state.engine.run(&request.code, &bars) {
            Ok(response) => (StatusCode::OK, Json(response)),
            Err(errors) => {
                let response = ApiResponse::error(errors);
                (StatusCode::OK, Json(response))
            }
        }
    }
}
