//! Execution context for Pine Script
//!
//! This module provides `ExecutionContext` which manages the runtime state
//! during script execution, including variables, series buffers, and call-site isolation.

use crate::config::RuntimeConfig;
use crate::module::{ModuleId, ModuleNamespace, ModuleRegistry};
use crate::series::SeriesBuf;
use crate::value::Value;
use indexmap::IndexMap;
use std::path::PathBuf;
use std::sync::Arc;

/// A slot index for variable storage (inspired by Rhai)
///
/// Variables are stored in a flat array indexed by SlotId,
/// allowing O(1) access without hash lookups.
pub type SlotId = usize;

/// A unique identifier for a call site
///
/// In Pine Script, the same function can be called from multiple locations,
/// and each call site maintains its own independent series state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallSiteId(pub usize);

/// Key for series slot lookup
///
/// Combines the variable name with the call site for proper isolation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SeriesKey {
    /// Variable name (e.g., "my_sma")
    name: String,
    /// Call site ID (for function call isolation)
    call_site: CallSiteId,
}

/// Persistence mode for Pine variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistentVarKind {
    /// `var`: persisted across bars, rolled back on realtime ticks.
    Var,
    /// `varip`: persisted across bars and intrabar updates.
    Varip,
}

/// Runtime bar state flags exposed through `barstate.*`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BarState {
    /// True on historical bars.
    pub ishistory: bool,
    /// True on realtime executions.
    pub isrealtime: bool,
    /// True on first update of a bar.
    pub isnew: bool,
    /// True when the current bar is confirmed/closed.
    pub isconfirmed: bool,
    /// True on the last visible bar.
    pub islast: bool,
    /// True only on the first bar in the dataset.
    pub isfirst: bool,
}

/// Execution context for bar-by-bar script execution
///
/// The context maintains:
/// - Simple variable values (scalars) in slot-based storage
/// - Series buffers (historical values)
/// - Call-site isolated series (for function calls)
/// - Runtime configuration
///
/// # Slot-Based Variable Storage
///
/// Variables are stored in a flat Vec indexed by SlotId (usize),
/// allowing O(1) access without hash lookups. This is inspired by Rhai's
/// implementation and provides significant performance improvements.
///
/// # Call-Site Series Isolation
///
/// Pine Script has a unique behavior where series variables inside functions
/// are isolated by call site. This means:
///
/// ```pine
/// f() =>
///     var count = 0
///     count := count + 1
///     count
///
/// a = f()  // count starts at 0
/// b = f()  // different count, also starts at 0
/// ```
///
/// This is implemented using `CallSiteId` in the series key.
#[derive(Debug)]
pub struct ExecutionContext {
    /// Runtime configuration
    config: Arc<RuntimeConfig>,

    /// Slot-based variable storage (non-series)
    ///
    /// Variables are indexed by SlotId for O(1) access.
    /// These are reset on each bar.
    variables: Vec<Option<Value>>,

    /// Variable name to slot mapping (for debugging and error messages)
    name_to_slot: IndexMap<String, SlotId>,

    /// Persistent series buffers
    ///
    /// These maintain history across bars.
    series: IndexMap<SeriesKey, SeriesBuf<Value>>,

    /// Last `bar_index` that appended a new element per series (not same-bar overwrites).
    series_last_bar: IndexMap<SeriesKey, i64>,

    /// var/varip variable storage (persistent across bars)
    ///
    /// These are preserved between bars.
    persistent_vars: IndexMap<String, Value>,

    /// varip variables (reset on strategy reset)
    varip_vars: IndexMap<String, Value>,

    /// `var` / `varip` values keyed by `(name, call_site)` for Pine call-site isolation.
    ///
    /// Global script `var` uses [`ExecutionContext::global_call_site`]. UDF `var` uses the stable
    /// ID interned for each call expression (see pine-eval).
    var_scoped: IndexMap<(String, CallSiteId), Value>,

    /// Persistence mode for scoped variables.
    scoped_var_kinds: IndexMap<(String, CallSiteId), PersistentVarKind>,

    /// Snapshot of `var` values at the last committed bar close.
    committed_var_scoped: IndexMap<(String, CallSiteId), Value>,

    /// Snapshot of series state at the last committed bar close.
    committed_series: IndexMap<SeriesKey, SeriesBuf<Value>>,

    /// Snapshot of per-series current bar tracking at the last committed bar close.
    committed_series_last_bar: IndexMap<SeriesKey, i64>,

