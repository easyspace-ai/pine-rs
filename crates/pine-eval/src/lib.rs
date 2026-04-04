//! Pine Script v6 Expression Evaluator
//!
//! This crate provides the expression evaluation engine for Pine Script,
//! including bar-by-bar execution and series alignment.

#![warn(missing_docs)]

pub mod eval_expr;
pub mod eval_stmt;
pub mod fn_call;
#[cfg(feature = "parallel")]
pub mod parallel;
pub mod runner;

use indexmap::IndexMap;
use pine_lexer::Span;
use pine_parser::ast;
use pine_runtime::config::RuntimeConfig;
use pine_runtime::context::{BarState, CallSiteId, ExecutionContext};
use pine_runtime::module::{ModuleId, ModuleRegistry};
use pine_runtime::value::{Object, Value};
use pine_stdlib::registry::FunctionRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

/// User-defined function stored for invocation (AST body).
#[derive(Debug, Clone)]
pub struct UserFn {
    /// Parameters
    pub params: Vec<ast::Param>,
    /// Body (`=> expr` or block)
    pub body: ast::FnBody,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedModule {
    pub path: PathBuf,
    pub exports: HashMap<String, Value>,
    pub exported_functions: HashMap<String, UserFn>,
}

/// Evaluation errors
#[derive(Debug, Error)]
pub enum EvalError {
    /// Placeholder error
    #[error("evaluation not yet implemented")]
    NotImplemented,

    /// Undefined variable
    #[error("undefined variable: {name}")]
    UndefinedVariable {
        /// Variable name
        name: String,
        /// Span where the variable was referenced
        span: Span,
    },

    /// Type error
    #[error("type error: {message}")]
    TypeError {
        /// Error message
        message: String,
        /// Span of the error
        span: Span,
    },

    /// Not an object
    #[error("expected an object, got {found:?}")]
    NotAnObject {
        /// Found value
        found: Value,
        /// Span of the error
        span: Span,
    },

    /// Undefined field
    #[error("object has no field {field_name}")]
    UndefinedField {
        /// Field name
        field_name: String,
        /// Span of the field access
        span: Span,
    },

    /// Undefined method
    #[error("object has no method {method_name}")]
    UndefinedMethod {
        /// Method name
        method_name: String,
        /// Span of the method call
        span: Span,
    },

    /// Module not found
    #[error("module not found: {path}")]
    ModuleNotFound {
        /// Module path
        path: String,
        /// Span of the import statement
        span: Span,
    },

    /// Export not found in module
    #[error("'{name}' is not exported from module")]
    ExportNotFound {
        /// Name of the export
        name: String,
        /// Span of the reference
        span: Span,
    },

    /// Circular module dependency
    #[error("circular module dependency: {cycle}")]
    CircularDependency {
        /// Dependency cycle description
        cycle: String,
        /// Span of the import statement
        span: Span,
    },

    /// Semantic analysis failed
    #[error(transparent)]
    Semantic(#[from] pine_sema::SemaError),
}

/// Result type for evaluation operations
pub type Result<T> = std::result::Result<T, EvalError>;

/// Plot output entry for a single bar
#[derive(Debug, Clone)]
pub struct PlotOutput {
    /// Plot name/title
    pub title: String,
    /// Value at this bar (None = na)
    pub value: Option<f64>,
    /// Color (if specified)
    pub color: Option<pine_runtime::value::Color>,
}

/// Plot outputs collector
#[derive(Debug, Clone, Default)]
pub struct PlotOutputs {
    /// Map of plot title to series of values
    plots: HashMap<String, Vec<Option<f64>>>,
    /// Current bar index
    current_bar: usize,
}

/// Strategy signal entry
#[derive(Debug, Clone)]
pub struct StrategySignal {
    /// Bar index where signal occurred
    pub bar_index: usize,
    /// Signal type: "entry", "close", or "exit"
    pub signal_type: String,
    /// Signal id (e.g., "Long", "Short")
    pub id: String,
    /// Direction: "long", "short", or empty for close
    pub direction: String,
    /// Quantity
    pub qty: f64,
    /// Price (optional, None for market orders)
    pub price: Option<f64>,
    /// Comment
    pub comment: Option<String>,
}

/// Strategy signals collector
#[derive(Debug, Clone, Default)]
pub struct StrategySignals {
    /// List of all signals
    signals: Vec<StrategySignal>,
    /// Current bar index
    current_bar: usize,
}

impl StrategySignals {
    /// Create a new strategy signals collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an entry signal
    pub fn record_entry(&mut self, id: impl Into<String>, direction: impl Into<String>, qty: f64) {
        self.signals.push(StrategySignal {
            bar_index: self.current_bar,
            signal_type: "entry".to_string(),
            id: id.into(),
            direction: direction.into(),
            qty,
            price: None,
            comment: None,
        });
    }

