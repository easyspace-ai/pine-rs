//! Pine Script v6 Semantic Analysis
//!
//! This crate provides semantic analysis for Pine Script, including:
//! - Type inference and checking
//! - Scope resolution
//! - Series annotation
//! - var/varip lifting

#![warn(missing_docs)]

pub mod infer;
pub mod scope;
pub mod types;

use pine_lexer::Span;
use pine_parser::ast as ast;
use scope::SymbolTable;
use thiserror::Error;
use types::{PineType, TypeDef};

/// Semantic analysis errors
#[derive(Debug, Error)]
pub enum SemaError {
    /// Placeholder error
    #[error("semantic analysis not yet implemented")]
    NotImplemented,

    /// Undefined symbol
    #[error("undefined symbol: {name}")]
    UndefinedSymbol {
        /// Symbol name
        name: String,
        /// Span where the symbol was referenced
        span: Span,
    },

    /// Undefined type
    #[error("undefined type: {name}")]
    UndefinedType {
        /// Type name
        name: String,
        /// Span where the type was referenced
        span: Span,
    },

    /// Type mismatch
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        /// Expected type
        expected: PineType,
        /// Found type
        found: PineType,
        /// Span of the expression
        span: Span,
    },

    /// Undefined field
    #[error("type {type_name} has no field {field_name}")]
    UndefinedField {
        /// Type name
        type_name: String,
        /// Field name
        field_name: String,
        /// Span of the field access
        span: Span,
    },

    /// Undefined method
    #[error("type {type_name} has no method {method_name}")]
    UndefinedMethod {
        /// Type name
        type_name: String,
        /// Method name
        method_name: String,
        /// Span of the method call
        span: Span,
    },

    /// Duplicate type definition
    #[error("type {name} is already defined")]
    DuplicateType {
        /// Type name
        name: String,
        /// Span of the duplicate definition
        span: Span,
    },

    /// Duplicate symbol definition
    #[error("symbol {name} is already defined in this scope")]
    DuplicateSymbol {
        /// Symbol name
        name: String,
        /// Span of the duplicate definition
        span: Span,
    },
}

/// Result type for semantic analysis operations
pub type Result<T> = std::result::Result<T, SemaError>;

/// Semantic analyzer for Pine Script
#[derive(Debug)]
pub struct SemanticAnalyzer {
    /// Symbol table
    symbol_table: SymbolTable,
    /// Collected errors
    errors: Vec<SemaError>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    /// Analyze a script AST
    pub fn analyze(&mut self, script: &ast::Script) -> Result<()> {
        // First pass: collect all type definitions
        for stmt in &script.stmts {
            if let ast::Stmt::TypeDef { name, fields, span } = stmt {
                self.define_type(name, fields, *span)?;
            }
        }

        // Second pass: collect all method definitions
        for stmt in &script.stmts {
            if let ast::Stmt::MethodDef {
                type_name,
                name,
                params,
                ret_type,
                ..
            } = stmt
            {
                self.define_method(type_name, name, params, ret_type.as_ref())?;
            }
        }

        // Third pass: analyze statements
        for stmt in &script.stmts {
            self.analyze_stmt(stmt)?;
        }

        Ok(())
    }

    /// Define a user-defined type
    fn define_type(&mut self, name: &ast::Ident, fields: &[ast::Field], span: Span) -> Result<()> {
        // Check for duplicate type definition
        if self.symbol_table.lookup_type(&name.name).is_some() {
            return Err(SemaError::DuplicateType {
                name: name.name.clone(),
                span,
            });
        }

        let mut type_def = TypeDef::new(&name.name);

        // Add fields to the type definition
        for field in fields {
            let field_type = self.convert_type_ann(field.type_ann.as_ref());
            type_def.add_field(
                &field.name.name,
                field_type,
                field.default.as_ref().map(|_| "TODO".to_string()),
            );
        }

        self.symbol_table.define_type(type_def);
        Ok(())
    }

    /// Define a method for a user-defined type
    fn define_method(
        &mut self,
        type_name: &ast::Ident,
        name: &ast::Ident,
        params: &[ast::Param],
        ret_type: Option<&ast::TypeAnn>,
    ) -> Result<()> {
        // Check if the type exists first
        if self.symbol_table.lookup_type(&type_name.name).is_none() {
            return Err(SemaError::UndefinedType {
                name: type_name.name.clone(),
                span: type_name.span,
            });
        }

        // Convert parameter types (before getting mutable reference)
        let param_types: Vec<PineType> = params
            .iter()
            .map(|p| self.convert_type_ann(p.type_ann.as_ref()))
            .collect();

        // Convert return type (before getting mutable reference)
        let return_type = self.convert_type_ann(ret_type);
        let method_name = name.name.clone();

        // Now get the type definition mutably and add the method
        let Some(type_def) = self.symbol_table.lookup_type_mut(&type_name.name) else {
            return Err(SemaError::UndefinedType {
                name: type_name.name.clone(),
                span: type_name.span,
            });
        };

        type_def.add_method(method_name, param_types, return_type);

        Ok(())
    }

