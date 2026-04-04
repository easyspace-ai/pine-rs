//! Pine Script execution engine wrapper for pine-tv
//! Wraps pine-rs interpreter and manages execution lifecycle.

use std::time::Instant;

use crate::data::OhlcvBar;
use crate::engine::output::{ApiError, ApiResponse, Plot, PlotData, StrategyOutput, TradeSignal};

use pine_lexer::Lexer;
use pine_parser::ast::{Arg, Expr, Lit, Script, Stmt};
use pine_runtime::value::Value;
use pine_stdlib::registry::FunctionRegistry;

/// Script declaration kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptKind {
    /// `indicator(...)`
    Indicator,
    /// `strategy(...)`
    Strategy,
    /// No explicit top-level declaration found.
    Unknown,
}

/// Realtime execution policy for a script session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealtimeExecutionPolicy {
    /// Execute on every forming tick and bar close.
    EveryTick,
    /// Execute only on bar close.
    BarCloseOnly,
}

/// Realtime trigger source for a script execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeExecutionTrigger {
    /// Initial full snapshot / explicit rerun.
    Snapshot,
    /// Forming bar tick update.
    Tick,
    /// Closed bar commit.
    BarClose,
    /// Re-execution after an order fill.
    OrderFill,
}

/// Top-level declaration information extracted from the script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptDeclaration {
    /// Declaration kind.
    pub kind: ScriptKind,
    /// Whether the script overlays on the main pane.
    pub overlay: bool,
    /// Whether strategy requested every-tick execution.
    pub calc_on_every_tick: bool,
    /// Whether strategy requested re-execution on fills.
    pub calc_on_order_fills: bool,
    /// Whether strategy requested processing orders on close.
    pub process_orders_on_close: bool,
}

impl ScriptDeclaration {
    /// Realtime execution policy implied by the declaration.
    pub fn realtime_execution_policy(&self) -> RealtimeExecutionPolicy {
        match self.kind {
            ScriptKind::Indicator => RealtimeExecutionPolicy::EveryTick,
            ScriptKind::Strategy if self.calc_on_every_tick => RealtimeExecutionPolicy::EveryTick,
            ScriptKind::Strategy => RealtimeExecutionPolicy::BarCloseOnly,
            ScriptKind::Unknown => RealtimeExecutionPolicy::EveryTick,
        }
    }

    /// Whether this script should execute for the given realtime trigger.
    pub fn should_execute_on(&self, trigger: RealtimeExecutionTrigger) -> bool {
        match trigger {
            RealtimeExecutionTrigger::Snapshot => true,
            RealtimeExecutionTrigger::Tick => {
                self.realtime_execution_policy() == RealtimeExecutionPolicy::EveryTick
            }
            RealtimeExecutionTrigger::BarClose => true,
            RealtimeExecutionTrigger::OrderFill => {
                self.kind == ScriptKind::Strategy && self.calc_on_order_fills
            }
        }
    }
}

/// Read top-level `indicator(...)` / `strategy(...)` declaration details from the script AST.
fn extract_script_declaration(script: &Script) -> ScriptDeclaration {
    for stmt in &script.stmts {
        let Stmt::Expr(expr) = stmt else {
            continue;
        };
        let Expr::FnCall { func, args, .. } = expr else {
            continue;
        };
        let Expr::Ident(callee) = func.as_ref() else {
            continue;
        };
        match callee.name.as_str() {
            "indicator" | "strategy" => {
                return ScriptDeclaration {
                    kind: if callee.name == "strategy" {
                        ScriptKind::Strategy
                    } else {
                        ScriptKind::Indicator
                    },
                    overlay: overlay_from_decl_args(callee.name.as_str(), args),
                    calc_on_order_fills: calc_on_order_fills_from_decl_args(
                        callee.name.as_str(),
                        args,
                    ),
                    process_orders_on_close: process_orders_on_close_from_decl_args(
                        callee.name.as_str(),
                        args,
                    ),
                    calc_on_every_tick: calc_on_every_tick_from_decl_args(
                        callee.name.as_str(),
                        args,
                    ),
                };
            }
            _ => {}
        }
    }

    ScriptDeclaration {
        kind: ScriptKind::Unknown,
        overlay: true,
        calc_on_every_tick: false,
        calc_on_order_fills: false,
        process_orders_on_close: false,
    }
}