    /// Record a close signal
    pub fn record_close(&mut self, id: impl Into<String>, comment: Option<String>) {
        self.signals.push(StrategySignal {
            bar_index: self.current_bar,
            signal_type: "close".to_string(),
            id: id.into(),
            direction: String::new(),
            qty: 0.0,
            price: None,
            comment,
        });
    }

    /// Record an exit signal
    pub fn record_exit(
        &mut self,
        id: impl Into<String>,
        from_entry: impl Into<String>,
        qty: Option<f64>,
    ) {
        self.signals.push(StrategySignal {
            bar_index: self.current_bar,
            signal_type: "exit".to_string(),
            id: id.into(),
            direction: String::new(),
            qty: qty.unwrap_or(0.0),
            price: None,
            comment: Some(from_entry.into()),
        });
    }

    /// Advance to the next bar
    pub fn next_bar(&mut self) {
        self.current_bar += 1;
    }

    /// Get all signals
    pub fn get_signals(&self) -> &[StrategySignal] {
        &self.signals
    }

    /// Get entry signals
    pub fn get_entries(&self) -> Vec<&StrategySignal> {
        self.signals
            .iter()
            .filter(|s| s.signal_type == "entry")
            .collect()
    }

    /// Get exit/close signals
    pub fn get_exits(&self) -> Vec<&StrategySignal> {
        self.signals
            .iter()
            .filter(|s| s.signal_type == "exit" || s.signal_type == "close")
            .collect()
    }
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

/// Evaluation context for Pine Script
#[derive(Debug)]
pub struct EvaluationContext {
    /// Variable bindings
    variables: HashMap<String, Value>,
    /// Per-variable history for series-aware ta.* calls
    variable_history: HashMap<String, Vec<Value>>,
    /// User-defined functions by name
    user_functions: HashMap<String, UserFn>,
    /// UDT methods: type name -> method name -> function
    type_methods: HashMap<String, HashMap<String, UserFn>>,
    /// Module registry for loaded libraries
    module_registry: ModuleRegistry,
    /// Function registry for built-in functions
    function_registry: FunctionRegistry,
    /// Currently loading modules (for circular dependency detection)
    loading_modules: Vec<String>,
    /// Base path for resolving relative imports
    base_path: PathBuf,
    /// Current module ID (if executing a library)
    current_module: Option<ModuleId>,
    /// Loaded module cache with executable exports.
    loaded_modules: HashMap<PathBuf, LoadedModule>,
    /// Series data for historical access (open, high, low, close, volume, time)
    pub series_data: Option<SeriesData>,
    /// Plot outputs collector
    pub plot_outputs: PlotOutputs,
    /// Strategy signals collector
    pub strategy_signals: StrategySignals,
    /// Runtime: call-site IDs, per-site `var`, series ring buffers.
    runtime: ExecutionContext,
    /// Nested UDF call sites.
    call_site_stack: Vec<CallSiteId>,
    /// Stable mapping from `(fn name, call span)` to [`CallSiteId`] across bars.
    call_site_intern: IndexMap<(String, usize, usize), CallSiteId>,
}

/// Series data for historical index access
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
    /// Current bar index
    pub current_bar: usize,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        let mut function_registry = FunctionRegistry::new();
        // Initialize with standard library functions
        pine_stdlib::init(&mut function_registry);

