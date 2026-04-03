//! Pine Script execution engine wrapper for pine-tv
//! Wraps pine-rs interpreter and manages execution lifecycle.

use std::time::Instant;

use crate::data::OhlcvBar;
use crate::engine::output::{ApiError, ApiResponse, Plot, PlotData};

use pine_lexer::Lexer;
use pine_stdlib::registry::FunctionRegistry;

/// Execution mode for PineEngine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Use pine-vm (bytecode VM)
    Vm,
    /// Use pine-eval (tree-walking interpreter)
    Eval,
}

impl ExecutionMode {
    /// Get execution mode from environment variable PINE_TV_MODE
    /// Defaults to VM if not set or invalid
    pub fn from_env() -> Self {
        match std::env::var("PINE_TV_MODE").as_deref() {
            Ok("eval") => Self::Eval,
            _ => Self::Vm, // Default to VM since parity is achieved
        }
    }
}

/// Pine Script execution engine
pub struct PineEngine {
    #[allow(dead_code)]
    registry: FunctionRegistry,
    mode: ExecutionMode,
}

impl PineEngine {
    /// Create a new PineEngine with default mode (from environment)
    pub fn new() -> Self {
        let mut registry = FunctionRegistry::new();
        pine_stdlib::init(&mut registry);

        Self {
            registry,
            mode: ExecutionMode::from_env(),
        }
    }

    /// Create a new PineEngine with explicit mode
    #[allow(dead_code)]
    pub fn with_mode(mode: ExecutionMode) -> Self {
        let mut registry = FunctionRegistry::new();
        pine_stdlib::init(&mut registry);

        Self { registry, mode }
    }

    /// Check Pine Script code without executing
    pub fn check(&self, code: &str) -> Result<(), Vec<ApiError>> {
        // Lex with indentation
        let tokens = match Lexer::lex_with_indentation(code) {
            Ok(t) => t,
            Err(e) => {
                return Err(vec![ApiError::simple(format!("Lex error: {:?}", e))]);
            }
        };

        // Parse
        let _ast = match pine_parser::parser::parse(tokens) {
            Ok(ast) => ast,
            Err(e) => {
                return Err(vec![ApiError::simple(format!("Parse error: {:?}", e))]);
            }
        };

        Ok(())
    }

    /// Run Pine Script code with OHLCV data
    pub fn run(&self, code: &str, bars: &[OhlcvBar]) -> Result<ApiResponse, Vec<ApiError>> {
        let start = Instant::now();

        // 1. Lex with indentation
        let tokens = match Lexer::lex_with_indentation(code) {
            Ok(t) => t,
            Err(e) => {
                return Err(vec![ApiError::simple(format!("Lex error: {:?}", e))]);
            }
        };

        // 2. Parse
        let ast = match pine_parser::parser::parse(tokens) {
            Ok(ast) => ast,
            Err(e) => {
                return Err(vec![ApiError::simple(format!("Parse error: {:?}", e))]);
            }
        };

        // 3. Convert OhlcvBar to BarData
        let bar_data: Vec<pine_eval::runner::BarData> = bars
            .iter()
            .map(|b| {
                pine_eval::runner::BarData::new(b.open, b.high, b.low, b.close, b.volume, b.time)
            })
            .collect();

        // 4. Execute based on mode
        let plots = match self.mode {
            ExecutionMode::Vm => self.run_with_vm(&ast, &bar_data, bars)?,
            ExecutionMode::Eval => self.run_with_eval(&ast, &bar_data, bars)?,
        };

        let exec_ms = start.elapsed().as_millis() as u64;

        Ok(ApiResponse::success(exec_ms, plots))
    }

    /// Execute using pine-vm
    fn run_with_vm(
        &self,
        ast: &pine_parser::ast::Script,
        bar_data: &[pine_eval::runner::BarData],
        bars: &[OhlcvBar],
    ) -> Result<Vec<Plot>, Vec<ApiError>> {
        use pine_vm::executor::{execute_script_with_vm, SeriesData as VmSeriesData};

        let series_data = VmSeriesData::new(
            bar_data.iter().map(|b| b.open).collect(),
            bar_data.iter().map(|b| b.high).collect(),
            bar_data.iter().map(|b| b.low).collect(),
            bar_data.iter().map(|b| b.close).collect(),
            bar_data.iter().map(|b| b.volume).collect(),
            bar_data.iter().map(|b| b.time).collect(),
        );

        // Execute with VM
        let result = execute_script_with_vm(ast, &series_data)
            .map_err(|e| vec![ApiError::simple(format!("VM execution error: {:?}", e))])?;

        // Convert to API format
        self.convert_plots_to_api(result.plot_outputs.get_plots(), bars)
    }

