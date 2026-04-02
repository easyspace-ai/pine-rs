//! Function call handling
//!
//! This module provides function call execution including:
//! - User-defined function calls
//! - Built-in function dispatch
//! - Call-site series isolation (each call site maintains independent series history)

use crate::eval_expr::eval_expr;
use crate::{EvalError, EvaluationContext, Result};
use pine_lexer::Span;
use pine_parser::ast;
use pine_runtime::value::{Closure, Value};
use std::sync::Arc;

/// Call a function
///
/// This function handles both user-defined functions and built-in functions.
/// For user-defined functions, it creates a new execution context with
/// call-site isolation for series variables.
pub fn call_fn(
    func: &ast::Expr,
    args: &[ast::Arg],
    ctx: &mut EvaluationContext,
    call_span: Span,
    call_site_id: &str,
) -> Result<Value> {
    // Evaluate arguments first
    let mut arg_values = Vec::with_capacity(args.len());
    for arg in args {
        let val = eval_expr(&arg.value, ctx)?;
        arg_values.push(val);
    }

    match func {
        ast::Expr::Ident(ident) => {
            if let Some(user_fn) = ctx.get_user_fn(&ident.name).cloned() {
                return crate::eval_stmt::invoke_user_function(
                    ctx,
                    &user_fn,
                    &ident.name,
                    call_span,
                    &arg_values,
                    ident.span,
                );
            }
            // Look up the function in the context
            if let Some(fn_value) = ctx.get_var(&ident.name).cloned() {
                match fn_value {
                    Value::Closure(closure) => {
                        // Call user-defined function with call-site isolation
                        call_user_fn(
                            &closure,
                            &arg_values,
                            ctx,
                            &format!("{}.{}", ident.name, call_site_id),
                        )
                    }
                    _ => {
                        // Try built-in function dispatch
                        call_builtin(&ident.name, &arg_values, ctx)
                    }
                }
            } else {
                // Try built-in function without having it in context
                call_builtin(&ident.name, &arg_values, ctx)
            }
        }
        _ => {
            // Complex function expression (e.g., lambda)
            let fn_value = eval_expr(func, ctx)?;
            match fn_value {
                Value::Closure(closure) => call_user_fn(&closure, &arg_values, ctx, call_site_id),
                _ => Err(EvalError::TypeError {
                    message: format!("Expected function, got {:?}", fn_value),
                    span: call_span,
                }),
            }
        }
    }
}

/// Call a user-defined function with call-site series isolation
///
/// # Call-site Series Isolation
///
/// In Pine Script, each call site of a function maintains its own independent
/// series history. For example, `f(close)` and `f(high)` each have independent
/// series histories.
///
/// This is implemented by creating a unique call-site key for each invocation,
/// which is used to isolate var/varip variables within that function call.
fn call_user_fn(
    closure: &Arc<Closure>,
    args: &[Value],
    _ctx: &mut EvaluationContext,
    call_site_key: &str,
) -> Result<Value> {
    // Create a new context for the function execution
    // The call_site_key ensures series isolation
    let mut fn_ctx = EvaluationContext::new();

    // Bind parameters to arguments
    if args.len() != closure.params.len() {
        return Err(EvalError::TypeError {
            message: format!(
                "Function expected {} arguments, got {}",
                closure.params.len(),
                args.len()
            ),
            span: pine_lexer::Span::default(),
        });
    }

    for ((param_name, _kind), arg) in closure.params.iter().zip(args.iter()) {
        fn_ctx.set_var(param_name, arg.clone());
    }

    // Store the call-site key for series isolation
    // This is used by the runtime to maintain independent series history
    fn_ctx.set_var("__call_site__", Value::String(call_site_key.into()));

    // Execute the function body
    // Note: In a full implementation, we'd execute the closure's body here
    // For now, return NA as the body execution requires AST storage in Closure

    Ok(Value::Na)
}

/// Call a built-in function
///
/// This dispatches to the appropriate built-in function from pine-stdlib.
pub fn call_builtin(name: &str, args: &[Value], _ctx: &mut EvaluationContext) -> Result<Value> {
    // Dispatch to built-in functions from pine-stdlib
    // This is a simplified version - full implementation would use
    // the FunctionRegistry from pine-stdlib

    match name {
        // NA checking function
        "na" => {
            // na(value) - returns true if value is NA
            if args.is_empty() {
                return Ok(Value::Bool(true));
            }
            Ok(Value::Bool(args[0].is_na()))
        }
        // Math functions
        "nz" => {
            // nz(value, default) - returns value if not na, else default
            if args.is_empty() {
                return Ok(Value::Na);
            }
            let val = &args[0];
            let default = args.get(1).cloned().unwrap_or(Value::Int(0));

            if val.is_na() {
                Ok(default)
            } else {
                Ok(val.clone())
            }
        }

        "abs" => {
            if args.is_empty() {
                return Ok(Value::Na);
            }
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Ok(Value::Na),
            }
        }

        "max" => {
            if args.len() < 2 {
                return Ok(Value::Na);
            }
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.max(*b as f64))),
                _ => Ok(Value::Na),
            }
        }

        "min" => {
            if args.len() < 2 {
                return Ok(Value::Na);
            }
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.min(*b as f64))),
                _ => Ok(Value::Na),
            }
        }

        // String functions
        "tostring" => {
            if args.is_empty() {
                return Ok(Value::String("na".into()));
            }
            let s = match &args[0] {
                Value::Int(i) => i.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::String(s) => s.to_string(),
                Value::Na => "na".to_string(),
                _ => "na".to_string(),
            };
            Ok(Value::String(s.into()))
        }

        // For other built-ins, return NA
        // Full implementation would use FunctionRegistry
        _ => Ok(Value::Na),
    }
}

/// Create a unique call-site key for series isolation
///
/// The key format: "{function_name}@{file}:{line}:{column}"
pub fn make_call_site_key(func_name: &str, span: pine_lexer::Span) -> String {
    format!("{}@{}:{}", func_name, span.start, span.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_lexer::Span;

    #[test]
    fn test_nz_builtin() {
        let mut ctx = EvaluationContext::new();

        // nz(na) should return 0 (default)
        let args = vec![Value::Na];
        let result = call_builtin("nz", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::Int(0));

        // nz(na, 10) should return 10
        let args = vec![Value::Na, Value::Int(10)];
        let result = call_builtin("nz", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::Int(10));

        // nz(5) should return 5
        let args = vec![Value::Int(5)];
        let result = call_builtin("nz", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_abs_builtin() {
        let mut ctx = EvaluationContext::new();

        // abs(-5) should return 5
        let args = vec![Value::Int(-5)];
        let result = call_builtin("abs", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::Int(5));

        // abs(-3.14) should return 3.14
        let args = vec![Value::Float(-3.14)];
        let result = call_builtin("abs", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_max_min_builtin() {
        let mut ctx = EvaluationContext::new();

        // max(5, 10) should return 10
        let args = vec![Value::Int(5), Value::Int(10)];
        let result = call_builtin("max", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::Int(10));

        // min(5, 10) should return 5
        let args = vec![Value::Int(5), Value::Int(10)];
        let result = call_builtin("min", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_tostring_builtin() {
        let mut ctx = EvaluationContext::new();

        // tostring(42) should return "42"
        let args = vec![Value::Int(42)];
        let result = call_builtin("tostring", &args, &mut ctx).unwrap();
        assert_eq!(result, Value::String("42".into()));
    }

    #[test]
    fn test_call_site_key() {
        let key = make_call_site_key("myFunc", Span::new(0, 100));
        assert!(key.starts_with("myFunc@"));
    }
}
