//! Execution context

use crate::value::Value;
use indexmap::IndexMap;

/// Execution context for a bar
#[derive(Debug, Default)]
pub struct ExecutionContext {
    /// Variable values
    variables: IndexMap<String, Value>,
    /// Series slot map (for call-site isolation)
    series_slots: IndexMap<String, Vec<Value>>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable value
    pub fn set_var(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Get a variable value
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Push a value to a series slot
    pub fn push_series(&mut self, slot: String, value: Value) {
        self.series_slots.entry(slot).or_default().push(value);
    }

    /// Get series values for a slot
    pub fn get_series(&self, slot: &str) -> Option<&Vec<Value>> {
        self.series_slots.get(slot)
    }
}
