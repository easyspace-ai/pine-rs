//! Function registry for Pine Script standard library
//!
//! This module provides a hash-based dispatch system for built-in functions,
//! inspired by Rhai's function dispatch mechanism.

use indexmap::IndexMap;
use pine_runtime::value::Value;
use std::fmt;
use std::sync::Arc;

/// Type alias for builtin function implementation
pub type BuiltinFn = Arc<dyn Fn(&[Value]) -> Value + Send + Sync>;

/// Function metadata
#[derive(Debug, Clone)]
pub struct FunctionMeta {
    /// Function name (as called in Pine Script)
    pub name: String,
    /// Namespace (e.g., "ta", "math", "str")
    pub namespace: Option<String>,
    /// Number of required arguments
    pub required_args: usize,
    /// Number of optional arguments
    pub optional_args: usize,
    /// Whether the function accepts a variable number of arguments
    pub variadic: bool,
    /// Whether this function returns a series
    pub returns_series: bool,
}

impl FunctionMeta {
    /// Create a new function metadata
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: None,
            required_args: 0,
            optional_args: 0,
            variadic: false,
            returns_series: false,
        }
    }

    /// Set the namespace
    pub fn with_namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = Some(ns.into());
        self
    }

    /// Set the number of required arguments
    pub fn with_required_args(mut self, count: usize) -> Self {
        self.required_args = count;
        self
    }

    /// Set the number of optional arguments
    pub fn with_optional_args(mut self, count: usize) -> Self {
        self.optional_args = count;
        self
    }

    /// Mark as variadic
    pub fn with_variadic(mut self) -> Self {
        self.variadic = true;
        self
    }

    /// Mark as returning a series
    pub fn with_series_return(mut self) -> Self {
        self.returns_series = true;
        self
    }

    /// Get the full name (namespace.name or just name)
    pub fn full_name(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{}.{}", ns, self.name),
            None => self.name.clone(),
        }
    }
}

/// Function registry entry
#[derive(Clone)]
pub struct FunctionEntry {
    /// Function metadata
    pub meta: FunctionMeta,
    /// Function implementation
    pub func: BuiltinFn,
}

impl fmt::Debug for FunctionEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionEntry")
            .field("meta", &self.meta)
            .field("func", &"<builtin fn>")
            .finish()
    }
}

/// Function registry with hash-based dispatch
///
/// This registry stores built-in functions and provides O(1) lookup
/// by function name. It supports namespaced functions (e.g., "ta.sma").
#[derive(Default)]
pub struct FunctionRegistry {
    /// Registered functions (full_name -> entry)
    functions: IndexMap<String, FunctionEntry>,
    /// Fast lookup cache for common functions
    hot_functions: IndexMap<String, BuiltinFn>,
}

impl fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionRegistry")
            .field("functions", &self.functions)
            .field("hot_functions", &self.hot_functions.len())
            .finish()
    }
}

impl FunctionRegistry {
    /// Create a new empty function registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new registry with all standard library functions
    pub fn with_stdlib() -> Self {
        let mut registry = Self::new();
        registry.register_stdlib();
        registry
    }

    /// Register a function
    pub fn register(&mut self, meta: FunctionMeta, func: BuiltinFn) {
        let full_name = meta.full_name();
        let entry = FunctionEntry { meta, func };
        self.functions.insert(full_name, entry);
    }

    /// Register with hot path optimization
    pub fn register_hot(&mut self, meta: FunctionMeta, func: BuiltinFn) {
        let full_name = meta.full_name();
        self.hot_functions.insert(full_name.clone(), func.clone());
        self.register(meta, func);
    }

    /// Look up a function by name
    pub fn lookup(&self, name: &str) -> Option<&FunctionEntry> {
        self.functions.get(name)
    }

    /// Fast dispatch for hot functions
    pub fn dispatch_hot(&self, name: &str, args: &[Value]) -> Option<Value> {
        self.hot_functions.get(name).map(|f| f(args))
    }

    /// Dispatch a function call
    pub fn dispatch(&self, name: &str, args: &[Value]) -> Option<Value> {
        // Try hot path first
        if let Some(result) = self.dispatch_hot(name, args) {
            return Some(result);
        }

        // Fall back to full lookup
        self.functions.get(name).map(|entry| (entry.func)(args))
    }

    /// Check if a function exists
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all registered function names
    pub fn names(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }

    /// Get functions by namespace
    pub fn by_namespace(&self, namespace: &str) -> Vec<&FunctionEntry> {
        self.functions
            .values()
            .filter(|e| e.meta.namespace.as_deref() == Some(namespace))
            .collect()
    }

    /// Register all standard library functions
    fn register_stdlib(&mut self) {
        crate::ta::register_functions(self);
        crate::math::register_functions(self);
        crate::array::register_functions(self);
        crate::map::register_functions(self);
        crate::str::register_functions(self);
        crate::color::register_functions(self);
    }

    /// Get the number of registered functions
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}

/// Global function registry singleton
///
/// This provides a global registry for convenience.
/// Use `FunctionRegistry::with_stdlib()` for a fresh instance.
pub struct GlobalRegistry;

impl GlobalRegistry {
    /// Get the global registry instance
    ///
    /// Note: This uses a lazy-initialized static.
    pub fn get() -> &'static std::sync::Mutex<FunctionRegistry> {
        use std::sync::LazyLock;

        static REGISTRY: LazyLock<std::sync::Mutex<FunctionRegistry>> =
            LazyLock::new(|| std::sync::Mutex::new(FunctionRegistry::with_stdlib()));

        &REGISTRY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut registry = FunctionRegistry::new();

        let meta = FunctionMeta::new("test_fn").with_required_args(1);
        let func: BuiltinFn = Arc::new(|args| {
            if let Some(arg) = args.first() {
                arg.clone()
            } else {
                Value::Na
            }
        });

        registry.register(meta, func);

        assert!(registry.contains("test_fn"));
        assert!(!registry.contains("nonexistent"));

        let entry = registry.lookup("test_fn").unwrap();
        assert_eq!(entry.meta.name, "test_fn");
    }

    #[test]
    fn test_dispatch() {
        let mut registry = FunctionRegistry::new();

        let meta = FunctionMeta::new("add").with_required_args(2);
        let func: BuiltinFn = Arc::new(|args| {
            if args.len() >= 2 {
                match (args[0].as_float(), args[1].as_float()) {
                    (Some(a), Some(b)) => Value::Float(a + b),
                    _ => Value::Na,
                }
            } else {
                Value::Na
            }
        });

        registry.register(meta, func);

        let result = registry.dispatch("add", &[Value::Int(2), Value::Int(3)]);
        assert_eq!(result, Some(Value::Float(5.0)));
    }

    #[test]
    fn test_namespaced_function() {
        let meta = FunctionMeta::new("sma")
            .with_namespace("ta")
            .with_required_args(2);
        assert_eq!(meta.full_name(), "ta.sma");
    }

    #[test]
    fn test_hot_dispatch() {
        let mut registry = FunctionRegistry::new();

        let meta = FunctionMeta::new("hot_add");
        let func: BuiltinFn = Arc::new(|_| Value::Int(42));

        registry.register_hot(meta, func);

        // Should work through hot path
        let result = registry.dispatch_hot("hot_add", &[]);
        assert_eq!(result, Some(Value::Int(42)));

        // Should also work through regular dispatch
        let result = registry.dispatch("hot_add", &[]);
        assert_eq!(result, Some(Value::Int(42)));
    }
}