        Self {
            variables: HashMap::new(),
            variable_history: HashMap::new(),
            user_functions: HashMap::new(),
            type_methods: HashMap::new(),
            module_registry: ModuleRegistry::new(),
            function_registry,
            loading_modules: Vec::new(),
            base_path: PathBuf::from("."),
            current_module: None,
            loaded_modules: HashMap::new(),
            series_data: None,
            plot_outputs: PlotOutputs::new(),
            strategy_signals: StrategySignals::new(),
            runtime: ExecutionContext::new(Arc::new(RuntimeConfig::default())),
            call_site_stack: Vec::new(),
            call_site_intern: IndexMap::new(),
        }
    }
}

impl EvaluationContext {
    /// Create a new evaluation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new evaluation context with a base path
    pub fn with_base_path(base_path: impl Into<PathBuf>) -> Self {
        let mut ctx = Self::new();
        let p = base_path.into();
        ctx.base_path = p.clone();
        ctx.runtime.set_base_path(p);
        ctx
    }

    /// Pine call site for `var` / scoped series (`global` when not in a UDF).
    pub fn current_call_site(&self) -> CallSiteId {
        self.call_site_stack
            .last()
            .copied()
            .unwrap_or_else(ExecutionContext::global_call_site)
    }

    /// Intern a stable call site for `fn_name` at the given source span (shared across bars).
    pub(crate) fn intern_call_site(&mut self, fn_name: &str, span: Span) -> CallSiteId {
        let key = (fn_name.to_string(), span.start, span.end);
        if let Some(id) = self.call_site_intern.get(&key).copied() {
            return id;
        }
        let id = self.runtime.new_call_site();
        self.call_site_intern.insert(key, id);
        id
    }

    pub(crate) fn push_call_site(&mut self, site: CallSiteId) {
        self.call_site_stack.push(site);
    }

    pub(crate) fn pop_call_site(&mut self) {
        self.call_site_stack.pop();
    }

    pub(crate) fn runtime(&self) -> &ExecutionContext {
        &self.runtime
    }

    pub(crate) fn runtime_mut(&mut self) -> &mut ExecutionContext {
        &mut self.runtime
    }

    /// Update the current runtime `barstate.*` flags.
    pub fn set_bar_state(&mut self, bar_state: BarState) {
        self.runtime.set_bar_state(bar_state);
    }

    /// Get a variable value
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        if let Some(v) = self.variables.get(name) {
            return Some(v);
        }
        let cs = self.current_call_site();
        self.runtime
            .get_var_scoped(name, cs)
            .or_else(|| self.runtime.get_persistent_var(name))
            .or_else(|| self.runtime.get_varip_var(name))
    }

