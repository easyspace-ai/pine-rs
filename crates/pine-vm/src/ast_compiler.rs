//! AST to Bytecode compiler
//!
//! This module compiles Pine Script AST into VM bytecode.

use pine_parser::ast::{self, AssignOp, BinOp, Expr, Lit, Stmt, UnaryOp, VarKind};
use pine_runtime::value::Value;

use crate::compiler::{BinaryOp as VmBinOp, Compiler, UnaryOp as VmUnaryOp};
use crate::debug::vm_debug;
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
    compiler.register_series("time");

    for stmt in &script.stmts {
        compile_stmt(&mut compiler, stmt)?;
    }

    compiler.compile_op(OpCode::Halt);
    Ok(compiler)
}

/// Compile a single statement
fn compile_stmt(compiler: &mut Compiler, stmt: &Stmt) -> Result<(), CompileError> {
    match stmt {
        Stmt::VarDecl {
            name, kind, init, ..
        } => {
            let is_persistent = matches!(kind, VarKind::Var | VarKind::Varip);
            let is_series_expr = init
                .as_ref()
                .map(|expr| contains_series_reference(compiler, expr))
                .unwrap_or(false);
            let should_track_as_series = is_persistent || is_series_expr;
            if should_track_as_series {
                compiler.mark_series_var(&name.name);
            }

            let value_slot = compiler.declare_var(&name.name);

            if is_persistent {
                let init_slot = compiler.declare_var(format!(
                    "__var_init_{}_{}",
                    name.name,
                    compiler.chunk().current_pos()
                ));
                compiler.compile_load_slot(init_slot);
                let skip_init = compiler.compile_jump(crate::compiler::JumpOp::IfTrue);

                if let Some(init_expr) = init {
                    compile_expr(compiler, init_expr)?;
                } else {
                    compiler.compile_const(Value::Na);
                }
                if should_track_as_series {
                    compiler.compile_dup();
                }
                compiler.compile_store_slot(value_slot);
                if should_track_as_series {
                    compiler.compile_update_user_series(&name.name);
                }
                compiler.compile_const(Value::Bool(true));
                compiler.compile_store_slot(init_slot);
                compiler.patch_jump(skip_init);
            } else {
                if let Some(init_expr) = init {
                    compile_expr(compiler, init_expr)?;
                    if should_track_as_series {
                        compiler.compile_dup();
                    }
                } else {
                    compiler.compile_const(Value::Na);
                }
                compiler.compile_store_slot(value_slot);
                if should_track_as_series {
                    compiler.compile_update_user_series(&name.name);
                }
            }
            Ok(())
        }
        Stmt::Assign {
            target, op, value, ..
        } => {
            // For compound assignment (+=, -=, etc.), first load current value,
            // compile the RHS, apply the operation, then store.
            let compound_op = match op {
                AssignOp::PlusEq => Some(VmBinOp::Add),
                AssignOp::MinusEq => Some(VmBinOp::Sub),
                AssignOp::StarEq => Some(VmBinOp::Mul),
                AssignOp::SlashEq => Some(VmBinOp::Div),
                AssignOp::PercentEq => Some(VmBinOp::Mod),
                AssignOp::Assign | AssignOp::ColonEq => None,
            };

            match target {
                ast::AssignTarget::Var(ident) => {
                    if let Some(bin_op) = compound_op {
                        // Load current value, compile RHS, apply op
                        if !compiler.compile_load_var(&ident.name) {
                            return Err(CompileError::UndefinedVariable(ident.name.clone()));
                        }
                        compile_expr(compiler, value)?;
                        compiler.compile_binary(bin_op);
                    } else {
                        compile_expr(compiler, value)?;
                    }
                    let is_series_expr = contains_series_reference(compiler, value);
                    let should_update_series =
                        is_series_expr || compiler.is_series_var(&ident.name);
                    if should_update_series {
                        compiler.mark_series_var(&ident.name);
                        compiler.compile_dup();
                    }
                    if !compiler.compile_store_var(&ident.name) {
                        return Err(CompileError::UndefinedVariable(ident.name.clone()));
                    }
                    if should_update_series {
                        compiler.compile_update_user_series(&ident.name);
                    }
                    Ok(())
                }
                ast::AssignTarget::Tuple(idents) => {
                    compile_expr(compiler, value)?;
                    let tuple_slot = compiler
                        .declare_var(format!("__tuple_tmp_{}", compiler.chunk().current_pos()));
                    compiler.compile_store_slot(tuple_slot);

                    for (idx, ident) in idents.iter().enumerate() {
                        compiler.compile_load_slot(tuple_slot);
                        compiler.compile_const(Value::Int(idx as i64));
                        let func_idx = compiler.register_external_function("__tuple_get");
                        compiler.compile_call(func_idx, 2);

                        if compiler.lookup_var(&ident.name).is_some() {
                            compiler.compile_store_var(&ident.name);
                        } else {
                            compiler.compile_var_decl(&ident.name);
                        }

                        compiler.mark_series_var(&ident.name);
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
        Stmt::Switch {
            scrutinee,
            arms,
            span: _,
        } => {
            compile_switch_stmt(compiler, scrutinee.as_ref(), arms)?;
            Ok(())
        }
        Stmt::ForIn {
            pattern,
            iterable,
            body,
            ..
        } => {
            compile_for_in_loop(compiler, pattern, iterable, body)?;
            Ok(())
        }
        Stmt::Break { .. } => {
            compiler.compile_break();
            Ok(())
        }
        Stmt::Continue { .. } => {
            compiler.compile_continue();
            Ok(())
        }
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
                vm_debug!("DEBUG compile_if_stmt: error compiling cond: {:?}", e);
            }
        },
        |c| {
            for stmt in &then_block.stmts {
                if let Err(e) = compile_stmt(c, stmt) {
                    vm_debug!("DEBUG compile_if_stmt: error compiling then stmt: {:?}", e);
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
                    vm_debug!("DEBUG compile_for_loop: error compiling stmt: {:?}", e);
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
            vm_debug!("DEBUG Index: compiling index expression");
            if let Expr::Ident(ident) = base.as_ref() {
                vm_debug!("DEBUG Index: base ident = {}", ident.name);
                if is_builtin_series(&ident.name) || compiler.is_series_var(&ident.name) {
                    let series_idx = compiler
                        .lookup_series(&ident.name)
                        .ok_or_else(|| CompileError::UndefinedVariable(ident.name.clone()))?;
                    compile_expr(compiler, offset)?;
                    compiler.compile_push_series_dynamic(series_idx);
                    Ok(())
                } else {
                    Err(CompileError::UndefinedVariable(ident.name.clone()))
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
                                    "blue" => Some(Value::Color(pine_runtime::value::Color::new(
                                        0, 120, 255,
                                    ))),
                                    "red" => Some(Value::Color(pine_runtime::value::Color::new(
                                        255, 0, 0,
                                    ))),
                                    "green" => Some(Value::Color(pine_runtime::value::Color::new(
                                        0, 128, 0,
                                    ))),
                                    "yellow" => Some(Value::Color(
                                        pine_runtime::value::Color::new(255, 255, 0),
                                    )),
                                    "white" => Some(Value::Color(pine_runtime::value::Color::new(
                                        255, 255, 255,
                                    ))),
                                    "black" => {
                                        Some(Value::Color(pine_runtime::value::Color::new(0, 0, 0)))
                                    }
                                    "gray" => Some(Value::Color(pine_runtime::value::Color::new(
                                        128, 128, 128,
                                    ))),
                                    "orange" => Some(Value::Color(
                                        pine_runtime::value::Color::new(255, 165, 0),
                                    )),
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
                                    compiler
                                        .compile_const(Value::String(field.name.clone().into()));
                                    return Ok(());
                                }
                            }
                            _ => {}
                        }
                        if base_ident.name == "ta" && matches!(field.name.as_str(), "obv" | "pvt") {
                            compiler.compile_const(Value::SeriesRef("close".to_string()));
                            compiler.compile_const(Value::SeriesRef("volume".to_string()));
                            let func_idx = compiler.register_external_function(&full_name);
                            compiler.compile_call(func_idx, 2);
                            return Ok(());
                        }

                        // Preserve namespace values for unsupported bare references.
                        compiler.compile_const(Value::Namespace(full_name));
                        Ok(())
                    }
                    _ => {
                        // Regular field access: compile base, then access field
                        compile_expr(compiler, base)?;
                        let field_name = &field.name;
                        // For now, treat as external function call
                        let func_idx =
                            compiler.register_external_function(format!("__field_{}", field_name));
                        compiler.compile_call(func_idx, 1);
                        Ok(())
                    }
                }
            } else {
                Err(CompileError::Unsupported("Complex field access base"))
            }
        }
        Expr::MethodCall {
            base, method, args, ..
        } => {
            // Method call: base.method(args)
            // For namespace.method(args), compile as external function "namespace.method"
            if let Expr::Ident(base_ident) = base.as_ref() {
                if matches!(
                    base_ident.name.as_str(),
                    "ta" | "math" | "color" | "strategy" | "plot" | "input"
                ) {
                    let full_name = format!("{}.{}", base_ident.name, method.name);

                    let injected_args = compile_ta_method_call_args(compiler, &method.name, args)?;

                    // Call as external function
                    let func_idx = compiler.register_external_function(&full_name);
                    let total_args = args.len() + injected_args;
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
        Expr::SwitchExpr {
            scrutinee,
            arms,
            span: _,
        } => compile_switch_expr(compiler, scrutinee.as_deref(), arms),
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
        let func_name = &ident.name;
        vm_debug!(
            "DEBUG compile_fn_call: func_name={}, args_count={}",
            func_name,
            args.len()
        );
        // First, compile all arguments onto the stack
        for (i, arg) in args.iter().enumerate() {
            vm_debug!(
                "DEBUG compile_fn_call: compiling arg {} for {}",
                i,
                func_name
            );
            if ta_method_needs_series_arg(func_name, i) {
                compile_series_function_arg(compiler, &arg.value)?;
            } else {
                compile_expr(compiler, &arg.value)?;
            }
        }

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

/// Compile a switch statement (no value left on stack)
fn compile_switch_stmt(
    compiler: &mut Compiler,
    scrutinee: Option<&Expr>,
    arms: &[ast::SwitchArm],
) -> Result<(), CompileError> {
    // If there's a scrutinee, evaluate it once and store in a temp slot.
    let scrutinee_slot = if let Some(scr) = scrutinee {
        compile_expr(compiler, scr)?;
        let slot = compiler.declare_var(format!("__switch_scr_{}", compiler.chunk().current_pos()));
        compiler.compile_store_slot(slot);
        Some(slot)
    } else {
        None
    };

    // Collect end-jump positions so we can patch them all to the end.
    let mut end_jumps: Vec<usize> = Vec::new();

    // Separate default arm from patterned arms.
    let (patterned, default_arm): (Vec<_>, Vec<_>) =
        arms.iter().partition(|arm| arm.pattern.is_some());

    for arm in &patterned {
        // Compile condition
        if let Some(scr_slot) = scrutinee_slot {
            // scrutinee == pattern
            compiler.compile_load_slot(scr_slot);
            compile_expr(compiler, arm.pattern.as_ref().unwrap())?;
            compiler.compile_binary(VmBinOp::Eq);
        } else {
            // No scrutinee: the pattern itself is the boolean condition
            compile_expr(compiler, arm.pattern.as_ref().unwrap())?;
        }
        let skip_jump = compiler.compile_jump(crate::compiler::JumpOp::IfFalse);

        // Compile arm body
        compile_switch_arm_body_stmt(compiler, &arm.body)?;

        // Jump to end of switch
        end_jumps.push(compiler.compile_jump(crate::compiler::JumpOp::Unconditional));

        // Patch the skip_jump to here (next arm)
        compiler.patch_jump(skip_jump);
    }

    // Default arm (always taken if reached)
    if let Some(arm) = default_arm.first() {
        compile_switch_arm_body_stmt(compiler, &arm.body)?;
    }

    // Patch all end jumps
    for j in end_jumps {
        compiler.patch_jump(j);
    }

    Ok(())
}

/// Compile a switch arm body as a statement (pop value if Expr body)
fn compile_switch_arm_body_stmt(
    compiler: &mut Compiler,
    body: &ast::SwitchArmBody,
) -> Result<(), CompileError> {
    match body {
        ast::SwitchArmBody::Expr(expr) => {
            compile_expr(compiler, expr)?;
            compiler.compile_pop();
        }
        ast::SwitchArmBody::Block(block) => {
            for stmt in &block.stmts {
                compile_stmt(compiler, stmt)?;
            }
        }
    }
    Ok(())
}

/// Compile a switch expression (leaves a value on the stack)
fn compile_switch_expr(
    compiler: &mut Compiler,
    scrutinee: Option<&Expr>,
    arms: &[ast::SwitchArm],
) -> Result<(), CompileError> {
    let scrutinee_slot = if let Some(scr) = scrutinee {
        compile_expr(compiler, scr)?;
        let slot = compiler.declare_var(format!("__switch_scr_{}", compiler.chunk().current_pos()));
        compiler.compile_store_slot(slot);
        Some(slot)
    } else {
        None
    };

    let mut end_jumps: Vec<usize> = Vec::new();
    let (patterned, default_arm): (Vec<_>, Vec<_>) =
        arms.iter().partition(|arm| arm.pattern.is_some());

    for arm in &patterned {
        if let Some(scr_slot) = scrutinee_slot {
            compiler.compile_load_slot(scr_slot);
            compile_expr(compiler, arm.pattern.as_ref().unwrap())?;
            compiler.compile_binary(VmBinOp::Eq);
        } else {
            compile_expr(compiler, arm.pattern.as_ref().unwrap())?;
        }
        let skip_jump = compiler.compile_jump(crate::compiler::JumpOp::IfFalse);

        // Compile arm body – must leave a value on the stack
        compile_switch_arm_body_expr(compiler, &arm.body)?;

        end_jumps.push(compiler.compile_jump(crate::compiler::JumpOp::Unconditional));
        compiler.patch_jump(skip_jump);
    }

    // Default arm
    if let Some(arm) = default_arm.first() {
        compile_switch_arm_body_expr(compiler, &arm.body)?;
    } else {
        // No default – push na
        compiler.compile_const(Value::Na);
    }

    for j in end_jumps {
        compiler.patch_jump(j);
    }

    Ok(())
}

/// Compile a switch arm body as an expression (leaves value on stack)
fn compile_switch_arm_body_expr(
    compiler: &mut Compiler,
    body: &ast::SwitchArmBody,
) -> Result<(), CompileError> {
    match body {
        ast::SwitchArmBody::Expr(expr) => {
            compile_expr(compiler, expr)?;
        }
        ast::SwitchArmBody::Block(block) => {
            // Compile all statements; last expression result stays on stack.
            // For blocks used as expressions, the last statement should be an Expr.
            for (i, stmt) in block.stmts.iter().enumerate() {
                if i == block.stmts.len() - 1 {
                    // If last stmt is an Expr, compile without popping
                    if let Stmt::Expr(expr) = stmt {
                        compile_expr(compiler, expr)?;
                    } else {
                        compile_stmt(compiler, stmt)?;
                        compiler.compile_const(Value::Na);
                    }
                } else {
                    compile_stmt(compiler, stmt)?;
                }
            }
            if block.stmts.is_empty() {
                compiler.compile_const(Value::Na);
            }
        }
    }
    Ok(())
}

/// Compile a for-in loop: for x in iterable / for [i, v] in iterable
fn compile_for_in_loop(
    compiler: &mut Compiler,
    pattern: &ast::ForInPattern,
    iterable: &Expr,
    body: &ast::Block,
) -> Result<(), CompileError> {
    compiler.enter_scope();

    // Compile iterable and store in temp
    compile_expr(compiler, iterable)?;
    let arr_slot = compiler.declare_var(format!("__forin_arr_{}", compiler.chunk().current_pos()));
    compiler.compile_store_slot(arr_slot);

    // Get array size and store
    compiler.compile_load_slot(arr_slot);
    let size_fn = compiler.register_external_function("__array_size");
    compiler.compile_call(size_fn, 1);
    let len_slot = compiler.declare_var(format!("__forin_len_{}", compiler.chunk().current_pos()));
    compiler.compile_store_slot(len_slot);

    // Index variable = 0
    let idx_slot = compiler.declare_var(format!("__forin_idx_{}", compiler.chunk().current_pos()));
    compiler.compile_const(Value::Int(0));
    compiler.compile_store_slot(idx_slot);

    // Declare loop binding variables
    match pattern {
        ast::ForInPattern::Single(ident) => {
            compiler.declare_var(&ident.name);
        }
        ast::ForInPattern::Tuple(idx_ident, val_ident) => {
            compiler.declare_var(&idx_ident.name);
            compiler.declare_var(&val_ident.name);
        }
    }

    // Loop start
    let start_pos = compiler.chunk().current_pos();
    compiler.push_loop(start_pos);

    // Condition: idx < len
    compiler.compile_load_slot(idx_slot);
    compiler.compile_load_slot(len_slot);
    compiler.compile_binary(VmBinOp::Lt);
    let end_jump = compiler.compile_jump(crate::compiler::JumpOp::IfFalse);

    // Get element: __array_get(arr, idx)
    compiler.compile_load_slot(arr_slot);
    compiler.compile_load_slot(idx_slot);
    let get_fn = compiler.register_external_function("__array_get");
    compiler.compile_call(get_fn, 2);

    // Store into pattern variable(s)
    match pattern {
        ast::ForInPattern::Single(ident) => {
            compiler.compile_store_var(&ident.name);
        }
        ast::ForInPattern::Tuple(idx_ident, val_ident) => {
            // Store value
            compiler.compile_store_var(&val_ident.name);
            // Store index
            compiler.compile_load_slot(idx_slot);
            compiler.compile_store_var(&idx_ident.name);
        }
    }

    // Compile body
    for stmt in &body.stmts {
        compile_stmt(compiler, stmt)?;
    }

    // Increment index
    compiler.compile_load_slot(idx_slot);
    compiler.compile_const(Value::Int(1));
    compiler.compile_binary(VmBinOp::Add);
    compiler.compile_store_slot(idx_slot);

    // Jump back to start
    let loop_jump = compiler.compile_jump(crate::compiler::JumpOp::Unconditional);
    compiler.patch_jump_to(loop_jump, start_pos);

    // Patch end jump
    compiler.patch_jump(end_jump);

    compiler.pop_loop();
    compiler.exit_scope();
    Ok(())
}

/// Compile function definition
fn compile_fn_def(
    compiler: &mut Compiler,
    name: &ast::Ident,
    params: &[ast::Param],
    body: &ast::FnBody,
) -> Result<(), CompileError> {
    // Skip over function bodies during top-level execution.
    let skip_body_jump = compiler.compile_jump(crate::compiler::JumpOp::Unconditional);

    // Reserve a function slot first (for forward references)
    let func_idx = compiler.reserve_function_slot();

    // Register the function name to index mapping
    compiler.register_function_name(&name.name, func_idx);

    // Remember the current position - we'll compile the body here
    let body_start = compiler.chunk().current_pos();

    // Patch the function address
    compiler.patch_function_address(func_idx, body_start);

    // Enter a function-local scope for the body
    compiler.enter_function_scope();

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
            for (idx, stmt) in block.stmts.iter().enumerate() {
                let is_last = idx + 1 == block.stmts.len();
                if is_last {
                    if let Stmt::Expr(expr) = stmt {
                        compile_expr(compiler, expr)?;
                    } else {
                        compile_stmt(compiler, stmt)?;
                        compiler.compile_const(Value::Na);
                    }
                } else {
                    compile_stmt(compiler, stmt)?;
                }
            }
            compiler.compile_op(OpCode::Return);
        }
    }

    // Exit the function scope
    compiler.exit_scope();

    // Resume top-level execution after the function body.
    compiler.patch_jump(skip_body_jump);

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
    matches!(
        name,
        "close" | "open" | "high" | "low" | "volume" | "time" | "hl2" | "hlc3" | "ohlc4"
    )
}

fn compile_series_function_arg(compiler: &mut Compiler, expr: &Expr) -> Result<(), CompileError> {
    if let Expr::Ident(ident) = expr {
        if is_builtin_series(&ident.name) || compiler.is_series_var(&ident.name) {
            compiler.compile_const(Value::SeriesRef(ident.name.clone()));
            return Ok(());
        }
    }

    if contains_series_reference(compiler, expr) {
        let series_name = compiler.next_synthetic_series_name();
        compile_expr(compiler, expr)?;
        compiler.compile_dup();
        compiler.compile_update_user_series(&series_name);
        compiler.compile_pop();
        compiler.compile_const(Value::SeriesRef(series_name));
        return Ok(());
    }

    compile_expr(compiler, expr)
}

fn ta_method_needs_series_arg(func_name: &str, arg_index: usize) -> bool {
    match func_name {
        "ta.sma" | "ta.ema" | "ta.rsi" | "ta.mom" | "ta.cci" | "ta.bb" | "ta.highest"
        | "ta.lowest" | "ta.highestbars" | "ta.lowestbars" => arg_index == 0,
        "ta.macd" => arg_index == 0,
        "ta.stoch" => arg_index <= 2,
        "ta.vwma" | "ta.mfi" => arg_index <= 1,
        "ta.crossover" | "ta.crossunder" => arg_index <= 1,
        "ta.barssince" => arg_index == 0,
        _ => false,
    }
}

fn compile_ta_method_call_args(
    compiler: &mut Compiler,
    method_name: &str,
    args: &[ast::Arg],
) -> Result<usize, CompileError> {
    match method_name {
        "tr" | "atr" | "dmi" | "supertrend" => {
            compiler.compile_const(Value::SeriesRef("high".to_string()));
            compiler.compile_const(Value::SeriesRef("low".to_string()));
            compiler.compile_const(Value::SeriesRef("close".to_string()));
            for arg in args {
                compile_expr(compiler, &arg.value)?;
            }
            Ok(3)
        }
        "vwma" | "mfi" => {
            if let Some(first) = args.first() {
                compile_series_function_arg(compiler, &first.value)?;
            }
            compiler.compile_const(Value::SeriesRef("volume".to_string()));
            for arg in args.iter().skip(1) {
                compile_expr(compiler, &arg.value)?;
            }
            Ok(1)
        }
        _ => {
            let full_name = format!("ta.{method_name}");
            for (idx, arg) in args.iter().enumerate() {
                if ta_method_needs_series_arg(&full_name, idx) {
                    compile_series_function_arg(compiler, &arg.value)?;
                } else {
                    compile_expr(compiler, &arg.value)?;
                }
            }
            Ok(0)
        }
    }
}

/// Check if an expression contains series references
fn contains_series_reference(compiler: &Compiler, expr: &Expr) -> bool {
    match expr {
        Expr::Ident(ident) => is_builtin_series(&ident.name) || compiler.is_series_var(&ident.name),
        Expr::BinOp { lhs, rhs, .. } => {
            contains_series_reference(compiler, lhs) || contains_series_reference(compiler, rhs)
        }
        Expr::UnaryOp { operand, .. } => contains_series_reference(compiler, operand),
        Expr::Index { base, .. } => {
            if let Expr::Ident(ident) = base.as_ref() {
                is_builtin_series(&ident.name) || compiler.is_series_var(&ident.name)
            } else {
                contains_series_reference(compiler, base)
            }
        }
        Expr::FieldAccess { base, .. } => contains_series_reference(compiler, base),
        Expr::MethodCall {
            base, method, args, ..
        } => {
            let returns_series = matches!(base.as_ref(), Expr::Ident(base_ident) if base_ident.name == "ta")
                || matches!(base.as_ref(), Expr::Ident(base_ident) if base_ident.name == "math" && matches!(method.name.as_str(), "max" | "min"));
            returns_series
                || contains_series_reference(compiler, base)
                || args
                    .iter()
                    .any(|arg| contains_series_reference(compiler, &arg.value))
        }
        Expr::FnCall { func, args, .. } => {
            let returns_series =
                matches!(func.as_ref(), Expr::Ident(ident) if ident.name.starts_with("ta."));
            returns_series
                || args
                    .iter()
                    .any(|arg| contains_series_reference(compiler, &arg.value))
        }
        Expr::Ternary {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            contains_series_reference(compiler, cond)
                || contains_series_reference(compiler, then_branch)
                || contains_series_reference(compiler, else_branch)
        }
        Expr::NaCoalesce { lhs, rhs, .. } => {
            contains_series_reference(compiler, lhs) || contains_series_reference(compiler, rhs)
        }
        _ => false,
    }
}
