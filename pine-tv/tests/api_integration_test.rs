//! HTTP-level integration tests for pine-tv endpoints.

use axum::{body::Body, http::Request, routing::post, Router};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

use pine_tv::engine::runner::{ExecutionMode, PineEngine};
use pine_tv::routes::{CheckHandler, RunHandler};

#[derive(Debug, Deserialize)]
struct VmParityCase {
    script_path: String,
}

/// Helper to build the run/check router for testing.
fn test_app() -> Router {
    test_app_with_mode(ExecutionMode::Vm)
}

fn test_app_with_mode(mode: ExecutionMode) -> Router {
    let engine = Arc::new(PineEngine::with_mode(mode));
    let data_loader = Arc::new(pine_tv::data::loader::DataLoader::new(
        "../tests/data".to_string(),
    ));
    let run_handler = Arc::new(RunHandler::new(engine.clone(), data_loader));
    let check_handler = Arc::new(CheckHandler::new(engine));

    Router::new()
        .route("/api/run", post(RunHandler::handle).with_state(run_handler))
        .route(
            "/api/check",
            post(CheckHandler::handle).with_state(check_handler),
        )
}

fn load_vm_parity_cases() -> Vec<VmParityCase> {
    let content =
        std::fs::read_to_string("../tests/vm_parity_cases.json").expect("read VM parity manifest");
    serde_json::from_str(&content).expect("parse VM parity manifest")
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

async fn run_api_script(mode: ExecutionMode, code: &str, bars: usize) -> Value {
    let app = test_app_with_mode(mode);
    let body = serde_json::json!({
        "code": code,
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "bars": bars
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));
    json
}

fn plot_map(json: &Value) -> std::collections::BTreeMap<String, Vec<Value>> {
    json.get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array")
        .iter()
        .map(|plot| {
            let title = plot
                .get("title")
                .and_then(|v| v.as_str())
                .expect("plot title")
                .to_string();
            let data = plot
                .get("data")
                .and_then(|v| v.as_array())
                .expect("plot data")
                .clone();
            (title, data)
        })
        .collect()
}

#[tokio::test]
async fn test_api_check_valid_script() {
    let app = test_app();

    let body = serde_json::json!({
        "code": "//@version=6\nindicator(\"Test\")\nplot(close)"
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/check")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));
}

#[tokio::test]
async fn test_api_check_invalid_script() {
    let app = test_app();

    let body = serde_json::json!({
        "code": "//@version=6\nindicator(\"Test\"\nplot(close)"
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/check")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!json.get("ok").and_then(|v| v.as_bool()).unwrap_or(true));
    let errors = json
        .get("errors")
        .and_then(|v| v.as_array())
        .expect("expected errors array");
    assert!(!errors.is_empty());
}

#[tokio::test]
async fn test_api_run_sma_script() {
    let app = test_app();

    let code = r#"//@version=6
indicator("SMA Test")
plot(ta.sma(close, 5), title="SMA 5")
"#;

    let body = serde_json::json!({
        "code": code,
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "bars": 20
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    assert!(!plots.is_empty());

    let plot = &plots[0];
    assert_eq!(plot.get("title").and_then(|v| v.as_str()), Some("SMA 5"));
    let data = plot
        .get("data")
        .and_then(|v| v.as_array())
        .expect("expected plot data array");
    assert!(!data.is_empty());
}

#[tokio::test]
async fn test_api_run_for_na_math() {
    let app = test_app();

    let code = std::fs::read_to_string("../tests/scripts/language/for_na_math.pine")
        .expect("read for_na_math.pine");

    let body = serde_json::json!({
        "code": code,
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "bars": 20
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    assert!(!plots.is_empty());

    let titles: Vec<&str> = plots
        .iter()
        .filter_map(|p| p.get("title").and_then(|v| v.as_str()))
        .collect();
    assert!(titles.contains(&"For Math Result"));
}

#[tokio::test]
async fn test_api_run_udf_basic_eval() {
    // UDF has VM parity issues; run in Eval mode for coverage.
    let app = test_app_with_mode(ExecutionMode::Eval);

    let code = std::fs::read_to_string("../tests/scripts/language/udf_basic.pine")
        .expect("read udf_basic.pine");

    let body = serde_json::json!({
        "code": code,
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "bars": 20
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    assert!(!plots.is_empty());

    let titles: Vec<&str> = plots
        .iter()
        .filter_map(|p| p.get("title").and_then(|v| v.as_str()))
        .collect();
    assert!(titles.contains(&"UDF Diff"));
    assert!(titles.contains(&"UDF Scale"));
}

#[tokio::test]
async fn test_api_run_while_loop_eval() {
    // while + var has VM parity issues; run in Eval mode for coverage.
    let app = test_app_with_mode(ExecutionMode::Eval);

    let code = std::fs::read_to_string("../tests/scripts/language/while_loop.pine")
        .expect("read while_loop.pine");

    let body = serde_json::json!({
        "code": code,
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "bars": 20
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    assert!(!plots.is_empty());

    let titles: Vec<&str> = plots
        .iter()
        .filter_map(|p| p.get("title").and_then(|v| v.as_str()))
        .collect();
    assert!(titles.contains(&"While Avg 5"));
}

#[tokio::test]
async fn test_api_run_switch_basic_eval() {
    // switch is only implemented in Eval mode.
    let app = test_app_with_mode(ExecutionMode::Eval);

    let code = std::fs::read_to_string("../tests/scripts/language/switch_basic.pine")
        .expect("read switch_basic.pine");

    let body = serde_json::json!({
        "code": code,
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "bars": 20
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    assert!(!plots.is_empty());

    let titles: Vec<&str> = plots
        .iter()
        .filter_map(|p| p.get("title").and_then(|v| v.as_str()))
        .collect();
    assert!(titles.contains(&"Switch Result"));
}

#[tokio::test]
async fn test_api_vm_matches_eval_for_regression_scripts() {
    let cases = load_vm_parity_cases();
    assert_eq!(cases.len(), 59, "unexpected VM parity manifest size");
    let root = workspace_root();

    for case in cases {
        let code =
            std::fs::read_to_string(root.join(&case.script_path)).expect("read regression script");

        let eval_json = run_api_script(ExecutionMode::Eval, &code, 83).await;
        let vm_json = run_api_script(ExecutionMode::Vm, &code, 83).await;

        assert_eq!(
            plot_map(&eval_json),
            plot_map(&vm_json),
            "API plot mismatch for {}",
            case.script_path
        );
    }
}
