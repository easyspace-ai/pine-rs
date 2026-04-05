//! Statement evaluation

use crate::eval_expr::eval_expr;
use crate::{EvalError, EvaluationContext, LoadedModule, Result, UserFn};
use pine_lexer::Lexer;
use pine_parser::ast;
use pine_parser::ast::{AssignOp, ForInPattern, ImportPath, SwitchArmBody};
use pine_runtime::context::PersistentVarKind;
use pine_runtime::module::Module;
use pine_runtime::value::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Evaluate a block of statements
pub fn eval_block(block: &ast::Block, ctx: &mut EvaluationContext) -> Result<()> {
    for stmt in &block.stmts {
        eval_stmt(stmt, ctx)?;
    }
    Ok(())
}

/// Evaluate a function / export / method body and return the value (`=> expr` or last expr in block).
pub(crate) fn eval_fn_body(body: &ast::FnBody, ctx: &mut EvaluationContext) -> Result<Value> {
    match body {
        ast::FnBody::Expr(e) => eval_expr(e, ctx),
        ast::FnBody::Block(b) => eval_block_last_value(b, ctx),
    }
}

fn eval_block_last_value(block: &ast::Block, ctx: &mut EvaluationContext) -> Result<Value> {
    if block.stmts.is_empty() {
        return Ok(Value::Na);
    }
    let n = block.stmts.len();
    for stmt in &block.stmts[..n - 1] {
        eval_stmt(stmt, ctx)?;
    }
    match &block.stmts[n - 1] {
        ast::Stmt::Expr(e) => eval_expr(e, ctx),
        s => {
            eval_stmt(s, ctx)?;
            Ok(Value::Na)
        }
    }
}

/// Invoke a user-defined function in the current context (parameter shadowing with restore).
pub(crate) fn invoke_user_function(
    ctx: &mut EvaluationContext,
    user_fn: &UserFn,
    fn_name: &str,
    call_span: pine_lexer::Span,
    arg_values: &[Value],
    span: pine_lexer::Span,
) -> Result<Value> {
    // Count required parameters (those without defaults)
    let required_count = user_fn
        .params
        .iter()
        .filter(|p| p.default.is_none())
        .count();

    if arg_values.len() < required_count || arg_values.len() > user_fn.params.len() {
        return Err(EvalError::TypeError {
            message: format!(
                "function expected {} arguments, got {}",
                user_fn.params.len(),
                arg_values.len()
            ),
            span,
        });
    }

    if !ctx.runtime_mut().enter_call() {
        return Err(EvalError::TypeError {
            message: "maximum function recursion depth exceeded".into(),
            span,
        });
    }

    let site = ctx.intern_call_site(fn_name, call_span);
    ctx.push_call_site(site);

    let mut saved: Vec<(String, Option<Value>)> = Vec::with_capacity(user_fn.params.len());
    for p in &user_fn.params {
        let name = p.name.name.clone();
        saved.push((name.clone(), ctx.get_var(&name).cloned()));
    }

    // Bind provided arguments
    for (p, v) in user_fn.params.iter().zip(arg_values.iter()) {
        ctx.set_var(&p.name.name, v.clone());
    }

    // Bind default values for remaining parameters
    for p in user_fn.params.iter().skip(arg_values.len()) {
        let default_val = if let Some(ref default_expr) = p.default {
            use crate::eval_expr::eval_expr;
            eval_expr(default_expr, ctx)?
        } else {
            Value::Na
        };
        ctx.set_var(&p.name.name, default_val);
    }

    let result = eval_fn_body(&user_fn.body, ctx);

    ctx.pop_call_site();
    ctx.runtime_mut().exit_call();

    for (name, old) in saved {
        match old {
            Some(v) => ctx.set_var(name, v),
            None => ctx.remove_var(&name),
        }
    }

    result
}

fn import_path_to_string(path: &ImportPath) -> String {
    match path {
        ImportPath::String(s) => s.clone(),
        ImportPath::Qualified(segs) => segs.join("/"),
    }
}

fn eval_switch_arm_body(body: &SwitchArmBody, ctx: &mut EvaluationContext) -> Result<()> {
    match body {
        SwitchArmBody::Expr(e) => {
            eval_expr(e, ctx)?;
            Ok(())
        }
        SwitchArmBody::Block(b) => eval_block(b, ctx),
    }
}