    /// Set a variable value
    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        let name = name.into();
        if let Some(series) = &self.series_data {
            let history = self.variable_history.entry(name.clone()).or_default();
            if history.len() <= series.current_bar {
                history.resize(series.current_bar + 1, Value::Na);
            }
            history[series.current_bar] = value.clone();
        }
        self.variables.insert(name, value);
    }

    /// Remove a variable (restore after UDF call).
    pub fn remove_var(&mut self, name: &str) {
        self.variables.remove(name);
        self.variable_history.remove(name);
    }

    /// Register a user-defined function.
    pub fn register_user_fn(&mut self, name: impl Into<String>, user_fn: UserFn) {
        self.user_functions.insert(name.into(), user_fn);
    }

    /// Look up a user-defined function.
    pub fn get_user_fn(&self, name: &str) -> Option<&UserFn> {
        self.user_functions.get(name)
    }

    /// Register a method on a UDT.
    pub fn register_type_method(
        &mut self,
        type_name: impl Into<String>,
        method_name: impl Into<String>,
        user_fn: UserFn,
    ) {
        self.type_methods
            .entry(type_name.into())
            .or_default()
            .insert(method_name.into(), user_fn);
    }

    /// Look up a UDT method.
    pub fn get_type_method(&self, type_name: &str, method_name: &str) -> Option<&UserFn> {
        self.type_methods
            .get(type_name)
            .and_then(|m| m.get(method_name))
    }

    /// Get historical values for a variable up to the current bar.
    pub fn get_var_history(&self, name: &str) -> Option<&[Value]> {
        let series = self.series_data.as_ref()?;
        let history = self.variable_history.get(name)?;
        history.get(..series.current_bar + 1)
    }

    /// Create a new object of the given type
    pub fn create_object(&mut self, type_name: impl Into<String>) -> Value {
        let obj = Object::new(type_name);
        Value::from(obj)
    }

    /// Create a runtime object with fields.
    pub fn create_object_with_fields(
        &mut self,
        type_name: impl Into<String>,
        fields: impl IntoIterator<Item = (String, Value)>,
    ) -> Value {
        Value::from(Object::with_fields(type_name, fields))
    }

    /// Get a reference to the module registry
    pub fn module_registry(&self) -> &ModuleRegistry {
        &self.module_registry
    }

    /// Get a mutable reference to the module registry
    pub fn module_registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.module_registry
    }

    /// Get a reference to the function registry
    pub fn function_registry(&self) -> &FunctionRegistry {
        &self.function_registry
    }

    /// Get a mutable reference to the function registry
    pub fn function_registry_mut(&mut self) -> &mut FunctionRegistry {
        &mut self.function_registry
    }

    /// Get the base path for resolving imports
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Set the base path for resolving imports
    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
        let p = path.into();
        self.base_path = p.clone();
        self.runtime.set_base_path(p);
    }

    /// Check if a module is currently being loaded
    pub fn is_loading_module(&self, path: &str) -> bool {
        self.loading_modules.iter().any(|p| p == path)
    }

    /// Begin loading a module
    pub fn begin_module_load(&mut self, path: impl Into<String>) {
        self.loading_modules.push(path.into());
    }

    /// End loading a module
    pub fn end_module_load(&mut self) {
        self.loading_modules.pop();
    }

    /// Get the current module ID
    pub fn current_module(&self) -> Option<ModuleId> {
        self.current_module
    }

    /// Set the current module ID
    pub fn set_current_module(&mut self, module_id: Option<ModuleId>) {
        self.current_module = module_id;
    }

    pub(crate) fn cache_loaded_module(&mut self, module: LoadedModule) {
        self.loaded_modules.insert(module.path.clone(), module);
    }

    pub(crate) fn get_loaded_module(&self, path: &Path) -> Option<&LoadedModule> {
        self.loaded_modules.get(path)
    }

    /// Get a historical value from a series
    ///
    /// # Arguments
    /// * `series_name` - Name of the series ("open", "high", "low", "close", "volume", "time")
    /// * `offset` - Number of bars back (0 = current bar, 1 = previous bar)
    ///
    /// # Returns
    /// The value at the given offset, or None if the offset is out of bounds
    pub fn get_series_value(&self, series_name: &str, offset: usize) -> Option<f64> {
        let series_data = self.series_data.as_ref()?;
        let idx = series_data.current_bar.checked_sub(offset)?;

        match series_name {
            "open" => series_data.open.get(idx).copied(),
            "high" => series_data.high.get(idx).copied(),
            "low" => series_data.low.get(idx).copied(),
            "close" => series_data.close.get(idx).copied(),
            "volume" => series_data.volume.get(idx).copied(),
            "time" => series_data.time.get(idx).map(|t| *t as f64),
            _ => None,
        }
    }
}

/// Evaluate a Pine Script program
pub fn evaluate() -> Result<()> {
    // TODO: Implement evaluation
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert!(evaluate().is_ok());
    }
}
