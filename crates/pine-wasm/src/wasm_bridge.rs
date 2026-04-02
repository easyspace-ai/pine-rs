use pine_eval::runner::{run_bar_by_bar, BarData};
use pine_eval::EvaluationContext;
use pine_lexer::Lexer;
use pine_parser::StmtParser;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct JsBar {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    time: i64,
}

fn to_js_err(msg: impl Into<String>) -> JsValue {
    JsValue::from_str(&msg.into())
}

/// Parse-only check (indentation-aware lexer + script parser).
#[wasm_bindgen(js_name = checkScript)]
pub fn check_script(source: &str) -> Result<(), JsValue> {
    let tokens = Lexer::lex_with_indentation(source).map_err(|_| to_js_err("lex error"))?;
    let mut parser = StmtParser::new(tokens);
    parser
        .parse_script()
        .map_err(|e| to_js_err(format!("{e}")))?;
    Ok(())
}

/// Run script on OHLCV bars; returns JSON map of plot title → value series.
#[wasm_bindgen(js_name = runScriptJson)]
pub fn run_script_json(source: &str, bars_json: &str) -> Result<String, JsValue> {
    let parsed: Vec<JsBar> =
        serde_json::from_str(bars_json).map_err(|e| to_js_err(e.to_string()))?;
    let bars: Vec<BarData> = parsed
        .into_iter()
        .map(|b| BarData::new(b.open, b.high, b.low, b.close, b.volume, b.time))
        .collect();

    let tokens = Lexer::lex_with_indentation(source).map_err(|_| to_js_err("lex error"))?;
    let mut parser = StmtParser::new(tokens);
    let script = parser
        .parse_script()
        .map_err(|e| to_js_err(format!("{e}")))?;

    let mut ctx = EvaluationContext::new();
    run_bar_by_bar(&script, &bars, &mut ctx).map_err(|e| to_js_err(format!("{e}")))?;

    let plots = ctx.plot_outputs.get_plots().clone();
    serde_json::to_string(&plots).map_err(|e| to_js_err(e.to_string()))
}
