//! VM vs Eval parity tests
//!
//! These tests verify that the VM produces the same results as pine-eval
//! for the same computations.

use pine_eval::eval_expr::eval_expr;
use pine_eval::EvaluationContext;
use pine_parser::ast::{BinOp, Expr, Lit, UnaryOp};
use pine_runtime::value::Value;
use pine_vm::compiler::{BinaryOp as VmBinOp, Compiler, UnaryOp as VmUnaryOp};
use pine_vm::vm::execute_chunk;

/// Compile a simple expression to VM bytecode
fn compile_expr_to_vm(expr: &Expr) -> Compiler {
    let mut compiler = Compiler::new();
    compile_expr(&mut compiler, expr);
    compiler.compile_op(pine_vm::opcode::OpCode::Return);
    compiler
}

/// Compile an expression recursively
fn compile_expr(compiler: &mut Compiler, expr: &Expr) {
    match expr {
        Expr::Literal(lit, _) => {
            let value = match lit {
                Lit::Int(i) => Value::Int(*i),
                Lit::Float(f) => Value::Float(*f),
                Lit::Bool(b) => Value::Bool(*b),
                Lit::Na => Value::Na,
                _ => Value::Na, // Other literals not yet supported
            };
            compiler.compile_const(value);
        }
        Expr::BinOp { op, lhs, rhs, .. } => {
            compile_expr(compiler, lhs);
            compile_expr(compiler, rhs);
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
                _ => panic!("Binary operator {:?} not yet supported", op),
            };
            compiler.compile_binary(vm_op);
        }
        Expr::UnaryOp { op, operand, .. } => {
            compile_expr(compiler, operand);
            let vm_op = match op {
                UnaryOp::Neg => VmUnaryOp::Neg,
                UnaryOp::Not => VmUnaryOp::Not,
            };
            compiler.compile_unary(vm_op);
        }
        _ => panic!("Expression type not yet supported: {:?}", expr),
    }
}

/// Compare VM result with eval result
fn assert_parity(expr: &Expr, expected: Value) {
    // Compile to VM and execute
    let compiler = compile_expr_to_vm(expr);
    let chunk = compiler.finish();
    let vm_result = execute_chunk(chunk).expect("VM execution failed");

    // Use eval
    let mut ctx = EvaluationContext::new();
    let eval_result = eval_expr(expr, &mut ctx).expect("Eval failed");

    // Both should match expected
    assert_eq!(vm_result, Some(expected.clone()), "VM result mismatch");
    assert_eq!(eval_result, expected, "Eval result mismatch");
}

fn int_lit(n: i64) -> Expr {
    use pine_lexer::Span;
    Expr::Literal(Lit::Int(n), Span::default())
}

#[allow(dead_code)]
fn float_lit(f: f64) -> Expr {
    use pine_lexer::Span;
    Expr::Literal(Lit::Float(f), Span::default())
}

fn bool_lit(b: bool) -> Expr {
    use pine_lexer::Span;
    Expr::Literal(Lit::Bool(b), Span::default())
}

fn na_lit() -> Expr {
    use pine_lexer::Span;
    Expr::Literal(Lit::Na, Span::default())
}

fn bin_op(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    use pine_lexer::Span;
    Expr::BinOp {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span: Span::default(),
    }
}

fn unary_op(op: UnaryOp, operand: Expr) -> Expr {
    use pine_lexer::Span;
    Expr::UnaryOp {
        op,
        operand: Box::new(operand),
        span: Span::default(),
    }
}

#[test]
fn test_parity_arithmetic() {
    // Simple arithmetic: 1 + 2 = 3
    assert_parity(&bin_op(BinOp::Add, int_lit(1), int_lit(2)), Value::Int(3));
    // 10 - 3 = 7
    assert_parity(&bin_op(BinOp::Sub, int_lit(10), int_lit(3)), Value::Int(7));
    // 4 * 5 = 20
    assert_parity(&bin_op(BinOp::Mul, int_lit(4), int_lit(5)), Value::Int(20));
    // 20 / 4 = 5.0 (Pine Script / always returns float)
    assert_parity(
        &bin_op(BinOp::Div, int_lit(20), int_lit(4)),
        Value::Float(5.0),
    );
}

