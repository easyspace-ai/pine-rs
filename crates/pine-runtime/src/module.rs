//! Module system for Pine Script v6
//!
//! This module provides the infrastructure for `import`/`export`/`library`
//! functionality in Pine Script v6.
//!
//! A module represents a loaded library with its exported functions and variables.
//! The module registry tracks all loaded modules and prevents circular dependencies.

use crate::value::Value;
use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A unique identifier for a loaded module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub usize);

/// A loaded Pine Script module (library)
///
/// Modules are created when a script with a `library()` declaration is loaded
/// via `import`. They contain the exported functions and variables.
#[derive(Debug, Clone)]
pub struct Module {
    /// Unique identifier for this module
    pub id: ModuleId,
    /// Module name (from library() declaration)
    pub name: String,
    /// Path to the source file
    pub path: PathBuf,
    /// Exported values (functions, variables, types)
    pub exports: IndexMap<String, Value>,
    /// Module properties from library() declaration
    pub props: IndexMap<String, Value>,
}

impl Module {
    /// Create a new module
    pub fn new(id: ModuleId, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id,
            name: name.into(),
            path: path.into(),
            exports: IndexMap::new(),
            props: IndexMap::new(),
        }
    }

    /// Export a value from this module
    pub fn export(&mut self, name: impl Into<String>, value: Value) {
        self.exports.insert(name.into(), value);
    }

    /// Get an exported value by name
    pub fn get_export(&self, name: &str) -> Option<&Value> {
        self.exports.get(name)
    }

    /// Check if a name is exported
    pub fn has_export(&self, name: &str) -> bool {
        self.exports.contains_key(name)
    }

    /// Set a module property
    pub fn set_prop(&mut self, name: impl Into<String>, value: Value) {
        self.props.insert(name.into(), value);
    }

    /// Get a module property
    pub fn get_prop(&self, name: &str) -> Option<&Value> {
        self.props.get(name)
    }
}

/// Error types for module operations
#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    /// Module not found at the specified path
    #[error("module not found: {path}")]
    NotFound {
        /// Path that was not found
        path: PathBuf,
    },

    /// Circular dependency detected
    #[error("circular dependency detected: {cycle}")]
    CircularDependency {
        /// Dependency cycle description
        cycle: String,
    },

    /// Export not found in module
    #[error("'{name}' is not exported from module '{module}'")]
    ExportNotFound {
        /// Name of the export that was not found
        name: String,
        /// Name of the module
        module: String,
    },

    /// Invalid module path
    #[error("invalid module path: {path}")]
    InvalidPath {
        /// Invalid path
        path: PathBuf,
    },

    /// Module already loaded with different path
    #[error("module '{name}' already loaded from different path")]
    DuplicateModule {
        /// Name of the duplicate module
        name: String,
    },

    /// IO error loading module
    #[error("failed to load module: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for module operations
pub type Result<T> = std::result::Result<T, ModuleError>;

/// Registry of loaded modules
///
/// The registry tracks all loaded modules and provides resolution
/// for import statements. It prevents circular dependencies.
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    /// All loaded modules by ID
    modules: IndexMap<ModuleId, Arc<Module>>,
    /// Module lookup by name
    by_name: IndexMap<String, ModuleId>,
    /// Module lookup by absolute path
    by_path: IndexMap<PathBuf, ModuleId>,
    /// Next module ID
    next_id: usize,
    /// Currently loading modules (for circular dependency detection)
    loading_stack: Vec<String>,
}

