//! AST to Bytecode compiler
//!
//! This module compiles Pine Script AST into VM bytecode.

use pine_parser::ast::{self, BinOp, Expr, Lit, Stmt, UnaryOp};
use pine_runtime::value::Value;

use crate::compiler::{BinaryOp as VmBinOp, Compiler, UnaryOp as VmUnaryOp};
use crate::opcode::OpCode;

/// Compile a Pine Script AST into VM bytecode
pub fn compile_script(script: &ast::Script) -> Result<Compiler, CompileError> {
    let mut compiler = Compiler::new();

    // Register built-in series names so they can be resolved during compilation
    compiler.register_series("close");
    compiler.register_series("open");
    compiler.register_series("high");
    compiler.register_series("low");
    compiler.register_series("volume");

    for stmt in &script.stmts {
        compile_stmt(&mut compiler, stmt)?;
    }

    compiler.compile_op(OpCode::Halt);
    Ok(compiler)
}

/// Compile a single statement
fn compile_stmt(compiler: &mut Compiler, stmt: &Stmt) -> Result<(), CompileError> {
    match stmt {
        Stmt::VarDecl { name, init, .. } => {
            let is_series_expr = init.as_ref().map(contains_series_reference).unwrap_or(false);
            if let Some(init_expr) = init {
                compile_expr(compiler, init_expr)?;
            } else {
                compiler.compile_const(Value::Na);
            }
            compiler.compile_var_decl(&name.name);
            // If the variable is assigned a series expression, update it in context
            // so that ta.* functions can access its history
            if is_series_expr {
                compiler.compile_dup();
                compiler.compile_update_user_series(&name.name);
            }
            Ok(())
        }
        Stmt::Assign { target, value, .. } => {
            let is_series_expr = contains_series_reference(value);
            compile_expr(compiler, value)?;
            match target {
                ast::AssignTarget::Var(ident) => {
                    if !compiler.compile_store_var(&ident.name) {
                        return Err(CompileError::UndefinedVariable(ident.name.clone()));
                    }
                    // If the variable is assigned a series expression, update it in context
                    if is_series_expr {
                        compiler.compile_load_var(&ident.name);
                        compiler.compile_update_user_series(&ident.name);
                    }
                    Ok(())
                }
                _ => Err(CompileError::Unsupported("Complex assignment target")),
            }
        }
        Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            ..
        } => {
            compile_if_stmt(compiler, cond, then_block, elifs, else_block)?;
            Ok(())
        }
        Stmt::For {
            var,
            from,
            to,
            by,
            body,
            ..
        } => {
            compile_for_loop(compiler, var, from, to, by.as_ref(), body)?;
            Ok(())
        }
        Stmt::While { cond, body, .. } => {
            compile_while_loop(compiler, cond, body)?;
            Ok(())
        }
        Stmt::Expr(expr) => {
            compile_expr(compiler, expr)?;
            // Pop the result of expression statement
            compiler.compile_pop();
            Ok(())
        }
        Stmt::Return { value, .. } => {
            if let Some(ret_expr) = value {
                compile_expr(compiler, ret_expr)?;
            } else {
                compiler.compile_const(Value::Na);
            }
            compiler.compile_op(OpCode::Return);
            Ok(())
        }
        Stmt::FnDef {
            name, params, body, ..
        } => compile_fn_def(compiler, name, params, body),
        _ => Err(CompileError::Unsupported(
            "Statement type not yet supported",
        )),
    }
}

/// Compile if statement
fn compile_if_stmt(
    compiler: &mut Compiler,
    cond: &Expr,
    then_block: &ast::Block,
    elifs: &[(Expr, ast::Block)],
    else_block: &Option<ast::Block>,
) -> Result<(), CompileError> {
    compiler.compile_if(
        |c| {
            if let Err(e) = compile_expr(c, cond) {
                eprintln!("DEBUG compile_if_stmt: error compiling cond: {:?}", e);
            }
        },
        |c| {
            for stmt in &then_block.stmts {
                if let Err(e) = compile_stmt(c, stmt) {
                    eprintln!("DEBUG compile_if_stmt: error compiling then stmt: {:?}", e);
                }
            }
        },
        if elifs.is_empty() && else_block.is_none() {
            None
        } else {
            Some(|c: &mut Compiler| {
                // Handle elifs and else
                if elifs.is_empty() {
                    if let Some(else_b) = else_block {
                        for stmt in &else_b.stmts {
                            compile_stmt(c, stmt).unwrap();
                        }
                    }
                } else {
                    // For simplicity, elifs are chained as nested ifs
                    let (elif_cond, elif_block) = &elifs[0];
                    let remaining_elifs = &elifs[1..];
                    compile_if_stmt(c, elif_cond, elif_block, remaining_elifs, else_block).unwrap();
                }
            })
        },
    );
    Ok(())
}