fn overlay_from_decl_args(callee: &str, args: &[Arg]) -> bool {
    for arg in args {
        if arg.name.as_ref().is_some_and(|n| n.name == "overlay") {
            if let Expr::Literal(Lit::Bool(b), _) = &arg.value {
                return *b;
            }
        }
    }
    if args.len() > 2 && args[2].name.is_none() {
        if let Expr::Literal(Lit::Bool(b), _) = &args[2].value {
            return *b;
        }
    }
    match callee {
        "indicator" => false,
        "strategy" => true,
        _ => true,
    }
}

fn calc_on_every_tick_from_decl_args(callee: &str, args: &[Arg]) -> bool {
    if callee != "strategy" {
        return false;
    }

    for arg in args {
        if arg
            .name
            .as_ref()
            .is_some_and(|n| n.name == "calc_on_every_tick")
        {
            if let Expr::Literal(Lit::Bool(b), _) = &arg.value {
                return *b;
            }
        }
    }

    if args.len() > 14 && args[14].name.is_none() {
        if let Expr::Literal(Lit::Bool(b), _) = &args[14].value {
            return *b;
        }
    }

    false
}

fn calc_on_order_fills_from_decl_args(callee: &str, args: &[Arg]) -> bool {
    if callee != "strategy" {
        return false;
    }

    for arg in args {
        if arg
            .name
            .as_ref()
            .is_some_and(|n| n.name == "calc_on_order_fills")
        {
            if let Expr::Literal(Lit::Bool(b), _) = &arg.value {
                return *b;
            }
        }
    }

    if args.len() > 13 && args[13].name.is_none() {
        if let Expr::Literal(Lit::Bool(b), _) = &args[13].value {
            return *b;
        }
    }

    false
}

fn process_orders_on_close_from_decl_args(callee: &str, args: &[Arg]) -> bool {
    if callee != "strategy" {
        return false;
    }

    for arg in args {
        if arg
            .name
            .as_ref()
            .is_some_and(|n| n.name == "process_orders_on_close")
        {
            if let Expr::Literal(Lit::Bool(b), _) = &arg.value {
                return *b;
            }
        }
    }

    if args.len() > 15 && args[15].name.is_none() {
        if let Expr::Literal(Lit::Bool(b), _) = &args[15].value {
            return *b;
        }
    }

    false
}

/// Execution mode for PineEngine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Use pine-vm (bytecode VM)
    Vm,
    /// Use pine-eval (tree-walking interpreter)
    Eval,
}

