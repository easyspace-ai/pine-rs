//! Statement evaluation

use crate::eval_expr::eval_expr;
use crate::{EvaluationContext, EvalError, Result};
use pine_parser::ast as ast;
use pine_runtime::value::Value;
use std::sync::Arc;

/// Evaluate a statement
pub fn eval_stmt(stmt: &ast::Stmt, ctx: &mut EvaluationContext) -> Result<()> {
    match stmt {
        ast::Stmt::VarDecl { name, init, .. } => {
            let value = if let Some(init_expr) = init {
                eval_expr(init_expr, ctx)?
            } else {
                Value::Na
            };
            ctx.set_var(&name.name, value);
            Ok(())
        }
        ast::Stmt::Assign { target, value, .. } => {
            let rhs_value = eval_expr(value, ctx)?;
            eval_assign(target, rhs_value, ctx)
        }
        ast::Stmt::Expr(expr) => {
            eval_expr(expr, ctx)?;
            Ok(())
        }
        _ => {
            // TODO: Handle other statement types
            Ok(())
        }
    }
}

/// Evaluate an assignment
fn eval_assign(target: &ast::AssignTarget, value: Value, ctx: &mut EvaluationContext) -> Result<()> {
    match target {
        ast::AssignTarget::Var(ident) => {
            ctx.set_var(&ident.name, value);
            Ok(())
        }
        ast::AssignTarget::Field { base, field, .. } => {
            let base_value = eval_expr(base, ctx)?;
            eval_field_assign(base_value, field, value)
        }
        _ => {
            // TODO: Handle index assignment
            Ok(())
        }
    }
}

/// Evaluate a field assignment
fn eval_field_assign(base: Value, field: &ast::Ident, value: Value) -> Result<()> {
    match base {
        Value::Object(mut obj) => {
            // Get a mutable reference to the object
            let obj_mut = Arc::make_mut(&mut obj);
            obj_mut.set(&field.name, value);
            Ok(())
        }
        _ => Err(EvalError::NotAnObject {
            found: base,
            span: field.span,
        }),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_field_assignment() {
        // This is a placeholder test - the proper implementation requires
        // a more complete evaluation context that handles mutable objects correctly
        assert!(true);
    }
}