    /// Current bar index
    bar_index: i64,

    /// Current timestamp (milliseconds since epoch)
    timestamp: i64,

    /// Call site counter (for generating unique IDs)
    next_call_site_id: usize,

    /// Recursion depth (for stack protection)
    recursion_depth: usize,

    /// Module registry for loaded libraries
    module_registry: ModuleRegistry,

    /// Current module ID (if this context is executing a library module)
    current_module: Option<ModuleId>,

    /// Module namespace bindings (alias -> module_id)
    module_namespaces: IndexMap<String, ModuleNamespace>,

    /// Base path for resolving relative imports
    base_path: PathBuf,

    /// Runtime `barstate.*` flags.
    bar_state: BarState,
}

impl ExecutionContext {
    /// Create a new execution context with the given number of variable slots
    ///
    /// # Arguments
    ///
    /// * `config` - Runtime configuration
    /// * `num_slots` - Number of variable slots to pre-allocate (from semantic analysis)
    pub fn with_slots(config: Arc<RuntimeConfig>, num_slots: usize) -> Self {
        Self {
            config,
            variables: vec![None; num_slots],
            name_to_slot: IndexMap::new(),
            series: IndexMap::new(),
            series_last_bar: IndexMap::new(),
            persistent_vars: IndexMap::new(),
            varip_vars: IndexMap::new(),
            var_scoped: IndexMap::new(),
            scoped_var_kinds: IndexMap::new(),
            committed_var_scoped: IndexMap::new(),
            committed_series: IndexMap::new(),
            committed_series_last_bar: IndexMap::new(),
            bar_index: 0,
            timestamp: 0,
            next_call_site_id: 1, // 0 is reserved for global scope
            recursion_depth: 0,
            module_registry: ModuleRegistry::new(),
            current_module: None,
            module_namespaces: IndexMap::new(),
            base_path: PathBuf::from("."),
            bar_state: BarState::default(),
        }
    }

    /// Create a new execution context
    ///
    /// Defaults to 64 variable slots. Use `with_slots` if you know
    /// the exact number of slots needed from semantic analysis.
    pub fn new(config: Arc<RuntimeConfig>) -> Self {
        Self::with_slots(config, 64)
    }

    /// Create a new execution context with default config
    pub fn default_with_config() -> Self {
        Self::new(Arc::new(RuntimeConfig::default()))
    }

    /// Create a new execution context with a base path for imports
    pub fn with_base_path(config: Arc<RuntimeConfig>, base_path: impl Into<PathBuf>) -> Self {
        let mut ctx = Self::new(config);
        ctx.base_path = base_path.into();
        ctx
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get current bar index
    pub fn bar_index(&self) -> i64 {
        self.bar_index
    }

    /// Set bar index
    pub fn set_bar_index(&mut self, index: i64) {
        self.bar_index = index;
    }

    /// Get current timestamp
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Set timestamp
    pub fn set_timestamp(&mut self, ts: i64) {
        self.timestamp = ts;
    }

    /// Generate a new unique call site ID
    pub fn new_call_site(&mut self) -> CallSiteId {
        let id = self.next_call_site_id;
        self.next_call_site_id += 1;
        CallSiteId(id)
    }

    /// Get the global call site (for top-level code)
    pub fn global_call_site() -> CallSiteId {
        CallSiteId(0)
    }

    //========================================================================
    // Slot-Based Variable Management
    //========================================================================

    /// Set a variable value by slot index
    ///
    /// This is the preferred method for variable access as it provides O(1)
    /// performance without hash lookups.
    pub fn set_slot(&mut self, slot: SlotId, value: Value) {
        if slot < self.variables.len() {
            self.variables[slot] = Some(value);
        }
    }

    /// Get a variable value by slot index
    #[inline]
    pub fn get_slot(&self, slot: SlotId) -> Option<&Value> {
        self.variables.get(slot).and_then(|v| v.as_ref())
    }

    /// Get a mutable reference to a variable by slot index
    #[inline]
    pub fn get_slot_mut(&mut self, slot: SlotId) -> Option<&mut Value> {
        self.variables.get_mut(slot).and_then(|v| v.as_mut())
    }

    /// Check if a slot has a value
    #[inline]
    pub fn slot_has_value(&self, slot: SlotId) -> bool {
        self.variables
            .get(slot)
            .map(|v| v.is_some())
            .unwrap_or(false)
    }

    /// Get or insert a value in a slot
    pub fn get_or_insert_slot(&mut self, slot: SlotId, default: Value) -> &mut Value {
        if slot >= self.variables.len() {
            self.variables.resize_with(slot + 1, || None);
        }
        if self.variables[slot].is_none() {
            self.variables[slot] = Some(default);
        }
        self.variables[slot].as_mut().unwrap()
    }

    /// Register a variable name to slot mapping (for debugging)
    pub fn register_var_name(&mut self, name: impl Into<String>, slot: SlotId) {
        self.name_to_slot.insert(name.into(), slot);
    }

    /// Look up a slot by variable name
    pub fn lookup_slot(&self, name: &str) -> Option<SlotId> {
        self.name_to_slot.get(name).copied()
    }

    /// Get the number of variable slots
    pub fn num_slots(&self) -> usize {
        self.variables.len()
    }

    /// Clear all simple variables (set all slots to None)
    ///
    /// Called at the start of each bar.
    pub fn clear_vars(&mut self) {
        for slot in &mut self.variables {
            *slot = None;
        }
    }

    //========================================================================
    // Legacy Variable Management (by name - for compatibility)
    //========================================================================

    /// Set a simple variable value by name
    ///
    /// This variable will be cleared on the next bar.
    /// Prefer `set_slot` for better performance.
    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        let name = name.into();
        if let Some(&slot) = self.name_to_slot.get(&name) {
            self.set_slot(slot, value);
        } else {
            // Dynamic variable - store in a new slot
            let slot = self.variables.len();
            self.variables.push(Some(value));
            self.name_to_slot.insert(name, slot);
        }
    }