impl ModuleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            modules: IndexMap::new(),
            by_name: IndexMap::new(),
            by_path: IndexMap::new(),
            next_id: 1, // 0 is reserved
            loading_stack: Vec::new(),
        }
    }

    /// Generate a new module ID
    fn next_module_id(&mut self) -> ModuleId {
        let id = self.next_id;
        self.next_id += 1;
        ModuleId(id)
    }

    /// Check if a module is currently being loaded (circular dependency check)
    fn is_loading(&self, path: &str) -> bool {
        self.loading_stack.iter().any(|p| p == path)
    }

    /// Begin loading a module (push to stack)
    fn begin_load(&mut self, path: impl Into<String>) {
        self.loading_stack.push(path.into());
    }

    /// Finish loading a module (pop from stack)
    fn end_load(&mut self) {
        self.loading_stack.pop();
    }

    /// Get the current dependency chain (for error messages)
    fn dependency_chain(&self) -> String {
        self.loading_stack.join(" -> ")
    }

    /// Register a newly loaded module
    pub fn register(&mut self, module: Module) -> ModuleId {
        let id = module.id;
        let name = module.name.clone();
        let path = module.path.clone();

        let arc_module = Arc::new(module);
        self.modules.insert(id, arc_module);
        self.by_name.insert(name, id);
        self.by_path.insert(path, id);

        id
    }

    /// Get a module by ID
    pub fn get(&self, id: ModuleId) -> Option<Arc<Module>> {
        self.modules.get(&id).cloned()
    }

    /// Get a module by name
    pub fn get_by_name(&self, name: &str) -> Option<Arc<Module>> {
        self.by_name.get(name).and_then(|id| self.get(*id))
    }

    /// Get a module by path
    pub fn get_by_path(&self, path: &Path) -> Option<Arc<Module>> {
        self.by_path.get(path).and_then(|id| self.get(*id))
    }

    /// Check if a module is loaded (by path)
    pub fn is_loaded(&self, path: &Path) -> bool {
        self.by_path.contains_key(path)
    }

    /// Get or load a module
    ///
    /// This is the main entry point for resolving imports.
    /// Returns the module ID if found or loaded successfully.
    pub fn resolve(&mut self, path: &Path) -> Result<ModuleId> {
        // Check if already loaded
        if let Some(id) = self.by_path.get(path) {
            return Ok(*id);
        }

        // Check for circular dependency
        let path_str = path.to_string_lossy().to_string();
        if self.is_loading(&path_str) {
            let cycle = format!("{} -> {}", self.dependency_chain(), path_str);
            return Err(ModuleError::CircularDependency { cycle });
        }

        // Mark as loading
        self.begin_load(path_str);

        // Module will need to be loaded by the caller
        // They will call register() after loading

        self.end_load();
        Ok(self.next_module_id())
    }

    /// Get all loaded modules
    pub fn all_modules(&self) -> impl Iterator<Item = &Arc<Module>> {
        self.modules.values()
    }

    /// Clear all modules
    pub fn clear(&mut self) {
        self.modules.clear();
        self.by_name.clear();
        self.by_path.clear();
        self.next_id = 1;
        self.loading_stack.clear();
    }
}

/// Module namespace in the execution context
///
/// When a module is imported with an alias (e.g., `import "utils" as u`),
/// all exported values are accessible via the namespace (e.g., `u.my_func()`).
#[derive(Debug, Clone)]
pub struct ModuleNamespace {
    /// Module ID
    pub module_id: ModuleId,
    /// Alias used in the import statement
    pub alias: String,
}

impl ModuleNamespace {
    /// Create a new namespace binding
    pub fn new(module_id: ModuleId, alias: impl Into<String>) -> Self {
        Self {
            module_id,
            alias: alias.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Closure;

    #[test]
    fn test_module_creation() {
        let module = Module::new(ModuleId(1), "test_lib", "/path/to/lib.pine");
        assert_eq!(module.name, "test_lib");
        assert_eq!(module.path, PathBuf::from("/path/to/lib.pine"));
        assert!(module.exports.is_empty());
    }

    #[test]
    fn test_module_exports() {
        let mut module = Module::new(ModuleId(1), "test_lib", "/path/to/lib.pine");

        module.export("add", Value::from(Closure::new("add", vec![])));
        module.export("PI", Value::from(3.14159));

        assert!(module.has_export("add"));
        assert!(module.has_export("PI"));
        assert!(!module.has_export("missing"));

        assert!(module.get_export("add").unwrap().is_closure());
        assert!(module.get_export("PI").unwrap().is_number());
    }

    #[test]
    fn test_module_registry() {
        let mut registry = ModuleRegistry::new();

        // Register a module
        let module = Module::new(ModuleId(1), "math", "/lib/math.pine");
        let id = registry.register(module);

        // Retrieve by ID
        let retrieved = registry.get(id).unwrap();
        assert_eq!(retrieved.name, "math");

        // Retrieve by name
        let by_name = registry.get_by_name("math").unwrap();
        assert_eq!(by_name.id, id);

        // Retrieve by path
        let by_path = registry.get_by_path(Path::new("/lib/math.pine")).unwrap();
        assert_eq!(by_path.id, id);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut registry = ModuleRegistry::new();

        // Simulate loading A -> B -> C -> A
        registry.begin_load("/lib/a.pine");
        registry.begin_load("/lib/b.pine");
        registry.begin_load("/lib/c.pine");

        // Trying to load A again should detect the cycle
        assert!(registry.is_loading("/lib/a.pine"));
        assert!(!registry.is_loading("/lib/d.pine"));

        assert_eq!(
            registry.dependency_chain(),
            "/lib/a.pine -> /lib/b.pine -> /lib/c.pine"
        );
    }

    #[test]
    fn test_module_namespace() {
        let ns = ModuleNamespace::new(ModuleId(42), "math");
        assert_eq!(ns.module_id, ModuleId(42));
        assert_eq!(ns.alias, "math");
    }
}