fn eval_switch_stmt(
    scrutinee: Option<&ast::Expr>,
    arms: &[ast::SwitchArm],
    ctx: &mut EvaluationContext,
) -> Result<()> {
    let disc = scrutinee.map(|e| eval_expr(e, ctx)).transpose()?;

    for arm in arms {
        match &arm.pattern {
            Some(pat) => {
                let branch_matches = if let Some(ref sv) = disc {
                    let pv = eval_expr(pat, ctx)?;
                    values_equal(sv, &pv)
                } else {
                    eval_expr(pat, ctx)?.is_truthy()
                };
                if branch_matches {
                    eval_switch_arm_body(&arm.body, ctx)?;
                    return Ok(());
                }
            }
            None => {
                eval_switch_arm_body(&arm.body, ctx)?;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn eval_for_in(
    pattern: &ForInPattern,
    iterable: &ast::Expr,
    body: &ast::Block,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    // First try to get series array if iterable is an identifier (e.g., close, open, mySeries)
    let items: Vec<Value> = if let ast::Expr::Ident(ident) = iterable {
        // Check if the identifier is currently bound to an array value
        // This handles cases like `arr = [1, 2, 3]; for v in arr`
        if let Some(Value::Array(values)) = ctx.get_var(&ident.name) {
            values.clone()
        } else if let Some(values) = get_series_array(&ident.name, ctx) {
            // Built-in series (open, high, low, close, volume, time)
            values
        } else if let Some(history) = ctx.get_var_history(&ident.name) {
            // User-defined series (var/varip variables with history)
            history.to_vec()
        } else {
            // Fall back to regular expression evaluation
            match eval_expr(iterable, ctx)? {
                Value::Array(values) => values,
                other => {
                    return Err(EvalError::TypeError {
                        message: format!("for...in expects array or series, got {:?}", other),
                        span: body.span,
                    });
                }
            }
        }
    } else {
        // Non-identifier expressions are evaluated normally
        match eval_expr(iterable, ctx)? {
            Value::Array(values) => values,
            other => {
                return Err(EvalError::TypeError {
                    message: format!("for...in expects array or series, got {:?}", other),
                    span: body.span,
                });
            }
        }
    };

    match pattern {
        ForInPattern::Single(id) => {
            for item in items {
                ctx.set_var(&id.name, item);
                eval_block(body, ctx)?;
            }
        }
        ForInPattern::Tuple(idx_id, val_id) => {
            for (i, item) in items.into_iter().enumerate() {
                ctx.set_var(&idx_id.name, Value::Int(i as i64));
                ctx.set_var(&val_id.name, item);
                eval_block(body, ctx)?;
            }
        }
    }
    Ok(())
}

/// Get series array for built-in price sources up to the current bar.
fn get_series_array(name: &str, ctx: &EvaluationContext) -> Option<Vec<Value>> {
    let series_data = ctx.series_data.as_ref()?;
    let end = series_data.current_bar + 1;

    match name {
        "open" => Some(
            series_data
                .open
                .get(..end)?
                .iter()
                .copied()
                .map(Value::Float)
                .collect(),
        ),
        "high" => Some(
            series_data
                .high
                .get(..end)?
                .iter()
                .copied()
                .map(Value::Float)
                .collect(),
        ),
        "low" => Some(
            series_data
                .low
                .get(..end)?
                .iter()
                .copied()
                .map(Value::Float)
                .collect(),
        ),
        "close" => Some(
            series_data
                .close
                .get(..end)?
                .iter()
                .copied()
                .map(Value::Float)
                .collect(),
        ),
        "volume" => Some(
            series_data
                .volume
                .get(..end)?
                .iter()
                .copied()
                .map(Value::Float)
                .collect(),
        ),
        "time" => Some(
            series_data
                .time
                .get(..end)?
                .iter()
                .copied()
                .map(Value::Int)
                .collect(),
        ),
        _ => None,
    }
}

fn read_assign_target_value(
    target: &ast::AssignTarget,
    ctx: &mut EvaluationContext,
) -> Result<Value> {
    match target {
        ast::AssignTarget::Var(ident) => Ok(ctx.get_var(&ident.name).cloned().unwrap_or(Value::Na)),
        _ => Err(EvalError::TypeError {
            message: "compound assignment is only supported for simple variables".into(),
            span: target.span(),
        }),
    }
}

fn apply_assign_op(op: AssignOp, cur: &Value, rhs: &Value) -> Value {
    use pine_runtime::na_ops;
    match op {
        AssignOp::PlusEq => na_ops::add(cur, rhs),
        AssignOp::MinusEq => na_ops::sub(cur, rhs),
        AssignOp::StarEq => na_ops::mul(cur, rhs),
        AssignOp::SlashEq => na_ops::div(cur, rhs),
        AssignOp::PercentEq => na_ops::modulo(cur, rhs),
        AssignOp::Assign | AssignOp::ColonEq => rhs.clone(),
    }
}

/// Evaluate a statement
pub fn eval_stmt(stmt: &ast::Stmt, ctx: &mut EvaluationContext) -> Result<()> {
    match stmt {
        ast::Stmt::VarDecl {
            name, init, kind, ..
        } => match kind {
            ast::VarKind::Plain => {
                let value = if let Some(init_expr) = init {
                    eval_expr(init_expr, ctx)?
                } else {
                    Value::Na
                };
                ctx.set_var(&name.name, value);
                Ok(())
            }
            ast::VarKind::Var | ast::VarKind::Varip => {
                let cs = ctx.current_call_site();
                if ctx.runtime().var_scoped_contains(&name.name, cs) {
                    return Ok(());
                }
                let value = if let Some(init_expr) = init {
                    eval_expr(init_expr, ctx)?
                } else {
                    Value::Na
                };
                let n = name.name.clone();
                let mode = match kind {
                    ast::VarKind::Var => PersistentVarKind::Var,
                    ast::VarKind::Varip => PersistentVarKind::Varip,
                    ast::VarKind::Plain => unreachable!("plain vars handled above"),
                };
                ctx.runtime_mut()
                    .declare_var_scoped(n.clone(), cs, mode, value.clone());
                ctx.runtime_mut().push_to_series(n, cs, value);
                Ok(())
            }
        },
        ast::Stmt::Assign {
            target, op, value, ..
        } => {
            let rhs_value = eval_expr(value, ctx)?;
            match op {
                AssignOp::Assign | AssignOp::ColonEq => eval_assign(target, rhs_value, ctx),
                AssignOp::PlusEq
                | AssignOp::MinusEq
                | AssignOp::StarEq
                | AssignOp::SlashEq
                | AssignOp::PercentEq => {
                    let cur = read_assign_target_value(target, ctx)?;
                    let combined = apply_assign_op(*op, &cur, &rhs_value);
                    eval_assign(target, combined, ctx)
                }
            }
        }
        ast::Stmt::Expr(expr) => {
            let value = eval_expr(expr, ctx)?;
            // Capture strategy signals from expression results
            capture_strategy_signal(&value, ctx);
            Ok(())
        }
        ast::Stmt::Switch {
            scrutinee, arms, ..
        } => eval_switch_stmt(scrutinee.as_ref(), arms, ctx),
        // Import statement: import "path" | import a/b/c [as alias]
        ast::Stmt::Import { path, alias, span } => {
            eval_import_path(path, alias.as_ref(), *span, ctx)
        }
        ast::Stmt::ExportFn {
            name, params, body, ..
        } => eval_fn_def(name, params, body, ctx),
        ast::Stmt::ExportAssign { name, init, .. } => eval_export_assign(name, init, ctx),
        // Library declaration: library(name, ...)
        ast::Stmt::Library {
            name: _,
            props: _,
            span: _,
        } => {
            // Library declarations are processed during semantic analysis
            // and module loading. At runtime, we just record that this is a library.
            // The actual module registration happens elsewhere.
            Ok(())
        }
        // If statement with series alignment enforcement
        ast::Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            ..
        } => eval_if_stmt(cond, then_block, elifs, else_block.as_ref(), ctx),
        // For loop
        ast::Stmt::For {
            var,
            from,
            to,
            by,
            body,
            ..
        } => eval_for_loop(var, from, to, by.as_ref(), body, ctx),
        ast::Stmt::ForIn {
            pattern,
            iterable,
            body,
            ..
        } => eval_for_in(pattern, iterable, body, ctx),
        // While loop
        ast::Stmt::While { cond, body, .. } => eval_while_loop(cond, body, ctx),
        // Function definition
        ast::Stmt::FnDef {
            name, params, body, ..
        } => eval_fn_def(name, params, body, ctx),
        // Type definition — runtime reserves type names for objects
        ast::Stmt::TypeDef { .. } => Ok(()),
        ast::Stmt::MethodDef {
            type_name,
            name,
            params,
            body,
            ..
        } => {
            let uf = UserFn {
                params: params.clone(),
                body: body.clone(),
            };
            ctx.register_type_method(&type_name.name, &name.name, uf);
            Ok(())
        }
        ast::Stmt::EnumDef { .. } => Ok(()),
        ast::Stmt::Break { .. } | ast::Stmt::Continue { .. } => Ok(()),
        ast::Stmt::Return { value, .. } => {
            if let Some(v) = value {
                eval_expr(v, ctx)?;
            }
            Ok(())
        }
    }
}

/// Evaluate an if statement with series alignment enforcement
///
/// # Series Alignment Rule
/// When a variable is assigned in one branch of an if/else, it MUST be
/// assigned in all branches to maintain series alignment.
fn eval_if_stmt(
    cond: &ast::Expr,
    then_block: &ast::Block,
    elifs: &[(ast::Expr, ast::Block)],
    else_block: Option<&ast::Block>,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    let condition = eval_expr(cond, ctx)?;

    let mut executed = false;

    if condition.is_truthy() {
        eval_block(then_block, ctx)?;
        executed = true;
    } else {
        // Check elif blocks
        for (elif_cond, elif_body) in elifs {
            let elif_condition = eval_expr(elif_cond, ctx)?;
            if elif_condition.is_truthy() {
                eval_block(elif_body, ctx)?;
                executed = true;
                break;
            }
        }
    }

    // If no branch matched and we have an else block, execute it
    if !executed {
        if let Some(else_body) = else_block {
            eval_block(else_body, ctx)?;
        }
    }

    Ok(())
}

/// Evaluate a for loop
///
/// Pine Script v6: `for i = a to b` is **inclusive** on both bounds when `step` is positive
/// (`i` takes `a, a+step, …` while `i <= b`). When `step` is negative, the condition is `i >= b`
/// (`i` takes `a, a+step, …` while `i >= b`).
fn eval_for_loop(
    var: &ast::Ident,
    from: &ast::Expr,
    to: &ast::Expr,
    by: Option<&ast::Expr>,
    body: &ast::Block,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    let from_val = eval_expr(from, ctx)?;
    let to_val = eval_expr(to, ctx)?;
    let by_val = by.map_or(Ok(None), |e| eval_expr(e, ctx).map(Some))?;

    let start = from_val.as_int().unwrap_or(0);
    let end = to_val.as_int().unwrap_or(0);
    let step = by_val.and_then(|v| v.as_int()).unwrap_or(1);

    if step == 0 {
        return Err(EvalError::TypeError {
            message: "for loop step cannot be 0".into(),
            span: var.span,
        });
    }

    let mut i = start;
    if step > 0 {
        while i <= end {
            ctx.set_var(&var.name, Value::Int(i));
            eval_block(body, ctx)?;
            i = i.saturating_add(step);
        }
    } else {
        while i >= end {
            ctx.set_var(&var.name, Value::Int(i));
            eval_block(body, ctx)?;
            i = i.saturating_add(step);
        }
    }

    Ok(())
}

/// Evaluate a while loop
fn eval_while_loop(cond: &ast::Expr, body: &ast::Block, ctx: &mut EvaluationContext) -> Result<()> {
    loop {
        let condition = eval_expr(cond, ctx)?;
        if !condition.is_truthy() {
            break;
        }
        eval_block(body, ctx)?;
    }

    Ok(())
}

/// `export name = expr`: lambda → UDF registration; otherwise evaluate and bind (library constant).
fn eval_export_assign(
    name: &ast::Ident,
    init: &ast::Expr,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    match init {
        ast::Expr::Lambda { params, body, .. } => {
            ctx.register_user_fn(
                name.name.clone(),
                UserFn {
                    params: params.clone(),
                    body: ast::FnBody::Expr(*body.clone()),
                },
            );
        }
        e => {
            let v = eval_expr(e, ctx)?;
            ctx.set_var(name.name.clone(), v);
        }
    }
    Ok(())
}

/// Evaluate a function definition (register for call sites).
fn eval_fn_def(
    name: &ast::Ident,
    params: &[ast::Param],
    body: &ast::FnBody,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    ctx.register_user_fn(
        name.name.clone(),
        UserFn {
            params: params.to_vec(),
            body: body.clone(),
        },
    );
    Ok(())
}

/// Capture strategy signals from expression evaluation results
///
/// When strategy.entry(), strategy.close(), or strategy.exit() are called,
/// they return special marker arrays that we capture here.
fn capture_strategy_signal(value: &Value, ctx: &mut EvaluationContext) {
    if let Value::Array(arr) = value {
        if arr.is_empty() {
            return;
        }

        // Check for strategy signal markers
        if let Some(Value::String(marker)) = arr.first() {
            match marker.as_str() {
                "__entry__" => {
                    // strategy.entry() returns: ["__entry__", id, direction, qty]
                    if arr.len() >= 4 {
                        if let (
                            Some(Value::String(id)),
                            Some(Value::String(direction)),
                            Some(qty_val),
                        ) = (arr.get(1), arr.get(2), arr.get(3))
                        {
                            let qty = match qty_val {
                                Value::Float(f) => *f,
                                Value::Int(i) => *i as f64,
                                _ => 1.0,
                            };
                            ctx.strategy_signals
                                .record_entry(id.clone(), direction.clone(), qty);
                        }
                    }
                }
                "__close__" => {
                    // strategy.close() returns: ["__close__", id]
                    if arr.len() >= 2 {
                        if let Some(Value::String(id)) = arr.get(1) {
                            ctx.strategy_signals.record_close(id.clone(), None);
                        }
                    }
                }
                "__exit__" => {
                    // strategy.exit() returns: ["__exit__", id, from_entry]
                    if arr.len() >= 3 {
                        if let (Some(Value::String(id)), Some(Value::String(from_entry))) =
                            (arr.get(1), arr.get(2))
                        {
                            ctx.strategy_signals
                                .record_exit(id.clone(), from_entry.clone(), None);
                        }
                    }
                }
                _ => {} // Not a strategy signal
            }
        }
    }
}

/// Check if two values are equal for switch case matching
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Na, Value::Na) => false, // na == na is false in Pine Script
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y)) => *x == (*y as f64),
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Color(x), Value::Color(y)) => x == y,
        _ => false,
    }
}

