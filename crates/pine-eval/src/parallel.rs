//! Parallel execution support for Pine Script using Rayon
//!
//! This module provides parallel execution capabilities for:
//! - Running multiple scripts simultaneously
//! - Processing multiple stocks/symbols in parallel
//! - Parallel bar-by-bar computation for independent series

use rayon::prelude::*;
use std::sync::Arc;

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of threads to use (0 = use Rayon default)
    pub num_threads: usize,
    /// Minimum batch size for parallel processing
    pub min_batch_size: usize,
    /// Enable parallel execution
    pub enabled: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: 0, // Use Rayon default
            min_batch_size: 100,
            enabled: true,
        }
    }
}

impl ParallelConfig {
    /// Create a new parallel config with the specified thread count
    pub fn with_threads(num_threads: usize) -> Self {
        Self {
            num_threads,
            ..Default::default()
        }
    }

    /// Disable parallel execution
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Initialize the Rayon thread pool with this configuration
    pub fn init_thread_pool(&self) {
        if self.num_threads > 0 {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(self.num_threads)
                .build_global();
        }
    }
}

/// A script execution task
#[derive(Debug, Clone)]
pub struct ScriptTask {
    /// Script ID
    pub id: String,
    /// Script source code
    pub source: String,
    /// Symbol/ticker to run the script on
    pub symbol: String,
    /// Timeframe (e.g., "1h", "1d")
    pub timeframe: String,
}

/// Result of a script execution
#[derive(Debug, Clone)]
pub struct ScriptResult {
    /// Script ID
    pub id: String,
    /// Symbol that was processed
    pub symbol: String,
    /// Execution result (JSON output)
    pub output: serde_json::Value,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Execute multiple scripts in parallel
///
/// This function runs multiple Pine Script programs simultaneously,
/// each potentially on different symbols/timeframes.
///
/// # Example
///
/// ```rust,ignore
/// use pine_eval::parallel::{execute_scripts_parallel, ScriptTask, ParallelConfig};
///
/// let tasks = vec![
///     ScriptTask {
///         id: "script1".to_string(),
///         source: "plot(close)".to_string(),
///         symbol: "BTCUSDT".to_string(),
///         timeframe: "1h".to_string(),
///     },
///     ScriptTask {
///         id: "script2".to_string(),
///         source: "plot(open)".to_string(),
///         symbol: "ETHUSDT".to_string(),
///         timeframe: "1h".to_string(),
///     },
/// ];
///
/// let results = execute_scripts_parallel(tasks, &ParallelConfig::default());
/// ```
pub fn execute_scripts_parallel(
    tasks: Vec<ScriptTask>,
    config: &ParallelConfig,
) -> Vec<ScriptResult> {
    if !config.enabled || tasks.len() < config.min_batch_size {
        // Sequential execution for small batches or when disabled
        tasks.into_iter().map(execute_single_script).collect()
    } else {
        // Parallel execution
        tasks.into_par_iter().map(execute_single_script).collect()
    }
}

/// Execute a single script task
fn execute_single_script(task: ScriptTask) -> ScriptResult {
    let start = std::time::Instant::now();

    // TODO: Implement actual script execution
    // For now, return a mock result
    let output = serde_json::json!({
        "script_id": task.id,
        "symbol": task.symbol,
        "timeframe": task.timeframe,
        "status": "executed",
    });

    ScriptResult {
        id: task.id,
        symbol: task.symbol,
        output,
        execution_time_ms: start.elapsed().as_millis() as u64,
    }
}

/// Process multiple symbols in parallel with the same script
///
/// This is useful for scanning multiple stocks with the same strategy.
///
/// # Arguments
///
/// * `script_source` - The Pine Script source code
/// * `symbols` - List of symbols to process
/// * `timeframe` - The timeframe to use for all symbols
/// * `config` - Parallel execution configuration
pub fn scan_symbols_parallel(
    script_source: String,
    symbols: Vec<String>,
    timeframe: String,
    config: &ParallelConfig,
) -> Vec<ScriptResult> {
    let tasks: Vec<ScriptTask> = symbols
        .into_iter()
        .enumerate()
        .map(|(i, symbol)| ScriptTask {
            id: format!("scan_{}", i),
            source: script_source.clone(),
            symbol,
            timeframe: timeframe.clone(),
        })
        .collect();

    execute_scripts_parallel(tasks, config)
}

/// Parallel map operation for series data
///
/// Applies a function to each element of a series in parallel.
/// This is useful for computationally expensive operations on large datasets.
pub fn parallel_series_map<T, F, R>(data: &[T], f: F) -> Vec<R>
where
    T: Sync,
    F: Fn(&T) -> R + Sync + Send,
    R: Send,
{
    data.par_iter().map(f).collect()
}

/// Parallel reduce operation for series data
///
/// Reduces a series to a single value using parallel computation.
pub fn parallel_series_reduce<T, F>(data: &[T], init: T, f: F) -> T
where
    T: Send + Sync + Clone,
    F: Fn(T, T) -> T + Sync + Send,
{
    data.par_iter().cloned().reduce(|| init.clone(), f)
}

/// Batch process multiple data series in parallel
///
/// This is useful for processing multiple time series simultaneously,
/// such as when computing indicators for multiple symbols.
pub fn batch_process_parallel<T, R, F>(batches: Vec<Vec<T>>, processor: F) -> Vec<Vec<R>>
where
    T: Send,
    R: Send,
    F: Fn(Vec<T>) -> Vec<R> + Sync + Send,
{
    batches.into_par_iter().map(processor).collect()
}

/// Thread-safe script cache for parallel execution
#[derive(Debug, Default, Clone)]
pub struct ParallelScriptCache {
    // TODO: Implement a concurrent hash map for compiled scripts
}

impl ParallelScriptCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a cached script or compile it
    pub fn get_or_compile(&self, _source: &str) -> Arc<CompiledScript> {
        // TODO: Implement actual caching
        Arc::new(CompiledScript {
            source_hash: 0,
            bytecode: Vec::new(),
        })
    }
}

