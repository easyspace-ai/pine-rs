//! POST /api/run endpoint
//!
//! Runs the script via [`crate::engine::runner::PineEngine`] (default **pine-vm**; set
//! `PINE_TV_MODE=eval` for the interpreter fallback). Response `plots` are built from execution
//! outputs aligned to request bars.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

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
        let bars = match state
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
            Err(_) => {
                match state
                    .data_loader
                    .load_from_binance(&request.symbol, &request.timeframe, request.bars)
                    .await
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
