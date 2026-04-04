//! Bug Condition Exploration Test - Multi-Pane Display Issue (API Level)
//!
//! **Validates: Requirements 1.1, 1.2, 1.3**
//!
//! **Property 1: Bug Condition** - Multiple Indicator Panes Not Displayed
//!
//! This test verifies that Pine Scripts with multiple pane indices return
//! the correct plot data from the API. The bug manifests in the frontend
//! (chart.js), but we can verify the backend correctly provides multiple
//! pane indices.
//!
//! **EXPECTED OUTCOME**: This API test should PASS (backend is correct)
//! The bug is in the frontend chart.js applyResult function.

use axum::{body::Body, http::Request, routing::post, Router};
use std::sync::Arc;
use tower::ServiceExt;

use pine_tv::engine::runner::{ExecutionMode, PineEngine};
use pine_tv::routes::{RunHandler};

fn test_app() -> Router {
    let engine = Arc::new(PineEngine::with_mode(ExecutionMode::Eval));
    let data_loader = Arc::new(pine_tv::data::loader::DataLoader::new(
        "tests/data".to_string(),
    ));
    let run_handler = Arc::new(RunHandler::new(engine.clone(), data_loader));

    Router::new()
        .route("/api/run", post(RunHandler::handle).with_state(run_handler))
}

/// Test Case 1: Two Panes (pane=1, pane=2)
///
/// Verifies that the API returns plots with correct pane indices.
/// The bug is in the frontend, so this test should PASS.
#[tokio::test]
async fn test_two_panes_api_response() {
    let app = test_app();

    let code = r#"//@version=6
indicator("Two Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(rsi_val, "RSI", color=color.orange, pane=2)
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
    
    println!("API Response: {}", serde_json::to_string_pretty(&json).unwrap());
    
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    
    assert_eq!(plots.len(), 2, "Should have 2 plots");

    // Verify pane indices
    let pane_indices: Vec<i64> = plots
        .iter()
        .filter_map(|p| p.get("pane").and_then(|v| v.as_i64()))
        .collect();
    
    println!("Pane indices from API: {:?}", pane_indices);
    
    assert!(pane_indices.contains(&1), "Should have plot with pane=1");
    assert!(pane_indices.contains(&2), "Should have plot with pane=2");
    
    // Verify titles
    let titles: Vec<&str> = plots
        .iter()
        .filter_map(|p| p.get("title").and_then(|v| v.as_str()))
        .collect();
    
    assert!(titles.contains(&"SMA"), "Should have SMA plot");
    assert!(titles.contains(&"RSI"), "Should have RSI plot");
}

/// Test Case 2: Three Panes (pane=1, pane=2, pane=3)
#[tokio::test]
async fn test_three_panes_api_response() {
    let app = test_app();

    let code = r#"//@version=6
indicator("Three Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
rsi_val = ta.rsi(close, 14)
macd_val = ta.macd(close, 12, 26, 9)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(rsi_val, "RSI", color=color.orange, pane=2)
plot(macd_val[0], "MACD", color=color.green, pane=3)
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
    
    println!("API Response: {}", serde_json::to_string_pretty(&json).unwrap());
    
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    
    assert_eq!(plots.len(), 3, "Should have 3 plots");

    let pane_indices: Vec<i64> = plots
        .iter()
        .filter_map(|p| p.get("pane").and_then(|v| v.as_i64()))
        .collect();
    
    println!("Pane indices from API: {:?}", pane_indices);
    
    assert!(pane_indices.contains(&1), "Should have plot with pane=1");
    assert!(pane_indices.contains(&2), "Should have plot with pane=2");
    assert!(pane_indices.contains(&3), "Should have plot with pane=3");
}

/// Test Case 3: Non-Contiguous Panes (pane=1, pane=3)
#[tokio::test]
async fn test_non_contiguous_panes_api_response() {
    let app = test_app();

    let code = r#"//@version=6
indicator("Non-Contiguous Pane Test", overlay=false)
sma_val = ta.sma(close, 14)
macd_val = ta.macd(close, 12, 26, 9)
plot(sma_val, "SMA", color=color.blue, pane=1)
plot(macd_val[0], "MACD", color=color.green, pane=3)
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
    
    println!("API Response: {}", serde_json::to_string_pretty(&json).unwrap());
    
    assert!(json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    let plots = json
        .get("plots")
        .and_then(|v| v.as_array())
        .expect("expected plots array");
    
    assert_eq!(plots.len(), 2, "Should have 2 plots");

    let pane_indices: Vec<i64> = plots
        .iter()
        .filter_map(|p| p.get("pane").and_then(|v| v.as_i64()))
        .collect();
    
    println!("Pane indices from API: {:?}", pane_indices);
    
    assert!(pane_indices.contains(&1), "Should have plot with pane=1");
    assert!(pane_indices.contains(&3), "Should have plot with pane=3");
    assert!(!pane_indices.contains(&2), "Should NOT have plot with pane=2");
}

/// Documentation of Bug Condition
///
/// This test suite confirms that:
/// 1. The backend API correctly returns plots with multiple pane indices
/// 2. The pane indices are correctly set in the JSON response
/// 3. The bug is NOT in the backend
///
/// **Root Cause**: The bug is in `pine-tv/static/chart.js`, function `applyResult`
/// (lines 289-293), which only calls `setHeight` on the maximum pane index without
/// ensuring all intermediate panes are properly created and visible.
///
/// **Counterexamples** (to be verified in browser):
/// - When API returns plots with pane=1 and pane=2, only pane 2 is visible in browser
/// - When API returns plots with pane=1, pane=2, pane=3, only pane 3 is visible
/// - Intermediate panes have zero or minimal height in the DOM
///
/// **Next Steps**:
/// 1. Run manual browser verification (see manual_bug_verification.md)
/// 2. Document observed pane heights and visibility in browser
/// 3. Proceed to implement fix in chart.js
#[test]
fn document_bug_condition() {
    println!("\n=== Bug Condition Documentation ===\n");
    println!("Bug: Multiple Indicator Panes Not Displayed");
    println!("Location: pine-tv/static/chart.js, function applyResult (lines 289-293)");
    println!("\nCurrent buggy code:");
    println!("  if (maxPane > 0 && chart.panes().length > maxPane && chartContainer) {{");
    println!("      const sub = chart.panes()[maxPane];");
    println!("      const h = Math.max(100, Math.floor(chartContainer.clientHeight * 0.28));");
    println!("      sub.setHeight(h);");
    println!("  }}");
    println!("\nProblem: Only sets height for maxPane, doesn't iterate through all unique pane indices.");
    println!("\nExpected behavior:");
    println!("  - Collect all unique pane indices from plots array");
    println!("  - Iterate through each unique pane index");
    println!("  - Set appropriate height for each pane");
    println!("  - Ensure all panes are visible with height >= 80px");
    println!("\nVerification:");
    println!("  1. Run: cargo test -p pine-tv test_two_panes_api_response -- --nocapture");
    println!("  2. Observe API returns correct pane indices");
    println!("  3. Run manual browser test (see manual_bug_verification.md)");
    println!("  4. Observe only highest pane is visible in browser");
    println!("  5. This confirms bug is in frontend chart.js, not backend");
    println!("\n=== End Documentation ===\n");
}