/// Compile for loop (for i = start to end [by step])
fn compile_for_loop(
    compiler: &mut Compiler,
    var: &ast::Ident,
    from: &Expr,
    to: &Expr,
    by: Option<&Expr>,
    body: &ast::Block,
) -> Result<(), CompileError> {
    // Declare loop variable
    compiler.enter_scope();
    let loop_var_slot = compiler.declare_var(&var.name);

    compiler.compile_for(
        loop_var_slot,
        |c| compile_expr(c, from).unwrap(),
        |c| compile_expr(c, to).unwrap(),
        by.map(|step| |c: &mut Compiler| compile_expr(c, step).unwrap()),
        |c| {
            for stmt in &body.stmts {
                if let Err(e) = compile_stmt(c, stmt) {
                    eprintln!("DEBUG compile_for_loop: error compiling stmt: {:?}", e);
                }
            }
        },
    );

    compiler.exit_scope();
    Ok(())
}

/// Compile while loop
fn compile_while_loop(
    compiler: &mut Compiler,
    cond: &Expr,
    body: &ast::Block,
) -> Result<(), CompileError> {
    compiler.compile_while(
        |c| {
            compile_expr(c, cond).unwrap();
        },
        |c| {
            for stmt in &body.stmts {
                compile_stmt(c, stmt).unwrap();
            }
        },
    );
    Ok(())
}

