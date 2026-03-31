//! Scope and symbol table for semantic analysis

use indexmap::IndexMap;
use std::collections::HashMap;

/// Symbol table entry
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol type (as string representation for now)
    pub ty: String,
}

/// Scope for variable resolution
#[derive(Debug, Default)]
pub struct Scope {
    /// Symbols defined in this scope
    symbols: IndexMap<String, Symbol>,
    /// Parent scope (if any)
    parent: Option<Box<Scope>>,
}

impl Scope {
    /// Create a new scope
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new scope with a parent
    pub fn with_parent(parent: Scope) -> Self {
        Self {
            symbols: IndexMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Define a symbol in this scope
    pub fn define(&mut self, name: String, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    /// Look up a symbol in this scope and parent scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }
}

/// Symbol table for the entire program
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Global symbols
    globals: HashMap<String, Symbol>,
    /// Current scope stack
    scopes: Vec<Scope>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self::default()
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a symbol in the current scope
    pub fn define(&mut self, name: String, symbol: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.define(name, symbol);
        } else {
            self.globals.insert(name, symbol);
        }
    }

    /// Look up a symbol
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.lookup(name) {
                return Some(sym);
            }
        }
        self.globals.get(name)
    }
}
