//! VM-based script execution with series support and plot output collection
//!
//! This module provides high-level execution interfaces for running Pine Script
//! with the VM, including series data management and plot output collection.

use crate::ast_compiler::compile_script;
use crate::vm::VM;
use pine_parser::ast::Script;
use pine_runtime::context::CallSiteId;
use pine_runtime::value::Value;
use std::collections::HashMap;

/// Plot outputs collector
#[derive(Debug, Clone, Default)]
pub struct PlotOutputs {
    /// Map of plot title to series of values
    plots: HashMap<String, Vec<Option<f64>>>,
    /// Current bar index
    current_bar: usize,
}

impl PlotOutputs {
    /// Create a new plot outputs collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a plot value for the current bar
    pub fn record(&mut self, title: impl Into<String>, value: Option<f64>) {
        let title = title.into();
        let plot = self.plots.entry(title).or_default();

        // Ensure the vector is long enough to hold values up to current_bar
        while plot.len() <= self.current_bar {
            plot.push(None);
        }
        plot[self.current_bar] = value;
    }

    /// Advance to the next bar
    pub fn next_bar(&mut self) {
        self.current_bar += 1;
    }

    /// Get all plot outputs
    pub fn get_plots(&self) -> &HashMap<String, Vec<Option<f64>>> {
        &self.plots
    }

    /// Get plot values by title
    pub fn get_plot(&self, title: &str) -> Option<&Vec<Option<f64>>> {
        self.plots.get(title)
    }
}

/// Series data for bar-by-bar execution
#[derive(Debug, Clone)]
pub struct SeriesData {
    /// Open price series
    pub open: Vec<f64>,
    /// High price series
    pub high: Vec<f64>,
    /// Low price series
    pub low: Vec<f64>,
    /// Close price series
    pub close: Vec<f64>,
    /// Volume series
    pub volume: Vec<f64>,
    /// Time series
    pub time: Vec<i64>,
}

impl SeriesData {
    /// Create series data from vectors
    pub fn new(
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
        volume: Vec<f64>,
        time: Vec<i64>,
    ) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
            time,
        }
    }

    /// Get the number of bars
    pub fn len(&self) -> usize {
        self.close.len()
    }

    /// Check if series is empty
    pub fn is_empty(&self) -> bool {
        self.close.is_empty()
    }
}

/// Execution result from VM
#[derive(Debug)]
pub struct VmExecutionResult {
    /// Plot outputs
    pub plot_outputs: PlotOutputs,
    /// Number of bars processed
    pub bars_processed: usize,
    /// Success flag
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Execute a Pine Script using the VM with series data
///
/// This is the main entry point for VM-based script execution.
/// It compiles the AST to bytecode and executes bar-by-bar.
///
/// # Arguments
/// * `script` - The parsed Pine Script AST
/// * `series_data` - OHLCV series data for execution
///
/// # Returns
/// Execution result with plot outputs
pub fn execute_script_with_vm(
    script: &Script,
    series_data: &SeriesData,
) -> Result<VmExecutionResult, crate::VmError> {
    // Compile AST to bytecode
    let compiler = match compile_script(script) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("DEBUG: Compile error: {:?}", e);
            return Err(crate::VmError::CompileError(format!("{:?}", e)));
        }
    };
    let chunk = compiler.finish();
    eprintln!("DEBUG: chunk constants:");
    for (i, c) in chunk.constants.iter().enumerate() {
        eprintln!("  [{}]: {:?}", i, c);
    }
    eprintln!("DEBUG: chunk instructions:");
    for (i, inst) in chunk.instructions.iter().enumerate() {
        eprintln!("  [{}]: {:?} {:?}", i, inst.opcode, inst.operands);
    }

    // Collect external function names
    let external_functions: Vec<String> = chunk.external_functions.clone();
    eprintln!("DEBUG: external_functions = {:?}", external_functions);

    // Initialize plot outputs
    let mut plot_outputs = PlotOutputs::new();

    // Execute bar-by-bar
    let num_bars = series_data.len();

    // Create a shared context that persists across bars
    let mut context = ExecutionContext::new();
    context.set_bar_index(0);

    for bar_idx in 0..num_bars {
        // Create VM for this bar, but share the context
        let mut vm = VM::with_context(context);

        // Register external functions
        for func_name in &external_functions {
            vm.register_external_function(func_name);
        }

        // Set bar index so series push works correctly
        vm.context_mut().set_bar_index(bar_idx as i64);

        // Register series values for this bar
        register_series_for_bar(&mut vm, series_data, bar_idx);

        // Load and execute
        vm.load_chunk(chunk.clone());
        let result = vm.execute();
        if let Err(ref e) = result {
            eprintln!("DEBUG: VM execution error at bar {}: {:?}", bar_idx, e);
        }
        let _result = result?;

        // Collect plot outputs from VM
        eprintln!("DEBUG: bar {} - plot_outputs count: {}", bar_idx, vm.plot_outputs().len());
        for record in vm.plot_outputs() {
            eprintln!("DEBUG: bar {} - plot: {} = {:?}", bar_idx, record.title, record.value);
            plot_outputs.record(record.title.clone(), record.value);
        }

        plot_outputs.next_bar();

        // Extract context for next iteration
        context = vm.into_context();
    }

    Ok(VmExecutionResult {
        plot_outputs,
        bars_processed: num_bars,
        success: true,
        error: None,
    })
}

/// Register series values for the current bar in VM context
///
/// This pushes all historical values from bar 0 to bar_idx so that series
/// functions like ta.sma can access the full series history.
fn register_series_for_bar(vm: &mut VM, series_data: &SeriesData, bar_idx: usize) {
    let call_site = CallSiteId(0); // Use global call site for built-in series

    eprintln!("DEBUG register_series_for_bar: bar_idx={}, close.len={}", bar_idx, series_data.close.len());

    // Push all values from bar 0 to bar_idx for each series
    // Use get_or_create_series().push() directly to bypass bar_index check
    for i in 0..=bar_idx {
        // Register close
        if let Some(value) = series_data.close.get(i) {
            eprintln!("DEBUG register_series_for_bar: pushing close[{}] = {}", i, value);
            vm.context_mut()
                .get_or_create_series("close", call_site)
                .push(Value::Float(*value));
        }

        // Register open
        if let Some(value) = series_data.open.get(i) {
            vm.context_mut()
                .get_or_create_series("open", call_site)
                .push(Value::Float(*value));
        }

        // Register high
        if let Some(value) = series_data.high.get(i) {
            vm.context_mut()
                .get_or_create_series("high", call_site)
                .push(Value::Float(*value));
        }

        // Register low
        if let Some(value) = series_data.low.get(i) {
            vm.context_mut()
                .get_or_create_series("low", call_site)
                .push(Value::Float(*value));
        }

        // Register volume
        if let Some(value) = series_data.volume.get(i) {
            vm.context_mut()
                .get_or_create_series("volume", call_site)
                .push(Value::Float(*value));
        }
    }
}