/// Compile an expression
fn compile_expr(compiler: &mut Compiler, expr: &Expr) -> Result<(), CompileError> {
    match expr {
        Expr::Literal(lit, _) => {
            let value = match lit {
                Lit::Int(i) => Value::Int(*i),
                Lit::Float(f) => Value::Float(*f),
                Lit::Bool(b) => Value::Bool(*b),
                Lit::Na => Value::Na,
                Lit::String(s) => Value::String(s.clone().into()),
                Lit::Color(c) => {
                    // u32 color is typically 0xAARRGGBB or 0xRRGGBB
                    // Extract components from u32
                    let r = ((*c >> 16) & 0xFF) as u8;
                    let g = ((*c >> 8) & 0xFF) as u8;
                    let b = (*c & 0xFF) as u8;
                    let a = ((*c >> 24) & 0xFF) as u8;
                    Value::Color(pine_runtime::value::Color::with_alpha(r, g, b, a))
                }
            };
            compiler.compile_const(value);
            Ok(())
        }
        Expr::Ident(ident) => {
            // First try to load as a local variable
            if compiler.compile_load_var(&ident.name) {
                return Ok(());
            }
            // Then try to load as a series (close, open, high, low, volume)
            // For built-in series, we push a SeriesRef so that series functions
            // can access the full series data from context
            if is_builtin_series(&ident.name) {
                compiler.compile_const(Value::SeriesRef(ident.name.clone()));
                return Ok(());
            }
            Err(CompileError::UndefinedVariable(ident.name.clone()))
        }
        Expr::BinOp { op, lhs, rhs, .. } => {
            compile_expr(compiler, lhs)?;
            compile_expr(compiler, rhs)?;
            let vm_op = match op {
                BinOp::Add => VmBinOp::Add,
                BinOp::Sub => VmBinOp::Sub,
                BinOp::Mul => VmBinOp::Mul,
                BinOp::Div => VmBinOp::Div,
                BinOp::Mod => VmBinOp::Mod,
                BinOp::Eq => VmBinOp::Eq,
                BinOp::Neq => VmBinOp::Ne,
                BinOp::Lt => VmBinOp::Lt,
                BinOp::Le => VmBinOp::Le,
                BinOp::Gt => VmBinOp::Gt,
                BinOp::Ge => VmBinOp::Ge,
                BinOp::And => VmBinOp::And,
                BinOp::Or => VmBinOp::Or,
                _ => return Err(CompileError::Unsupported("Binary operator")),
            };
            compiler.compile_binary(vm_op);
            Ok(())
        }
        Expr::UnaryOp { op, operand, .. } => {
            compile_expr(compiler, operand)?;
            let vm_op = match op {
                UnaryOp::Neg => VmUnaryOp::Neg,
                UnaryOp::Not => VmUnaryOp::Not,
            };
            compiler.compile_unary(vm_op);
            Ok(())
        }
        Expr::Index { base, offset, .. } => {
            // Series index access: close[i] or close[1]
            // base should be an identifier (series name)
            eprintln!("DEBUG Index: compiling index expression");
            if let Expr::Ident(ident) = base.as_ref() {
                eprintln!("DEBUG Index: base ident = {}", ident.name);
                if is_builtin_series(&ident.name) {
                    eprintln!("DEBUG Index: {} is builtin series", ident.name);
                    // For built-in series (close, open, high, low, volume),
                    // we need to use the runtime series access
                    // Push series name as string, then offset, then call runtime helper
                    compiler.compile_const(Value::String(ident.name.clone().into()));
                    compile_expr(compiler, offset)?;
                    // Call external function to access series at dynamic offset
                    let func_idx = compiler.register_external_function("__series_at");
                    compiler.compile_call(func_idx, 2);
                    Ok(())
                } else {
                    // For user-defined series, use compile-time series index
                    let series_idx = compiler
                        .lookup_series(&ident.name)
                        .ok_or_else(|| CompileError::UndefinedVariable(ident.name.clone()))?;
                    compile_expr(compiler, offset)?;
                    compiler.compile_push_series_dynamic(series_idx);
                    Ok(())
                }
            } else {
                Err(CompileError::Unsupported("Non-identifier series base"))
            }
        }
        Expr::Ternary {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            // condition ? then : else
            compile_ternary(compiler, cond, then_branch, else_branch)
        }
        Expr::NaCoalesce { lhs, rhs, .. } => {
            // lhs ?? rhs
            compile_expr(compiler, lhs)?;
            compile_expr(compiler, rhs)?;
            compiler.compile_op(OpCode::Coalesce);
            Ok(())
        }
        Expr::FnCall { func, args, .. } => {
            // Function call: compile arguments, then call
            compile_fn_call(compiler, func, args)
        }
        Expr::FieldAccess { base, field, .. } => {
            // Field access: base.field (e.g., ta.sma, color.blue)
            if let Expr::Ident(base_ident) = base.as_ref() {
                match base_ident.name.as_str() {
                    "ta" | "math" | "color" | "strategy" | "plot" | "input" | "display" => {
                        // Namespace access: create namespace value and access field
                        // For now, treat as external function reference "namespace.field"
                        let full_name = format!("{}.{}", base_ident.name, field.name);
                        // Check for namespace constants
                        match base_ident.name.as_str() {
                            "color" => {
                                let color_value = match field.name.as_str() {
                                    "blue" => Some(Value::Color(pine_runtime::value::Color::new(0, 120, 255))),
                                    "red" => Some(Value::Color(pine_runtime::value::Color::new(255, 0, 0))),
                                    "green" => Some(Value::Color(pine_runtime::value::Color::new(0, 128, 0))),
                                    "yellow" => Some(Value::Color(pine_runtime::value::Color::new(255, 255, 0))),
                                    "white" => Some(Value::Color(pine_runtime::value::Color::new(255, 255, 255))),
                                    "black" => Some(Value::Color(pine_runtime::value::Color::new(0, 0, 0))),
                                    "gray" => Some(Value::Color(pine_runtime::value::Color::new(128, 128, 128))),
                                    "orange" => Some(Value::Color(pine_runtime::value::Color::new(255, 165, 0))),
                                    _ => None,
                                };
                                if let Some(v) = color_value {
                                    compiler.compile_const(v);
                                    return Ok(());
                                }
                            }
                            "display" => {
                                // display.none, display.all, etc - for now just push 0 or 1
                                let display_value = match field.name.as_str() {
                                    "none" => Some(Value::Int(0)),
                                    "all" => Some(Value::Int(1)),
                                    _ => None,
                                };
                                if let Some(v) = display_value {
                                    compiler.compile_const(v);
                                    return Ok(());
                                }
                            }
                            "plot" => {
                                // plot.style_line, etc - for now just push string
                                if field.name.starts_with("style_") {
                                    compiler.compile_const(Value::String(field.name.clone().into()));
                                    return Ok(());
                                }
                            }
                            _ => {}
                        }
                        // For other namespaces (ta, math), push namespace value
                        compiler.compile_const(Value::Namespace(full_name));
                        Ok(())
                    }
                    _ => {
                        // Regular field access: compile base, then access field
                        compile_expr(compiler, base)?;
                        let field_name = &field.name;
                        // For now, treat as external function call
                        let func_idx = compiler.register_external_function(&format!("__field_{}", field_name));
                        compiler.compile_call(func_idx, 1);
                        Ok(())
                    }
                }
            } else {
                Err(CompileError::Unsupported("Complex field access base"))
            }
        }
        Expr::MethodCall { base, method, args, .. } => {
            // Method call: base.method(args)
            // For namespace.method(args), compile as external function "namespace.method"
            if let Expr::Ident(base_ident) = base.as_ref() {
                if matches!(base_ident.name.as_str(), "ta" | "math" | "color" | "strategy" | "plot" | "input") {
                    let full_name = format!("{}.{}", base_ident.name, method.name);

                    // Handle special TA functions that need implicit high/low/close
                    let needs_ohlc = matches!(method.name.as_str(), "tr" | "atr" | "bb");

                    // Inject implicit arguments for OHLC-dependent functions FIRST
                    // (they become the first 3 arguments)
                    if base_ident.name == "ta" && needs_ohlc {
                        // Inject high, low, close as SeriesRef
                        compiler.compile_const(Value::SeriesRef("high".to_string()));
                        compiler.compile_const(Value::SeriesRef("low".to_string()));
                        compiler.compile_const(Value::SeriesRef("close".to_string()));
                    }

                    // Special handling for ta.cci: first argument should be series data
                    if method.name == "cci" && !args.is_empty() {
                        // First argument to ta.cci should be loaded as series data
                        if let Expr::Ident(var_ident) = &args[0].value {
                            // Variable reference - load series data from context
                            compiler.compile_const(Value::String(var_ident.name.clone().into()));
                            let func_idx = compiler.register_external_function("__load_series_data");
                            compiler.compile_call(func_idx, 1);
                        } else {
                            // Complex expression - compile normally (will get current value)
                            compile_expr(compiler, &args[0].value)?;
                        }
                        // Compile remaining arguments
                        for arg in args.iter().skip(1) {
                            compile_expr(compiler, &arg.value)?;
                        }
                    } else {
                        // Compile user-provided arguments normally
                        for arg in args {
                            compile_expr(compiler, &arg.value)?;
                        }
                    }

                    // Call as external function
                    let func_idx = compiler.register_external_function(&full_name);
                    let total_args = if method.name == "cci" && !args.is_empty() { args.len() } else { args.len() + if needs_ohlc { 3 } else { 0 } };
                    compiler.compile_call(func_idx, total_args);
                    return Ok(());
                }
            }
            // Regular method call: compile base, then arguments, then call method
            compile_expr(compiler, base)?;
            for arg in args {
                compile_expr(compiler, &arg.value)?;
            }
            let method_name = &method.name;
            let func_idx = compiler.register_external_function(method_name);
            compiler.compile_call(func_idx, args.len() + 1); // +1 for self
            Ok(())
        }
        _ => Err(CompileError::Unsupported("Expression type")),
    }
}