impl ExecutionMode {
    /// Get execution mode from environment variable `PINE_TV_MODE`.
    ///
    /// - Default **`eval`**: pine-tv `/api/run` uses `pine-eval` + `run_bar_by_bar` so JSON
    ///   `plots` match interpreter semantics.
    /// - Set `PINE_TV_MODE=vm` to use the bytecode VM instead.
    pub fn from_env() -> Self {
        match std::env::var("PINE_TV_MODE").as_deref() {
            Ok("vm") => Self::Vm,
            _ => Self::Eval,
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

    /// Active backend (`eval` or `vm`).
    pub fn execution_mode(&self) -> ExecutionMode {
        self.mode
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

    /// Inspect the top-level declaration without executing the script.
    pub fn inspect_script(&self, code: &str) -> Result<ScriptDeclaration, Vec<ApiError>> {
        let tokens = match Lexer::lex_with_indentation(code) {
            Ok(t) => t,
            Err(e) => {
                return Err(vec![ApiError::simple(format!("Lex error: {:?}", e))]);
            }
        };

        let ast = match pine_parser::parser::parse(tokens) {
            Ok(ast) => ast,
            Err(e) => {
                return Err(vec![ApiError::simple(format!("Parse error: {:?}", e))]);
            }
        };

        Ok(extract_script_declaration(&ast))
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

        let declaration = extract_script_declaration(&ast);
        let overlay = declaration.overlay;
        // 4. Execute based on mode
        let (plots, strategy) = match self.mode {
            ExecutionMode::Vm => {
                let plots = self.run_with_vm(&ast, &bar_data, bars, overlay)?;
                (plots, None) // VM mode doesn't support strategy signals yet
            }
            ExecutionMode::Eval => self.run_with_eval(&ast, &bar_data, bars, overlay)?,
        };

        let exec_ms = start.elapsed().as_millis() as u64;

        // Return response with or without strategy signals
        if let Some(strategy_output) = strategy {
            Ok(ApiResponse::success_with_strategy(
                exec_ms,
                plots,
                strategy_output,
            ))
        } else {
            Ok(ApiResponse::success(exec_ms, plots))
        }
    }

    /// Execute using pine-vm
    fn run_with_vm(
        &self,
        ast: &pine_parser::ast::Script,
        bar_data: &[pine_eval::runner::BarData],
        bars: &[OhlcvBar],
        overlay: bool,
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
        self.convert_plots_to_api(result.plot_outputs.get_plots(), bars, overlay)
    }

    /// Execute using pine-eval
    fn run_with_eval(
        &self,
        ast: &pine_parser::ast::Script,
        bar_data: &[pine_eval::runner::BarData],
        bars: &[OhlcvBar],
        overlay: bool,
    ) -> Result<(Vec<Plot>, Option<StrategyOutput>), Vec<ApiError>> {
        // Create evaluation context
        let mut ctx = pine_eval::EvaluationContext::new();

        // Execute bar by bar
        pine_eval::runner::run_bar_by_bar(ast, bar_data, &mut ctx)
            .map_err(|e| vec![ApiError::simple(format!("Eval execution error: {:?}", e))])?;

        // Convert plots to API format
        let plots = self.convert_plots_to_api(ctx.plot_outputs.get_plots(), bars, overlay)?;

        // Convert strategy signals to API format
        let strategy_output = self.convert_strategy_signals(&ctx, bars);

        Ok((plots, strategy_output))
    }

    /// Convert PlotOutputs to API Plot format
    fn convert_plots_to_api(
        &self,
        plots_map: &std::collections::HashMap<String, Vec<Option<f64>>>,
        bars: &[OhlcvBar],
        overlay: bool,
    ) -> Result<Vec<Plot>, Vec<ApiError>> {
        let times: Vec<i64> = bars.iter().map(|b| b.time).collect();
        let mut plots = Vec::new();
        let pane = if overlay { 0 } else { 1 };

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
                pane,
                data: plot_data,
            });
        }

        Ok(plots)
    }

    /// Convert strategy signals to API format
    fn convert_strategy_signals(
        &self,
        ctx: &pine_eval::EvaluationContext,
        bars: &[OhlcvBar],
    ) -> Option<StrategyOutput> {
        let signals = ctx.strategy_signals.get_signals();
        if signals.is_empty() {
            return None;
        }

        // Try to extract strategy name from the script
        let strategy_name = self.extract_strategy_name(ctx);

        let mut entries = Vec::new();
        let mut exits = Vec::new();

        for signal in signals {
            let bar_idx = signal.bar_index;
            let time = bars.get(bar_idx).map(|b| b.time).unwrap_or(0);

            let trade_signal = TradeSignal {
                bar_index: bar_idx,
                time,
                signal_type: signal.signal_type.clone(),
                id: signal.id.clone(),
                direction: signal.direction.clone(),
                qty: signal.qty,
                price: signal.price,
                comment: signal.comment.clone(),
            };

            match signal.signal_type.as_str() {
                "entry" => entries.push(trade_signal),
                "exit" | "close" => exits.push(trade_signal),
                _ => {}
            }
        }

        // Determine position direction
        let position_direction = if ctx.get_var("strategy.position_size").is_some() {
            // Try to get actual position size from runtime
            match ctx.get_var("strategy.position_size") {
                Some(Value::Float(size)) if *size > 0.0 => "long",
                Some(Value::Float(size)) if *size < 0.0 => "short",
                Some(Value::Int(size)) if *size > 0 => "long",
                Some(Value::Int(size)) if *size < 0 => "short",
                _ => "none",
            }
        } else {
            // Infer from signals
            let entry_count = entries.len();
            let exit_count = exits.len();
            if entry_count > exit_count {
                "long"
            } else {
                "none"
            }
        };

        // Calculate position size (simplified)
        let position_size = entries.len().saturating_sub(exits.len()) as f64;

        Some(StrategyOutput {
            name: strategy_name,
            entries,
            exits,
            position_size,
            position_direction: position_direction.to_string(),
        })
    }

