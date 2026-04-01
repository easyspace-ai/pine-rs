//! Type inference for Pine Script
//!
//! This module provides type inference using fixed-point iteration (constraint solving).
//!
//! # Algorithm
//!
//! 1. Collect type constraints from AST traversal
//! 2. Build constraint graph
//! 3. Iteratively solve constraints until fixed point is reached
//! 4. Propagate type information through the graph

use crate::types::PineType;
use std::collections::{HashMap, HashSet};

/// A type variable for inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

/// Type constraint for constraint solving
#[derive(Debug, Clone)]
pub enum Constraint {
    /// lhs must equal rhs
    Eq(TypeVar, TypeVar),
    /// lhs must be a subtype of rhs
    SubType(TypeVar, TypeVar),
    /// type_var must be concrete type
    Concrete(TypeVar, PineType),
    /// type_var must be a series of inner type
    SeriesOf(TypeVar, TypeVar),
}

/// Type inference engine using fixed-point iteration
#[derive(Debug)]
pub struct TypeInference {
    /// Type variables
    vars: Vec<TypeVarInfo>,
    /// Constraints to solve
    constraints: Vec<Constraint>,
    /// Next type variable ID
    next_var: usize,
    /// Type variable substitutions (reserved for future use)
    #[allow(dead_code)]
    substitutions: HashMap<TypeVar, PineType>,
}

/// Information about a type variable
#[derive(Debug, Clone)]
struct TypeVarInfo {
    /// Current inferred type
    ty: Option<PineType>,
    /// Possible types (for union types during inference)
    possible_types: HashSet<PineType>,
    /// Whether this variable is a series
    is_series: bool,
}

impl TypeVarInfo {
    fn new() -> Self {
        Self {
            ty: None,
            possible_types: HashSet::new(),
            is_series: false,
        }
    }
}