/// Compile function call
fn compile_fn_call(
    compiler: &mut Compiler,
    func: &Expr,
    args: &[ast::Arg],
) -> Result<(), CompileError> {
    // For now, only support direct function name calls (not higher-order functions)
    if let Expr::Ident(ident) = func {
        eprintln!("DEBUG compile_fn_call: func_name={}, args_count={}", ident.name, args.len());
        // First, compile all arguments onto the stack
        for (i, arg) in args.iter().enumerate() {
            eprintln!("DEBUG compile_fn_call: compiling arg {} for {}", i, ident.name);
            compile_expr(compiler, &arg.value)?;
        }

        let func_name = &ident.name;

        // Check if it's a user-defined function
        if let Some(func_idx) = compiler.lookup_function(func_name) {
            compiler.compile_call(func_idx, args.len());
            Ok(())
        } else {
            // Treat as external function (e.g., ta.sma, math.abs)
            let func_idx = compiler.register_external_function(func_name);
            compiler.compile_call(func_idx, args.len());
            Ok(())
        }
    } else {
        Err(CompileError::Unsupported("Non-identifier function call"))
    }
}

/// Compile function definition
fn compile_fn_def(
    compiler: &mut Compiler,
    name: &ast::Ident,
    params: &[ast::Param],
    body: &ast::FnBody,
) -> Result<(), CompileError> {
    // Reserve a function slot first (for forward references)
    let func_idx = compiler.reserve_function_slot();

    // Register the function name to index mapping
    compiler.register_function_name(&name.name, func_idx);

    // Remember the current position - we'll compile the body here
    let body_start = compiler.chunk().current_pos();

    // Patch the function address
    compiler.patch_function_address(func_idx, body_start);

    // Enter a new scope for the function body
    compiler.enter_scope();

    // Declare parameters as local variables
    for param in params {
        compiler.declare_var(&param.name.name);
    }

    // Compile the function body
    match body {
        ast::FnBody::Expr(expr) => {
            // Arrow function body: => expr
            compile_expr(compiler, expr)?;
            compiler.compile_op(OpCode::Return);
        }
        ast::FnBody::Block(block) => {
            // Block body: =>
            //   ... statements ...
            //   return expr (optional)
            for stmt in &block.stmts {
                compile_stmt(compiler, stmt)?;
            }
            // If last statement wasn't a return, add one
            compiler.compile_op(OpCode::Return);
        }
    }

    // Exit the function scope
    compiler.exit_scope();

    Ok(())
}

