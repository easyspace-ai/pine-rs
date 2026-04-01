//! Bar-by-bar execution runner
//!
//! This module provides the main execution loop for Pine Script programs,
//! processing data bar by bar (candle by candle).

use crate::eval_stmt::eval_stmt;
use crate::{EvaluationContext, Result, SeriesData};
use pine_parser::ast;
use pine_runtime::series::SeriesBufF64;
use pine_runtime::value::Value;

/// Input data for a single bar
#[derive(Debug, Clone)]
pub struct BarData {
    /// Opening price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Volume
    pub volume: f64,
    /// Timestamp (milliseconds since epoch)
    pub time: i64,
}

impl BarData {
    /// Create a new bar with OHLCV data
    pub fn new(open: f64, high: f64, low: f64, close: f64, volume: f64, time: i64) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
            time,
        }
    }

    /// Get the typical price (hlc3)
    pub fn hlc3(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Get the OHLC average (ohlc4)
    pub fn ohlc4(&self) -> f64 {
        (self.open + self.high + self.low + self.close) / 4.0
    }

    /// Get the HL average (hl2)
    pub fn hl2(&self) -> f64 {
        (self.high + self.low) / 2.0
    }
}

/// Execution state for bar-by-bar processing
#[derive(Debug)]
pub struct ExecutionState {
    /// Current bar index (0-based)
    pub current_bar: usize,
    /// Total number of bars processed
    pub total_bars: usize,
    /// Built-in series data
    /// Open price series
    pub open_series: SeriesBufF64,
    /// High price series
    pub high_series: SeriesBufF64,
    /// Low price series
    pub low_series: SeriesBufF64,
    /// Close price series
    pub close_series: SeriesBufF64,
    /// Volume series
    pub volume_series: SeriesBufF64,
    /// Time series (timestamps)
    pub time_series: SeriesBufF64,
}

impl ExecutionState {
    /// Create a new execution state with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            current_bar: 0,
            total_bars: 0,
            open_series: SeriesBufF64::new(capacity),
            high_series: SeriesBufF64::new(capacity),
            low_series: SeriesBufF64::new(capacity),
            close_series: SeriesBufF64::new(capacity),
            volume_series: SeriesBufF64::new(capacity),
            time_series: SeriesBufF64::new(capacity),
        }
    }

    /// Load historical data into the state
    pub fn load_history(&mut self, bars: &[BarData]) {
        let opens: Vec<f64> = bars.iter().map(|b| b.open).collect();
        let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
        let lows: Vec<f64> = bars.iter().map(|b| b.low).collect();
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
        let volumes: Vec<f64> = bars.iter().map(|b| b.volume).collect();
        let times: Vec<f64> = bars.iter().map(|b| b.time as f64).collect();

        self.open_series.extend(&opens);
        self.high_series.extend(&highs);
        self.low_series.extend(&lows);
        self.close_series.extend(&closes);
        self.volume_series.extend(&volumes);
        self.time_series.extend(&times);

        self.total_bars = bars.len();
    }

    /// Add a new bar and advance
    pub fn add_bar(&mut self, bar: &BarData) {
        self.open_series.push(bar.open);
        self.high_series.push(bar.high);
        self.low_series.push(bar.low);
        self.close_series.push(bar.close);
        self.volume_series.push(bar.volume);
        self.time_series.push(bar.time as f64);

        self.total_bars += 1;
        self.current_bar = self.total_bars - 1;
    }

    /// Get current bar data
    pub fn current_bar_data(&self) -> Option<BarData> {
        if self.current_bar >= self.total_bars {
            return None;
        }

        // Get data at current_bar index (0 is the oldest bar)
        Some(BarData {
            open: self.open_series.get(self.current_bar)?,
            high: self.high_series.get(self.current_bar)?,
            low: self.low_series.get(self.current_bar)?,
            close: self.close_series.get(self.current_bar)?,
            volume: self.volume_series.get(self.current_bar)?,
            time: self.time_series.get(self.current_bar)? as i64,
        })
    }
}

/// Run a script on historical data bar-by-bar
///
/// This is the main entry point for executing a Pine Script program.
/// It processes each bar sequentially, maintaining state across bars.
pub fn run_bar_by_bar(
    script: &ast::Script,
    bars: &[BarData],
    ctx: &mut EvaluationContext,
) -> Result<Vec<Value>> {
    let mut state = ExecutionState::new(bars.len());
    let mut results = Vec::with_capacity(bars.len());

    // Load all historical data
    state.load_history(bars);

    // Execute script for each bar
    for bar_idx in 0..bars.len() {
        state.current_bar = bar_idx;

        // Set up built-in variables for this bar
        setup_builtin_vars(ctx, &state, bar_idx)?;

        // Execute the script
        for stmt in &script.stmts {
            match eval_stmt(stmt, ctx) {
                Ok(_) => {}
                Err(e) => {
                    // Log error but continue to next bar
                    eprintln!("Error at bar {}: {}", bar_idx, e);
                    break;
                }
            }
        }

        // Advance plot outputs to next bar
        ctx.plot_outputs.next_bar();

        // TODO: Capture the actual result from statement evaluation
        results.push(Value::Na);
    }

    Ok(results)
}

/// Run a script on a single new bar (for real-time updates)
pub fn run_single_bar(
    script: &ast::Script,
    bar: &BarData,
    state: &mut ExecutionState,
    ctx: &mut EvaluationContext,
) -> Result<Value> {
    // Add the new bar to state
    state.add_bar(bar);

    // Set up built-in variables
    setup_builtin_vars(ctx, state, state.current_bar)?;

    // Execute the script
    for stmt in &script.stmts {
        match eval_stmt(stmt, ctx) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }

    // TODO: Return actual result from statement evaluation
    Ok(Value::Na)
}