    /// Get a variable value by name
    /// Prefer `get_slot` for better performance.
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.lookup_slot(name).and_then(|slot| self.get_slot(slot))
    }

    /// Get a mutable reference to a variable by name
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.lookup_slot(name)
            .and_then(|slot| self.get_slot_mut(slot))
    }

    /// Check if a variable exists
    pub fn has_var(&self, name: &str) -> bool {
        self.lookup_slot(name)
            .map(|slot| self.slot_has_value(slot))
            .unwrap_or(false)
    }

    /// Remove a variable by name
    ///
    /// Returns the removed value if it existed.
    pub fn remove_var(&mut self, name: &str) -> Option<Value> {
        self.lookup_slot(name)
            .and_then(|slot| self.variables.get_mut(slot).and_then(|v| v.take()))
    }

    //========================================================================
    // Series Management
    //========================================================================

    /// Get or create a series buffer
    ///
    /// # Panics
    ///
    /// Panics if the series buffer cannot be created.
    pub fn get_or_create_series(
        &mut self,
        name: impl Into<String>,
        call_site: CallSiteId,
    ) -> &mut SeriesBuf<Value> {
        let key = SeriesKey {
            name: name.into(),
            call_site,
        };

        let max_len = self.config.max_bars_back;
        self.series
            .entry(key)
            .or_insert_with(|| SeriesBuf::new(max_len))
    }

    /// Get a series buffer
    pub fn get_series(&self, name: &str, call_site: CallSiteId) -> Option<&SeriesBuf<Value>> {
        let key = SeriesKey {
            name: name.to_string(),
            call_site,
        };
        self.series.get(&key)
    }

    /// Push a value to a series
    ///
    /// Creates the series if it doesn't exist.
    pub fn push_to_series(&mut self, name: impl Into<String>, call_site: CallSiteId, value: Value) {
        let name = name.into();
        let key = SeriesKey {
            name: name.clone(),
            call_site,
        };
        let bar = self.bar_index;
        let same_bar = self.series_last_bar.get(&key).copied() == Some(bar);
        let series = self.get_or_create_series(name, call_site);
        if same_bar {
            series.update_current(value);
        } else {
            series.push(value);
            self.series_last_bar.insert(key, bar);
        }
    }

    /// Get the latest value from a series
    pub fn get_series_current(&self, name: &str, call_site: CallSiteId) -> Option<&Value> {
        self.get_series(name, call_site).and_then(|s| s.current())
    }

    /// Get a historical value from a series
    pub fn get_series_at(
        &self,
        name: &str,
        call_site: CallSiteId,
        offset: usize,
    ) -> Option<&Value> {
        self.get_series(name, call_site).and_then(|s| s.get(offset))
    }

    /// Get the full series history as a vector (newest first)
    pub fn get_series_history(&self, name: &str, call_site: CallSiteId) -> Option<Vec<Value>> {
        self.get_series(name, call_site).map(|s| s.to_vec())
    }

    /// Get the full series history as a vector (oldest first)
    pub fn get_series_history_oldest_first(
        &self,
        name: &str,
        call_site: CallSiteId,
    ) -> Option<Vec<Value>> {
        self.get_series(name, call_site)
            .map(|s| s.iter_oldest_first().cloned().collect())
    }

    /// Clear all series
    ///
    /// Called when the script is reset.
    pub fn clear_series(&mut self) {
        self.series.clear();
        self.series_last_bar.clear();
    }

    //========================================================================
    // Persistent Variables (var)
    //========================================================================

    /// Set a persistent variable (var)
    ///
    /// These variables persist across bars.
    pub fn set_persistent_var(&mut self, name: impl Into<String>, value: Value) {
        self.persistent_vars.insert(name.into(), value);
    }

    /// Get a persistent variable
    pub fn get_persistent_var(&self, name: &str) -> Option<&Value> {
        self.persistent_vars.get(name)
    }

    /// Get or insert a persistent variable
    pub fn get_or_insert_persistent(
        &mut self,
        name: impl Into<String>,
        default: Value,
    ) -> &mut Value {
        self.persistent_vars.entry(name.into()).or_insert(default)
    }

    /// Clear persistent variables
    pub fn clear_persistent_vars(&mut self) {
        self.persistent_vars.clear();
    }

    //========================================================================
    // Call-site scoped `var` / `varip` (Pine isolation)
    //========================================================================

    /// Whether a scoped `var` exists for this name at `call_site`.
    pub fn var_scoped_contains(&self, name: &str, call_site: CallSiteId) -> bool {
        self.var_scoped.contains_key(&(name.to_string(), call_site))
    }

    /// Get the persistence mode for a scoped variable.
    pub fn get_var_scoped_kind(
        &self,
        name: &str,
        call_site: CallSiteId,
    ) -> Option<PersistentVarKind> {
        self.scoped_var_kinds
            .get(&(name.to_string(), call_site))
            .copied()
    }

    /// Get scoped `var` value.
    pub fn get_var_scoped(&self, name: &str, call_site: CallSiteId) -> Option<&Value> {
        self.var_scoped.get(&(name.to_string(), call_site))
    }

    /// Declare a scoped persistent variable with its persistence mode.
    pub fn declare_var_scoped(
        &mut self,
        name: impl Into<String>,
        call_site: CallSiteId,
        kind: PersistentVarKind,
        value: Value,
    ) {
        let n = name.into();
        self.scoped_var_kinds.insert((n.clone(), call_site), kind);
        self.var_scoped.insert((n, call_site), value);
    }

    /// Set scoped `var` value (creates or replaces).
    pub fn set_var_scoped(&mut self, name: impl Into<String>, call_site: CallSiteId, value: Value) {
        let n = name.into();
        self.var_scoped.insert((n, call_site), value);
    }

    /// Clear all call-site scoped vars (e.g. full script reset).
    pub fn clear_var_scoped(&mut self) {
        self.var_scoped.clear();
        self.scoped_var_kinds.clear();
        self.committed_var_scoped.clear();
    }

    //========================================================================
    // Varip Variables (varip)
    //========================================================================

    /// Set a varip variable
    ///
    /// These persist across bars but reset on strategy reset.
    pub fn set_varip_var(&mut self, name: impl Into<String>, value: Value) {
        self.varip_vars.insert(name.into(), value);
    }

    /// Get a varip variable
    pub fn get_varip_var(&self, name: &str) -> Option<&Value> {
        self.varip_vars.get(name)
    }

    /// Get or insert a varip variable
    pub fn get_or_insert_varip(&mut self, name: impl Into<String>, default: Value) -> &mut Value {
        self.varip_vars.entry(name.into()).or_insert(default)
    }

    /// Clear varip variables
    pub fn clear_varip_vars(&mut self) {
        self.varip_vars.clear();
    }

    /// Snapshot current `var` state at bar close.
    pub fn commit_bar_state(&mut self) {
        self.committed_var_scoped = self
            .var_scoped
            .iter()
            .filter(|(key, _)| {
                matches!(
                    self.scoped_var_kinds.get(*key),
                    Some(PersistentVarKind::Var)
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        self.committed_series = self.series.clone();
        self.committed_series_last_bar = self.series_last_bar.clone();
    }

    /// Restore all `var` state to the last committed bar-close snapshot.
    ///
    /// `varip` values intentionally keep their intrabar updates.
    pub fn rollback_bar_state(&mut self) {
        let var_keys: Vec<_> = self
            .scoped_var_kinds
            .iter()
            .filter_map(|(key, kind)| (*kind == PersistentVarKind::Var).then_some(key.clone()))
            .collect();

        for key in var_keys {
            match self.committed_var_scoped.get(&key).cloned() {
                Some(value) => {
                    self.var_scoped.insert(key, value);
                }
                None => {
                    self.var_scoped.shift_remove(&key);
                }
            }
        }

        let series_keys: Vec<_> = self
            .series
            .keys()
            .filter(|key| {
                matches!(
                    self.scoped_var_kinds
                        .get(&(key.name.clone(), key.call_site)),
                    Some(PersistentVarKind::Var)
                )
            })
            .cloned()
            .collect();

        for key in series_keys {
            match self.committed_series.get(&key).cloned() {
                Some(series) => {
                    self.series.insert(key.clone(), series);
                }
                None => {
                    self.series.shift_remove(&key);
                }
            }

            match self.committed_series_last_bar.get(&key).copied() {
                Some(last_bar) => {
                    self.series_last_bar.insert(key, last_bar);
                }
                None => {
                    self.series_last_bar.shift_remove(&key);
                }
            }
        }
    }

    /// Set current `barstate.*` flags.
    pub fn set_bar_state(&mut self, bar_state: BarState) {
        self.bar_state = bar_state;
    }

    /// Get current `barstate.*` flags.
    pub fn bar_state(&self) -> &BarState {
        &self.bar_state
    }

    //========================================================================
    // Recursion Tracking
    //========================================================================

    /// Enter a function call (increment recursion depth)
    ///
    /// Returns false if max recursion depth would be exceeded.
    pub fn enter_call(&mut self) -> bool {
        if self.recursion_depth >= self.config.max_recursion_depth {
            return false;
        }
        self.recursion_depth += 1;
        true
    }

    /// Exit a function call (decrement recursion depth)
    pub fn exit_call(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    /// Get current recursion depth
    pub fn recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    //========================================================================
    // Bar Management
    //========================================================================

    /// Advance to the next bar
    ///
    /// This:
    /// - Clears simple variables
    /// - Increments bar index
    /// - Preserves persistent variables and series
    pub fn next_bar(&mut self) {
        self.clear_vars();
        self.bar_index += 1;
        self.bar_state = BarState::default();
    }

    /// Reset the context
    ///
    /// This:
    /// - Clears all variables
    /// - Clears all series
    /// - Resets bar index
    /// - Keeps persistent vars (unless `full` is true)
    pub fn reset(&mut self, full: bool) {
        self.clear_vars();
        self.clear_series();
        self.bar_index = 0;
        self.timestamp = 0;
        self.recursion_depth = 0;
        self.next_call_site_id = 1;
        self.bar_state = BarState::default();
        self.committed_series.clear();
        self.committed_series_last_bar.clear();
        self.committed_var_scoped.clear();

        if full {
            self.persistent_vars.clear();
            self.varip_vars.clear();
            self.var_scoped.clear();
            self.scoped_var_kinds.clear();
        }
    }

    /// Get debug information
    pub fn debug_info(&self) -> ContextDebugInfo {
        ContextDebugInfo {
            bar_index: self.bar_index,
            var_count: self.variables.len(),
            series_count: self.series.len(),
            persistent_count: self.persistent_vars.len(),
            varip_count: self.varip_vars.len(),
            recursion_depth: self.recursion_depth,
        }
    }

    //========================================================================
    // Module System
    //========================================================================

    /// Get a reference to the module registry
    pub fn module_registry(&self) -> &ModuleRegistry {
        &self.module_registry
    }

    /// Get a mutable reference to the module registry
    pub fn module_registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.module_registry
    }

    /// Set the current module ID (when executing a library)
    pub fn set_current_module(&mut self, module_id: Option<ModuleId>) {
        self.current_module = module_id;
    }

    /// Get the current module ID
    pub fn current_module(&self) -> Option<ModuleId> {
        self.current_module
    }

    /// Add a module namespace binding (from import statement)
    pub fn add_module_namespace(&mut self, alias: impl Into<String>, module_id: ModuleId) {
        let alias = alias.into();
        let ns = ModuleNamespace::new(module_id, alias.clone());
        self.module_namespaces.insert(alias, ns);
    }

    /// Get a module namespace by alias
    pub fn get_module_namespace(&self, alias: &str) -> Option<&ModuleNamespace> {
        self.module_namespaces.get(alias)
    }

    /// Check if a namespace alias exists
    pub fn has_module_namespace(&self, alias: &str) -> bool {
        self.module_namespaces.contains_key(alias)
    }

    /// Get the base path for resolving relative imports
    pub fn base_path(&self) -> &std::path::Path {
        &self.base_path
    }

    /// Set the base path for resolving relative imports
    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
        self.base_path = path.into();
    }
}

/// Debug information about context state
#[derive(Debug, Clone)]
pub struct ContextDebugInfo {
    /// Current bar index
    pub bar_index: i64,
    /// Number of simple variables
    pub var_count: usize,
    /// Number of series buffers
    pub series_count: usize,
    /// Number of persistent variables
    pub persistent_count: usize,
    /// Number of varip variables
    pub varip_count: usize,
    /// Current recursion depth
    pub recursion_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context() -> ExecutionContext {
        ExecutionContext::new(Arc::new(RuntimeConfig::default()))
    }

    #[test]
    fn test_variable_management() {
        let mut ctx = test_context();

        ctx.set_var("x", Value::Int(10));
        assert_eq!(ctx.get_var("x"), Some(&Value::Int(10)));

        ctx.set_var("y", Value::Float(3.14));
        assert_eq!(ctx.get_var("y"), Some(&Value::Float(3.14)));

        assert!(ctx.has_var("x"));
        assert!(!ctx.has_var("z"));

        ctx.remove_var("x");
        assert!(!ctx.has_var("x"));
    }

    #[test]
    fn test_series_management() {
        let mut ctx = test_context();
        let cs = ExecutionContext::global_call_site();

        // One committed value per bar; history deepens when `bar_index` advances.
        ctx.push_to_series("close", cs, Value::Float(100.0));
        ctx.set_bar_index(1);
        ctx.push_to_series("close", cs, Value::Float(101.0));
        ctx.set_bar_index(2);
        ctx.push_to_series("close", cs, Value::Float(102.0));

        assert_eq!(
            ctx.get_series_current("close", cs),
            Some(&Value::Float(102.0))
        );
        assert_eq!(
            ctx.get_series_at("close", cs, 1),
            Some(&Value::Float(101.0))
        );
        assert_eq!(
            ctx.get_series_at("close", cs, 2),
            Some(&Value::Float(100.0))
        );
    }

    #[test]
    fn test_series_same_bar_overwrites_current() {
        let mut ctx = test_context();
        let cs = ExecutionContext::global_call_site();
        ctx.push_to_series("close", cs, Value::Float(100.0));
        ctx.push_to_series("close", cs, Value::Float(101.0));
        assert_eq!(
            ctx.get_series_current("close", cs),
            Some(&Value::Float(101.0))
        );
        assert_eq!(ctx.get_series_at("close", cs, 1), None);
    }

    #[test]
    fn test_call_site_isolation() {
        let mut ctx = test_context();
        let cs1 = ExecutionContext::global_call_site();
        let cs2 = ctx.new_call_site();

        ctx.push_to_series("count", cs1, Value::Int(1));
        ctx.push_to_series("count", cs2, Value::Int(100));

        // Different call sites should have different values
        assert_eq!(ctx.get_series_current("count", cs1), Some(&Value::Int(1)));
        assert_eq!(ctx.get_series_current("count", cs2), Some(&Value::Int(100)));
    }

    #[test]
    fn test_persistent_vars() {
        let mut ctx = test_context();

        ctx.set_persistent_var("total", Value::Int(0));
        *ctx.get_or_insert_persistent("total", Value::Int(0)) = Value::Int(10);

        ctx.next_bar(); // Advance to next bar

        // Simple vars should be cleared
        assert!(!ctx.has_var("temp"));

        // Persistent vars should remain
        assert_eq!(ctx.get_persistent_var("total"), Some(&Value::Int(10)));
    }

    #[test]
    fn test_rollback_restores_var_but_keeps_varip() {
        let mut ctx = test_context();
        let cs = ExecutionContext::global_call_site();

        ctx.declare_var_scoped("v", cs, PersistentVarKind::Var, Value::Int(1));
        ctx.push_to_series("v", cs, Value::Int(1));
        ctx.declare_var_scoped("vip", cs, PersistentVarKind::Varip, Value::Int(10));
        ctx.push_to_series("vip", cs, Value::Int(10));
        ctx.commit_bar_state();

        ctx.set_bar_index(1);
        ctx.set_var_scoped("v", cs, Value::Int(2));
        ctx.push_to_series("v", cs, Value::Int(2));
        ctx.set_var_scoped("vip", cs, Value::Int(20));
        ctx.push_to_series("vip", cs, Value::Int(20));

        ctx.rollback_bar_state();

        assert_eq!(ctx.get_var_scoped("v", cs), Some(&Value::Int(1)));
        assert_eq!(ctx.get_series_current("v", cs), Some(&Value::Int(1)));
        assert_eq!(ctx.get_var_scoped("vip", cs), Some(&Value::Int(20)));
        assert_eq!(ctx.get_series_current("vip", cs), Some(&Value::Int(20)));
    }

    #[test]
    fn test_commit_advances_snapshot_to_latest_bar() {
        let mut ctx = test_context();
        let cs = ExecutionContext::global_call_site();

        ctx.declare_var_scoped("count", cs, PersistentVarKind::Var, Value::Int(1));
        ctx.push_to_series("count", cs, Value::Int(1));
        ctx.commit_bar_state();

        ctx.set_bar_index(1);
        ctx.set_var_scoped("count", cs, Value::Int(2));
        ctx.push_to_series("count", cs, Value::Int(2));
        ctx.commit_bar_state();

        ctx.set_bar_index(2);
        ctx.set_var_scoped("count", cs, Value::Int(99));
        ctx.push_to_series("count", cs, Value::Int(99));
        ctx.rollback_bar_state();

        assert_eq!(ctx.get_var_scoped("count", cs), Some(&Value::Int(2)));
        assert_eq!(ctx.get_series_current("count", cs), Some(&Value::Int(2)));
        assert_eq!(ctx.get_series_at("count", cs, 1), Some(&Value::Int(1)));
    }

    #[test]
    fn test_bar_state_round_trip() {
        let mut ctx = test_context();
        let state = BarState {
            ishistory: false,
            isrealtime: true,
            isnew: true,
            isconfirmed: false,
            islast: true,
            isfirst: false,
        };

        ctx.set_bar_state(state.clone());

        assert_eq!(ctx.bar_state(), &state);
    }

    #[test]
    fn test_recursion_tracking() {
        let mut ctx = test_context();

        assert_eq!(ctx.recursion_depth(), 0);

        assert!(ctx.enter_call());
        assert_eq!(ctx.recursion_depth(), 1);

        assert!(ctx.enter_call());
        assert_eq!(ctx.recursion_depth(), 2);

        ctx.exit_call();
        assert_eq!(ctx.recursion_depth(), 1);

        ctx.exit_call();
        assert_eq!(ctx.recursion_depth(), 0);
    }

    #[test]
    fn test_recursion_limit() {
        let config = Arc::new(RuntimeConfig::default().with_max_recursion_depth(2));
        let mut ctx = ExecutionContext::new(config);

        assert!(ctx.enter_call()); // depth 1
        assert!(ctx.enter_call()); // depth 2
        assert!(!ctx.enter_call()); // would be depth 3, exceeds limit
    }

    #[test]
    fn test_bar_advance() {
        let mut ctx = test_context();

        ctx.set_var("temp", Value::Int(5));
        ctx.set_bar_index(10);

        ctx.next_bar();

        assert_eq!(ctx.bar_index(), 11);
        assert!(!ctx.has_var("temp")); // Simple vars cleared
    }

    #[test]
    fn test_reset() {
        let mut ctx = test_context();
        let cs = ExecutionContext::global_call_site();

        ctx.set_var("x", Value::Int(1));
        ctx.push_to_series("s", cs, Value::Int(2));
        ctx.set_persistent_var("p", Value::Int(3));

        ctx.reset(false);

        assert!(!ctx.has_var("x"));
        assert!(ctx.get_series("s", cs).is_none());
        assert_eq!(ctx.get_persistent_var("p"), Some(&Value::Int(3))); // Preserved

        ctx.reset(true);
        assert!(ctx.get_persistent_var("p").is_none()); // Now cleared
    }
}