    /// Extract strategy name from context
    fn extract_strategy_name(&self, ctx: &pine_eval::EvaluationContext) -> String {
        // Look for strategy variable
        if let Some(Value::Array(arr)) = ctx.get_var("strategy") {
            if arr.len() >= 2 {
                if let Some(Value::String(name)) = arr.get(1) {
                    return name.to_string();
                }
            }
        }
        "Strategy".to_string()
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
        let engine = PineEngine::new();
        let _ = engine.execution_mode();
    }

    #[test]
    fn test_engine_with_mode() {
        let engine_vm = PineEngine::with_mode(ExecutionMode::Vm);
        assert_eq!(engine_vm.execution_mode(), ExecutionMode::Vm);

        let engine_eval = PineEngine::with_mode(ExecutionMode::Eval);
        assert_eq!(engine_eval.execution_mode(), ExecutionMode::Eval);
    }

    #[test]
    fn test_eval_run_produces_sma_plot_series() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let bars: Vec<OhlcvBar> = (0_i64..40)
            .map(|i| {
                let c = 100.0 + i as f64 * 0.1;
                OhlcvBar::new(i, c, c + 0.5, c - 0.5, c, 1000.0)
            })
            .collect();
        let code = r#"//@version=6
indicator("SMA test")
plot(ta.sma(close, 5), title="SMA 5", color=#2196F3)
"#;
        let res = engine.run(code, &bars).expect("run");
        assert!(res.ok, "{:?}", res.errors);
        let plots = res.plots.expect("plots");
        assert_eq!(plots.len(), 1);
        assert_eq!(plots[0].title, "SMA 5");
        let non_na = plots[0].data.iter().filter(|p| p.value.is_some()).count();
        assert!(
            non_na >= 35,
            "expected most bars to have SMA, got {} non-na points",
            non_na
        );
        assert_eq!(
            plots[0].pane, 1,
            "default indicator() uses overlay=false → separate pane"
        );
    }

    #[test]
    fn test_indicator_overlay_true_is_main_pane() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let bars: Vec<OhlcvBar> = (0_i64..30)
            .map(|i| {
                let c = 100.0 + i as f64 * 0.1;
                OhlcvBar::new(i, c, c + 0.5, c - 0.5, c, 1000.0)
            })
            .collect();
        let code = r#"//@version=6
indicator("SMA overlay", overlay=true)
plot(ta.sma(close, 5), title="SMA 5", color=#2196F3)
"#;
        let res = engine.run(code, &bars).expect("run");
        assert!(res.ok);
        let plots = res.plots.expect("plots");
        assert_eq!(plots.len(), 1);
        assert_eq!(plots[0].pane, 0);
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

    #[test]
    fn test_strategy_signals_output() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let bars: Vec<OhlcvBar> = (0_i64..50)
            .map(|i| {
                let c = 100.0 + (i as f64 * 0.5).sin() * 10.0;
                OhlcvBar::new(i * 3600, c, c + 1.0, c - 1.0, c, 1000.0)
            })
            .collect();

        let code = r#"//@version=6
strategy("Test Strategy", overlay=true)
sma = ta.sma(close, 14)
longCondition = ta.crossover(close, sma)
if longCondition
    strategy.entry("Long", strategy.long)
shortCondition = ta.crossunder(close, sma)
if shortCondition
    strategy.close("Long")
plot(sma, title="SMA", color=color.blue)
"#;

        let res = engine.run(code, &bars).expect("run");
        assert!(res.ok, "{:?}", res.errors);

        // Check that strategy output is present
        let strategy = res.strategy.expect("strategy output should be present");
        // Strategy name defaults to "Strategy" since the namespace variable
        // overwrites the strategy() call result
        assert!(
            !strategy.name.is_empty(),
            "Strategy name should not be empty"
        );

        // Verify that we have some signals (the exact count depends on the data pattern)
        let total_signals = strategy.entries.len() + strategy.exits.len();
        assert!(
            total_signals > 0,
            "Expected some strategy signals, got {} entries and {} exits",
            strategy.entries.len(),
            strategy.exits.len()
        );
    }

