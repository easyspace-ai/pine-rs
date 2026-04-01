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
            eval_field_access(base_value, field, *span, ctx)
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
        ast::Expr::BinOp { op, lhs, rhs, .. } => {
            let left = eval_expr(lhs, ctx)?;
            let right = eval_expr(rhs, ctx)?;
            eval_binary_op(*op, &left, &right)
        }
        ast::Expr::UnaryOp { op, operand, .. } => {
            let val = eval_expr(operand, ctx)?;
            eval_unary_op(*op, &val)
        }
        ast::Expr::Ternary {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            let condition = eval_expr(cond, ctx)?;
            if condition.is_truthy() {
                eval_expr(then_branch, ctx)
            } else {
                eval_expr(else_branch, ctx)
            }
        }
        ast::Expr::FnCall { func, args, .. } => eval_fn_call(func, args, ctx),
        ast::Expr::Index { base, offset, .. } => eval_index_access(base, offset, ctx),
        _ => {
            // TODO: Implement other expression types (ArrayLit, MapLit, Lambda, etc.)
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
fn eval_field_access(
    base: Value,
    field: &ast::Ident,
    span: Span,
    ctx: &mut EvaluationContext,
) -> Result<Value> {
    match base {
        Value::Object(obj) => {
            obj.get(&field.name)
                .cloned()
                .ok_or_else(|| EvalError::UndefinedField {
                    field_name: field.name.clone(),
                    span,
                })
        }
        Value::Namespace(ns) => {
            // Namespace field access: color.blue, color.red, etc.
            // These are color constants from the color namespace
            if ns == "color" {
                match field.name.as_str() {
                    "blue" => Ok(Value::Color(pine_runtime::value::Color::new(0, 120, 255))),
                    "red" => Ok(Value::Color(pine_runtime::value::Color::new(255, 0, 0))),
                    "green" => Ok(Value::Color(pine_runtime::value::Color::new(0, 128, 0))),
                    "yellow" => Ok(Value::Color(pine_runtime::value::Color::new(255, 255, 0))),
                    "white" => Ok(Value::Color(pine_runtime::value::Color::new(255, 255, 255))),
                    "black" => Ok(Value::Color(pine_runtime::value::Color::new(0, 0, 0))),
                    "gray" => Ok(Value::Color(pine_runtime::value::Color::new(128, 128, 128))),
                    "orange" => Ok(Value::Color(pine_runtime::value::Color::new(255, 165, 0))),
                    "purple" => Ok(Value::Color(pine_runtime::value::Color::new(128, 0, 128))),
                    "lime" => Ok(Value::Color(pine_runtime::value::Color::new(0, 255, 0))),
                    "maroon" => Ok(Value::Color(pine_runtime::value::Color::new(128, 0, 0))),
                    "olive" => Ok(Value::Color(pine_runtime::value::Color::new(128, 128, 0))),
                    "navy" => Ok(Value::Color(pine_runtime::value::Color::new(0, 0, 128))),
                    "teal" => Ok(Value::Color(pine_runtime::value::Color::new(0, 128, 128))),
                    "silver" => Ok(Value::Color(pine_runtime::value::Color::new(192, 192, 192))),
                    "fuchsia" => Ok(Value::Color(pine_runtime::value::Color::new(255, 0, 255))),
                    "aqua" => Ok(Value::Color(pine_runtime::value::Color::new(0, 255, 255))),
                    _ => {
                        // Try to dispatch as color.* function (like color.rgb)
                        let full_name = format!("color.{}", field.name);
                        if ctx.function_registry().contains(&full_name) {
                            // Return a closure-like value that can be called
                            // For now, we'll handle this in the calling context
                            Ok(Value::Namespace(full_name))
                        } else {
                            Err(EvalError::UndefinedField {
                                field_name: field.name.clone(),
                                span,
                            })
                        }
                    }
                }
            } else {
                // For other namespaces, return a namespaced identifier
                // This allows things like input.int to be resolved later
                Ok(Value::Namespace(format!("{}.{}", ns, field.name)))
            }
        }
        _ => Err(EvalError::NotAnObject { found: base, span }),
    }
}

/// Evaluate a method call expression
fn eval_method_call(
    base: Value,
    method: &ast::Ident,
    args: &[ast::Arg],
    span: Span,
    ctx: &mut EvaluationContext,
) -> Result<Value> {
    match base {
        Value::Object(_obj) => {
            // TODO: Implement method dispatch
            // For now, return NA as a placeholder
            Ok(Value::Na)
        }
        Value::Namespace(ns) => {
            // Namespace function call: input.int(...), ta.sma(...), etc.
            let full_name = format!("{}.{}", ns, method.name);

            // Evaluate arguments
            let mut arg_values = Vec::with_capacity(args.len());
            for arg in args {
                let val = eval_namespace_arg(&ns, &arg.value, ctx)?;
                arg_values.push(val);
            }

            // Some ta.* functions use implicit built-in series in Pine Script.
            if (full_name == "ta.atr" && arg_values.len() == 1)
                || (full_name == "ta.tr" && arg_values.len() <= 1)
            {
                if let (Some(high), Some(low), Some(close)) = (
                    builtin_series_value("high", ctx),
                    builtin_series_value("low", ctx),
                    builtin_series_value("close", ctx),
                ) {
                    arg_values.insert(0, close);
                    arg_values.insert(0, low);
                    arg_values.insert(0, high);
                }
            }

            // Dispatch the function
            if let Some(result) = ctx.function_registry().dispatch(&full_name, &arg_values) {
                Ok(result)
            } else {
                Err(EvalError::TypeError {
                    message: format!("Undefined function: {}", full_name),
                    span,
                })
            }
        }
        _ => Err(EvalError::NotAnObject { found: base, span }),
    }
}

/// Evaluate an argument passed to a namespaced function.
///
/// For series-aware namespaces such as `ta` and `input`, built-in price sources
/// need the full history up to the current bar rather than only the current bar.
fn eval_namespace_arg(
    namespace: &str,
    expr: &ast::Expr,
    ctx: &mut EvaluationContext,
) -> Result<Value> {
    if matches!(namespace, "ta" | "input") {
        if let Some(series_value) = eval_builtin_series_arg(expr, ctx) {
            return Ok(series_value);
        }
    }

    eval_expr(expr, ctx)
}

/// Build a series array for built-in price sources up to the current bar.
fn eval_builtin_series_arg(expr: &ast::Expr, ctx: &EvaluationContext) -> Option<Value> {
    let ident = match expr {
        ast::Expr::Ident(ident) => ident,
        _ => return None,
    };

    if let Some(series) = builtin_series_value(&ident.name, ctx) {
        return Some(series);
    }

    ctx.get_var_history(&ident.name)
        .map(|values| Value::Array(values.to_vec()))
}

fn builtin_series_value(name: &str, ctx: &EvaluationContext) -> Option<Value> {
    let series_data = ctx.series_data.as_ref()?;
    let end = series_data.current_bar + 1;

    let values = match name {
        "open" => series_data.open.get(..end)?,
        "high" => series_data.high.get(..end)?,
        "low" => series_data.low.get(..end)?,
        "close" => series_data.close.get(..end)?,
        "volume" => series_data.volume.get(..end)?,
        "time" => {
            return Some(Value::Array(
                series_data
                    .time
                    .get(..end)?
                    .iter()
                    .map(|v| Value::Int(*v))
                    .collect(),
            ))
        }
        _ => return None,
    };

    Some(Value::Array(
        values.iter().copied().map(Value::Float).collect(),
    ))
}

/// Evaluate a binary operation
fn eval_binary_op(op: ast::BinOp, left: &Value, right: &Value) -> Result<Value> {
    use ast::BinOp;
    use pine_runtime::na_ops;

    match op {
        BinOp::Add => Ok(na_ops::add(left, right)),
        BinOp::Sub => Ok(na_ops::sub(left, right)),
        BinOp::Mul => Ok(na_ops::mul(left, right)),
        BinOp::Div => Ok(na_ops::div(left, right)),
        BinOp::Mod => {
            // Modulo operation
            match (left.as_float(), right.as_float()) {
                (Some(a), Some(b)) if b != 0.0 => Ok(Value::Float(a % b)),
                _ => Ok(Value::Na),
            }
        }
        BinOp::Pow => {
            // Power operation
            match (left.as_float(), right.as_float()) {
                (Some(a), Some(b)) => Ok(Value::Float(a.powf(b))),
                _ => Ok(Value::Na),
            }
        }
        BinOp::Eq => Ok(na_ops::eq(left, right)),
        BinOp::Neq => Ok(na_ops::ne(left, right)),
        BinOp::Lt => Ok(na_ops::lt(left, right)),
        BinOp::Le => Ok(na_ops::le(left, right)),
        BinOp::Gt => Ok(na_ops::gt(left, right)),
        BinOp::Ge => Ok(na_ops::ge(left, right)),
        BinOp::And => Ok(na_ops::and(left, right)),
        BinOp::Or => Ok(na_ops::or(left, right)),
    }
}

/// Evaluate a unary operation
fn eval_unary_op(op: ast::UnaryOp, operand: &Value) -> Result<Value> {
    use ast::UnaryOp;
    use pine_runtime::na_ops;

    match op {
        UnaryOp::Neg => Ok(na_ops::neg(operand)),
        UnaryOp::Not => Ok(na_ops::not(operand)),
    }
}

/// Evaluate a function call
fn eval_fn_call(func: &ast::Expr, args: &[ast::Arg], ctx: &mut EvaluationContext) -> Result<Value> {
    // Check if this is a plot() call
    if let ast::Expr::Ident(ident) = func {
        if ident.name == "plot" {
            return eval_plot_call(args, ctx);
        }

        // Try built-in functions directly
        use crate::fn_call::call_builtin;
        // First evaluate arguments
        let mut arg_values = Vec::with_capacity(args.len());
        for arg in args {
            let val = eval_expr(&arg.value, ctx)?;
            arg_values.push(val);
        }
        return call_builtin(&ident.name, &arg_values, ctx);
    }

    // For non-ident function expressions, return NA for now
    Ok(Value::Na)
}

/// Evaluate a plot() function call
fn eval_plot_call(args: &[ast::Arg], ctx: &mut EvaluationContext) -> Result<Value> {
    // First argument is the value to plot
    let value = if let Some(arg) = args.first() {
        eval_expr(&arg.value, ctx)?
    } else {
        return Ok(Value::Na);
    };

    // Second argument (optional) is the title
    let title = if let Some(arg) = args.get(1) {
        match eval_expr(&arg.value, ctx)? {
            Value::String(s) => s.to_string(),
            _ => "plot".to_string(),
        }
    } else {
        "plot".to_string()
    };

    // Convert value to Option<f64>
    let plot_value = match value {
        Value::Float(f) => Some(f),
        Value::Int(i) => Some(i as f64),
        _ => None, // NA or other types = no value
    };

    // Record the plot value
    ctx.plot_outputs.record(title.clone(), plot_value);

    // Return the plotted value
    Ok(match plot_value {
        Some(f) => Value::Float(f),
        None => Value::Na,
    })
}

/// Evaluate index access (e.g., close[1])
///
/// This function handles historical series access like `close[1]` which accesses
/// the previous bar's close value. The offset is evaluated and used to look up
/// the historical value from the series data.
fn eval_index_access(
    base: &ast::Expr,
    offset: &ast::Expr,
    ctx: &mut EvaluationContext,
) -> Result<Value> {
    // Get the series name from the base expression
    let series_name = match base {
        ast::Expr::Ident(ident) => &ident.name,
        _ => {
            // For complex expressions, evaluate the base first
            let base_val = eval_expr(base, ctx)?;
            // Try to get series value from the evaluated base
            return Ok(base_val);
        }
    };

    // Evaluate the offset expression
    let offset_val = eval_expr(offset, ctx)?;
    let offset = match offset_val {
        Value::Int(i) => i as usize,
        Value::Float(f) => f as usize,
        _ => return Ok(Value::Na),
    };

    // Get the historical value from series data
    match ctx.get_series_value(series_name, offset) {
        Some(val) => Ok(Value::Float(val)),
        None => Ok(Value::Na),
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
