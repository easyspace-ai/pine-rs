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
use std::collections::HashSet;
use std::sync::Arc;

use crate::data::{RealtimeDataManager, RealtimeUpdate};
use crate::engine::output::{ApiError, ApiResponse};
use crate::engine::realtime_runner::{RealtimeRunner, ScriptUpdateKind};
use crate::engine::runner::{RealtimeExecutionTrigger, ScriptKind};

const WS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, serde::Serialize)]
struct WsEnvelope<T> {
    schema_version: u32,
    channel: &'static str,
    seq: u64,
    #[serde(flatten)]
    message: T,
}

fn serialize_ws_message<T: serde::Serialize>(
    channel: &'static str,
    seq: u64,
    message: &T,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&WsEnvelope {
        schema_version: WS_SCHEMA_VERSION,
        channel,
        seq,
        message,
    })
}

fn trigger_from_update_kind(update_kind: ScriptUpdateKind) -> RealtimeExecutionTrigger {
    match update_kind {
        ScriptUpdateKind::Snapshot => RealtimeExecutionTrigger::Snapshot,
        ScriptUpdateKind::BarOpen | ScriptUpdateKind::BarUpdate => RealtimeExecutionTrigger::Tick,
        ScriptUpdateKind::BarClose => RealtimeExecutionTrigger::BarClose,
        ScriptUpdateKind::OrderFill => RealtimeExecutionTrigger::OrderFill,
    }
}

/// WebSocket handler state
pub struct WsHandler {
    data_manager: Arc<RealtimeDataManager>,
    realtime_runner: Arc<RealtimeRunner>,
}

