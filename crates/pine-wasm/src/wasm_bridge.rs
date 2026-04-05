use pine_eval::runner::{run_bar_by_bar, BarData};
use pine_eval::EvaluationContext;
use pine_lexer::Lexer;
use pine_parser::StmtParser;
use pine_vm::executor::{execute_script_with_vm, SeriesData};
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

fn parse_source(source: &str) -> Result<pine_parser::ast::Script, JsValue> {
    let tokens = Lexer::lex_with_indentation(source).map_err(|_| to_js_err("lex error"))?;
    let mut parser = StmtParser::new(tokens);
    parser.parse_script().map_err(|e| to_js_err(format!("{e}")))
}

fn parse_bars(bars_json: &str) -> Result<Vec<JsBar>, JsValue> {
    serde_json::from_str(bars_json).map_err(|e| to_js_err(e.to_string()))
}

/// Parse-only check (indentation-aware lexer + script parser).
#[wasm_bindgen(js_name = checkScript)]
pub fn check_script(source: &str) -> Result<(), JsValue> {
    parse_source(source).map(|_| ())
}

/// Run script on OHLCV bars using eval; returns JSON map of plot title → value series.
#[wasm_bindgen(js_name = runScriptJson)]
pub fn run_script_json(source: &str, bars_json: &str) -> Result<String, JsValue> {
    let parsed = parse_bars(bars_json)?;
    let bars: Vec<BarData> = parsed
        .into_iter()
        .map(|b| BarData::new(b.open, b.high, b.low, b.close, b.volume, b.time))
        .collect();

    let script = parse_source(source)?;

    let mut ctx = EvaluationContext::new();
    run_bar_by_bar(&script, &bars, &mut ctx).map_err(|e| to_js_err(format!("{e}")))?;

    let plots = ctx.plot_outputs.get_plots().clone();
    serde_json::to_string(&plots).map_err(|e| to_js_err(e.to_string()))
}

/// Run script on OHLCV bars using VM; returns JSON map of plot title → value series.
#[wasm_bindgen(js_name = runScriptJsonVm)]
pub fn run_script_json_vm(source: &str, bars_json: &str) -> Result<String, JsValue> {
    let parsed = parse_bars(bars_json)?;
    let series_data = SeriesData::new(
        parsed.iter().map(|b| b.open).collect(),
        parsed.iter().map(|b| b.high).collect(),
        parsed.iter().map(|b| b.low).collect(),
        parsed.iter().map(|b| b.close).collect(),
        parsed.iter().map(|b| b.volume).collect(),
        parsed.iter().map(|b| b.time).collect(),
    );

    let script = parse_source(source)?;

    let result =
        execute_script_with_vm(&script, &series_data).map_err(|e| to_js_err(format!("{e:?}")))?;

    let plots = result.plot_outputs.get_plots().clone();
    serde_json::to_string(&plots).map_err(|e| to_js_err(e.to_string()))
}
