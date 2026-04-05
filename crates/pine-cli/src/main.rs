//! Pine Script v6 CLI
//!
//! Command-line interface for the Pine Script interpreter.

use clap::{Parser, Subcommand, ValueEnum};
use miette::{miette, IntoDiagnostic, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Data feed trait for loading market data
trait DataFeed {
    /// Load OHLCV data from a source
    fn load(&self) -> Result<Vec<OHLCV>>;
}

/// OHLCV data point (Open, High, Low, Close, Volume)
#[derive(Debug, Clone, Serialize)]
#[allow(clippy::upper_case_acronyms)]
struct OHLCV {
    /// Timestamp (milliseconds since epoch)
    pub time: i64,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
}

/// CSV data feed implementation
struct CsvDataFeed {
    path: String,
}

impl CsvDataFeed {
    /// Create a new CSV data feed
    fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl DataFeed for CsvDataFeed {
    fn load(&self) -> Result<Vec<OHLCV>> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Err(miette!("Data file not found: {}", self.path));
        }

        let file = fs::File::open(path).into_diagnostic()?;
        let mut rdr = csv::Reader::from_reader(file);

        let mut data = Vec::new();

        for result in rdr.records() {
            let record = result.into_diagnostic()?;

            // Parse CSV record - supports multiple formats:
            // Format 1: time,open,high,low,close,volume (standard OHLCV)
            // Format 2: timestamp,close (close only)
            let ohlcv = if record.len() >= 6 {
                OHLCV {
                    time: parse_timestamp(&record[0])?,
                    open: record[1].parse().into_diagnostic()?,
                    high: record[2].parse().into_diagnostic()?,
                    low: record[3].parse().into_diagnostic()?,
                    close: record[4].parse().into_diagnostic()?,
                    volume: record[5].parse().into_diagnostic()?,
                }
            } else if record.len() >= 2 {
                // Close-only format - use close for all price fields
                let close: f64 = record[1].parse().into_diagnostic()?;
                OHLCV {
                    time: parse_timestamp(&record[0])?,
                    open: close,
                    high: close,
                    low: close,
                    close,
                    volume: 0.0,
                }
            } else {
                return Err(miette!(
                    "Invalid CSV format: expected at least 2 columns, got {}",
                    record.len()
                ));
            };

            data.push(ohlcv);
        }

        // Sort by timestamp (oldest first)
        data.sort_by_key(|d| d.time);

        Ok(data)
    }
}

/// Parse timestamp from string (supports Unix timestamp in ms or ISO 8601)
fn parse_timestamp(s: &str) -> Result<i64> {
    // Try parsing as integer (Unix timestamp in milliseconds)
    if let Ok(ts) = s.parse::<i64>() {
        // If it's a small number, it's likely seconds, convert to milliseconds
        if ts < 1_000_000_000_000 {
            return Ok(ts * 1000);
        }
        return Ok(ts);
    }

    // Try parsing as ISO 8601 date string
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp_millis());
    }

    // Try other common date formats
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt.and_utc().timestamp_millis());
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis());
    }

    Err(miette!("Cannot parse timestamp: {}", s))
}

/// Script execution result for JSON output
#[derive(Serialize)]
struct ExecutionResult {
    /// Whether execution was successful
    success: bool,
    /// Output values (plot values, etc.)
    outputs: HashMap<String, Vec<f64>>,
    /// Plots indexed by bar index (for golden test comparison)
    #[serde(skip_serializing_if = "Option::is_none")]
    plots: Option<HashMap<String, Option<f64>>>,
    /// Strategy signals (entries and exits)
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<StrategyResult>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    /// Number of bars processed
    bars_processed: usize,
}

/// Strategy execution result
#[derive(Serialize)]
struct StrategyResult {
    /// Strategy name
    name: String,
    /// Entry signals (bar_index, direction, qty, price)
    entries: Vec<Signal>,
    /// Exit signals (bar_index, direction, qty, price)
    exits: Vec<Signal>,
    /// Final position size
    position_size: f64,
    /// Final position direction ("long", "short", "none")
    position_direction: String,
}