#[test]
fn test_parity_comparison() {
    // 1 < 2 = true
    assert_parity(
        &bin_op(BinOp::Lt, int_lit(1), int_lit(2)),
        Value::Bool(true),
    );
    // 2 < 1 = false
    assert_parity(
        &bin_op(BinOp::Lt, int_lit(2), int_lit(1)),
        Value::Bool(false),
    );
    // 1 == 1 = true
    assert_parity(
        &bin_op(BinOp::Eq, int_lit(1), int_lit(1)),
        Value::Bool(true),
    );
}

#[test]
fn test_parity_logical() {
    // true and false = false
    assert_parity(
        &bin_op(BinOp::And, bool_lit(true), bool_lit(false)),
        Value::Bool(false),
    );
    // true or false = true
    assert_parity(
        &bin_op(BinOp::Or, bool_lit(true), bool_lit(false)),
        Value::Bool(true),
    );
}

#[test]
fn test_parity_unary() {
    // -5
    assert_parity(&unary_op(UnaryOp::Neg, int_lit(5)), Value::Int(-5));
    // not false
    assert_parity(&unary_op(UnaryOp::Not, bool_lit(false)), Value::Bool(true));
}

#[test]
fn test_parity_nested() {
    // (1 + 2) * 3 = 9
    let expr = bin_op(
        BinOp::Mul,
        bin_op(BinOp::Add, int_lit(1), int_lit(2)),
        int_lit(3),
    );
    assert_parity(&expr, Value::Int(9));
}

#[test]
fn test_parity_na_propagation() {
    // na + 1 = na
    let expr = bin_op(BinOp::Add, na_lit(), int_lit(1));
    assert_parity(&expr, Value::Na);
}

/// Compile expression using the new AST compiler (via VM path)
fn compile_expr_to_vm_ast(expr: &Expr) -> pine_vm::compiler::Compiler {
    use pine_parser::ast::{Script, Stmt};

    // Use return statement to preserve the expression result
    let stmt = Stmt::Return {
        value: Some(expr.clone()),
        span: pine_lexer::Span::default(),
    };
    let script = Script {
        stmts: vec![stmt],
        span: pine_lexer::Span::default(),
    };

    pine_vm::ast_compiler::compile_script(&script).expect("AST compilation failed")
}

#[test]
fn test_parity_external_function_call() {
    // Test external function: math.abs(-5) = 5
    use pine_lexer::Span;
    use pine_parser::ast::{Arg, Expr, Ident};

    // Create function call: math.abs(-5)
    let func_expr = Expr::Ident(Ident::new("math.abs", Span::default()));
    let arg = Arg {
        name: None,
        value: int_lit(-5),
    };
    let call_expr = Expr::FnCall {
        func: Box::new(func_expr),
        args: vec![arg],
        span: Span::default(),
    };

    // Compile and execute with VM
    let compiler = compile_expr_to_vm_ast(&call_expr);
    let chunk = compiler.finish();

    // Verify external function was registered
    assert!(
        chunk.external_functions.contains(&"math.abs".to_string()),
        "math.abs should be registered as external function"
    );

    // Execute with VM
    let mut vm = pine_vm::vm::VM::new();
    for func_name in chunk.external_functions.iter() {
        vm.register_external_function(func_name);
    }
    vm.load_chunk(chunk);
    let vm_result = vm.execute().expect("VM execution failed");

    // Execute with eval
    let mut ctx = EvaluationContext::new();
    let eval_result = eval_expr(&call_expr, &mut ctx).expect("Eval failed");

    // math.abs returns Int for Int input, Float for Float input
    assert_eq!(vm_result, Some(Value::Int(5)), "VM result mismatch");
    assert_eq!(eval_result, Value::Int(5), "Eval result mismatch");
}