impl WsHandler {
    /// Create a new WsHandler
    pub fn new(
        data_manager: Arc<RealtimeDataManager>,
        realtime_runner: Arc<RealtimeRunner>,
    ) -> Self {
        Self {
            data_manager,
            realtime_runner,
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
        let mut script_rx = state.realtime_runner.subscribe();
        let mut session_ids: HashSet<String> = HashSet::new();
        let mut seq: u64 = 0;

        // Send initial snapshot
        let bars = state.data_manager.get_bars().await;
        let snapshot = RealtimeUpdate::Snapshot { bars };
        seq += 1;
        if let Ok(json) = serialize_ws_message("market", seq, &snapshot) {
            let _ = socket.send(Message::Text(json)).await;
        }

        // Main loop: receive from client and send updates
        loop {
            tokio::select! {
                // Receive update from data manager
                Ok(update) = rx.recv() => {
                    seq += 1;
                    if let Ok(json) = serialize_ws_message("market", seq, &update) {
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                Ok(update) = script_rx.recv() => {
                    if session_ids.contains(update.session_id.as_str()) {
                        let response = WsServerMessage::Result {
                            session_id: update.session_id,
                            is_full: update.is_full,
                            update_kind: update.update_kind,
                            script_kind: update.script_kind,
                            trigger: update.trigger,
                            bar_time: update.bar_time,
                            timestamp: update.timestamp,
                            result: update.result,
                        };
                        seq += 1;
                        if let Ok(json) = serialize_ws_message("script", seq, &response) {
                            if socket.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                // Receive message from client
                Some(msg) = socket.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // Handle client messages (e.g., request script execution)
                            if let Ok(req) = serde_json::from_str::<WsClientMessage>(&text) {
                                match req.action.as_str() {
                                    "run" => {
                                        if let Some(code) = req.code {
                                            let requested_symbol =
                                                req.symbol.clone().unwrap_or_else(|| "BTCUSDT".to_string());
                                            let requested_timeframe =
                                                req.timeframe.clone().unwrap_or_else(|| "1h".to_string());
                                            let requested_bars = req.bars.unwrap_or(500);
                                            let result = if let Some(session_id) = req.session_id.as_deref() {
                                                if session_ids.contains(session_id) {
                                                    state
                                                        .realtime_runner
                                                        .update_script(
                                                            session_id,
                                                            code,
                                                            Some(requested_symbol.clone()),
                                                            Some(requested_timeframe.clone()),
                                                            Some(requested_bars),
                                                        )
                                                        .await
                                                        .map(|_| session_id.to_string())
                                                } else {
                                                    Err("Session not owned by this connection".to_string())
                                                }
                                            } else if session_ids.len() == 1 {
                                                let session_id = session_ids.iter().next().cloned().unwrap();
                                                state
                                                    .realtime_runner
                                                    .update_script(
                                                        &session_id,
                                                        code,
                                                        Some(requested_symbol.clone()),
                                                        Some(requested_timeframe.clone()),
                                                        Some(requested_bars),
                                                    )
                                                    .await
                                                    .map(|_| session_id)
                                            } else {
                                                state
                                                    .realtime_runner
                                                    .register_script(
                                                        code,
                                                        requested_symbol,
                                                        requested_timeframe,
                                                        requested_bars,
                                                    )
                                                    .await
                                            };

                                            match result {
                                                Ok(session_id) => {
                                                    session_ids.insert(session_id.clone());
                                                    let response = WsServerMessage::Session {
                                                        session_id: session_id.clone(),
                                                        status: "active".to_string(),
                                                    };
                                                    seq += 1;
                                                    if let Ok(json) = serialize_ws_message("control", seq, &response) {
                                                        let _ = socket.send(Message::Text(json)).await;
                                                    }

                                                    if let Some(result) = state
                                                        .realtime_runner
                                                        .get_current_result(&session_id)
                                                        .await
                                                    {
                                                        let declaration = state
                                                            .realtime_runner
                                                            .get_session_declaration(&session_id)
                                                            .await;
                                                        let response = WsServerMessage::Result {
                                                            session_id,
                                                            is_full: true,
                                                            update_kind: ScriptUpdateKind::Snapshot,
                                                            script_kind: declaration
                                                                .as_ref()
                                                                .map(|d| d.kind)
                                                                .unwrap_or(ScriptKind::Unknown),
                                                            trigger: trigger_from_update_kind(
                                                                ScriptUpdateKind::Snapshot,
                                                            ),
                                                            bar_time: None,
                                                            timestamp: chrono::Utc::now()
                                                                .timestamp_millis(),
                                                            result,
                                                        };
                                                        seq += 1;
                                                        if let Ok(json) = serialize_ws_message("script", seq, &response) {
                                                            let _ = socket
                                                                .send(Message::Text(json))
                                                                .await;
                                                        }
                                                    }
                                                }
                                                Err(error) => {
                                                    let response = WsServerMessage::Error {
                                                        errors: vec![ApiError::simple(error)],
                                                    };
                                                    seq += 1;
                                                    if let Ok(json) = serialize_ws_message("control", seq, &response) {
                                                        let _ = socket.send(Message::Text(json)).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    "stop" => {
                                        let targets: Vec<String> = if let Some(session_id) =
                                            req.session_id.as_deref()
                                        {
                                            if session_ids.contains(session_id) {
                                                vec![session_id.to_string()]
                                            } else {
                                                Vec::new()
                                            }
                                        } else {
                                            session_ids.iter().cloned().collect()
                                        };

                                        for session_id in targets {
                                            session_ids.remove(&session_id);
                                            state.realtime_runner.unregister_script(&session_id).await;
                                            let response = WsServerMessage::Session {
                                                session_id,
                                                status: "stopped".to_string(),
                                            };
                                            seq += 1;
                                            if let Ok(json) = serialize_ws_message("control", seq, &response) {
                                                let _ = socket.send(Message::Text(json)).await;
                                            }
                                        }
                                    }
                                    _ => {}
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

        for session_id in session_ids {
            state.realtime_runner.unregister_script(&session_id).await;
        }
        tracing::info!("WebSocket client disconnected");
    }
}

/// Message from client
#[derive(Debug, serde::Deserialize)]
struct WsClientMessage {
    action: String,
    code: Option<String>,
    session_id: Option<String>,
    symbol: Option<String>,
    timeframe: Option<String>,
    bars: Option<usize>,
}

/// Message to client
#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    #[serde(rename = "result")]
    Result {
        session_id: String,
        is_full: bool,
        update_kind: ScriptUpdateKind,
        script_kind: ScriptKind,
        trigger: RealtimeExecutionTrigger,
        bar_time: Option<i64>,
        timestamp: i64,
        result: ApiResponse,
    },
    #[serde(rename = "error")]
    Error { errors: Vec<ApiError> },
    #[serde(rename = "session")]
    Session { session_id: String, status: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_messages_use_expected_type_tags() {
        let result = WsServerMessage::Result {
            session_id: "session_1".to_string(),
            is_full: true,
            update_kind: ScriptUpdateKind::Snapshot,
            script_kind: ScriptKind::Indicator,
            trigger: RealtimeExecutionTrigger::Snapshot,
            bar_time: Some(123),
            timestamp: 123,
            result: ApiResponse::success(1, vec![]),
        };
        let session = WsServerMessage::Session {
            session_id: "session_1".to_string(),
            status: "active".to_string(),
        };
        let error = WsServerMessage::Error {
            errors: vec![ApiError::simple("boom".to_string())],
        };

        let result_json = serialize_ws_message("script", 7, &result).unwrap();
        let session_json = serialize_ws_message("control", 8, &session).unwrap();
        let error_json = serialize_ws_message("control", 9, &error).unwrap();

        assert!(result_json.contains("\"type\":\"result\""));
        assert!(result_json.contains("\"schema_version\":1"));
        assert!(result_json.contains("\"channel\":\"script\""));
        assert!(result_json.contains("\"seq\":7"));
        assert!(result_json.contains("\"session_id\":\"session_1\""));
        assert!(result_json.contains("\"update_kind\":\"snapshot\""));
        assert!(result_json.contains("\"script_kind\":\"indicator\""));
        assert!(result_json.contains("\"trigger\":\"snapshot\""));
        assert!(result_json.contains("\"bar_time\":123"));
        assert!(session_json.contains("\"type\":\"session\""));
        assert!(session_json.contains("\"schema_version\":1"));
        assert!(session_json.contains("\"channel\":\"control\""));
        assert!(error_json.contains("\"type\":\"error\""));
        assert!(error_json.contains("\"channel\":\"control\""));
    }
}