/// Trade signal
#[derive(Serialize)]
struct Signal {
    /// Bar index where signal occurred
    bar_index: usize,
    /// Signal direction ("long", "short", "close")
    direction: String,
    /// Quantity
    qty: f64,
    /// Price (optional)
    price: Option<f64>,
    /// Signal comment/label
    comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliExecutionEngine {
    Auto,
    Eval,
    Vm,
}

impl CliExecutionEngine {
    fn from_env() -> Self {
        match std::env::var("PINE_CLI_ENGINE").as_deref() {
            Ok("eval") => Self::Eval,
            Ok("vm") => Self::Vm,
            _ => Self::Auto,
        }
    }
}

#[derive(Parser)]
#[command(name = "pine")]
#[command(about = "Pine Script v6 interpreter")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Pine Script file
    Run {
        /// Path to the Pine Script file
        script: String,
        /// Path to the data CSV file
        #[arg(short, long)]
        data: Option<String>,
        /// Output format (json or text)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Execution engine (auto, eval, vm)
        #[arg(long, value_enum)]
        engine: Option<CliExecutionEngine>,
    },
    /// Check a Pine Script file for errors
    Check {
        /// Path to the Pine Script file
        script: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            script,
            data,
            format,
            engine,
        } => {
            // Load script
            let script_path = Path::new(&script);
            if !script_path.exists() {
                return Err(miette!("Script file not found: {}", script));
            }

            let script_content = fs::read_to_string(script_path).into_diagnostic()?;

            // Load data if provided
            let data_feed: Option<Box<dyn DataFeed>> = data.as_ref().map(|d| {
                let feed: Box<dyn DataFeed> = Box::new(CsvDataFeed::new(d));
                feed
            });

            let ohlcv_data = if let Some(feed) = data_feed {
                Some(feed.load()?)
            } else {
                None
            };

            let engine = engine.unwrap_or_else(CliExecutionEngine::from_env);

            let result = execute_script(&script_content, ohlcv_data.as_deref(), engine)?;

            // Output result
            match format.as_str() {
                "json" => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&result).into_diagnostic()?
                    );
                }
                _ => {
                    if result.success {
                        println!("Script executed successfully");
                        println!("Bars processed: {}", result.bars_processed);
                        for (name, values) in result.outputs {
                            println!("{}: {:?}", name, values);
                        }
                    } else {
                        println!("Error: {}", result.error.unwrap_or_default());
                    }
                }
            }

            Ok(())
        }
        Commands::Check { script } => check_script(&script),
    }
}

/// Convert OHLCV data to BarData for pine-eval
fn convert_to_bar_data(data: &[OHLCV]) -> Vec<pine_eval::runner::BarData> {
    data.iter()
        .map(|d| pine_eval::runner::BarData::new(d.open, d.high, d.low, d.close, d.volume, d.time))
        .collect()
}

fn convert_to_series_data(data: &[OHLCV]) -> pine_vm::executor::SeriesData {
    let open = data.iter().map(|d| d.open).collect();
    let high = data.iter().map(|d| d.high).collect();
    let low = data.iter().map(|d| d.low).collect();
    let close = data.iter().map(|d| d.close).collect();
    let volume = data.iter().map(|d| d.volume).collect();
    let time = data.iter().map(|d| d.time).collect();
    pine_vm::executor::SeriesData::new(open, high, low, close, volume, time)
}

fn parse_script(script: &str) -> Result<pine_parser::ast::Script> {
    let tokens = pine_lexer::Lexer::lex_with_indentation(script)
        .map_err(|e| miette!("Lexical error: {:?}", e))?;
    pine_parser::parser::parse(tokens).map_err(|e| miette!("Parse error: {:?}", e))
}

fn collect_plot_outputs(
    plot_map: &HashMap<String, Vec<Option<f64>>>,
) -> (HashMap<String, Vec<f64>>, HashMap<String, Option<f64>>) {
    let mut outputs = HashMap::new();
    let mut plots = HashMap::new();

    for (title, values) in plot_map {
        let plot_values: Vec<f64> = values
            .iter()
            .map(|v| match v {
                Some(f) => *f,
                None => f64::NAN,
            })
            .collect();

        outputs.insert(title.clone(), plot_values);
        plots.insert(title.clone(), values.last().copied().flatten());
    }

    (outputs, plots)
}

