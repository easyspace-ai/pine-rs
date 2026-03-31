//! Function registry

use indexmap::IndexMap;

/// Function registry entry
#[derive(Debug, Clone)]
pub struct FunctionEntry {
    /// Function name
    pub name: String,
    /// Function signature
    pub signature: String,
}

/// Function registry
#[derive(Debug, Default)]
pub struct FunctionRegistry {
    /// Registered functions
    functions: IndexMap<String, FunctionEntry>,
}

impl FunctionRegistry {
    /// Create a new function registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a function
    pub fn register(&mut self, entry: FunctionEntry) {
        self.functions.insert(entry.name.clone(), entry);
    }

    /// Look up a function by name
    pub fn lookup(&self, name: &str) -> Option<&FunctionEntry> {
        self.functions.get(name)
    }

    /// Get all registered function names
    pub fn names(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
}
