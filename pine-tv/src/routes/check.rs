//! POST /api/check endpoint
//! Check Pine Script code for syntax and type errors without executing.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::engine::output::{ApiResponse, CheckRequest};
use crate::engine::runner::PineEngine;

/// Check handler state
pub struct CheckHandler {
    engine: Arc<PineEngine>,
}

impl CheckHandler {
    /// Create a new CheckHandler
    pub fn new(engine: Arc<PineEngine>) -> Self {
        Self { engine }
    }

    /// Handle POST /api/check
    pub async fn handle(
        State(state): State<Arc<Self>>,
        Json(request): Json<CheckRequest>,
    ) -> impl IntoResponse {
        match state.engine.check(&request.code) {
            Ok(_) => {
                let response = ApiResponse {
                    ok: true,
                    exec_ms: None,
                    plots: None,
                    errors: None,
                };
                (StatusCode::OK, Json(response))
            }
            Err(errors) => {
                let response = ApiResponse::error(errors);
                (StatusCode::OK, Json(response))
            }
        }
    }
}