fn strategy_result_from_ctx(ctx: &pine_eval::EvaluationContext) -> StrategyResult {
    let entries: Vec<Signal> = ctx
        .strategy_signals
        .get_entries()
        .iter()
        .map(|s| Signal {
            bar_index: s.bar_index,
            direction: s.direction.clone(),
            qty: s.qty,
            price: s.price,
            comment: s.comment.clone(),
        })
        .collect();

    let exits: Vec<Signal> = ctx
        .strategy_signals
        .get_exits()
        .iter()
        .map(|s| Signal {
            bar_index: s.bar_index,
            direction: if s.signal_type == "close" {
                "close".to_string()
            } else {
                s.direction.clone()
            },
            qty: s.qty,
            price: s.price,
            comment: s.comment.clone(),
        })
        .collect();

    let final_position = entries.len() as f64 - exits.len() as f64;
    let position_direction = if final_position > 0.0 {
        "long"
    } else if final_position < 0.0 {
        "short"
    } else {
        "none"
    };

    StrategyResult {
        name: "Pine Strategy".to_string(),
        entries,
        exits,
        position_size: final_position.abs(),
        position_direction: position_direction.to_string(),
    }
}

fn execute_with_eval(
    ast: &pine_parser::ast::Script,
    data: &[OHLCV],
    is_strategy: bool,
) -> Result<ExecutionResult> {
    let bar_data = convert_to_bar_data(data);
    let mut ctx = pine_eval::EvaluationContext::new();

    if let Err(e) = pine_eval::runner::run_bar_by_bar(ast, &bar_data, &mut ctx) {
        return Ok(ExecutionResult {
            success: false,
            outputs: HashMap::new(),
            plots: None,
            strategy: None,
            error: Some(format!("Execution error: {:?}", e)),
            bars_processed: 0,
        });
    }

    let (outputs, plots) = collect_plot_outputs(ctx.plot_outputs.get_plots());

    Ok(ExecutionResult {
        success: true,
        outputs,
        plots: if plots.is_empty() { None } else { Some(plots) },
        strategy: is_strategy.then(|| strategy_result_from_ctx(&ctx)),
        error: None,
        bars_processed: data.len(),
    })
}

fn execute_with_vm(ast: &pine_parser::ast::Script, data: &[OHLCV]) -> Result<ExecutionResult> {
    let series_data = convert_to_series_data(data);
    let vm_result = pine_vm::executor::execute_script_with_vm(ast, &series_data)
        .map_err(|e| miette!("VM execution error: {:?}", e))?;

    let (outputs, plots) = collect_plot_outputs(vm_result.plot_outputs.get_plots());

    Ok(ExecutionResult {
        success: vm_result.success,
        outputs,
        plots: if plots.is_empty() { None } else { Some(plots) },
        strategy: None,
        error: vm_result.error,
        bars_processed: vm_result.bars_processed,
    })
}

/// Execute a Pine Script using pine-eval
fn execute_script(
    script: &str,
    data: Option<&[OHLCV]>,
    engine: CliExecutionEngine,
) -> Result<ExecutionResult> {
    let ast = parse_script(script)?;
    let bars_processed = data.map(|d| d.len()).unwrap_or(0);
    let is_strategy = script.contains("strategy(");

    if let Some(ohlcv_data) = data {
        return match engine {
            CliExecutionEngine::Eval => execute_with_eval(&ast, ohlcv_data, is_strategy),
            CliExecutionEngine::Vm => {
                if is_strategy {
                    Ok(ExecutionResult {
                        success: false,
                        outputs: HashMap::new(),
                        plots: None,
                        strategy: None,
                        error: Some(
                            "VM mode does not yet support strategy outputs in pine-cli".to_string(),
                        ),
                        bars_processed: 0,
                    })
                } else {
                    execute_with_vm(&ast, ohlcv_data)
                }
            }
            CliExecutionEngine::Auto => {
                if is_strategy {
                    execute_with_eval(&ast, ohlcv_data, true)
                } else {
                    match execute_with_vm(&ast, ohlcv_data) {
                        Ok(result) => Ok(result),
                        Err(_) => execute_with_eval(&ast, ohlcv_data, false),
                    }
                }
            }
        };
    }

    Ok(ExecutionResult {
        success: true,
        outputs: HashMap::new(),
        plots: None,
        strategy: None,
        error: None,
        bars_processed,
    })
}

