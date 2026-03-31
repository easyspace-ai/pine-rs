//! Pine Script v6 Expression Evaluator
//!
//! This crate provides the expression evaluation engine for Pine Script,
//! including bar-by-bar execution and series alignment.

#![warn(missing_docs)]

pub mod eval_expr;
pub mod eval_stmt;
pub mod fn_call;
pub mod runner;

use pine_lexer::Span;
use pine_runtime::module::{ModuleId, ModuleRegistry};
use pine_runtime::value::{Object, Value};
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

/// Evaluation context for Pine Script
#[derive(Debug)]
pub struct EvaluationContext {
    /// Variable bindings
    variables: HashMap<String, Value>,
    /// Module registry for loaded libraries
    module_registry: ModuleRegistry,
    /// Currently loading modules (for circular dependency detection)
    loading_modules: Vec<String>,
    /// Base path for resolving relative imports
    base_path: PathBuf,
    /// Current module ID (if executing a library)
    current_module: Option<ModuleId>,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            module_registry: ModuleRegistry::new(),
            loading_modules: Vec::new(),
            base_path: PathBuf::from("."),
            current_module: None,
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
