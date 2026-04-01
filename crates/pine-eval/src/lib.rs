//! Pine Script v6 Expression Evaluator
//!
//! This crate provides the expression evaluation engine for Pine Script,
//! including bar-by-bar execution and series alignment.

#![warn(missing_docs)]

pub mod eval_expr;
pub mod eval_stmt;
pub mod fn_call;
pub mod parallel;
pub mod runner;

use pine_lexer::Span;
use pine_runtime::module::{ModuleId, ModuleRegistry};
use pine_runtime::value::{Object, Value};
use pine_stdlib::registry::FunctionRegistry;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

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
    /// Series data for historical access (open, high, low, close, volume, time)
    pub series_data: Option<SeriesData>,
    /// Plot outputs collector
    pub plot_outputs: PlotOutputs,
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
            module_registry: ModuleRegistry::new(),
            function_registry,
            loading_modules: Vec::new(),
            base_path: PathBuf::from("."),
            current_module: None,
            series_data: None,
            plot_outputs: PlotOutputs::new(),
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
        ctx.base_path = base_path.into();
        ctx
    }

    /// Get a variable value
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        self.variables.insert(name.into(), value);
    }

    /// Create a new object of the given type
    pub fn create_object(&mut self, type_name: impl Into<String>) -> Value {
        let obj = Object::new(type_name);
        Value::from(obj)
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
        self.base_path = path.into();
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