    /// Convert an AST type annotation to a PineType
    fn convert_type_ann(&self, type_ann: Option<&ast::TypeAnn>) -> PineType {
        match type_ann {
            Some(ann) => self.convert_type_ann_inner(ann),
            None => PineType::Unknown,
        }
    }

    /// Convert an AST type annotation to a PineType (inner helper)
    fn convert_type_ann_inner(&self, type_ann: &ast::TypeAnn) -> PineType {
        match type_ann {
            ast::TypeAnn::Simple(name) => match name.as_str() {
                "int" => PineType::Int,
                "float" => PineType::Float,
                "bool" => PineType::Bool,
                "string" => PineType::String,
                "color" => PineType::Color,
                _ => {
                    // Check if it's a UDT
                    if self.symbol_table.lookup_type(name).is_some() {
                        PineType::Udt(name.clone())
                    } else {
                        PineType::Error
                    }
                }
            },
            ast::TypeAnn::Series(inner) => PineType::Series(Box::new(self.convert_type_ann_inner(inner))),
            ast::TypeAnn::Array(inner) => PineType::Array(Box::new(self.convert_type_ann_inner(inner))),
            ast::TypeAnn::Matrix(inner) => PineType::Matrix(Box::new(self.convert_type_ann_inner(inner))),
            ast::TypeAnn::Map(key, value) => PineType::Map(
                Box::new(self.convert_type_ann_inner(key)),
                Box::new(self.convert_type_ann_inner(value)),
            ),
            ast::TypeAnn::User(name) => PineType::Udt(name.clone()),
        }
    }

    /// Analyze a single statement
    fn analyze_stmt(&mut self, stmt: &ast::Stmt) -> Result<()> {
        match stmt {
            ast::Stmt::TypeDef { .. } | ast::Stmt::MethodDef { .. } => {
                // Already handled in first/second pass
                Ok(())
            }
            ast::Stmt::VarDecl { name, type_ann, .. } => {
                // Define the variable in the current scope
                let ty = self.convert_type_ann(type_ann.as_ref());
                self.symbol_table
                    .define(name.name.clone(), scope::Symbol::var(&name.name, ty));
                Ok(())
            }
            _ => {
                // TODO: Handle other statement types
                Ok(())
            }
        }
    }

    /// Get the type of an expression
    pub fn expr_type(&self, expr: &ast::Expr) -> Result<PineType> {
        match expr {
            ast::Expr::Ident(ident) => {
                let symbol = self.symbol_table.lookup(&ident.name).ok_or_else(|| {
                    SemaError::UndefinedSymbol {
                        name: ident.name.clone(),
                        span: ident.span,
                    }
                })?;
                Ok(symbol.ty.clone())
            }
            ast::Expr::FieldAccess { base, field, span } => {
                let base_type = self.expr_type(base)?;
                match base_type {
                    PineType::Udt(type_name) => {
                        let type_def = self.symbol_table.lookup_type(&type_name).ok_or_else(|| {
                            SemaError::UndefinedType {
                                name: type_name.clone(),
                                span: *span,
                            }
                        })?;
                        let field_def = type_def.get_field(&field.name).ok_or_else(|| {
                            SemaError::UndefinedField {
                                type_name,
                                field_name: field.name.clone(),
                                span: *span,
                            }
                        })?;
                        Ok(field_def.ty.clone())
                    }
                    _ => Err(SemaError::TypeMismatch {
                        expected: PineType::Udt("".to_string()),
                        found: base_type,
                        span: *span,
                    }),
                }
            }
            ast::Expr::MethodCall { base, method, span, .. } => {
                let base_type = self.expr_type(base)?;
                match base_type {
                    PineType::Udt(type_name) => {
                        let type_def = self.symbol_table.lookup_type(&type_name).ok_or_else(|| {
                            SemaError::UndefinedType {
                                name: type_name.clone(),
                                span: *span,
                            }
                        })?;
                        let method_def = type_def.get_method(&method.name).ok_or_else(|| {
                            SemaError::UndefinedMethod {
                                type_name,
                                method_name: method.name.clone(),
                                span: *span,
                            }
                        })?;
                        Ok(method_def.ret_ty.clone())
                    }
                    _ => Err(SemaError::TypeMismatch {
                        expected: PineType::Udt("".to_string()),
                        found: base_type,
                        span: *span,
                    }),
                }
            }
            _ => {
                // TODO: Handle other expression types
                Ok(PineType::Unknown)
            }
        }
    }

    /// Get the collected errors
    pub fn errors(&self) -> &[SemaError] {
        &self.errors
    }

    /// Consume the analyzer and return the symbol table
    pub fn into_symbol_table(self) -> SymbolTable {
        self.symbol_table
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Run semantic analysis on the parsed AST
pub fn analyze(script: &ast::Script) -> Result<SymbolTable> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(script)?;
    Ok(analyzer.into_symbol_table())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Test will be expanded as we implement more functionality
        assert!(true);
    }
}
