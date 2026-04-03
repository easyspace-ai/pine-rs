//! pine-tv - Local TradingView-like web playground for Pine Script v6
//!
//! This is the main entry point for the pine-tv web application.
//! It serves a static frontend and provides REST API endpoints for
//! executing Pine Script code.

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod data;
mod engine;
mod routes;

use data::{BinanceClient, BinanceWsClient, DataLoader, RealtimeDataManager};
use engine::{ExecutionMode, PineEngine};
use routes::{CheckHandler, DataHandler, RunHandler, WsHandler};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Path to static files directory
    pub static_path: String,
    /// Path to data directory
    pub data_path: String,
    /// Default symbol
    pub default_symbol: String,
    /// Default timeframe
    pub default_tf: String,
    /// Max bars to keep
    pub max_bars: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 7070,
            static_path: "pine-tv/static".to_string(),
            data_path: "tests/data".to_string(),
            default_symbol: "BTCUSDT".to_string(),
            default_tf: "1h".to_string(),
            max_bars: 500,
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pine_tv=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = ServerConfig::default();

    // Override static path from environment if provided
    let static_path = std::env::var("PINE_TV_STATIC").unwrap_or(config.static_path);

    // Create shared state
    let engine = Arc::new(PineEngine::new());
    let mode = engine.execution_mode();
    tracing::info!(
        "Pine execution backend: {}",
        match mode {
            ExecutionMode::Eval => "pine-eval (default; set PINE_TV_MODE=vm for VM)",
            ExecutionMode::Vm => "pine-vm (PINE_TV_MODE=vm)",
        }
    );
    let data_loader = Arc::new(DataLoader::new(config.data_path));
    let binance_client = BinanceClient::new();

    // Set up real-time data manager
    let data_manager = Arc::new(RealtimeDataManager::new(
        config.default_symbol.clone(),
        config.default_tf.clone(),
        config.max_bars,
    ));

    // Load initial historical data
    if let Err(e) = data_manager.load_historical(&binance_client).await {
        tracing::warn!("Failed to load initial historical data: {}", e);
    }

    // Start Binance WebSocket (spawns internally)
    let ws_client = Arc::new(BinanceWsClient::new(
        config.default_symbol.clone(),
        config.default_tf.clone(),
    ));

    // Start processing real-time updates
    data_manager.clone().start(ws_client).await;

    // Create handlers
    let run_handler = Arc::new(RunHandler::new(engine.clone(), data_loader.clone()));
    let check_handler = Arc::new(CheckHandler::new(engine.clone()));
    let data_handler = Arc::new(DataHandler::new(data_loader.clone()));
    let ws_handler = Arc::new(WsHandler::new(data_manager.clone(), engine.clone()));

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/run", post(RunHandler::handle).with_state(run_handler))
        .route(
            "/api/check",
            post(CheckHandler::handle).with_state(check_handler),
        )
        .route(
            "/api/data/:symbol/:tf",
            get(DataHandler::handle).with_state(data_handler),
        )
        .route("/api/ws", get(WsHandler::handle_ws).with_state(ws_handler))
        // Example scripts routes
        .merge(routes::examples::router())
        // Static files
        .fallback_service(ServeDir::new(static_path));

    // Start server
    use std::net::SocketAddr;
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    tracing::info!("pine-tv listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
