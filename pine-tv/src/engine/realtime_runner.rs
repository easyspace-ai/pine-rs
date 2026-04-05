//! Real-time script runner for pine-tv
//!
//! Manages active scripts and re-executes them when new K-line data arrives.
//! Provides incremental updates to connected WebSocket clients.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::data::{
    BinanceClient, BinanceWsClient, DataLoader, RealtimeDataManager, RealtimeUpdate,
};
use crate::engine::output::ApiResponse;
use crate::engine::runner::{PineEngine, RealtimeExecutionTrigger, ScriptDeclaration, ScriptKind};

/// Realtime update kind for a script session.
#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScriptUpdateKind {
    /// Full snapshot after register/update.
    Snapshot,
    /// First tick of a new bar.
    BarOpen,
    /// Intrabar update on the current forming bar.
    BarUpdate,
    /// Final recalculation on bar close.
    BarClose,
    /// Recalculation after order fill.
    OrderFill,
}

/// Result of script execution for real-time updates
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScriptUpdate {
    /// Session identifier for the active script.
    pub session_id: String,
    /// Whether this is a full recalculation
    pub is_full: bool,
    /// Incremental event type for this execution.
    pub update_kind: ScriptUpdateKind,
    /// Script declaration kind.
    pub script_kind: ScriptKind,
    /// Execution trigger that caused this update.
    pub trigger: RealtimeExecutionTrigger,
    /// Bar timestamp that triggered this execution, when applicable.
    pub bar_time: Option<i64>,
    /// Script execution result
    pub result: ApiResponse,
    /// Timestamp when this update was generated
    pub timestamp: i64,
}

/// Active script configuration
#[derive(Debug, Clone)]
struct ActiveScript {
    code: String,
    declaration: ScriptDeclaration,
    last_result: Option<ApiResponse>,
    last_executed: Instant,
    market_key: MarketKey,
    bars_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MarketKey {
    symbol: String,
    timeframe: String,
}

impl MarketKey {
    fn new(symbol: String, timeframe: String) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            timeframe,
        }
    }
}

/// Real-time runner that executes scripts on K-line updates
pub struct RealtimeRunner {
    engine: Arc<PineEngine>,
    data_loader: Arc<DataLoader>,
    market_runtimes: RwLock<HashMap<MarketKey, Arc<RealtimeDataManager>>>,
    active_scripts: RwLock<HashMap<String, ActiveScript>>,
    results_tx: broadcast::Sender<ScriptUpdate>,
    /// Minimum interval between forming bar recalculations (throttling)
    throttle_duration: Duration,
    max_bars: usize,
}

impl RealtimeRunner {
    /// Create a new real-time runner
    pub fn new(
        engine: Arc<PineEngine>,
        data_loader: Arc<DataLoader>,
        max_bars: usize,
    ) -> Arc<Self> {
        let (results_tx, _results_rx) = broadcast::channel::<ScriptUpdate>(100);

        Arc::new(Self {
            engine,
            data_loader,
            market_runtimes: RwLock::new(HashMap::new()),
            active_scripts: RwLock::new(HashMap::new()),
            results_tx,
            throttle_duration: Duration::from_millis(500), // 500ms throttle for forming updates
            max_bars,
        })
    }

    /// Subscribe to script execution results
    pub fn subscribe(&self) -> broadcast::Receiver<ScriptUpdate> {
        self.results_tx.subscribe()
    }

    /// Register a new script for real-time execution
    /// Returns the session ID
    pub async fn register_script(
        self: &Arc<Self>,
        code: String,
        symbol: String,
        timeframe: String,
        bars: usize,
    ) -> Result<String, String> {
        // Validate script first
        let declaration = self
            .engine
            .inspect_script(&code)
            .map_err(|errors| format!("Script validation failed: {:?}", errors))?;
        if let Err(errors) = self.engine.check(&code) {
            return Err(format!("Script validation failed: {:?}", errors));
        }
        let market_key = self.ensure_market_runtime(symbol, timeframe, bars).await?;

        let session_id = format!("session_{}", generate_id());
        let script = ActiveScript {
            code,
            declaration,
            last_result: None,
            last_executed: Instant::now() - Duration::from_secs(3600), // Force first execution
            market_key,
            bars_limit: bars,
        };

        self.active_scripts
            .write()
            .await
            .insert(session_id.clone(), script);
        info!("Registered script with session_id={}", session_id);

        // Execute immediately
        self.execute_script(&session_id, ScriptUpdateKind::Snapshot, None)
            .await;

        Ok(session_id)
    }

