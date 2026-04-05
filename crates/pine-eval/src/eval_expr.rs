//! Expression evaluation

use crate::{EvalError, EvaluationContext, Result};
use pine_lexer::Span;
use pine_parser::ast;
use pine_runtime::value::Value;
use std::sync::Arc;

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
        ast::Expr::FnCall {
            func, args, span, ..
        } => eval_fn_call(func, args, ctx, *span),
        ast::Expr::Index { base, offset, .. } => eval_index_access(base, offset, ctx),
        ast::Expr::ArrayLit(elements, _span) => {
            let mut values = Vec::with_capacity(elements.len());
            for e in elements {
                values.push(eval_expr(e, ctx)?);
            }
            Ok(Value::Array(values))
        }
        _ => {
            // TODO: Implement other expression types (MapLit, Lambda, etc.)
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
                let qualified_name = format!("{}.{}", ns, field.name);
                if let Some(value) = ctx.get_var(&qualified_name).cloned() {
                    return Ok(value);
                }
                if ctx.get_user_fn(&qualified_name).is_some() {
                    return Ok(Value::Namespace(qualified_name));
                }
                if let Some(value) = eval_builtin_namespace_variable(&qualified_name, ctx) {
                    return Ok(value);
                }
                // For other namespaces, return a namespaced identifier
                // This allows things like input.int to be resolved later
                Ok(Value::Namespace(qualified_name))
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
        Value::Object(obj) => {
            let type_name = obj.type_name.clone();
            let Some(user_fn) = ctx.get_type_method(&type_name, &method.name).cloned() else {
                return Err(EvalError::UndefinedMethod {
                    method_name: method.name.clone(),
                    span,
                });
            };
            let mut arg_values = Vec::with_capacity(1 + args.len());
            arg_values.push(Value::Object(Arc::clone(&obj)));
            for arg in args {
                arg_values.push(eval_expr(&arg.value, ctx)?);
            }
            let key = format!("{}.{}", type_name, method.name);
            crate::eval_stmt::invoke_user_function(
                ctx,
                &user_fn,
                &key,
                span,
                &arg_values,
                method.span,
            )
        }
        Value::Namespace(ns) => {
            // Namespace function call: input.int(...), ta.sma(...), etc.
            let full_name = format!("{}.{}", ns, method.name);

            if let Some(user_fn) = ctx.get_user_fn(&full_name).cloned() {
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(eval_expr(&arg.value, ctx)?);
                }
                return crate::eval_stmt::invoke_user_function(
                    ctx,
                    &user_fn,
                    &full_name,
                    span,
                    &arg_values,
                    method.span,
                );
            }

            // Evaluate arguments
            let mut arg_values = Vec::with_capacity(args.len());
            for arg in args {
                let val = eval_namespace_arg(&ns, &arg.value, ctx)?;
                arg_values.push(val);
            }

            // Some ta.* functions use implicit built-in series in Pine Script.
            if (full_name == "ta.atr" && arg_values.len() == 1)
                || (full_name == "ta.dmi" && arg_values.len() == 2)
                || (full_name == "ta.mfi" && arg_values.len() == 2)
                || (full_name == "ta.supertrend" && arg_values.len() == 2)
                || (full_name == "ta.vwma" && arg_values.len() == 2)
                || (full_name == "ta.tr" && arg_values.len() <= 1)
            {
                if full_name == "ta.vwma" || full_name == "ta.mfi" {
                    if let Some(volume) = builtin_series_value("volume", ctx) {
                        arg_values.insert(1, volume);
                    }
                } else if let (Some(high), Some(low), Some(close)) = (
                    builtin_series_value("high", ctx),
                    builtin_series_value("low", ctx),
                    builtin_series_value("close", ctx),
                ) {
                    {
                        arg_values.insert(0, close);
                        arg_values.insert(0, low);
                        arg_values.insert(0, high);
                    }
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
        "hl2" => {
            return Some(Value::Array(
                series_data
                    .high
                    .get(..end)?
                    .iter()
                    .zip(series_data.low.get(..end)?.iter())
                    .map(|(high, low)| Value::Float((high + low) / 2.0))
                    .collect(),
            ))
        }
        "hlc3" => {
            return Some(Value::Array(
                series_data
                    .high
                    .get(..end)?
                    .iter()
                    .zip(series_data.low.get(..end)?.iter())
                    .zip(series_data.close.get(..end)?.iter())
                    .map(|((high, low), close)| Value::Float((high + low + close) / 3.0))
                    .collect(),
            ))
        }
        "ohlc4" => {
            return Some(Value::Array(
                series_data
                    .open
                    .get(..end)?
                    .iter()
                    .zip(series_data.high.get(..end)?.iter())
                    .zip(series_data.low.get(..end)?.iter())
                    .zip(series_data.close.get(..end)?.iter())
                    .map(|(((open, high), low), close)| {
                        Value::Float((open + high + low + close) / 4.0)
                    })
                    .collect(),
            ))
        }
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

fn eval_builtin_namespace_variable(name: &str, ctx: &EvaluationContext) -> Option<Value> {
    match name {
        "ta.obv" => {
            let close = builtin_series_value("close", ctx)?;
            let volume = builtin_series_value("volume", ctx)?;
            ctx.function_registry().dispatch(name, &[close, volume])
        }
        "ta.pvt" => {
            let close = builtin_series_value("close", ctx)?;
            let volume = builtin_series_value("volume", ctx)?;
            ctx.function_registry().dispatch(name, &[close, volume])
        }
        _ => None,
    }
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
        BinOp::Mod => Ok(pine_runtime::na_ops::modulo(left, right)),
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
fn eval_fn_call(
    func: &ast::Expr,
    args: &[ast::Arg],
    ctx: &mut EvaluationContext,
    call_span: Span,
) -> Result<Value> {
    if let ast::Expr::Ident(ident) = func {
        if ident.name == "plot" {
            return eval_plot_call(args, ctx);
        }

        if let Some(user_fn) = ctx.get_user_fn(&ident.name).cloned() {
            let mut arg_values = Vec::with_capacity(args.len());
            for arg in args {
                arg_values.push(eval_expr(&arg.value, ctx)?);
            }
            return crate::eval_stmt::invoke_user_function(
                ctx,
                &user_fn,
                &ident.name,
                call_span,
                &arg_values,
                ident.span,
            );
        }

        let mut arg_values = Vec::with_capacity(args.len());
        for arg in args {
            let val = eval_expr(&arg.value, ctx)?;
            arg_values.push(val);
        }
        let math_name = format!("math.{}", ident.name);
        if let Some(v) = ctx.function_registry().dispatch(&math_name, &arg_values) {
            return Ok(v);
        }
        if let Some(v) = ctx.function_registry().dispatch(&ident.name, &arg_values) {
            return Ok(v);
        }
        use crate::fn_call::call_builtin;
        return call_builtin(&ident.name, &arg_values, ctx);
    }

    use crate::fn_call::{call_fn, make_call_site_key};
    let key = make_call_site_key("fn", func.span());
    call_fn(func, args, ctx, call_span, &key)
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

    // Extract pane from named arguments
    let pane = args.iter().find_map(|arg| {
        if arg.name.as_ref().map(|n| n.name.as_str()) == Some("pane") {
            match eval_expr(&arg.value, ctx).ok()? {
                Value::Int(i) => Some(i as i32),
                Value::Float(f) => Some(f as i32),
                _ => None,
            }
        } else {
            None
        }
    });

    // Convert value to Option<f64>
    let plot_value = match value {
        Value::Float(f) => Some(f),
        Value::Int(i) => Some(i as f64),
        _ => None, // NA or other types = no value
    };

    // Record the plot value with pane info
    ctx.plot_outputs.record_with_pane(title.clone(), plot_value, pane);

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
    let offset_val = eval_expr(offset, ctx)?;
    let off = match offset_val {
        Value::Int(i) => i as usize,
        Value::Float(f) => f as usize,
        _ => return Ok(Value::Na),
    };

    if let ast::Expr::Ident(ident) = base {
        // First check if the identifier is bound to an array
        if let Some(Value::Array(arr)) = ctx.get_var(&ident.name) {
            // Array indexing: arr[0], arr[1], etc.
            return Ok(arr.get(off).cloned().unwrap_or(Value::Na));
        }

        // Then check scoped series (var/varip)
        let cs = ctx.current_call_site();
        if ctx.runtime().var_scoped_contains(&ident.name, cs) {
            return Ok(ctx
                .runtime()
                .get_series_at(&ident.name, cs, off)
                .cloned()
                .unwrap_or(Value::Na));
        }

        // Finally check built-in series (close, open, high, low, etc.)
        return match ctx.get_series_value(&ident.name, off) {
            Some(val) => Ok(Value::Float(val)),
            None => Ok(Value::Na),
        };
    }

    // For non-identifier bases (e.g., function calls), evaluate and return
    let base_val = eval_expr(base, ctx)?;
    Ok(base_val)
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

    #[test]
    fn test_ta_obv_field_access_uses_builtin_series() {
        let mut ctx = EvaluationContext::new();
        ctx.set_var("ta", Value::Namespace("ta".to_string()));
        ctx.series_data = Some(crate::SeriesData {
            open: vec![99.0, 100.0, 103.0, 103.0, 101.0],
            high: vec![102.0, 106.0, 105.0, 106.0, 106.0],
            low: vec![96.0, 98.0, 100.0, 99.0, 98.0],
            close: vec![100.0, 103.0, 103.0, 101.0, 104.0],
            volume: vec![1000.0, 1200.0, 900.0, 1300.0, 1100.0],
            time: vec![1, 2, 3, 4, 5],
            current_bar: 4,
        });

        let expr = ast::Expr::FieldAccess {
            base: Box::new(ast::Expr::Ident(ast::Ident::new("ta", Span::default()))),
            field: ast::Ident::new("obv", Span::default()),
            span: Span::default(),
        };

        let result = eval_expr(&expr, &mut ctx).unwrap();
        assert_eq!(result, Value::Float(1000.0));
    }

    #[test]
    fn test_ta_pvt_field_access_uses_builtin_series() {
        let mut ctx = EvaluationContext::new();
        ctx.set_var("ta", Value::Namespace("ta".to_string()));
        ctx.series_data = Some(crate::SeriesData {
            open: vec![99.0, 100.0, 103.0, 103.0, 101.0],
            high: vec![102.0, 106.0, 105.0, 106.0, 106.0],
            low: vec![96.0, 98.0, 100.0, 99.0, 98.0],
            close: vec![100.0, 103.0, 103.0, 101.0, 104.0],
            volume: vec![1000.0, 1200.0, 900.0, 1300.0, 1100.0],
            time: vec![1, 2, 3, 4, 5],
            current_bar: 4,
        });

        let expr = ast::Expr::FieldAccess {
            base: Box::new(ast::Expr::Ident(ast::Ident::new("ta", Span::default()))),
            field: ast::Ident::new("pvt", Span::default()),
            span: Span::default(),
        };

        let result = eval_expr(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Value::Float(v) if (v - 43.43054888013073).abs() < 1e-9));
    }
}