    #[test]
    fn test_strategy_no_signals_for_indicator() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let bars: Vec<OhlcvBar> = (0_i64..20)
            .map(|i| {
                let c = 100.0 + i as f64 * 0.1;
                OhlcvBar::new(i, c, c + 0.5, c - 0.5, c, 1000.0)
            })
            .collect();

        // This is an indicator script (not a strategy), so no strategy signals
        let code = r#"//@version=6
indicator("SMA Indicator")
plot(ta.sma(close, 5), title="SMA")
"#;

        let res = engine.run(code, &bars).expect("run");
        assert!(res.ok, "{:?}", res.errors);

        // No strategy output for indicator scripts
        assert!(
            res.strategy.is_none(),
            "Indicator scripts should not have strategy output"
        );
    }

    #[test]
    fn test_inspect_indicator_defaults_to_every_tick() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let info = engine
            .inspect_script("//@version=6\nindicator(\"x\")\nplot(close)")
            .expect("inspect");
        assert_eq!(info.kind, ScriptKind::Indicator);
        assert_eq!(info.overlay, false);
        assert_eq!(
            info.realtime_execution_policy(),
            RealtimeExecutionPolicy::EveryTick
        );
    }

    #[test]
    fn test_inspect_strategy_defaults_to_bar_close_only() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let info = engine
            .inspect_script("//@version=6\nstrategy(\"x\")\nplot(close)")
            .expect("inspect");
        assert_eq!(info.kind, ScriptKind::Strategy);
        assert!(!info.calc_on_every_tick);
        assert_eq!(
            info.realtime_execution_policy(),
            RealtimeExecutionPolicy::BarCloseOnly
        );
    }

    #[test]
    fn test_inspect_strategy_calc_on_every_tick_named_arg() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let info = engine
            .inspect_script("//@version=6\nstrategy(\"x\", calc_on_every_tick=true)\nplot(close)")
            .expect("inspect");
        assert_eq!(info.kind, ScriptKind::Strategy);
        assert!(info.calc_on_every_tick);
        assert_eq!(
            info.realtime_execution_policy(),
            RealtimeExecutionPolicy::EveryTick
        );
    }

    #[test]
    fn test_inspect_strategy_calc_on_order_fills_named_arg() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let info = engine
            .inspect_script("//@version=6\nstrategy(\"x\", calc_on_order_fills=true)\nplot(close)")
            .expect("inspect");
        assert_eq!(info.kind, ScriptKind::Strategy);
        assert!(info.calc_on_order_fills);
        assert!(info.should_execute_on(RealtimeExecutionTrigger::OrderFill));
    }

    #[test]
    fn test_inspect_strategy_process_orders_on_close_named_arg() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let info = engine
            .inspect_script(
                "//@version=6\nstrategy(\"x\", process_orders_on_close=true)\nplot(close)",
            )
            .expect("inspect");
        assert_eq!(info.kind, ScriptKind::Strategy);
        assert!(info.process_orders_on_close);
    }

    #[test]
    fn test_strategy_bar_close_policy_skips_tick_trigger() {
        let engine = PineEngine::with_mode(ExecutionMode::Eval);
        let info = engine
            .inspect_script("//@version=6\nstrategy(\"x\")\nplot(close)")
            .expect("inspect");

        assert!(!info.should_execute_on(RealtimeExecutionTrigger::Tick));
        assert!(info.should_execute_on(RealtimeExecutionTrigger::BarClose));
        assert!(!info.should_execute_on(RealtimeExecutionTrigger::OrderFill));
    }
}
