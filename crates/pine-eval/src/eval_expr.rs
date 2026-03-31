//! Expression evaluation

use crate::{EvalError, EvaluationContext, Result};
use pine_lexer::Span;
use pine_parser::ast;
use pine_runtime::value::Value;

/// Evaluate an expression
pub fn eval_expr(expr: &ast::Expr, ctx: &mut EvaluationContext) -> Result<Value> {
    match expr {
        ast::Expr::Literal(lit, _span) => eval_literal(lit),
        ast::Expr::Ident(ident) => {
            ctx.get_var(&ident.name)
                .cloned()
                .ok_or_else(|| EvalError::UndefinedVariable {
                    name: ident.name.clone(),
                    span: ident.span,
                })
        }
        ast::Expr::FieldAccess { base, field, span } => {
            let base_value = eval_expr(base, ctx)?;
            eval_field_access(base_value, field, *span)
        }
        ast::Expr::MethodCall {
            base,
            method,
            args,
            span,
        } => {
            let base_value = eval_expr(base, ctx)?;
            eval_method_call(base_value, method, args, *span, ctx)
        }
        _ => {
            // TODO: Implement other expression types
            Ok(Value::Na)
        }
    }
}

/// Evaluate a literal
fn eval_literal(lit: &ast::Lit) -> Result<Value> {
    match lit {
        ast::Lit::Int(i) => Ok(Value::Int(*i)),
        ast::Lit::Float(f) => Ok(Value::Float(*f)),
        ast::Lit::Bool(b) => Ok(Value::Bool(*b)),
        ast::Lit::String(s) => Ok(Value::String(s.clone().into())),
        ast::Lit::Color(c) => {
            let r = ((c >> 24) & 0xFF) as u8;
            let g = ((c >> 16) & 0xFF) as u8;
            let b = ((c >> 8) & 0xFF) as u8;
            let a = (c & 0xFF) as u8;
            Ok(Value::Color(pine_runtime::value::Color::with_alpha(
                r, g, b, a,
            )))
        }
        ast::Lit::Na => Ok(Value::Na),
    }
}

/// Evaluate a field access expression
fn eval_field_access(base: Value, field: &ast::Ident, span: Span) -> Result<Value> {
    match base {
        Value::Object(obj) => {
            obj.get(&field.name)
                .cloned()
                .ok_or_else(|| EvalError::UndefinedField {
                    field_name: field.name.clone(),
                    span,
                })
        }
        _ => Err(EvalError::NotAnObject { found: base, span }),
    }
}

/// Evaluate a method call expression
fn eval_method_call(
    base: Value,
    _method: &ast::Ident,
    _args: &[ast::Arg],
    span: Span,
    _ctx: &mut EvaluationContext,
) -> Result<Value> {
    match base {
        Value::Object(_obj) => {
            // TODO: Implement method dispatch
            // For now, return NA as a placeholder
            Ok(Value::Na)
        }
        _ => Err(EvalError::NotAnObject { found: base, span }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_lexer::Span;
    use pine_runtime::value::{Object, Value};

    #[test]
    fn test_field_access() {
        let mut ctx = EvaluationContext::new();

        // Create an object
        let mut obj = Object::new("Point");
        obj.set("x", Value::Int(10));
        obj.set("y", Value::Int(20));

        // Store it in a variable
        ctx.set_var("p", Value::from(obj));

        // Create AST nodes for field access
        let ident_p = ast::Ident::new("p", Span::default());
        let field_x = ast::Ident::new("x", Span::default());
        let field_access = ast::Expr::FieldAccess {
            base: Box::new(ast::Expr::Ident(ident_p)),
            field: field_x,
            span: Span::default(),
        };

        // Evaluate the field access
        let result = eval_expr(&field_access, &mut ctx).unwrap();
        assert_eq!(result, Value::Int(10));
    }
}
