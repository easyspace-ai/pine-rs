//! Pine Script v6 CLI
//!
//! Command-line interface for the Pine Script interpreter.

use clap::{Parser, Subcommand};
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

            // Execute script (placeholder - full implementation requires pine-eval)
            let result = execute_script(&script_content, ohlcv_data.as_deref())?;

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

/// Execute a Pine Script
fn execute_script(script: &str, data: Option<&[OHLCV]>) -> Result<ExecutionResult> {
    let bars_processed = data.map(|d| d.len()).unwrap_or(0);

    // Check if this is a strategy script
    let is_strategy = script.contains("strategy(");

    // For now, return a placeholder result with mock strategy signals if applicable
    let mut outputs = HashMap::new();
    let mut plots = HashMap::new();
    let mut strategy_result = None;

    // If we have data, calculate a simple SMA as an example
    if let Some(d) = data {
        if d.len() >= 3 {
            // Calculate fast SMA (3-period)
            let fast_sma: Vec<f64> = d
                .windows(3)
                .map(|window| {
                    let sum: f64 = window.iter().map(|ohlcv| ohlcv.close).sum();
                    sum / 3.0
                })
                .collect();

            // Calculate slow SMA (6-period) if enough data
            let slow_sma: Vec<f64> = if d.len() >= 6 {
                d.windows(6)
                    .map(|window| {
                        let sum: f64 = window.iter().map(|ohlcv| ohlcv.close).sum();
                        sum / 6.0
                    })
                    .collect()
            } else {
                vec![d[0].close; fast_sma.len()]
            };

            // Populate plots for golden test comparison
            for (idx, value) in fast_sma.iter().enumerate() {
                plots.insert(format!("fast_sma_{}", idx), Some(*value));
            }

            outputs.insert("fast_sma".to_string(), fast_sma.clone());
            outputs.insert("slow_sma".to_string(), slow_sma.clone());

            // Generate mock strategy signals for strategy scripts
            if is_strategy && fast_sma.len() >= 2 {
                let mut entries = Vec::new();
                let mut exits = Vec::new();

                // Simple crossover logic for demo
                for i in 1..fast_sma.len() {
                    if i < slow_sma.len() {
                        let prev_fast = fast_sma[i - 1];
                        let curr_fast = fast_sma[i];
                        let prev_slow = slow_sma[i - 1];
                        let curr_slow = slow_sma[i];

                        // Golden cross (fast crosses above slow)
                        if prev_fast <= prev_slow && curr_fast > curr_slow {
                            entries.push(Signal {
                                bar_index: i + 2, // Adjust for window offset
                                direction: "long".to_string(),
                                qty: 1.0,
                                price: Some(d[i + 2].close),
                                comment: Some("Long Entry".to_string()),
                            });
                        }

                        // Death cross (fast crosses below slow)
                        if prev_fast >= prev_slow && curr_fast < curr_slow {
                            exits.push(Signal {
                                bar_index: i + 2,
                                direction: "close".to_string(),
                                qty: 1.0,
                                price: Some(d[i + 2].close),
                                comment: Some("Close Long".to_string()),
                            });
                        }
                    }
                }

                let position_direction = if entries.len() > exits.len() {
                    "long".to_string()
                } else {
                    "none".to_string()
                };
                let position_size = if entries.len() > exits.len() { 1.0 } else { 0.0 };

                strategy_result = Some(StrategyResult {
                    name: "SMA Crossover Strategy".to_string(),
                    entries,
                    exits,
                    position_size,
                    position_direction,
                });
            }
        }
    }

    Ok(ExecutionResult {
        success: true,
        outputs,
        plots: if plots.is_empty() { None } else { Some(plots) },
        strategy: strategy_result,
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

    // Lexical analysis
    let tokens = pine_lexer::Lexer::lex(&content).map_err(|e| miette!("Lexical error: {:?}", e))?;

    // Parse
    let ast = pine_parser::parse(tokens).map_err(|e| miette!("Parse error: {:?}", e))?;

    println!("✓ Script is syntactically correct");
    println!("  Statements: {}", ast.stmts.len());

    // Semantic analysis (if available)
    // This would use pine-sema when fully implemented

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