/// Set up built-in variables (open, high, low, close, volume, etc.)
fn setup_builtin_vars(
    ctx: &mut EvaluationContext,
    state: &ExecutionState,
    bar_idx: usize,
) -> Result<()> {
    // bar_idx 0 is the oldest bar, bar_idx total_bars-1 is the newest
    let _idx = bar_idx;

    // Set up series data for historical access
    // IMPORTANT: Use to_vec_oldest_first() so index 0 = oldest bar, index N = newest bar
    ctx.series_data = Some(SeriesData {
        open: state.open_series.to_vec_oldest_first(),
        high: state.high_series.to_vec_oldest_first(),
        low: state.low_series.to_vec_oldest_first(),
        close: state.close_series.to_vec_oldest_first(),
        volume: state.volume_series.to_vec_oldest_first(),
        time: state.time_series.to_vec_oldest_first().into_iter().map(|t| t as i64).collect(),
        current_bar: bar_idx,
    });

    // Set built-in series variables
    // IMPORTANT: SeriesBufF64::get() expects offset from newest (0 = current bar)
    // bar_idx is index from oldest (0 = oldest bar)
    // So we need to calculate: offset = total_bars - 1 - bar_idx
    let offset = state.total_bars.saturating_sub(1).saturating_sub(bar_idx);

    if let Some(v) = state.open_series.get(offset) {
        ctx.set_var("open", Value::Float(v));
    } else {
        ctx.set_var("open", Value::Na);
    }

    if let Some(v) = state.high_series.get(offset) {
        ctx.set_var("high", Value::Float(v));
    } else {
        ctx.set_var("high", Value::Na);
    }

    if let Some(v) = state.low_series.get(offset) {
        ctx.set_var("low", Value::Float(v));
    } else {
        ctx.set_var("low", Value::Na);
    }

    if let Some(v) = state.close_series.get(offset) {
        ctx.set_var("close", Value::Float(v));
    } else {
        ctx.set_var("close", Value::Na);
    }

    if let Some(v) = state.volume_series.get(offset) {
        ctx.set_var("volume", Value::Float(v));
    } else {
        ctx.set_var("volume", Value::Na);
    }

    if let Some(v) = state.time_series.get(offset) {
        ctx.set_var("time", Value::Int(v as i64));
    } else {
        ctx.set_var("time", Value::Na);
    }

    // Set computed series
    if let (Some(h), Some(l)) = (state.high_series.get(offset), state.low_series.get(offset)) {
        ctx.set_var("hl2", Value::Float((h + l) / 2.0));
    } else {
        ctx.set_var("hl2", Value::Na);
    }

    if let (Some(h), Some(l), Some(c)) = (
        state.high_series.get(offset),
        state.low_series.get(offset),
        state.close_series.get(offset),
    ) {
        ctx.set_var("hlc3", Value::Float((h + l + c) / 3.0));
    } else {
        ctx.set_var("hlc3", Value::Na);
    }

    if let (Some(o), Some(h), Some(l), Some(c)) = (
        state.open_series.get(offset),
        state.high_series.get(offset),
        state.low_series.get(offset),
        state.close_series.get(offset),
    ) {
        ctx.set_var("ohlc4", Value::Float((o + h + l + c) / 4.0));
    } else {
        ctx.set_var("ohlc4", Value::Na);
    }

    // Set up builtin namespace objects for function calls like input.int(), ta.sma(), etc.
    ctx.set_var("input", Value::Namespace("input".to_string()));
    ctx.set_var("ta", Value::Namespace("ta".to_string()));
    ctx.set_var("math", Value::Namespace("math".to_string()));
    ctx.set_var("str", Value::Namespace("str".to_string()));
    ctx.set_var("color", Value::Namespace("color".to_string()));
    ctx.set_var("array", Value::Namespace("array".to_string()));
    ctx.set_var("map", Value::Namespace("map".to_string()));

    Ok(())
}

/// Simple script execution (for testing)
pub fn run(script: &ast::Script) -> Result<()> {
    let mut ctx = EvaluationContext::new();

    for stmt in &script.stmts {
        eval_stmt(stmt, &mut ctx)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bars() -> Vec<BarData> {
        vec![
            BarData::new(100.0, 105.0, 99.0, 102.0, 1000.0, 1609459200000),
            BarData::new(102.0, 108.0, 101.0, 107.0, 1200.0, 1609545600000),
            BarData::new(107.0, 110.0, 106.0, 108.0, 1100.0, 1609632000000),
        ]
    }

    #[test]
    fn test_bar_data() {
        let bar = BarData::new(100.0, 110.0, 90.0, 105.0, 1000.0, 0);
        assert_eq!(bar.hl2(), 100.0);
        assert_eq!(bar.hlc3(), 101.66666666666667);
        assert_eq!(bar.ohlc4(), 101.25);
    }

    #[test]
    fn test_execution_state() {
        let bars = create_test_bars();
        let mut state = ExecutionState::new(bars.len());

        state.load_history(&bars);
        assert_eq!(state.total_bars, 3);

        // Test current bar data (SeriesBufF64 stores newest at index 0)
        let current = state.current_bar_data().unwrap();
        assert_eq!(current.close, 108.0); // Last bar's close is at index 0
    }

    #[test]
    fn test_run_simple() {
        // Create an empty script
        let script = ast::Script {
            stmts: vec![],
            span: pine_lexer::Span::default(),
        };

        let result = run(&script);
        assert!(result.is_ok());
    }
}
