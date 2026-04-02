//! Function registry for Pine Script standard library
//!
//! This module provides a hash-based dispatch system for built-in functions,
//! inspired by Rhai's function dispatch mechanism.
//!
//! # Hash Dispatch with Bloom Filter
//!
//! The registry uses a two-level lookup system for optimal performance:
//!
//! 1. **Bloom Filter**: Quickly check if a function might exist (no false negatives)
//! 2. **Hash Map**: Actual function lookup with O(1) complexity
//!
//! This avoids expensive string comparisons for non-existent functions.

use indexmap::IndexMap;
use pine_runtime::value::Value;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A simple Bloom filter for fast negative checks
///
/// Bloom filters allow us to quickly determine that a function definitely
/// does NOT exist, avoiding expensive hash table lookups for typos or
/// undefined functions.
#[derive(Debug, Clone)]
pub struct BloomFilter {
    /// Bit array (using u64 for efficiency)
    bits: Vec<u64>,
    /// Number of hash functions
    k: usize,
    /// Size in bits
    size: usize,
}

impl Default for BloomFilter {
    fn default() -> Self {
        Self::default_for_functions()
    }
}

impl BloomFilter {
    /// Create a new Bloom filter with expected number of items and false positive rate
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        // Calculate optimal size: m = -n * ln(p) / (ln(2)^2)
        let size = (-(expected_items as f64) * false_positive_rate.ln() / (2.0f64.ln().powi(2)))
            .ceil() as usize;
        // Calculate optimal k: k = m/n * ln(2)
        let k = ((size as f64 / expected_items as f64) * 2.0f64.ln()).ceil() as usize;
        let k = k.max(1);

        let num_u64s = size.div_ceil(64);
        Self {
            bits: vec![0; num_u64s],
            k,
            size: num_u64s * 64,
        }
    }

    /// Create a default Bloom filter (suitable for ~100 functions)
    pub fn default_for_functions() -> Self {
        Self::new(100, 0.01)
    }

    /// Hash a string to a bit index
    #[inline]
    fn hash_index(&self, s: &str, seed: u64) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        seed.hash(&mut hasher);
        let hash = hasher.finish();
        (hash as usize) % self.size
    }

    /// Add an item to the filter
    pub fn add(&mut self, item: &str) {
        for i in 0..self.k {
            let idx = self.hash_index(item, i as u64);
            let (word, bit) = (idx / 64, idx % 64);
            self.bits[word] |= 1u64 << bit;
        }
    }

    /// Check if an item might be in the set
    ///
    /// Returns `false` if the item is definitely NOT in the set.
    /// Returns `true` if the item MIGHT be in the set (could be false positive).
    pub fn might_contain(&self, item: &str) -> bool {
        for i in 0..self.k {
            let idx = self.hash_index(item, i as u64);
            let (word, bit) = (idx / 64, idx % 64);
            if (self.bits[word] & (1u64 << bit)) == 0 {
                return false;
            }
        }
        true
    }

    /// Clear the filter
    pub fn clear(&mut self) {
        for word in &mut self.bits {
            *word = 0;
        }
    }
}

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

/// A hash key for function lookup
///
/// Uses precomputed hash for fast comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionHash(pub u64);

impl FunctionHash {
    /// Compute hash for a function name
    pub fn compute(name: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Get the raw hash value
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Function registry with hash-based dispatch and Bloom filter
///
/// This registry stores built-in functions and provides O(1) lookup
/// by function name using a two-level system:
///
/// 1. Bloom filter for fast negative checks
/// 2. Hash map for actual function lookup
///
/// It supports namespaced functions (e.g., "ta.sma").
#[derive(Default)]
pub struct FunctionRegistry {
    /// Registered functions by full name (for metadata and iteration)
    functions: IndexMap<String, FunctionEntry>,
    /// Hash-based lookup (hash -> entry)
    /// This provides O(1) lookup without string comparison
    hash_lookup: HashMap<FunctionHash, FunctionEntry>,
    /// Bloom filter for fast negative checks
    bloom: BloomFilter,
    /// Fast lookup cache for hot functions (called frequently)
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
        Self {
            functions: IndexMap::new(),
            hash_lookup: HashMap::new(),
            bloom: BloomFilter::default_for_functions(),
            hot_functions: IndexMap::new(),
        }
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
        let hash = FunctionHash::compute(&full_name);

        let entry = FunctionEntry { meta, func };

        // Add to all lookup structures
        self.functions.insert(full_name.clone(), entry.clone());
        self.hash_lookup.insert(hash, entry);
        self.bloom.add(&full_name);
    }

    /// Register with hot path optimization
    ///
    /// Hot functions are stored in a separate cache for even faster dispatch.
    pub fn register_hot(&mut self, meta: FunctionMeta, func: BuiltinFn) {
        let full_name = meta.full_name();
        self.hot_functions.insert(full_name.clone(), func.clone());
        self.register(meta, func);
    }

    /// Look up a function by name (string-based)
    ///
    /// This is the traditional lookup method. For better performance,
    /// consider using `lookup_by_hash` if you have a precomputed hash.
    pub fn lookup(&self, name: &str) -> Option<&FunctionEntry> {
        self.functions.get(name)
    }

    /// Look up a function by precomputed hash
    ///
    /// This provides O(1) lookup without string comparison.
    #[inline]
    pub fn lookup_by_hash(&self, hash: FunctionHash) -> Option<&FunctionEntry> {
        self.hash_lookup.get(&hash)
    }

    /// Fast dispatch for hot functions
    #[inline]
    pub fn dispatch_hot(&self, name: &str, args: &[Value]) -> Option<Value> {
        self.hot_functions.get(name).map(|f| f(args))
    }

    /// Dispatch a function call using hash-based lookup
    ///
    /// Uses the two-level lookup system:
    /// 1. Bloom filter for fast negative check
    /// 2. Hash map for actual function lookup
    pub fn dispatch(&self, name: &str, args: &[Value]) -> Option<Value> {
        // Try hot path first (fastest)
        if let Some(result) = self.dispatch_hot(name, args) {
            return Some(result);
        }

        // Bloom filter check - fast negative check
        if !self.bloom.might_contain(name) {
            return None;
        }

        // Hash-based lookup - O(1) without string comparison
        let hash = FunctionHash::compute(name);
        self.hash_lookup.get(&hash).map(|entry| (entry.func)(args))
    }

    /// Check if a function exists using Bloom filter
    ///
    /// This is fast but may return false positives (rarely).
    /// Use `contains_exact` for definitive checks.
    pub fn might_contain(&self, name: &str) -> bool {
        self.bloom.might_contain(name)
    }

    /// Check if a function exists (exact check)
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
        crate::input::register_functions(self);
    }

    /// Get the number of registered functions
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    /// Get the Bloom filter (for testing/debugging)
    #[cfg(test)]
    #[allow(dead_code)]
    fn bloom_filter(&self) -> &BloomFilter {
        &self.bloom
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