/// Compile ternary expression (cond ? then : else)
fn compile_ternary(
    compiler: &mut Compiler,
    cond: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
) -> Result<(), CompileError> {
    compiler.compile_if(
        |c| {
            compile_expr(c, cond).unwrap();
        },
        |c| {
            compile_expr(c, then_branch).unwrap();
        },
        Some(|c: &mut Compiler| {
            compile_expr(c, else_branch).unwrap();
        }),
    );
    Ok(())
}

/// Compilation errors
#[derive(Debug, Clone)]
pub enum CompileError {
    /// Variable not found
    UndefinedVariable(String),
    /// Function not found
    UndefinedFunction(String),
    /// Unsupported feature
    Unsupported(&'static str),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: {}", name)
            }
            CompileError::UndefinedFunction(name) => {
                write!(f, "Undefined function: {}", name)
            }
            CompileError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

/// Check if a name is a built-in series
fn is_builtin_series(name: &str) -> bool {
    matches!(name, "close" | "open" | "high" | "low" | "volume" | "time" | "hl2" | "hlc3" | "ohlc4")
}

/// Check if an expression contains series references
fn contains_series_reference(expr: &Expr) -> bool {
    match expr {
        Expr::Ident(ident) => is_builtin_series(&ident.name),
        Expr::BinOp { lhs, rhs, .. } => {
            contains_series_reference(lhs) || contains_series_reference(rhs)
        }
        Expr::UnaryOp { operand, .. } => contains_series_reference(operand),
        Expr::Index { base, .. } => {
            if let Expr::Ident(ident) = base.as_ref() {
                is_builtin_series(&ident.name)
            } else {
                contains_series_reference(base)
            }
        }
        Expr::FieldAccess { base, .. } => contains_series_reference(base),
        Expr::MethodCall { base, args, .. } => {
            contains_series_reference(base)
                || args.iter().any(|arg| contains_series_reference(&arg.value))
        }
        Expr::FnCall { args, .. } => {
            args.iter().any(|arg| contains_series_reference(&arg.value))
        }
        Expr::Ternary { cond, then_branch, else_branch, .. } => {
            contains_series_reference(cond)
                || contains_series_reference(then_branch)
                || contains_series_reference(else_branch)
        }
        Expr::NaCoalesce { lhs, rhs, .. } => {
            contains_series_reference(lhs) || contains_series_reference(rhs)
        }
        _ => false,
    }
}
