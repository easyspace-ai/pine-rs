//! WebSocket endpoint for real-time updates
//! Handles frontend WebSocket connections and streams real-time data.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::StreamExt;
use std::sync::Arc;

use crate::data::{RealtimeDataManager, RealtimeUpdate};
use crate::engine::runner::PineEngine;

/// WebSocket handler state
pub struct WsHandler {
    data_manager: Arc<RealtimeDataManager>,
    engine: Arc<PineEngine>,
}

impl WsHandler {
    /// Create a new WsHandler
    pub fn new(data_manager: Arc<RealtimeDataManager>, engine: Arc<PineEngine>) -> Self {
        Self {
            data_manager,
            engine,
        }
    }

    /// Handle WebSocket upgrade
    pub async fn handle_ws(
        ws: WebSocketUpgrade,
        State(state): State<Arc<Self>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Self::handle_socket(socket, state))
    }

    /// Handle a WebSocket connection
    async fn handle_socket(mut socket: WebSocket, state: Arc<Self>) {
        tracing::info!("WebSocket client connected");

        // Subscribe to real-time updates
        let mut rx = state.data_manager.subscribe();

        // Send initial snapshot
        let bars = state.data_manager.get_bars().await;
        let snapshot = RealtimeUpdate::Snapshot { bars };
        if let Ok(json) = serde_json::to_string(&snapshot) {
            let _ = socket.send(Message::Text(json)).await;
        }

        // Main loop: receive from client and send updates
        loop {
            tokio::select! {
                // Receive update from data manager
                Ok(update) = rx.recv() => {
                    if let Ok(json) = serde_json::to_string(&update) {
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                // Receive message from client
                Some(msg) = socket.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // Handle client messages (e.g., request script execution)
                            if let Ok(req) = serde_json::from_str::<WsClientMessage>(&text) {
                                if req.action.as_str() == "run" {
                                    let bars = state.data_manager.get_bars().await;
                                    if let Some(code) = req.code {
                                        match state.engine.run(&code, &bars) {
                                            Ok(result) => {
                                                let response = WsServerMessage::Result { result };
                                                if let Ok(json) = serde_json::to_string(&response) {
                                                    let _ = socket.send(Message::Text(json)).await;
                                                }
                                            }
                                            Err(errors) => {
                                                let response = WsServerMessage::Error { errors };
                                                if let Ok(json) = serde_json::to_string(&response) {
                                                    let _ = socket.send(Message::Text(json)).await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }
            }
        }

        tracing::info!("WebSocket client disconnected");
    }
}

/// Message from client
#[derive(Debug, serde::Deserialize)]
struct WsClientMessage {
    action: String,
    code: Option<String>,
}

/// Message to client
#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    #[serde(rename = "result")]
    Result {
        result: crate::engine::output::ApiResponse,
    },
    #[serde(rename = "error")]
    Error {
        errors: Vec<crate::engine::output::ApiError>,
    },
}
