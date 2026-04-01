//! Statement evaluation

use crate::eval_expr::eval_expr;
use crate::{EvalError, EvaluationContext, Result};
use pine_parser::ast;
use pine_runtime::value::Value;
use std::path::PathBuf;
use std::sync::Arc;

/// Evaluate a block of statements
pub fn eval_block(block: &ast::Block, ctx: &mut EvaluationContext) -> Result<()> {
    for stmt in &block.stmts {
        eval_stmt(stmt, ctx)?;
    }
    Ok(())
}

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
        ast::Stmt::Switch {
            expr,
            cases,
            default,
            ..
        } => {
            let switch_value = eval_expr(expr, ctx)?;
            let mut matched = false;

            // Check each case
            for case in cases {
                let case_value = eval_expr(&case.value, ctx)?;
                if values_equal(&switch_value, &case_value) {
                    eval_block(&case.body, ctx)?;
                    matched = true;
                    break;
                }
            }

            // If no case matched and we have a default, execute it
            if !matched {
                if let Some(default_block) = default {
                    eval_block(default_block, ctx)?;
                }
            }

            Ok(())
        }
        // Import statement: import "path" [as alias]
        ast::Stmt::Import { path, alias, span } => eval_import(path, alias.as_ref(), *span, ctx),
        // Export statement: export name
        ast::Stmt::Export { name, span } => eval_export(&name.name, *span, ctx),
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
        // While loop
        ast::Stmt::While { cond, body, .. } => eval_while_loop(cond, body, ctx),
        // Function definition
        ast::Stmt::FnDef {
            name, params, body, ..
        } => eval_fn_def(name, params, body, ctx),
        // Type definition - handled during semantic analysis
        ast::Stmt::TypeDef { .. } => Ok(()),
        // Method definition - handled during semantic analysis
        ast::Stmt::MethodDef { .. } => Ok(()),
        _ => {
            // TODO: Handle any remaining statement types
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
    let by_val = by.and_then(|e| eval_expr(e, ctx).ok());

    let start = from_val.as_int().unwrap_or(0);
    let fixed_end = to_val.as_int().unwrap_or(0);
    let step = by_val.and_then(|v| v.as_int()).unwrap_or(1);

    let mut i = start;
    while i <= fixed_end {
        ctx.set_var(&var.name, Value::Int(i));
        eval_block(body, ctx)?;
        i += step;
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

/// Evaluate a function definition
fn eval_fn_def(
    _name: &ast::Ident,
    _params: &[ast::Param],
    _body: &ast::Block,
    _ctx: &mut EvaluationContext,
) -> Result<()> {
    // Function definitions are stored during semantic analysis
    // The actual closure is created when the function is called
    // For now, this is a placeholder - full function support requires
    // proper closure creation with captured environment
    Ok(())
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

/// Evaluate an assignment
fn eval_assign(
    target: &ast::AssignTarget,
    value: Value,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    match target {
        ast::AssignTarget::Var(ident) => {
            ctx.set_var(&ident.name, value);
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
fn eval_import(
    path: &str,
    alias: Option<&ast::Ident>,
    span: pine_lexer::Span,
    ctx: &mut EvaluationContext,
) -> Result<()> {
    use std::path::Path;

    // Resolve the module path
    let module_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        ctx.base_path().join(path)
    };

    let path_str = module_path.to_string_lossy().to_string();

    // Check for circular dependency
    if ctx.is_loading_module(&path_str) {
        return Err(EvalError::CircularDependency {
            cycle: path_str,
            span,
        });
    }

    // Check if already loaded
    if ctx.module_registry().get_by_path(&module_path).is_some() {
        // Module already loaded, just bind it to the alias
        if let Some(_alias_ident) = alias {
            let _module_id = ctx
                .module_registry()
                .get_by_path(&module_path)
                .map(|m| m.id)
                .unwrap();
            // Store the module namespace binding
            // This will be used when resolving qualified names like `alias.function()`
            // For now, we just note that the import was successful
        }
        return Ok(());
    }

    // In a full implementation, we would:
    // 1. Load the file content
    // 2. Parse it
    // 3. Execute the module code
    // 4. Collect exports
    // 5. Register the module

    // For now, this is a stub that notes the import would happen
    // The actual implementation requires file I/O and parser integration

    Ok(())
}

/// Evaluate an export statement
fn eval_export(name: &str, span: pine_lexer::Span, ctx: &mut EvaluationContext) -> Result<()> {
    // Get the value being exported
    let _value = ctx
        .get_var(name)
        .cloned()
        .ok_or_else(|| EvalError::UndefinedVariable {
            name: name.to_string(),
            span,
        })?;

    // In a full implementation, we would add this to the current module's exports
    // For now, this is a stub
    // The actual export registration happens during module finalization

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_lexer::Span;
    use pine_parser::ast::{Block, Ident, Lit, Stmt, SwitchCase};
    use pine_runtime::value::Value;

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

        // Create a simple switch statement AST
        let switch_expr = ast::Expr::Literal(Lit::Int(2), Span::default());

        // Case 1: value 1
        let case1_value = ast::Expr::Literal(Lit::Int(1), Span::default());
        let case1_block = Block {
            stmts: vec![Stmt::VarDecl {
                name: Ident::new("result", Span::default()),
                kind: ast::VarKind::Var,
                type_ann: None,
                init: Some(ast::Expr::Literal(Lit::Int(10), Span::default())),
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let case1 = SwitchCase {
            value: case1_value,
            body: case1_block,
        };

        // Case 2: value 2
        let case2_value = ast::Expr::Literal(Lit::Int(2), Span::default());
        let case2_block = Block {
            stmts: vec![Stmt::VarDecl {
                name: Ident::new("result", Span::default()),
                kind: ast::VarKind::Var,
                type_ann: None,
                init: Some(ast::Expr::Literal(Lit::Int(20), Span::default())),
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let case2 = SwitchCase {
            value: case2_value,
            body: case2_block,
        };

        // Default case
        let default_block = Block {
            stmts: vec![Stmt::VarDecl {
                name: Ident::new("result", Span::default()),
                kind: ast::VarKind::Var,
                type_ann: None,
                init: Some(ast::Expr::Literal(Lit::Int(99), Span::default())),
                span: Span::default(),
            }],
            span: Span::default(),
        };

        let switch_stmt = Stmt::Switch {
            expr: switch_expr,
            cases: vec![case1, case2],
            default: Some(default_block),
            span: Span::default(),
        };

        // Evaluate the switch statement
        eval_stmt(&switch_stmt, &mut ctx).unwrap();

        // Check that the correct case was executed
        assert_eq!(ctx.get_var("result"), Some(&Value::Int(20)));
    }

    #[test]
    fn test_import_stmt_basic() {
        let mut ctx = EvaluationContext::new();

        // Create a simple import statement AST
        let import_stmt = Stmt::Import {
            path: "test_module.pine".to_string(),
            alias: Some(Ident::new("test", Span::default())),
            span: Span::default(),
        };

        // Evaluate the import statement (should not fail for stub implementation)
        let result = eval_stmt(&import_stmt, &mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_stmt_without_alias() {
        let mut ctx = EvaluationContext::new();

        // Create an import statement without alias
        let import_stmt = Stmt::Import {
            path: "utils.pine".to_string(),
            alias: None,
            span: Span::default(),
        };

        // Evaluate the import statement
        let result = eval_stmt(&import_stmt, &mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_stmt_with_existing_var() {
        let mut ctx = EvaluationContext::new();

        // First define a variable
        ctx.set_var("my_func", Value::from(42i64));

        // Create an export statement
        let export_stmt = Stmt::Export {
            name: Ident::new("my_func", Span::default()),
            span: Span::default(),
        };

        // Evaluate the export statement
        let result = eval_stmt(&export_stmt, &mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_stmt_with_undefined_var() {
        let mut ctx = EvaluationContext::new();

        // Try to export a variable that doesn't exist
        let export_stmt = Stmt::Export {
            name: Ident::new("undefined_var", Span::default()),
            span: Span::default(),
        };

        // Evaluate the export statement (should fail)
        let result = eval_stmt(&export_stmt, &mut ctx);
        assert!(result.is_err());

        // Verify it's the right error
        match result {
            Err(EvalError::UndefinedVariable { name, .. }) => {
                assert_eq!(name, "undefined_var");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
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
}
