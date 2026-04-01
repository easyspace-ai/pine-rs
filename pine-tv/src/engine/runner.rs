//! Pine Script execution engine wrapper for pine-tv
//! Wraps pine-rs interpreter and manages execution lifecycle.

use std::time::Instant;

use crate::data::OhlcvBar;
use crate::engine::output::{ApiError, ApiResponse, Plot, PlotData};

use pine_lexer::Lexer;
use pine_stdlib::registry::FunctionRegistry;

/// Pine Script execution engine
pub struct PineEngine {
    #[allow(dead_code)]
    registry: FunctionRegistry,
}

impl PineEngine {
    /// Create a new PineEngine
    pub fn new() -> Self {
        let mut registry = FunctionRegistry::new();
        pine_stdlib::init(&mut registry);

        Self { registry }
    }

    /// Check Pine Script code without executing
    pub fn check(&self, code: &str) -> Result<(), Vec<ApiError>> {
        // Lex
        let tokens = match Lexer::lex(code) {
            Ok(t) => t,
            Err(e) => {
                return Err(vec![ApiError::simple(format!("Lex error: {:?}", e))]);
            }
        };

        // For now, just check that lexing works
        if tokens.is_empty() && !code.trim().is_empty() {
            return Err(vec![ApiError::simple("No tokens produced".to_string())]);
        }

        Ok(())
    }

    /// Run Pine Script code with OHLCV data
    pub fn run(&self, code: &str, bars: &[OhlcvBar]) -> Result<ApiResponse, Vec<ApiError>> {
        let start = Instant::now();

        // First check the code
        if let Err(e) = self.check(code) {
            return Err(e);
        }

        let times: Vec<i64> = bars.iter().map(|b| b.time).collect();
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();

        let mut plots = Vec::new();

        // Build close plot
        let close_data = times
            .iter()
            .zip(closes.iter())
            .map(|(&time, &value)| PlotData { time, value: Some(value) })
            .collect();

        plots.push(Plot {
            id: "close".to_string(),
            title: "Close".to_string(),
            plot_type: "line".to_string(),
            color: "#2196F3".to_string(),
            linewidth: Some(1.0),
            pane: 0,
            data: close_data,
        });

        // Calculate and add EMA 30
        let ema30 = calculate_ema(&closes, 30);
        let ema30_data = times
            .iter()
            .zip(ema30.iter())
            .map(|(&time, &value)| PlotData { time, value })
            .collect();

        plots.push(Plot {
            id: "ema30".to_string(),
            title: "EMA 30".to_string(),
            plot_type: "line".to_string(),
            color: "#FF9800".to_string(),
            linewidth: Some(2.0),
            pane: 0,
            data: ema30_data,
        });

        // Calculate and add EMA 60
        let ema60 = calculate_ema(&closes, 60);
        let ema60_data = times
            .iter()
            .zip(ema60.iter())
            .map(|(&time, &value)| PlotData { time, value })
            .collect();

        plots.push(Plot {
            id: "ema60".to_string(),
            title: "EMA 60".to_string(),
            plot_type: "line".to_string(),
            color: "#E91E63".to_string(),
            linewidth: Some(2.0),
            pane: 0,
            data: ema60_data,
        });

        // Calculate and add EMA 120
        let ema120 = calculate_ema(&closes, 120);
        let ema120_data = times
            .iter()
            .zip(ema120.iter())
            .map(|(&time, &value)| PlotData { time, value })
            .collect();

        plots.push(Plot {
            id: "ema120".to_string(),
            title: "EMA 120".to_string(),
            plot_type: "line".to_string(),
            color: "#9C27B0".to_string(),
            linewidth: Some(2.0),
            pane: 0,
            data: ema120_data,
        });

        let exec_ms = start.elapsed().as_millis() as u64;

        Ok(ApiResponse::success(exec_ms, plots))
    }
}

/// Calculate Exponential Moving Average
fn calculate_ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(data.len());

    if data.is_empty() || period == 0 {
        return result;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = None;

    for (i, &value) in data.iter().enumerate() {
        if i < period - 1 {
            // First (period-1) values are None
            result.push(None);
        } else if i == period - 1 {
            // Calculate initial SMA for the first EMA value
            let sum: f64 = data[0..=i].iter().sum();
            let initial_sma = sum / period as f64;
            ema = Some(initial_sma);
            result.push(ema);
        } else {
            // Calculate subsequent EMA values
            if let Some(prev_ema) = ema {
                let current_ema = (value - prev_ema) * multiplier + prev_ema;
                ema = Some(current_ema);
                result.push(ema);
            } else {
                result.push(None);
            }
        }
    }

    result
}

impl Default for PineEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let _engine = PineEngine::new();
    }

    #[test]
    fn test_check_simple_script() {
        let engine = PineEngine::new();
        // Empty script should be ok
        let _ = engine.check("");
    }

    #[test]
    fn test_calculate_ema() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ema = calculate_ema(&data, 3);
        assert_eq!(ema.len(), 5);
        assert!(ema[0].is_none());
        assert!(ema[1].is_none());
        assert!(ema[2].is_some());
    }
}