/// Check a Pine Script file for syntax errors
fn check_script(script_path: &str) -> Result<()> {
    let path = Path::new(script_path);
    if !path.exists() {
        return Err(miette!("Script file not found: {}", script_path));
    }

    let content = fs::read_to_string(path).into_diagnostic()?;

    // Lexical analysis - use lex_with_indentation to get INDENT/DEDENT tokens
    let tokens = pine_lexer::Lexer::lex_with_indentation(&content)
        .map_err(|e| miette!("Lexical error: {:?}", e))?;

    // Parse
    let ast = pine_parser::parser::parse(tokens).map_err(|e| miette!("Parse error: {:?}", e))?;

    println!("✓ Script is syntactically correct");
    println!("  Statements: {}", ast.stmts.len());

    // Semantic analysis (if available)
    // This would use pine-sema when fully implemented

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::path::PathBuf;

    #[derive(Debug, Deserialize)]
    struct VmParityCase {
        script_path: String,
        golden_path: String,
    }

    fn with_temp_env_var<T>(key: &str, value: Option<&str>, f: impl FnOnce() -> T) -> T {
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }
        let result = f();
        match previous.as_deref() {
            Some(previous) => unsafe { std::env::set_var(key, previous) },
            None => unsafe { std::env::remove_var(key) },
        }
        result
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn load_vm_parity_cases() -> Vec<VmParityCase> {
        let root = workspace_root();
        let content = fs::read_to_string(root.join("tests/vm_parity_cases.json"))
            .expect("read VM parity manifest");
        serde_json::from_str(&content).expect("parse VM parity manifest")
    }

    #[test]
    fn test_parse_timestamp() {
        // Unix timestamp in milliseconds
        assert_eq!(parse_timestamp("1609459200000").unwrap(), 1609459200000);

        // Unix timestamp in seconds (should be converted)
        assert_eq!(parse_timestamp("1609459200").unwrap(), 1609459200000);

        // ISO 8601
        assert!(parse_timestamp("2021-01-01T00:00:00Z").is_ok());

        // Date only
        assert!(parse_timestamp("2021-01-01").is_ok());
    }

    #[test]
    fn test_csv_data_feed() {
        // Create a temporary CSV file for testing
        let csv_content = "time,open,high,low,close,volume\n\
                          1609459200000,100.0,110.0,95.0,105.0,1000.0\n\
                          1609545600000,105.0,115.0,100.0,110.0,1500.0";

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_data.csv");
        fs::write(&temp_file, csv_content).unwrap();

        let feed = CsvDataFeed::new(temp_file.to_str().unwrap());
        let data = feed.load().unwrap();

        assert_eq!(data.len(), 2);
        assert_eq!(data[0].close, 105.0);
        assert_eq!(data[1].close, 110.0);

        // Cleanup
        fs::remove_file(&temp_file).ok();
    }

    fn sample_data() -> Vec<OHLCV> {
        vec![
            OHLCV {
                time: 1,
                open: 10.0,
                high: 12.0,
                low: 9.0,
                close: 11.0,
                volume: 100.0,
            },
            OHLCV {
                time: 2,
                open: 11.0,
                high: 13.0,
                low: 10.0,
                close: 12.0,
                volume: 110.0,
            },
            OHLCV {
                time: 3,
                open: 12.0,
                high: 14.0,
                low: 11.0,
                close: 13.0,
                volume: 120.0,
            },
        ]
    }

    fn assert_float_series_match(left: &[f64], right: &[f64], label: &str) {
        assert_eq!(left.len(), right.len(), "length mismatch for {label}");
        for (idx, (lhs, rhs)) in left.iter().zip(right.iter()).enumerate() {
            if lhs.is_nan() && rhs.is_nan() {
                continue;
            }
            assert!(
                (lhs - rhs).abs() <= 1e-8,
                "value mismatch for {label} at bar {idx}: left={lhs}, right={rhs}"
            );
        }
    }

    fn assert_output_maps_match(
        left: &HashMap<String, Vec<f64>>,
        right: &HashMap<String, Vec<f64>>,
        label: &str,
    ) {
        let mut left_keys: Vec<_> = left.keys().cloned().collect();
        let mut right_keys: Vec<_> = right.keys().cloned().collect();
        left_keys.sort();
        right_keys.sort();
        assert_eq!(left_keys, right_keys, "output keys mismatch for {label}");

        for key in left_keys {
            assert_float_series_match(&left[&key], &right[&key], &format!("{label}:{key}"));
        }
    }

    #[test]
    fn test_cli_engine_from_env() {
        with_temp_env_var("PINE_CLI_ENGINE", Some("vm"), || {
            assert_eq!(CliExecutionEngine::from_env(), CliExecutionEngine::Vm);
        });
        with_temp_env_var("PINE_CLI_ENGINE", Some("eval"), || {
            assert_eq!(CliExecutionEngine::from_env(), CliExecutionEngine::Eval);
        });
        with_temp_env_var("PINE_CLI_ENGINE", None, || {
            assert_eq!(CliExecutionEngine::from_env(), CliExecutionEngine::Auto);
        });
    }

    #[test]
    fn test_execute_script_vm_matches_eval_for_indicator() {
        let script = r#"
indicator("VM parity")
plot(close, "Close")
"#;
        let data = sample_data();

        let eval_result = execute_script(script, Some(&data), CliExecutionEngine::Eval).unwrap();
        let vm_result = execute_script(script, Some(&data), CliExecutionEngine::Vm).unwrap();
        let auto_result = execute_script(script, Some(&data), CliExecutionEngine::Auto).unwrap();

        assert!(eval_result.success);
        assert!(vm_result.success);
        assert!(auto_result.success);
        assert_eq!(eval_result.outputs, vm_result.outputs);
        assert_eq!(vm_result.outputs, auto_result.outputs);
        assert_eq!(eval_result.plots, vm_result.plots);
    }

    #[test]
    fn test_execute_script_auto_falls_back_to_eval_for_strategy() {
        let script = r#"
strategy("Fallback strategy")
if close > open
    strategy.entry("L", strategy.long)
"#;
        let data = sample_data();

        let auto_result = execute_script(script, Some(&data), CliExecutionEngine::Auto).unwrap();
        let vm_result = execute_script(script, Some(&data), CliExecutionEngine::Vm).unwrap();

        assert!(auto_result.success);
        assert!(auto_result.strategy.is_some());
        assert!(!vm_result.success);
        assert!(vm_result.error.is_some());
    }

    #[test]
    fn test_execute_script_vm_matches_eval_for_regression_scripts() {
        let root = workspace_root();
        let cases = load_vm_parity_cases();
        assert_eq!(cases.len(), 59, "unexpected VM parity manifest size");

        for case in cases {
            let script = fs::read_to_string(root.join(&case.script_path)).expect("read script");
            let feed = CsvDataFeed::new(root.join(&case.golden_path).display().to_string());
            let data = feed.load().expect("load golden csv as input");

            let eval_result =
                execute_script(&script, Some(&data), CliExecutionEngine::Eval).expect("eval run");
            let vm_result =
                execute_script(&script, Some(&data), CliExecutionEngine::Vm).expect("vm run");

            assert!(eval_result.success, "eval failed for {}", case.script_path);
            assert!(vm_result.success, "vm failed for {}", case.script_path);
            assert_output_maps_match(&eval_result.outputs, &vm_result.outputs, &case.script_path);
            assert_eq!(
                eval_result.plots, vm_result.plots,
                "plot mismatch for {}",
                case.script_path
            );
            assert_eq!(
                eval_result.bars_processed, vm_result.bars_processed,
                "bars mismatch for {}",
                case.script_path
            );
        }
    }
}
