//! Scope and symbol table for semantic analysis

use crate::types::PineType;
use crate::types::TypeDef;
use indexmap::IndexMap;
use std::collections::HashMap;

/// Symbol table entry kind
#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// Variable
    Variable,
    /// Function
    Function,
    /// Type (UDT)
    Type,
}

/// Symbol table entry
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Symbol type
    pub ty: PineType,
}

impl Symbol {
    /// Create a new variable symbol
    pub fn var(name: impl Into<String>, ty: PineType) -> Self {
        Self {
            name: name.into(),
            kind: SymbolKind::Variable,
            ty,
        }
    }

    /// Create a new function symbol
    pub fn func(name: impl Into<String>, params: Vec<PineType>, ret_ty: PineType) -> Self {
        Self {
            name: name.into(),
            kind: SymbolKind::Function,
            ty: PineType::Function(params, Box::new(ret_ty)),
        }
    }

    /// Create a new type symbol
    pub fn type_symbol(name: impl Into<String>) -> Self {
        let type_name = name.into();
        Self {
            name: type_name.clone(),
            kind: SymbolKind::Type,
            ty: PineType::Udt(type_name),
        }
    }
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
    /// User-defined type definitions
    type_defs: HashMap<String, TypeDef>,
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

    /// Define a type in the global scope
    pub fn define_type(&mut self, type_def: TypeDef) {
        let name = type_def.name.clone();
        self.type_defs.insert(name.clone(), type_def);
        self.globals.insert(name.clone(), Symbol::type_symbol(name));
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

    /// Look up a type definition
    pub fn lookup_type(&self, name: &str) -> Option<&TypeDef> {
        self.type_defs.get(name)
    }

    /// Get a mutable reference to a type definition
    pub fn lookup_type_mut(&mut self, name: &str) -> Option<&mut TypeDef> {
        self.type_defs.get_mut(name)
    }
}