    /// Unregister a script
    pub async fn unregister_script(&self, session_id: &str) {
        self.active_scripts.write().await.remove(session_id);
        info!("Unregistered script session_id={}", session_id);
    }

    /// Update script code for an existing session
    pub async fn update_script(
        self: &Arc<Self>,
        session_id: &str,
        new_code: String,
        symbol: Option<String>,
        timeframe: Option<String>,
        bars: Option<usize>,
    ) -> Result<(), String> {
        // Validate new code
        let declaration = self
            .engine
            .inspect_script(&new_code)
            .map_err(|errors| format!("Script validation failed: {:?}", errors))?;
        if let Err(errors) = self.engine.check(&new_code) {
            return Err(format!("Script validation failed: {:?}", errors));
        }

        let current = self.active_scripts.read().await.get(session_id).cloned();
        let Some(existing) = current else {
            return Err("Session not found".to_string());
        };

        let next_symbol = symbol.unwrap_or(existing.market_key.symbol.clone());
        let next_timeframe = timeframe.unwrap_or(existing.market_key.timeframe.clone());
        let next_bars = bars.unwrap_or(existing.bars_limit);
        let next_market_key = self
            .ensure_market_runtime(next_symbol, next_timeframe, next_bars)
            .await?;

        let mut scripts = self.active_scripts.write().await;
        if let Some(script) = scripts.get_mut(session_id) {
            script.code = new_code;
            script.declaration = declaration;
            script.last_executed = Instant::now() - Duration::from_secs(3600);
            script.market_key = next_market_key;
            script.bars_limit = next_bars;
            drop(scripts);

            // Execute immediately with full recalc
            self.execute_script(session_id, ScriptUpdateKind::Snapshot, None)
                .await;
            info!("Updated script session_id={}", session_id);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    async fn ensure_market_runtime(
        self: &Arc<Self>,
        symbol: String,
        timeframe: String,
        bars: usize,
    ) -> Result<MarketKey, String> {
        let key = MarketKey::new(symbol, timeframe);
        if self.market_runtimes.read().await.contains_key(&key) {
            return Ok(key);
        }

        let market_bars = self.max_bars.max(bars);
        let data_manager = Arc::new(RealtimeDataManager::new(
            key.symbol.clone(),
            key.timeframe.clone(),
            market_bars,
        ));

        let binance_client = BinanceClient::new();
        let historical_err = data_manager
            .load_historical(&binance_client)
            .await
            .err()
            .map(|err| err.to_string());
        if let Some(err_msg) = historical_err {
            match self.data_loader.load_local(&key.symbol, &key.timeframe) {
                Ok(mut fallback) => {
                    if fallback.len() > market_bars {
                        let start = fallback.len() - market_bars;
                        fallback.drain(..start);
                    }
                    data_manager.set_bars(fallback).await;
                    warn!(
                        "Falling back to local data for {} {} after historical load failed: {}",
                        key.symbol, key.timeframe, err_msg
                    );
                }
                Err(load_err) => {
                    return Err(format!(
                        "Data load error for {} {}: {}; fallback failed: {}",
                        key.symbol, key.timeframe, err_msg, load_err
                    ));
                }
            }
        }

        let ws_client = Arc::new(BinanceWsClient::new(
            key.symbol.clone(),
            key.timeframe.clone(),
        ));
        data_manager.clone().start(ws_client).await;

        {
            let mut runtimes = self.market_runtimes.write().await;
            if runtimes.contains_key(&key) {
                return Ok(key);
            }
            runtimes.insert(key.clone(), data_manager.clone());
        }

        let runner = Arc::clone(self);
        let market_key = key.clone();
        let mut rx = data_manager.subscribe();
        tokio::spawn(async move {
            info!(
                "RealtimeRunner subscribed to market {} {}",
                market_key.symbol, market_key.timeframe
            );
            while let Ok(update) = rx.recv().await {
                match update {
                    RealtimeUpdate::BarOpened { bar } => {
                        runner
                            .execute_market_scripts(
                                &market_key,
                                ScriptUpdateKind::BarOpen,
                                Some(bar.time),
                            )
                            .await;
                    }
                    RealtimeUpdate::FormingUpdate { bar } => {
                        runner
                            .execute_market_scripts(
                                &market_key,
                                ScriptUpdateKind::BarUpdate,
                                Some(bar.time),
                            )
                            .await;
                    }
                    RealtimeUpdate::BarClosed { bar } => {
                        runner
                            .execute_market_scripts(
                                &market_key,
                                ScriptUpdateKind::BarClose,
                                Some(bar.time),
                            )
                            .await;
                    }
                    RealtimeUpdate::Snapshot { .. } | RealtimeUpdate::NewBar { .. } => {}
                }
            }
        });

        Ok(key)
    }

    async fn execute_market_scripts(
        &self,
        market_key: &MarketKey,
        update_kind: ScriptUpdateKind,
        bar_time: Option<i64>,
    ) {
        let now = Instant::now();
        let session_ids: Vec<String> = {
            self.active_scripts
                .read()
                .await
                .iter()
                .filter_map(|(session_id, script)| {
                    if &script.market_key != market_key {
                        return None;
                    }
                    match update_kind {
                        ScriptUpdateKind::BarOpen | ScriptUpdateKind::BarUpdate => {
                            should_execute_forming_session(
                                script,
                                update_kind,
                                now,
                                self.throttle_duration,
                            )
                            .then_some(session_id.clone())
                        }
                        _ => should_execute_session(script, update_kind)
                            .then_some(session_id.clone()),
                    }
                })
                .collect()
        };

        for session_id in session_ids {
            self.execute_script(&session_id, update_kind, bar_time)
                .await;
        }
    }

    /// Execute a single script and broadcast result
    async fn execute_script(
        &self,
        session_id: &str,
        update_kind: ScriptUpdateKind,
        bar_time: Option<i64>,
    ) {
        let should_follow = self
            .execute_script_once(session_id, update_kind, bar_time, true)
            .await;
        if should_follow {
            let _ = self
                .execute_script_once(session_id, ScriptUpdateKind::OrderFill, bar_time, false)
                .await;
        }
    }

    async fn execute_script_once(
        &self,
        session_id: &str,
        update_kind: ScriptUpdateKind,
        bar_time: Option<i64>,
        allow_order_fill_followup: bool,
    ) -> bool {
        let (code, market_key, bars_limit, declaration) = {
            let scripts = self.active_scripts.read().await;
            let script = match scripts.get(session_id) {
                Some(s) => s,
                None => return false,
            };
            (
                script.code.clone(),
                script.market_key.clone(),
                script.bars_limit,
                script.declaration.clone(),
            )
        };
        let runtime = self.market_runtimes.read().await.get(&market_key).cloned();
        let Some(runtime) = runtime else {
            return false;
        };
        let mut bars = runtime.get_bars().await;
        if bars.len() > bars_limit {
            let start = bars.len() - bars_limit;
            bars.drain(..start);
        }

        let start = Instant::now();

        match self.engine.run(&code, &bars) {
            Ok(result) => {
                let exec_time = start.elapsed().as_millis() as u64;
                let mut result_with_time = result;
                result_with_time.exec_ms = Some(exec_time);
                let should_follow_with_order_fill = should_schedule_order_fill_followup(
                    &declaration,
                    update_kind,
                    &result_with_time,
                    bar_time,
                    allow_order_fill_followup,
                );

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
                    session_id: session_id.to_string(),
                    is_full: matches!(update_kind, ScriptUpdateKind::Snapshot),
                    update_kind,
                    script_kind: declaration.kind,
                    trigger: trigger_from_update_kind(update_kind),
                    bar_time,
                    result: result_with_time,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };

                let _ = self.results_tx.send(update);
                debug!(
                    "Executed session_id={} in {}ms (kind={:?})",
                    session_id, exec_time, update_kind
                );

                should_follow_with_order_fill
            }
            Err(errors) => {
                error!(
                    "Script execution failed for session_id={}: {:?}",
                    session_id, errors
                );
                false
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

    /// Get the parsed declaration for a session.
    pub async fn get_session_declaration(&self, session_id: &str) -> Option<ScriptDeclaration> {
        self.active_scripts
            .read()
            .await
            .get(session_id)
            .map(|s| s.declaration.clone())
    }
}

fn should_execute_session(script: &ActiveScript, update_kind: ScriptUpdateKind) -> bool {
    let trigger = match update_kind {
        ScriptUpdateKind::Snapshot => RealtimeExecutionTrigger::Snapshot,
        ScriptUpdateKind::BarOpen | ScriptUpdateKind::BarUpdate => RealtimeExecutionTrigger::Tick,
        ScriptUpdateKind::BarClose => RealtimeExecutionTrigger::BarClose,
        ScriptUpdateKind::OrderFill => RealtimeExecutionTrigger::OrderFill,
    };

    script.declaration.should_execute_on(trigger)
}

fn should_execute_forming_session(
    script: &ActiveScript,
    update_kind: ScriptUpdateKind,
    now: Instant,
    throttle_duration: Duration,
) -> bool {
    if !should_execute_session(script, update_kind) {
        return false;
    }

    matches!(update_kind, ScriptUpdateKind::BarOpen)
        || now.duration_since(script.last_executed) >= throttle_duration
}

fn trigger_from_update_kind(update_kind: ScriptUpdateKind) -> RealtimeExecutionTrigger {
    match update_kind {
        ScriptUpdateKind::Snapshot => RealtimeExecutionTrigger::Snapshot,
        ScriptUpdateKind::BarOpen | ScriptUpdateKind::BarUpdate => RealtimeExecutionTrigger::Tick,
        ScriptUpdateKind::BarClose => RealtimeExecutionTrigger::BarClose,
        ScriptUpdateKind::OrderFill => RealtimeExecutionTrigger::OrderFill,
    }
}

fn result_has_fill_on_bar(result: &ApiResponse, bar_time: Option<i64>) -> bool {
    let Some(bar_time) = bar_time else {
        return false;
    };
    let Some(strategy) = &result.strategy else {
        return false;
    };

    strategy
        .entries
        .iter()
        .any(|signal| signal.time == bar_time)
        || strategy.exits.iter().any(|signal| signal.time == bar_time)
}

fn should_schedule_order_fill_followup(
    declaration: &ScriptDeclaration,
    update_kind: ScriptUpdateKind,
    result: &ApiResponse,
    bar_time: Option<i64>,
    allow_order_fill_followup: bool,
) -> bool {
    allow_order_fill_followup
        && (declaration.should_execute_on(RealtimeExecutionTrigger::OrderFill)
            || (declaration.process_orders_on_close && update_kind == ScriptUpdateKind::BarClose))
        && update_kind != ScriptUpdateKind::OrderFill
        && result_has_fill_on_bar(result, bar_time)
}

/// Generate a short unique ID
fn generate_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed) as u128;
    let mixed = nanos ^ counter;
    let hex = format!("{:016x}", mixed);
    hex[hex.len() - 8..].to_string()
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

    #[test]
    fn strategy_defaults_to_bar_close_only_in_realtime_runner() {
        let script = ActiveScript {
            code: String::new(),
            declaration: ScriptDeclaration {
                kind: crate::engine::runner::ScriptKind::Strategy,
                overlay: true,
                calc_on_every_tick: false,
                calc_on_order_fills: false,
                process_orders_on_close: false,
            },
            last_result: None,
            last_executed: Instant::now(),
            market_key: MarketKey::new("BTCUSDT".to_string(), "1m".to_string()),
            bars_limit: 500,
        };

        assert!(!should_execute_session(&script, ScriptUpdateKind::BarOpen));
        assert!(!should_execute_session(
            &script,
            ScriptUpdateKind::BarUpdate
        ));
        assert!(should_execute_session(&script, ScriptUpdateKind::BarClose));
    }

    #[test]
    fn indicator_executes_on_every_tick_in_realtime_runner() {
        let script = ActiveScript {
            code: String::new(),
            declaration: ScriptDeclaration {
                kind: crate::engine::runner::ScriptKind::Indicator,
                overlay: false,
                calc_on_every_tick: false,
                calc_on_order_fills: false,
                process_orders_on_close: false,
            },
            last_result: None,
            last_executed: Instant::now(),
            market_key: MarketKey::new("BTCUSDT".to_string(), "1m".to_string()),
            bars_limit: 500,
        };

        assert!(should_execute_session(&script, ScriptUpdateKind::BarOpen));
        assert!(should_execute_session(&script, ScriptUpdateKind::BarUpdate));
        assert!(should_execute_session(&script, ScriptUpdateKind::BarClose));
    }

    #[test]
    fn bar_open_executes_immediately_but_bar_update_obeys_throttle() {
        let script = ActiveScript {
            code: String::new(),
            declaration: ScriptDeclaration {
                kind: crate::engine::runner::ScriptKind::Indicator,
                overlay: false,
                calc_on_every_tick: false,
                calc_on_order_fills: false,
                process_orders_on_close: false,
            },
            last_result: None,
            last_executed: Instant::now(),
            market_key: MarketKey::new("BTCUSDT".to_string(), "1m".to_string()),
            bars_limit: 500,
        };

        assert!(should_execute_forming_session(
            &script,
            ScriptUpdateKind::BarOpen,
            Instant::now(),
            Duration::from_secs(60),
        ));
        assert!(!should_execute_forming_session(
            &script,
            ScriptUpdateKind::BarUpdate,
            Instant::now(),
            Duration::from_secs(60),
        ));
    }

    #[test]
    fn strategy_with_calc_on_order_fills_accepts_order_fill_trigger() {
        let script = ActiveScript {
            code: String::new(),
            declaration: ScriptDeclaration {
                kind: crate::engine::runner::ScriptKind::Strategy,
                overlay: true,
                calc_on_every_tick: false,
                calc_on_order_fills: true,
                process_orders_on_close: false,
            },
            last_result: None,
            last_executed: Instant::now(),
            market_key: MarketKey::new("BTCUSDT".to_string(), "1m".to_string()),
            bars_limit: 500,
        };

        assert!(should_execute_session(&script, ScriptUpdateKind::OrderFill));
    }

    #[test]
    fn result_has_fill_only_when_signal_matches_trigger_bar() {
        let response = ApiResponse {
            ok: true,
            exec_ms: Some(1),
            plots: None,
            strategy: Some(crate::engine::output::StrategyOutput {
                name: "Strategy".to_string(),
                entries: vec![crate::engine::output::TradeSignal {
                    bar_index: 1,
                    time: 100,
                    signal_type: "entry".to_string(),
                    id: "Long".to_string(),
                    direction: "long".to_string(),
                    qty: 1.0,
                    price: None,
                    comment: None,
                }],
                exits: vec![],
                trades: vec![],
                report: crate::engine::output::StrategyReport {
                    initial_capital: 100000.0,
                    equity: 100000.0,
                    net_profit: 0.0,
                    net_profit_percent: 0.0,
                    gross_profit: 0.0,
                    gross_loss: 0.0,
                    total_commission: 0.0,
                    total_slippage_cost: 0.0,
                    total_closed_trades: 0,
                    winning_trades: 0,
                    losing_trades: 0,
                    win_rate: 0.0,
                    profit_factor: None,
                    avg_trade: 0.0,
                    avg_trade_percent: 0.0,
                    largest_win: 0.0,
                    largest_loss: 0.0,
                    max_drawdown: 0.0,
                    max_drawdown_percent: 0.0,
                    avg_bars_held: 0.0,
                    open_trades: 1,
                    long: crate::engine::output::StrategySideReport {
                        closed_trades: 0,
                        winning_trades: 0,
                        net_profit: 0.0,
                        win_rate: 0.0,
                    },
                    short: crate::engine::output::StrategySideReport {
                        closed_trades: 0,
                        winning_trades: 0,
                        net_profit: 0.0,
                        win_rate: 0.0,
                    },
                    equity_curve: Vec::new(),
                },
                position_size: 1.0,
                position_direction: "long".to_string(),
            }),
            errors: None,
        };

        assert!(result_has_fill_on_bar(&response, Some(100)));
        assert!(!result_has_fill_on_bar(&response, Some(200)));
        assert!(!result_has_fill_on_bar(&response, None));
    }

    #[test]
    fn order_fill_followup_requires_opt_in_and_matching_signal() {
        let response = ApiResponse {
            ok: true,
            exec_ms: Some(1),
            plots: None,
            strategy: Some(crate::engine::output::StrategyOutput {
                name: "Strategy".to_string(),
                entries: vec![crate::engine::output::TradeSignal {
                    bar_index: 1,
                    time: 100,
                    signal_type: "entry".to_string(),
                    id: "Long".to_string(),
                    direction: "long".to_string(),
                    qty: 1.0,
                    price: None,
                    comment: None,
                }],
                exits: vec![],
                trades: vec![],
                report: crate::engine::output::StrategyReport {
                    initial_capital: 100000.0,
                    equity: 100000.0,
                    net_profit: 0.0,
                    net_profit_percent: 0.0,
                    gross_profit: 0.0,
                    gross_loss: 0.0,
                    total_commission: 0.0,
                    total_slippage_cost: 0.0,
                    total_closed_trades: 0,
                    winning_trades: 0,
                    losing_trades: 0,
                    win_rate: 0.0,
                    profit_factor: None,
                    avg_trade: 0.0,
                    avg_trade_percent: 0.0,
                    largest_win: 0.0,
                    largest_loss: 0.0,
                    max_drawdown: 0.0,
                    max_drawdown_percent: 0.0,
                    avg_bars_held: 0.0,
                    open_trades: 1,
                    long: crate::engine::output::StrategySideReport {
                        closed_trades: 0,
                        winning_trades: 0,
                        net_profit: 0.0,
                        win_rate: 0.0,
                    },
                    short: crate::engine::output::StrategySideReport {
                        closed_trades: 0,
                        winning_trades: 0,
                        net_profit: 0.0,
                        win_rate: 0.0,
                    },
                    equity_curve: Vec::new(),
                },
                position_size: 1.0,
                position_direction: "long".to_string(),
            }),
            errors: None,
        };
        let opt_in = ScriptDeclaration {
            kind: crate::engine::runner::ScriptKind::Strategy,
            overlay: true,
            calc_on_every_tick: false,
            calc_on_order_fills: true,
            process_orders_on_close: false,
        };
        let opt_out = ScriptDeclaration {
            calc_on_order_fills: false,
            ..opt_in.clone()
        };

        assert!(opt_in.should_execute_on(RealtimeExecutionTrigger::OrderFill));
        assert!(result_has_fill_on_bar(&response, Some(100)));
        assert!(!result_has_fill_on_bar(&response, Some(101)));
        assert!(!opt_out.should_execute_on(RealtimeExecutionTrigger::OrderFill));
    }

    #[test]
    fn process_orders_on_close_allows_bar_close_followup_without_calc_on_order_fills() {
        let declaration = ScriptDeclaration {
            kind: crate::engine::runner::ScriptKind::Strategy,
            overlay: true,
            calc_on_every_tick: false,
            calc_on_order_fills: false,
            process_orders_on_close: true,
        };

        assert!(!declaration.should_execute_on(RealtimeExecutionTrigger::OrderFill));
        assert!(declaration.process_orders_on_close);
    }

    #[test]
    fn update_kind_maps_to_expected_trigger() {
        assert_eq!(
            trigger_from_update_kind(ScriptUpdateKind::BarOpen),
            RealtimeExecutionTrigger::Tick
        );
        assert_eq!(
            trigger_from_update_kind(ScriptUpdateKind::BarUpdate),
            RealtimeExecutionTrigger::Tick
        );
        assert_eq!(
            trigger_from_update_kind(ScriptUpdateKind::BarClose),
            RealtimeExecutionTrigger::BarClose
        );
        assert_eq!(
            trigger_from_update_kind(ScriptUpdateKind::OrderFill),
            RealtimeExecutionTrigger::OrderFill
        );
    }

    #[test]
    fn followup_matrix_covers_tick_close_and_order_fill_paths() {
        let response = ApiResponse {
            ok: true,
            exec_ms: Some(1),
            plots: None,
            strategy: Some(crate::engine::output::StrategyOutput {
                name: "Strategy".to_string(),
                entries: vec![crate::engine::output::TradeSignal {
                    bar_index: 1,
                    time: 100,
                    signal_type: "entry".to_string(),
                    id: "Long".to_string(),
                    direction: "long".to_string(),
                    qty: 1.0,
                    price: None,
                    comment: None,
                }],
                exits: vec![],
                trades: vec![],
                report: crate::engine::output::StrategyReport {
                    initial_capital: 100000.0,
                    equity: 100000.0,
                    net_profit: 0.0,
                    net_profit_percent: 0.0,
                    gross_profit: 0.0,
                    gross_loss: 0.0,
                    total_commission: 0.0,
                    total_slippage_cost: 0.0,
                    total_closed_trades: 0,
                    winning_trades: 0,
                    losing_trades: 0,
                    win_rate: 0.0,
                    profit_factor: None,
                    avg_trade: 0.0,
                    avg_trade_percent: 0.0,
                    largest_win: 0.0,
                    largest_loss: 0.0,
                    max_drawdown: 0.0,
                    max_drawdown_percent: 0.0,
                    avg_bars_held: 0.0,
                    open_trades: 1,
                    long: crate::engine::output::StrategySideReport {
                        closed_trades: 0,
                        winning_trades: 0,
                        net_profit: 0.0,
                        win_rate: 0.0,
                    },
                    short: crate::engine::output::StrategySideReport {
                        closed_trades: 0,
                        winning_trades: 0,
                        net_profit: 0.0,
                        win_rate: 0.0,
                    },
                    equity_curve: Vec::new(),
                },
                position_size: 1.0,
                position_direction: "long".to_string(),
            }),
            errors: None,
        };
        let calc_on_fill = ScriptDeclaration {
            kind: crate::engine::runner::ScriptKind::Strategy,
            overlay: true,
            calc_on_every_tick: false,
            calc_on_order_fills: true,
            process_orders_on_close: false,
        };
        let on_close_only = ScriptDeclaration {
            calc_on_order_fills: false,
            process_orders_on_close: true,
            ..calc_on_fill.clone()
        };

        assert!(should_schedule_order_fill_followup(
            &calc_on_fill,
            ScriptUpdateKind::BarOpen,
            &response,
            Some(100),
            true,
        ));
        assert!(should_schedule_order_fill_followup(
            &calc_on_fill,
            ScriptUpdateKind::BarUpdate,
            &response,
            Some(100),
            true,
        ));
        assert!(should_schedule_order_fill_followup(
            &on_close_only,
            ScriptUpdateKind::BarClose,
            &response,
            Some(100),
            true,
        ));
        assert!(!should_schedule_order_fill_followup(
            &on_close_only,
            ScriptUpdateKind::OrderFill,
            &response,
            Some(100),
            true,
        ));
        assert!(!should_schedule_order_fill_followup(
            &calc_on_fill,
            ScriptUpdateKind::BarUpdate,
            &response,
            Some(999),
            true,
        ));
    }
}
