//! Real-time script runner for pine-tv
//!
//! Manages active scripts and re-executes them when new K-line data arrives.
//! Provides incremental updates to connected WebSocket clients.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::data::{BarUpdate, BinanceWsClient, OhlcvBar, RealtimeDataManager, RealtimeUpdate};
use crate::engine::runner::PineEngine;
use crate::engine::output::ApiResponse;

/// Result of script execution for real-time updates
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScriptUpdate {
    /// Whether this is a full recalculation
    pub is_full: bool,
    /// Script execution result
    pub result: ApiResponse,
    /// Timestamp when this update was generated
    pub timestamp: i64,
}

/// Active script configuration
#[derive(Debug, Clone)]
struct ActiveScript {
    code: String,
    last_result: Option<ApiResponse>,
    last_executed: Instant,
}

/// Real-time runner that executes scripts on K-line updates
pub struct RealtimeRunner {
    engine: Arc<PineEngine>,
    data_manager: Arc<RealtimeDataManager>,
    active_scripts: RwLock<HashMap<String, ActiveScript>>,
    results_tx: broadcast::Sender<ScriptUpdate>,
    /// Minimum interval between forming bar recalculations (throttling)
    throttle_duration: Duration,
}

impl RealtimeRunner {
    /// Create a new real-time runner
    pub fn new(
        engine: Arc<PineEngine>,
        data_manager: Arc<RealtimeDataManager>,
    ) -> Arc<Self> {
        let (results_tx, _results_rx) = broadcast::channel::<ScriptUpdate>(100);

        Arc::new(Self {
            engine,
            data_manager,
            active_scripts: RwLock::new(HashMap::new()),
            results_tx,
            throttle_duration: Duration::from_millis(500), // 500ms throttle for forming updates
        })
    }

    /// Subscribe to script execution results
    pub fn subscribe(&self) -> broadcast::Receiver<ScriptUpdate> {
        self.results_tx.subscribe()
    }

    /// Start the real-time runner
    /// Spawns a background task that listens for bar updates and executes scripts
    pub async fn start(self: Arc<Self>, ws_client: Arc<BinanceWsClient>) {
        let mut bar_rx = ws_client.subscribe();

        tokio::spawn(async move {
            info!("RealtimeRunner started");

            while let Ok(update) = bar_rx.recv().await {
                match update {
                    BarUpdate::Forming { bar, is_new: _ } => {
                        // Throttle forming bar updates
                        self.handle_forming_update(bar).await;
                    }
                    BarUpdate::Closed { bar } => {
                        // Always execute on closed bar
                        self.execute_all_scripts(true).await;
                        debug!("Executed scripts on closed bar at time={}", bar.time);
                    }
                }
            }

            warn!("RealtimeRunner stopped - WebSocket disconnected");
        });
    }

    /// Register a new script for real-time execution
    /// Returns the session ID
    pub async fn register_script(&self, code: String) -> Result<String, String> {
        // Validate script first
        let bars = self.data_manager.get_bars().await;
        if let Err(errors) = self.engine.check(&code) {
            return Err(format!("Script validation failed: {:?}", errors));
        }

        let session_id = format!("session_{}", generate_id());
        let script = ActiveScript {
            code,
            last_result: None,
            last_executed: Instant::now() - Duration::from_secs(3600), // Force first execution
        };

        self.active_scripts.write().await.insert(session_id.clone(), script);
        info!("Registered script with session_id={}", session_id);

        // Execute immediately
        self.execute_script(&session_id, true).await;

        Ok(session_id)
    }

    /// Unregister a script
    pub async fn unregister_script(&self, session_id: &str) {
        self.active_scripts.write().await.remove(session_id);
        info!("Unregistered script session_id={}", session_id);
    }

    /// Update script code for an existing session
    pub async fn update_script(&self, session_id: &str, new_code: String) -> Result<(), String> {
        // Validate new code
        if let Err(errors) = self.engine.check(&new_code) {
            return Err(format!("Script validation failed: {:?}", errors));
        }

        let mut scripts = self.active_scripts.write().await;
        if let Some(script) = scripts.get_mut(session_id) {
            script.code = new_code;
            script.last_executed = Instant::now() - Duration::from_secs(3600);
            drop(scripts);

            // Execute immediately with full recalc
            self.execute_script(session_id, true).await;
            info!("Updated script session_id={}", session_id);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Handle forming bar update with throttling
    async fn handle_forming_update(&self, _bar: OhlcvBar) {
        let now = Instant::now();
        let should_execute = {
            let scripts = self.active_scripts.read().await;
            scripts.values().any(|script| {
                now.duration_since(script.last_executed) >= self.throttle_duration
            })
        };

        if should_execute {
            self.execute_all_scripts(false).await;
        }
    }

    /// Execute all active scripts
    async fn execute_all_scripts(&self, is_full: bool) {
        let session_ids: Vec<String> = {
            self.active_scripts.read().await.keys().cloned().collect()
        };

        for session_id in session_ids {
            self.execute_script(&session_id, is_full).await;
        }
    }

    /// Execute a single script and broadcast result
    async fn execute_script(&self, session_id: &str, is_full: bool) {
        let (code, bars) = {
            let scripts = self.active_scripts.read().await;
            let script = match scripts.get(session_id) {
                Some(s) => s,
                None => return,
            };

            let bars = self.data_manager.get_bars().await;
            (script.code.clone(), bars)
        };

        let start = Instant::now();

        match self.engine.run(&code, &bars) {
            Ok(result) => {
                let exec_time = start.elapsed().as_millis() as i64;
                let mut result_with_time = result;
                result_with_time.exec_ms = exec_time;

                // Update last result
                {
                    let mut scripts = self.active_scripts.write().await;
                    if let Some(script) = scripts.get_mut(session_id) {
                        script.last_result = Some(result_with_time.clone());
                        script.last_executed = Instant::now();
                    }
                }

                // Broadcast update
                let update = ScriptUpdate {
                    is_full,
                    result: result_with_time,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };

                let _ = self.results_tx.send(update);
                debug!(
                    "Executed session_id={} in {}ms (full={})",
                    session_id, exec_time, is_full
                );
            }
            Err(errors) => {
                error!("Script execution failed for session_id={}: {:?}", session_id, errors);
            }
        }
    }

    /// Get the current result for a session (for new subscribers)
    pub async fn get_current_result(&self, session_id: &str) -> Option<ApiResponse> {
        self.active_scripts
            .read()
            .await
            .get(session_id)
            .and_then(|s| s.last_result.clone())
    }

    /// Check if a session exists
    pub async fn has_session(&self, session_id: &str) -> bool {
        self.active_scripts.read().await.contains_key(session_id)
    }

    /// Get list of active sessions
    pub async fn get_active_sessions(&self) -> Vec<String> {
        self.active_scripts.read().await.keys().cloned().collect()
    }
}

/// Generate a short unique ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 8);
    }
}