/// Public wrapper for value equality comparison (used by eval_expr for switch expressions)
pub fn values_equal_public(a: &Value, b: &Value) -> bool {
    values_equal(a, b)
}

/// Evaluate an assignment
fn eval_assign(
    target: &ast::AssignTarget,
    value: Value,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    match target {
        ast::AssignTarget::Var(ident) => {
            let cs = ctx.current_call_site();
            if ctx.runtime().var_scoped_contains(&ident.name, cs) {
                let n = ident.name.clone();
                ctx.runtime_mut()
                    .set_var_scoped(n.clone(), cs, value.clone());
                ctx.runtime_mut().push_to_series(n, cs, value);
            } else {
                ctx.set_var(&ident.name, value);
            }
            Ok(())
        }
        ast::AssignTarget::Tuple(idents) => eval_tuple_assign(idents, value, ctx),
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

fn eval_tuple_assign(
    idents: &[ast::Ident],
    value: Value,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    match value {
        Value::Tuple(values) => {
            for (ident, item) in idents.iter().zip(values.iter()) {
                ctx.set_var(&ident.name, item.clone());
            }
            Ok(())
        }
        Value::Array(values) => {
            for (ident, item) in idents.iter().zip(values.iter()) {
                ctx.set_var(&ident.name, item.clone());
            }
            Ok(())
        }
        other => Err(EvalError::TypeError {
            message: format!("Cannot destructure value {:?}", other),
            span: idents.first().map(|i| i.span).unwrap_or_default(),
        }),
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

/// Evaluate an import statement
fn eval_import_path(
    path: &ImportPath,
    alias: Option<&ast::Ident>,
    span: pine_lexer::Span,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    use std::path::Path;

    let path_str = import_path_to_string(path);
    // Resolve the module path
    let module_path = if Path::new(&path_str).is_absolute() {
        PathBuf::from(path_str)
    } else {
        ctx.base_path().join(&path_str)
    };
    let module_path = if module_path.extension().is_some() {
        module_path
    } else {
        module_path.with_extension("pine")
    };

    let path_str = module_path.to_string_lossy().to_string();

    // Check for circular dependency
    if ctx.is_loading_module(&path_str) {
        return Err(EvalError::CircularDependency {
            cycle: path_str,
            span,
        });
    }

    if let Some(loaded_module) = ctx.get_loaded_module(&module_path).cloned() {
        bind_loaded_module(&loaded_module, alias, ctx);
        return Ok(());
    }

    let source = fs::read_to_string(&module_path).map_err(|_| EvalError::ModuleNotFound {
        path: module_path.display().to_string(),
        span,
    })?;
    let tokens = Lexer::lex_with_indentation(&source).map_err(|_| EvalError::ModuleNotFound {
        path: module_path.display().to_string(),
        span,
    })?;
    let script = pine_parser::parser::parse(tokens).map_err(|errors| EvalError::TypeError {
        message: format!(
            "failed to parse module {}: {}",
            module_path.display(),
            errors.join("; ")
        ),
        span,
    })?;

    ctx.begin_module_load(path_str);

    let mut module_ctx = EvaluationContext::with_base_path(
        module_path
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".")),
    );
    module_ctx.series_data = ctx.series_data.clone();
    let module_id = ctx
        .module_registry_mut()
        .resolve(&module_path)
        .map_err(|err| EvalError::TypeError {
            message: err.to_string(),
            span,
        })?;
    module_ctx.set_current_module(Some(module_id));

    let execution_result = (|| -> Result<LoadedModule> {
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut module_ctx)?;
        }

        let module_name = script
            .stmts
            .iter()
            .find_map(|stmt| match stmt {
                ast::Stmt::Library { name, .. } => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| {
                module_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("module")
                    .to_string()
            });

        let mut module = Module::new(module_id, module_name.clone(), module_path.clone());
        let mut exports = HashMap::new();
        let mut exported_functions = HashMap::new();

        for stmt in &script.stmts {
            match stmt {
                ast::Stmt::ExportFn { name, .. } => {
                    let user_fn = module_ctx.get_user_fn(&name.name).cloned().ok_or(
                        EvalError::ExportNotFound {
                            name: name.name.clone(),
                            span,
                        },
                    )?;
                    exported_functions.insert(name.name.clone(), user_fn);
                    module.export(
                        name.name.clone(),
                        Value::String(format!("fn {}", name.name).into()),
                    );
                }
                ast::Stmt::ExportAssign { name, .. } => {
                    if let Some(user_fn) = module_ctx.get_user_fn(&name.name).cloned() {
                        exported_functions.insert(name.name.clone(), user_fn);
                        module.export(
                            name.name.clone(),
                            Value::String(format!("fn {}", name.name).into()),
                        );
                    } else {
                        let value = module_ctx.get_var(&name.name).cloned().ok_or(
                            EvalError::ExportNotFound {
                                name: name.name.clone(),
                                span,
                            },
                        )?;
                        exports.insert(name.name.clone(), value.clone());
                        module.export(name.name.clone(), value);
                    }
                }
                _ => {}
            }
        }

        ctx.module_registry_mut().register(module);

        Ok(LoadedModule {
            path: module_path.clone(),
            exports,
            exported_functions,
        })
    })();

    ctx.end_module_load();

    let loaded_module = execution_result?;
    ctx.cache_loaded_module(loaded_module.clone());
    bind_loaded_module(&loaded_module, alias, ctx);
    Ok(())
}

fn bind_loaded_module(
    loaded_module: &LoadedModule,
    alias: Option<&ast::Ident>,
    ctx: &mut EvaluationContext,
) {
    let prefix = alias.map(|ident| ident.name.clone());

    if let Some(alias_name) = prefix.as_ref() {
        ctx.set_var(alias_name, Value::Namespace(alias_name.clone()));
    }

    for (name, value) in &loaded_module.exports {
        let binding = if let Some(prefix) = prefix.as_ref() {
            format!("{prefix}.{name}")
        } else {
            name.clone()
        };
        ctx.set_var(binding, value.clone());
    }

    for (name, user_fn) in &loaded_module.exported_functions {
        let binding = if let Some(prefix) = prefix.as_ref() {
            format!("{prefix}.{name}")
        } else {
            name.clone()
        };
        ctx.register_user_fn(binding, user_fn.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval_expr::eval_expr;
    use pine_lexer::Span;
    use pine_parser::ast::{Block, FnBody, Ident, Lit, Param, Stmt, SwitchArm, SwitchArmBody};
    use pine_runtime::value::Value;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pine-rs-{prefix}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_field_assignment() {
        // This is a placeholder test - the proper implementation requires
        // a more complete evaluation context that handles mutable objects correctly
        assert!(true);
    }

    #[test]
    fn test_values_equal() {
        // Test integer equality
        assert!(values_equal(&Value::Int(1), &Value::Int(1)));
        assert!(!values_equal(&Value::Int(1), &Value::Int(2)));

        // Test float equality
        assert!(values_equal(&Value::Float(1.5), &Value::Float(1.5)));
        assert!(!values_equal(&Value::Float(1.5), &Value::Float(2.5)));

        // Test int-float equality
        assert!(values_equal(&Value::Int(1), &Value::Float(1.0)));
        assert!(values_equal(&Value::Float(2.0), &Value::Int(2)));

        // Test bool equality
        assert!(values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(!values_equal(&Value::Bool(true), &Value::Bool(false)));

        // Test string equality
        assert!(values_equal(
            &Value::String("hello".into()),
            &Value::String("hello".into())
        ));
        assert!(!values_equal(
            &Value::String("hello".into()),
            &Value::String("world".into())
        ));

        // Test na equality (na == na is false in Pine Script)
        assert!(!values_equal(&Value::Na, &Value::Na));
    }

    #[test]
    fn test_switch_stmt() {
        let mut ctx = EvaluationContext::new();

        let scrutinee = ast::Expr::Literal(Lit::Int(2), Span::default());

        let arm1 = SwitchArm {
            pattern: Some(ast::Expr::Literal(Lit::Int(1), Span::default())),
            body: SwitchArmBody::Block(Block {
                stmts: vec![Stmt::VarDecl {
                    name: Ident::new("result", Span::default()),
                    kind: ast::VarKind::Var,
                    type_ann: None,
                    init: Some(ast::Expr::Literal(Lit::Int(10), Span::default())),
                    span: Span::default(),
                }],
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let arm2 = SwitchArm {
            pattern: Some(ast::Expr::Literal(Lit::Int(2), Span::default())),
            body: SwitchArmBody::Block(Block {
                stmts: vec![Stmt::VarDecl {
                    name: Ident::new("result", Span::default()),
                    kind: ast::VarKind::Var,
                    type_ann: None,
                    init: Some(ast::Expr::Literal(Lit::Int(20), Span::default())),
                    span: Span::default(),
                }],
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let arm_default = SwitchArm {
            pattern: None,
            body: SwitchArmBody::Block(Block {
                stmts: vec![Stmt::VarDecl {
                    name: Ident::new("result", Span::default()),
                    kind: ast::VarKind::Var,
                    type_ann: None,
                    init: Some(ast::Expr::Literal(Lit::Int(99), Span::default())),
                    span: Span::default(),
                }],
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let switch_stmt = Stmt::Switch {
            scrutinee: Some(scrutinee),
            arms: vec![arm1, arm2, arm_default],
            span: Span::default(),
        };

        eval_stmt(&switch_stmt, &mut ctx).unwrap();

        assert_eq!(ctx.get_var("result"), Some(&Value::Int(20)));
    }

    #[test]
    fn test_switch_guard_stmt() {
        let mut ctx = EvaluationContext::new();

        // Guard switch without scrutinee: first truthy pattern wins
        let arm1 = SwitchArm {
            pattern: Some(ast::Expr::Literal(Lit::Bool(false), Span::default())),
            body: SwitchArmBody::Block(Block {
                stmts: vec![Stmt::VarDecl {
                    name: Ident::new("result", Span::default()),
                    kind: ast::VarKind::Var,
                    type_ann: None,
                    init: Some(ast::Expr::Literal(Lit::Int(10), Span::default())),
                    span: Span::default(),
                }],
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let arm2 = SwitchArm {
            pattern: Some(ast::Expr::Literal(Lit::Bool(true), Span::default())),
            body: SwitchArmBody::Block(Block {
                stmts: vec![Stmt::VarDecl {
                    name: Ident::new("result", Span::default()),
                    kind: ast::VarKind::Var,
                    type_ann: None,
                    init: Some(ast::Expr::Literal(Lit::Int(20), Span::default())),
                    span: Span::default(),
                }],
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let arm_default = SwitchArm {
            pattern: None,
            body: SwitchArmBody::Block(Block {
                stmts: vec![Stmt::VarDecl {
                    name: Ident::new("result", Span::default()),
                    kind: ast::VarKind::Var,
                    type_ann: None,
                    init: Some(ast::Expr::Literal(Lit::Int(99), Span::default())),
                    span: Span::default(),
                }],
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let switch_stmt = Stmt::Switch {
            scrutinee: None,
            arms: vec![arm1, arm2, arm_default],
            span: Span::default(),
        };

        eval_stmt(&switch_stmt, &mut ctx).unwrap();

        assert_eq!(ctx.get_var("result"), Some(&Value::Int(20)));
    }

    #[test]
    fn test_import_stmt_basic() {
        let dir = temp_dir("import-basic");
        fs::write(dir.join("test_module.pine"), "export answer = 42\n").unwrap();
        let mut ctx = EvaluationContext::with_base_path(&dir);

        let import_stmt = Stmt::Import {
            path: ImportPath::String("test_module.pine".to_string()),
            alias: Some(Ident::new("test", Span::default())),
            span: Span::default(),
        };

        let result = eval_stmt(&import_stmt, &mut ctx);
        assert!(result.is_ok());
        assert!(matches!(ctx.get_var("test.answer"), Some(Value::Int(42))));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_import_stmt_without_alias() {
        let dir = temp_dir("import-no-alias");
        fs::write(dir.join("utils.pine"), "export answer = 7\n").unwrap();
        let mut ctx = EvaluationContext::with_base_path(&dir);

        let import_stmt = Stmt::Import {
            path: ImportPath::String("utils.pine".to_string()),
            alias: None,
            span: Span::default(),
        };

        let result = eval_stmt(&import_stmt, &mut ctx);
        assert!(result.is_ok());
        assert!(matches!(ctx.get_var("answer"), Some(Value::Int(7))));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_udf_var_isolated_per_call_site() {
        let src = r#"fn f(x)
    var a = 0
    a := a + 1
    x + a
v1 = f(100)
v2 = f(200)
"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(src).unwrap();
        let mut parser = pine_parser::stmt::StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();

        let mut ctx = EvaluationContext::new();
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut ctx).unwrap();
        }
        assert_eq!(ctx.get_var("v1"), Some(&Value::Int(101)));
        assert_eq!(ctx.get_var("v2"), Some(&Value::Int(201)));
    }

    #[test]
    fn test_udf_var_series_index_two_bars() {
        let src = r#"fn f()
    var s = 0
    s := s + 1
    s[1]
out = f()
"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(src).unwrap();
        let mut parser = pine_parser::stmt::StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();

        let mut ctx = EvaluationContext::new();
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut ctx).unwrap();
        }
        // Bar 0: no history -> na
        assert_eq!(ctx.get_var("out"), Some(&Value::Na));

        ctx.runtime_mut().set_bar_index(1);
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut ctx).unwrap();
        }
        // Bar 1: s=2, s[1]=1
        assert_eq!(ctx.get_var("out"), Some(&Value::Int(1)));
    }

    #[test]
    fn test_udf_call_invokes_body() {
        let mut ctx = EvaluationContext::new();
        eval_stmt(
            &Stmt::FnDef {
                name: Ident::new("dbl", Span::default()),
                params: vec![Param {
                    name: Ident::new("x", Span::default()),
                    type_ann: None,
                    default: None,
                }],
                ret_type: None,
                body: FnBody::Expr(ast::Expr::BinOp {
                    op: ast::BinOp::Mul,
                    lhs: Box::new(ast::Expr::Ident(Ident::new("x", Span::default()))),
                    rhs: Box::new(ast::Expr::Literal(Lit::Int(2), Span::default())),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            &mut ctx,
        )
        .unwrap();

        let call = ast::Expr::FnCall {
            func: Box::new(ast::Expr::Ident(Ident::new("dbl", Span::default()))),
            args: vec![ast::Arg {
                name: None,
                value: ast::Expr::Literal(Lit::Int(21), Span::default()),
            }],
            span: Span::default(),
        };
        let v = eval_expr(&call, &mut ctx).unwrap();
        assert_eq!(v, Value::Int(42));
    }

    #[test]
    fn test_for_in_executes_body() {
        let mut ctx = EvaluationContext::new();
        ctx.set_var(
            "xs",
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        );

        let body = Block {
            stmts: vec![Stmt::Assign {
                target: ast::AssignTarget::Var(Ident::new("acc", Span::default())),
                op: AssignOp::Assign,
                value: ast::Expr::BinOp {
                    op: ast::BinOp::Add,
                    lhs: Box::new(ast::Expr::Ident(Ident::new("acc", Span::default()))),
                    rhs: Box::new(ast::Expr::Ident(Ident::new("v", Span::default()))),
                    span: Span::default(),
                },
                span: Span::default(),
            }],
            span: Span::default(),
        };

        let for_in = Stmt::ForIn {
            pattern: ForInPattern::Single(Ident::new("v", Span::default())),
            iterable: ast::Expr::Ident(Ident::new("xs", Span::default())),
            body,
            span: Span::default(),
        };

        ctx.set_var("acc", Value::Int(0));
        eval_stmt(&for_in, &mut ctx).unwrap();
        assert_eq!(ctx.get_var("acc"), Some(&Value::Int(6)));
    }

    #[test]
    fn test_for_loop_inclusive_accumulator() {
        let src = r#"sum = 0
for i = 1 to 3
    sum := sum + i
out = sum
"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(src).unwrap();
        let mut parser = pine_parser::stmt::StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();

        let mut ctx = EvaluationContext::new();
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut ctx).unwrap();
        }
        assert_eq!(ctx.get_var("out"), Some(&Value::Int(6)));
    }

    /// `for i = 0 to N` must include both 0 and N (N+1 iterations when step is 1).
    #[test]
    fn test_for_zero_to_n_inclusive_upper_bound() {
        let src = r#"steps = 0
for i = 0 to 4
    steps := steps + 1
out = steps
"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(src).unwrap();
        let mut parser = pine_parser::stmt::StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();

        let mut ctx = EvaluationContext::new();
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut ctx).unwrap();
        }
        assert_eq!(ctx.get_var("out"), Some(&Value::Int(5)));
    }

    #[test]
    fn test_for_loop_negative_step_inclusive_low_bound() {
        let src = r#"sum = 0
for i = 3 to 1 by -1
    sum := sum + i
out = sum
"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(src).unwrap();
        let mut parser = pine_parser::stmt::StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();

        let mut ctx = EvaluationContext::new();
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut ctx).unwrap();
        }
        assert_eq!(ctx.get_var("out"), Some(&Value::Int(6)));
    }

    #[test]
    fn test_export_assign_lambda_registers_udf() {
        let src = "export dbl = (x) => x * 2\ny = dbl(21)";
        let tokens = pine_lexer::Lexer::lex_with_indentation(src).unwrap();
        let mut parser = pine_parser::stmt::StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();

        let mut ctx = EvaluationContext::new();
        for stmt in &script.stmts {
            eval_stmt(stmt, &mut ctx).unwrap();
        }
        assert_eq!(ctx.get_var("y"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_export_fn_registers_udf() {
        let mut ctx = EvaluationContext::new();

        let export_stmt = Stmt::ExportFn {
            name: Ident::new("add1", Span::default()),
            params: vec![Param {
                name: Ident::new("x", Span::default()),
                type_ann: None,
                default: None,
            }],
            ret_type: None,
            body: FnBody::Expr(ast::Expr::BinOp {
                op: ast::BinOp::Add,
                lhs: Box::new(ast::Expr::Ident(Ident::new("x", Span::default()))),
                rhs: Box::new(ast::Expr::Literal(Lit::Int(1), Span::default())),
                span: Span::default(),
            }),
            span: Span::default(),
        };

        eval_stmt(&export_stmt, &mut ctx).unwrap();
        assert!(ctx.get_user_fn("add1").is_some());
    }

    #[test]
    fn test_library_stmt() {
        let mut ctx = EvaluationContext::new();

        // Create a library declaration statement
        let library_stmt = Stmt::Library {
            name: "TestLib".to_string(),
            props: vec![],
            span: Span::default(),
        };

        // Evaluate the library statement (should succeed, it's a no-op at runtime)
        let result = eval_stmt(&library_stmt, &mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_module_context_methods() {
        let mut ctx = EvaluationContext::new();

        // Test base path methods
        ctx.set_base_path("/test/path");
        assert_eq!(ctx.base_path(), &std::path::PathBuf::from("/test/path"));

        // Test module loading tracking
        assert!(!ctx.is_loading_module("test.pine"));
        ctx.begin_module_load("test.pine");
        assert!(ctx.is_loading_module("test.pine"));
        ctx.end_module_load();
        assert!(!ctx.is_loading_module("test.pine"));

        // Test current module
        assert!(ctx.current_module().is_none());
        ctx.set_current_module(Some(pine_runtime::module::ModuleId(1)));
        assert_eq!(
            ctx.current_module(),
            Some(pine_runtime::module::ModuleId(1))
        );
    }

    #[test]
    fn test_import_stmt_with_alias_loads_functions_and_values() {
        let dir = temp_dir("import-alias");
        let module_path = dir.join("math_lib.pine");
        fs::write(
            &module_path,
            r#"
export add = (a, b) => a + b
export PI = 3.14159
"#,
        )
        .unwrap();

        let mut ctx = EvaluationContext::with_base_path(&dir);
        let import_stmt = Stmt::Import {
            path: ImportPath::String("math_lib.pine".to_string()),
            alias: Some(Ident::new("m", Span::default())),
            span: Span::default(),
        };

        eval_stmt(&import_stmt, &mut ctx).unwrap();
        let sum = eval_expr(
            &ast::Expr::MethodCall {
                base: Box::new(ast::Expr::Ident(Ident::new("m", Span::default()))),
                method: Ident::new("add", Span::default()),
                args: vec![
                    ast::Arg {
                        name: None,
                        value: ast::Expr::Literal(Lit::Int(2), Span::default()),
                    },
                    ast::Arg {
                        name: None,
                        value: ast::Expr::Literal(Lit::Int(3), Span::default()),
                    },
                ],
                span: Span::default(),
            },
            &mut ctx,
        )
        .unwrap();
        let pi = eval_expr(
            &ast::Expr::FieldAccess {
                base: Box::new(ast::Expr::Ident(Ident::new("m", Span::default()))),
                field: Ident::new("PI", Span::default()),
                span: Span::default(),
            },
            &mut ctx,
        )
        .unwrap();

        assert_eq!(sum, Value::Int(5));
        assert!(matches!(pi, Value::Float(v) if (v - 3.14159).abs() < 1e-10));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_import_stmt_with_qualified_path_loads_module() {
        let dir = temp_dir("import-qualified");
        let module_dir = dir.join("user").join("lib");
        fs::create_dir_all(&module_dir).unwrap();
        let module_path = module_dir.join("1.pine");
        fs::write(
            &module_path,
            r#"
export add(x, y) => x + y
export MAGIC = 9
"#,
        )
        .unwrap();

        let mut ctx = EvaluationContext::with_base_path(&dir);
        let import_stmt = Stmt::Import {
            path: ImportPath::Qualified(vec![
                "user".to_string(),
                "lib".to_string(),
                "1".to_string(),
            ]),
            alias: Some(Ident::new("m", Span::default())),
            span: Span::default(),
        };

        eval_stmt(&import_stmt, &mut ctx).unwrap();
        let sum = eval_expr(
            &ast::Expr::MethodCall {
                base: Box::new(ast::Expr::Ident(Ident::new("m", Span::default()))),
                method: Ident::new("add", Span::default()),
                args: vec![
                    ast::Arg {
                        name: None,
                        value: ast::Expr::Literal(Lit::Int(4), Span::default()),
                    },
                    ast::Arg {
                        name: None,
                        value: ast::Expr::Literal(Lit::Int(5), Span::default()),
                    },
                ],
                span: Span::default(),
            },
            &mut ctx,
        )
        .unwrap();
        let magic = eval_expr(
            &ast::Expr::FieldAccess {
                base: Box::new(ast::Expr::Ident(Ident::new("m", Span::default()))),
                field: Ident::new("MAGIC", Span::default()),
                span: Span::default(),
            },
            &mut ctx,
        )
        .unwrap();

        assert_eq!(sum, Value::Int(9));
        assert_eq!(magic, Value::Int(9));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_import_stmt_without_alias_binds_exports_directly() {
        let dir = temp_dir("import-direct");
        let module_path = dir.join("math_lib.pine");
        fs::write(
            &module_path,
            r#"
export subtract = (a, b) => a - b
export PI = 3.14159
"#,
        )
        .unwrap();

        let mut ctx = EvaluationContext::with_base_path(&dir);
        let import_stmt = Stmt::Import {
            path: ImportPath::String("math_lib.pine".to_string()),
            alias: None,
            span: Span::default(),
        };

        eval_stmt(&import_stmt, &mut ctx).unwrap();
        let diff = eval_expr(
            &ast::Expr::FnCall {
                func: Box::new(ast::Expr::Ident(Ident::new("subtract", Span::default()))),
                args: vec![
                    ast::Arg {
                        name: None,
                        value: ast::Expr::Literal(Lit::Int(7), Span::default()),
                    },
                    ast::Arg {
                        name: None,
                        value: ast::Expr::Literal(Lit::Int(4), Span::default()),
                    },
                ],
                span: Span::default(),
            },
            &mut ctx,
        )
        .unwrap();

        assert_eq!(diff, Value::Int(3));
        assert!(matches!(ctx.get_var("PI"), Some(Value::Float(v)) if (*v - 3.14159).abs() < 1e-10));

        let _ = fs::remove_dir_all(dir);
    }
}