    /// Execute using pine-eval
    fn run_with_eval(
        &self,
        ast: &pine_parser::ast::Script,
        bar_data: &[pine_eval::runner::BarData],
        bars: &[OhlcvBar],
    ) -> Result<Vec<Plot>, Vec<ApiError>> {
        // Create evaluation context
        let mut ctx = pine_eval::EvaluationContext::new();

        // Execute bar by bar
        pine_eval::runner::run_bar_by_bar(ast, bar_data, &mut ctx)
            .map_err(|e| vec![ApiError::simple(format!("Eval execution error: {:?}", e))])?;

        // Convert to API format
        self.convert_plots_to_api(ctx.plot_outputs.get_plots(), bars)
    }

    /// Convert PlotOutputs to API Plot format
    fn convert_plots_to_api(
        &self,
        plots_map: &std::collections::HashMap<String, Vec<Option<f64>>>,
        bars: &[OhlcvBar],
    ) -> Result<Vec<Plot>, Vec<ApiError>> {
        let times: Vec<i64> = bars.iter().map(|b| b.time).collect();
        let mut plots = Vec::new();

        for (title, values) in plots_map {
            let plot_data: Vec<PlotData> = times
                .iter()
                .zip(values.iter())
                .map(|(&time, &value)| PlotData { time, value })
                .collect();

            let title = title.clone();
            plots.push(Plot {
                id: title.clone(),
                title: title.clone(),
                plot_type: "line".to_string(),
                color: generate_color(&title),
                linewidth: Some(2.0),
                pane: 0,
                data: plot_data,
            });
        }

        Ok(plots)
    }
}

/// Generate a color for a plot based on its title
fn generate_color(title: &str) -> String {
    // Simple hash-based color generation
    let hash = title
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));

    // Predefined color palette for common indicators
    match title.to_lowercase().as_str() {
        s if s.contains("sma") => "#2196F3".to_string(),
        s if s.contains("ema") => "#FF9800".to_string(),
        s if s.contains("rsi") => "#9C27B0".to_string(),
        s if s.contains("macd") => "#4CAF50".to_string(),
        s if s.contains("signal") => "#F44336".to_string(),
        s if s.contains("histogram") => "#00BCD4".to_string(),
        s if s.contains("bb") || s.contains("band") => "#E91E63".to_string(),
        s if s.contains("upper") => "#FF5722".to_string(),
        s if s.contains("lower") => "#3F51B5".to_string(),
        s if s.contains("close") => "#607D8B".to_string(),
        _ => {
            // Generate color from hash
            let r = ((hash >> 16) & 0xFF) as u8;
            let g = ((hash >> 8) & 0xFF) as u8;
            let b = (hash & 0xFF) as u8;
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        }
    }
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
    fn test_engine_with_mode() {
        let engine_vm = PineEngine::with_mode(ExecutionMode::Vm);
        assert_eq!(engine_vm.mode, ExecutionMode::Vm);

        let engine_eval = PineEngine::with_mode(ExecutionMode::Eval);
        assert_eq!(engine_eval.mode, ExecutionMode::Eval);
    }

    #[test]
    fn test_check_simple_script() {
        let engine = PineEngine::new();

        // Valid script
        let result = engine.check("//@version=6\nindicator(\"Test\")\nplot(close)");
        assert!(result.is_ok());

        // Invalid script
        let result = engine.check("//@version=6\nindicator(\"Test\"\nplot(close)");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_color() {
        assert_eq!(generate_color("SMA 20"), "#2196F3");
        assert_eq!(generate_color("EMA 50"), "#FF9800");
        assert_eq!(generate_color("RSI 14"), "#9C27B0");
        assert_eq!(generate_color("close"), "#607D8B");

        // Unknown indicator should generate hash-based color
        let color = generate_color("custom_indicator");
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }
}