impl TypeInference {
    /// Create a new type inference engine
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            constraints: Vec::new(),
            next_var: 0,
            substitutions: HashMap::new(),
        }
    }

    /// Create a fresh type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar(self.next_var);
        self.next_var += 1;
        self.vars.push(TypeVarInfo::new());
        var
    }

    /// Add an equality constraint
    pub fn constrain_eq(&mut self, lhs: TypeVar, rhs: TypeVar) {
        self.constraints.push(Constraint::Eq(lhs, rhs));
    }

    /// Add a subtype constraint
    pub fn constrain_subtype(&mut self, lhs: TypeVar, rhs: TypeVar) {
        self.constraints.push(Constraint::SubType(lhs, rhs));
    }

    /// Add a concrete type constraint
    pub fn constrain_concrete(&mut self, var: TypeVar, ty: PineType) {
        self.constraints.push(Constraint::Concrete(var, ty));
    }

    /// Add a series constraint (var must be series of inner)
    pub fn constrain_series(&mut self, var: TypeVar, inner: TypeVar) {
        self.constraints.push(Constraint::SeriesOf(var, inner));
    }

    /// Solve type constraints using fixed-point iteration
    ///
    /// The algorithm:
    /// 1. Initialize all type variables to Unknown
    /// 2. Iteratively apply constraints until no changes occur
    /// 3. Return the final type assignments
    pub fn solve(&mut self) -> Result<InferenceResult, String> {
        // Initialize with known constraints
        self.initialize();

        // Fixed-point iteration
        let max_iterations = 100;
        for iteration in 0..max_iterations {
            let changed = self.iterate_constraints()?;

            if !changed {
                // Fixed point reached
                return Ok(self.build_result());
            }

            // Check for convergence issues
            if iteration == max_iterations - 1 {
                return Err("Type inference did not converge".to_string());
            }
        }

        Ok(self.build_result())
    }

    /// Initialize type variables from constraints
    fn initialize(&mut self) {
        // Apply concrete constraints first
        for constraint in &self.constraints {
            if let Constraint::Concrete(var, ty) = constraint {
                let idx = var.0;
                if idx < self.vars.len() {
                    self.vars[idx].ty = Some(ty.clone());
                }
            }
        }
    }

    /// One iteration of constraint solving
    fn iterate_constraints(&mut self) -> Result<bool, String> {
        let mut changed = false;

        // Clone constraints to avoid borrow issues
        let constraints: Vec<Constraint> = self.constraints.clone();

        for constraint in &constraints {
            match constraint {
                Constraint::Eq(lhs, rhs) => {
                    let lhs_ty = self.get_var_type(*lhs);
                    let rhs_ty = self.get_var_type(*rhs);

                    match (&lhs_ty, &rhs_ty) {
                        (Some(l), Some(r)) => {
                            if !types_compatible(l, r) {
                                return Err(format!(
                                    "Type mismatch: expected {}, found {}",
                                    l, r
                                ));
                            }
                            // Unify the types and update both variables
                            let unified = unify_types(l, r)?;
                            changed |= self.unify(*lhs, unified.clone())?;
                            changed |= self.unify(*rhs, unified)?;
                        }
                        (Some(t), None) => {
                            changed |= self.unify(*rhs, t.clone())?;
                        }
                        (None, Some(t)) => {
                            changed |= self.unify(*lhs, t.clone())?;
                        }
                        _ => {}
                    }
                }

                Constraint::SubType(lhs, rhs) => {
                    let lhs_ty = self.get_var_type(*lhs);
                    let rhs_ty = self.get_var_type(*rhs);

                    if let (Some(l), Some(r)) = (&lhs_ty, &rhs_ty) {
                        if !is_subtype(l, r) {
                            return Err(format!(
                                "Type {} is not a subtype of {}",
                                l, r
                            ));
                        }
                    }
                }

                Constraint::SeriesOf(var, inner) => {
                    let var_idx = var.0;
                    if var_idx < self.vars.len() {
                        if !self.vars[var_idx].is_series {
                            self.vars[var_idx].is_series = true;
                            changed = true;
                        }

                        // If inner type is known, update var type
                        if let Some(inner_ty) = self.get_var_type(*inner) {
                            let series_ty = PineType::Series(Box::new(inner_ty));
                            if self.vars[var_idx].ty.as_ref() != Some(&series_ty) {
                                self.vars[var_idx].ty = Some(series_ty);
                                changed = true;
                            }
                        }
                    }
                }

                Constraint::Concrete(var, ty) => {
                    changed |= self.unify(*var, ty.clone())?;
                }
            }
        }

        Ok(changed)
    }

    /// Get the current type of a variable
    fn get_var_type(&self, var: TypeVar) -> Option<PineType> {
        self.vars.get(var.0).and_then(|v| v.ty.clone())
    }

    /// Unify a variable with a type
    fn unify(&mut self, var: TypeVar, ty: PineType) -> Result<bool, String> {
        let idx = var.0;
        if idx >= self.vars.len() {
            return Ok(false);
        }

        let current = &self.vars[idx];

        if let Some(existing) = &current.ty {
            if existing != &ty {
                // Try to unify the types
                let unified = unify_types(existing, &ty)?;
                if self.vars[idx].ty.as_ref() != Some(&unified) {
                    self.vars[idx].ty = Some(unified);
                    return Ok(true);
                }
            }
            Ok(false)
        } else {
            self.vars[idx].ty = Some(ty);
            Ok(true)
        }
    }

    /// Add a possible type for a variable
    #[allow(dead_code)]
    fn add_possible_type(&mut self, var: TypeVar, ty: PineType) -> Result<bool, String> {
        let idx = var.0;
        if idx >= self.vars.len() {
            return Ok(false);
        }

        let info = &mut self.vars[idx];
        if info.possible_types.insert(ty.clone()) {
            // If only one possible type, set it
            if info.possible_types.len() == 1 {
                info.ty = Some(ty);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Build the final inference result
    fn build_result(&self) -> InferenceResult {
        let mut types = HashMap::new();

        for (idx, var_info) in self.vars.iter().enumerate() {
            let var = TypeVar(idx);
            let final_ty = var_info
                .ty
                .clone()
                .or_else(|| {
                    // If no definite type, use first possible type
                    var_info.possible_types.iter().next().cloned()
                })
                .unwrap_or(PineType::Unknown);

            types.insert(var, final_ty);
        }

        InferenceResult { types }
    }

    /// Get the inferred type of a variable
    pub fn get_type(&self, var: TypeVar) -> Option<PineType> {
        self.vars.get(var.0).and_then(|v| v.ty.clone())
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of type inference
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Inferred types for each type variable
    types: HashMap<TypeVar, PineType>,
}

impl InferenceResult {
    /// Get the inferred type for a variable
    pub fn get(&self, var: TypeVar) -> Option<&PineType> {
        self.types.get(&var)
    }
}

/// Check if two types are compatible (equal or can be unified)
fn types_compatible(a: &PineType, b: &PineType) -> bool {
    match (a, b) {
        (PineType::Unknown, _) | (_, PineType::Unknown) => true,
        (PineType::Error, _) | (_, PineType::Error) => true,
        // Int and Float are compatible
        (PineType::Int, PineType::Float) | (PineType::Float, PineType::Int) => true,
        (a, b) => a == b,
    }
}

/// Check if a is a subtype of b
fn is_subtype(a: &PineType, b: &PineType) -> bool {
    match (a, b) {
        // Unknown is subtype of everything
        (PineType::Unknown, _) => true,
        // Everything is subtype of Error
        (_, PineType::Error) => true,
        // Int is subtype of Float
        (PineType::Int, PineType::Float) => true,
        // Series covariance: Series<T> is subtype of Series<U> if T is subtype of U
        (PineType::Series(a_inner), PineType::Series(b_inner)) => {
            is_subtype(a_inner, b_inner)
        }
        // Equality
        (a, b) => a == b,
    }
}

/// Unify two types into a common type
fn unify_types(a: &PineType, b: &PineType) -> Result<PineType, String> {
    match (a, b) {
        (PineType::Unknown, t) | (t, PineType::Unknown) => Ok(t.clone()),
        (PineType::Error, t) | (t, PineType::Error) => Ok(t.clone()),
        (a, b) if a == b => Ok(a.clone()),
        // Int and Float unify to Float
        (PineType::Int, PineType::Float) | (PineType::Float, PineType::Int) => {
            Ok(PineType::Float)
        }
        // Series unification
        (PineType::Series(a_inner), PineType::Series(b_inner)) => {
            let unified = unify_types(a_inner, b_inner)?;
            Ok(PineType::Series(Box::new(unified)))
        }
        (a, b) => Err(format!("Cannot unify types {} and {}", a, b)),
    }
}

/// Type inference for expressions
pub fn infer_expr_type(expr: &pine_parser::ast::Expr) -> PineType {
    use pine_parser::ast::Expr;
    use pine_parser::ast::Lit;

    match expr {
        Expr::Literal(lit, _) => match lit {
            Lit::Int(_) => PineType::Int,
            Lit::Float(_) => PineType::Float,
            Lit::String(_) => PineType::String,
            Lit::Bool(_) => PineType::Bool,
            Lit::Color(_) => PineType::Color,
            Lit::Na => PineType::Unknown,
        },
        Expr::Ident(_) => PineType::Unknown,
        Expr::BinOp { .. } => PineType::Unknown,
        Expr::UnaryOp { .. } => PineType::Unknown,
        Expr::Ternary { .. } => PineType::Unknown,
        Expr::FnCall { .. } => PineType::Unknown,
        Expr::FieldAccess { .. } => PineType::Unknown,
        Expr::MethodCall { .. } => PineType::Unknown,
        Expr::Index { .. } => PineType::Unknown,
        Expr::NaCoalesce { .. } => PineType::Unknown,
        Expr::ArrayLit(_, _) => PineType::Array(Box::new(PineType::Unknown)),
        Expr::MapLit(_, _) => PineType::Map(Box::new(PineType::Unknown), Box::new(PineType::Unknown)),
        Expr::Lambda { .. } => PineType::Function(Vec::new(), Box::new(PineType::Unknown)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_unification() {
        let mut infer = TypeInference::new();
        let v1 = infer.fresh_var();
        let v2 = infer.fresh_var();

        infer.constrain_eq(v1, v2);
        infer.constrain_concrete(v1, PineType::Int);

        let result = infer.solve().unwrap();
        assert_eq!(result.get(v1), Some(&PineType::Int));
        assert_eq!(result.get(v2), Some(&PineType::Int));
    }

    #[test]
    fn test_series_constraint() {
        let mut infer = TypeInference::new();
        let series_var = infer.fresh_var();
        let inner_var = infer.fresh_var();

        infer.constrain_series(series_var, inner_var);
        infer.constrain_concrete(inner_var, PineType::Float);

        let result = infer.solve().unwrap();
        assert_eq!(
            result.get(series_var),
            Some(&PineType::Series(Box::new(PineType::Float)))
        );
    }

    #[test]
    fn test_int_float_unification() {
        let mut infer = TypeInference::new();
        let v1 = infer.fresh_var();
        let v2 = infer.fresh_var();

        infer.constrain_concrete(v1, PineType::Int);
        infer.constrain_concrete(v2, PineType::Float);
        infer.constrain_eq(v1, v2);

        let result = infer.solve().unwrap();
        // Int and Float should unify to Float
        assert_eq!(result.get(v1), Some(&PineType::Float));
        assert_eq!(result.get(v2), Some(&PineType::Float));
    }

    #[test]
    fn test_subtype_relation() {
        assert!(is_subtype(&PineType::Int, &PineType::Float));
        assert!(is_subtype(&PineType::Int, &PineType::Int));
        assert!(!is_subtype(&PineType::Float, &PineType::Int));
        assert!(is_subtype(&PineType::Unknown, &PineType::Int));
    }
}
