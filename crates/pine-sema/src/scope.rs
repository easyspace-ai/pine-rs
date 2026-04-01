//! Scope and symbol table for semantic analysis
//!
//! This module provides scope management with slot-based variable indexing
//! for efficient runtime access (inspired by Rhai).

use crate::types::PineType;
use crate::types::TypeDef;
use indexmap::IndexMap;
use std::collections::HashMap;

/// A unique slot index for variable storage
///
/// Variables are assigned sequential slot indices during semantic analysis,
/// allowing O(1) array-based lookup at runtime instead of hash map lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotId(pub usize);

impl SlotId {
    /// Get the slot index as a usize
    #[inline]
    pub fn index(&self) -> usize {
        self.0
    }
}

/// Symbol table entry kind
#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// Variable (with slot index for runtime storage)
    Variable {
        /// The slot index assigned to this variable for O(1) runtime access
        slot: SlotId,
    },
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
    /// Get the slot index if this is a variable
    pub fn slot(&self) -> Option<SlotId> {
        match self.kind {
            SymbolKind::Variable { slot } => Some(slot),
            _ => None,
        }
    }
}

impl Symbol {
    /// Create a new variable symbol with a specific slot
    pub fn var_with_slot(name: impl Into<String>, ty: PineType, slot: SlotId) -> Self {
        Self {
            name: name.into(),
            kind: SymbolKind::Variable { slot },
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
///
/// Manages variable slot allocation for efficient runtime access.
/// Each variable is assigned a unique slot index that can be used
/// for O(1) array-based lookup at runtime.
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Global symbols
    globals: HashMap<String, Symbol>,
    /// User-defined type definitions
    type_defs: HashMap<String, TypeDef>,
    /// Current scope stack
    scopes: Vec<Scope>,
    /// Next available slot index for variable allocation
    next_slot: usize,
    /// Total number of slots needed (max slot index + 1)
    total_slots: usize,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new slot index for a variable
    ///
    /// Returns the allocated slot ID and updates the total slot count.
    pub fn allocate_slot(&mut self) -> SlotId {
        let slot = SlotId(self.next_slot);
        self.next_slot += 1;
        self.total_slots = self.total_slots.max(self.next_slot);
        slot
    }

    /// Get the total number of slots needed
    ///
    /// This should be used by the runtime to pre-allocate storage.
    pub fn total_slots(&self) -> usize {
        self.total_slots
    }

    /// Reset the slot allocator (e.g., when entering a new function scope)
    ///
    /// Returns the previous next_slot value so it can be restored later.
    pub fn reset_slot_allocator(&mut self) -> usize {
        let prev = self.next_slot;
        self.next_slot = 0;
        prev
    }

    /// Restore the slot allocator to a previous value
    pub fn restore_slot_allocator(&mut self, prev_slot: usize) {
        self.next_slot = prev_slot;
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

    /// Define a variable with automatic slot allocation
    ///
    /// This is the preferred method for defining variables as it
    /// automatically assigns a unique slot index for runtime access.
    pub fn define_var(&mut self, name: impl Into<String>, ty: PineType) -> SlotId {
        let slot = self.allocate_slot();
        let symbol = Symbol::var_with_slot(name, ty, slot);
        if let Some(scope) = self.scopes.last_mut() {
            scope.define(symbol.name.clone(), symbol);
        } else {
            self.globals.insert(symbol.name.clone(), symbol);
        }
        slot
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

    /// Look up a symbol and return its slot index if it's a variable
    pub fn lookup_slot(&self, name: &str) -> Option<SlotId> {
        self.lookup(name).and_then(|s| s.slot())
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