/// A compiled script (placeholder)
#[derive(Debug)]
pub struct CompiledScript {
    /// Hash of the source code
    pub source_hash: u64,
    /// Compiled bytecode
    pub bytecode: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config() {
        let config = ParallelConfig::default();
        assert!(config.enabled);
        assert_eq!(config.num_threads, 0);

        let disabled = ParallelConfig::disabled();
        assert!(!disabled.enabled);
    }

    #[test]
    fn test_execute_scripts_parallel() {
        let tasks = vec![
            ScriptTask {
                id: "test1".to_string(),
                source: "plot(close)".to_string(),
                symbol: "BTCUSDT".to_string(),
                timeframe: "1h".to_string(),
            },
            ScriptTask {
                id: "test2".to_string(),
                source: "plot(open)".to_string(),
                symbol: "ETHUSDT".to_string(),
                timeframe: "1h".to_string(),
            },
        ];

        let results = execute_scripts_parallel(tasks, &ParallelConfig::disabled());
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "test1");
        assert_eq!(results[1].id, "test2");
    }

    #[test]
    fn test_scan_symbols_parallel() {
        let symbols = vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "SOLUSDT".to_string(),
        ];

        let results = scan_symbols_parallel(
            "plot(close)".to_string(),
            symbols,
            "1h".to_string(),
            &ParallelConfig::disabled(),
        );

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_parallel_series_map() {
        let data: Vec<i32> = (0..1000).collect();
        let result: Vec<i32> = parallel_series_map(&data, |x| x * 2);

        assert_eq!(result.len(), 1000);
        assert_eq!(result[0], 0);
        assert_eq!(result[500], 1000);
        assert_eq!(result[999], 1998);
    }

    #[test]
    fn test_parallel_series_reduce() {
        let data: Vec<i32> = (1..=100).collect();
        let sum = parallel_series_reduce(&data, 0, |a, b| a + b);

        assert_eq!(sum, 5050); // Sum of 1 to 100
    }

    #[test]
    fn test_batch_process_parallel() {
        let batches: Vec<Vec<i32>> = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

        let results: Vec<Vec<i32>> =
            batch_process_parallel(batches, |batch| batch.into_iter().map(|x| x * 2).collect());

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], vec![2, 4, 6]);
        assert_eq!(results[1], vec![8, 10, 12]);
        assert_eq!(results[2], vec![14, 16, 18]);
    }
}
