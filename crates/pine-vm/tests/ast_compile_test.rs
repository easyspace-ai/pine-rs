//! Tests for AST to Bytecode compiler

use pine_parser::ast::{Arg, BinOp, Block, Expr, FnBody, Ident, Lit, Param, Stmt};
use pine_runtime::value::Value;
use pine_vm::ast_compiler::compile_script;
use pine_vm::vm::execute_chunk;

fn int_lit(n: i64) -> Expr {
    use pine_lexer::Span;
    Expr::Literal(Lit::Int(n), Span::default())
}

fn bool_lit(b: bool) -> Expr {
    use pine_lexer::Span;
    Expr::Literal(Lit::Bool(b), Span::default())
}

fn ident(name: &str) -> Expr {
    Expr::Ident(Ident::new(name, pine_lexer::Span::default()))
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

fn fn_call(name: &str, args: Vec<Expr>) -> Expr {
    use pine_lexer::Span;
    Expr::FnCall {
        func: Box::new(ident(name)),
        args: args
            .into_iter()
            .map(|value| Arg { name: None, value })
            .collect(),
        span: Span::default(),
    }
}

fn param(name: &str) -> Param {
    use pine_lexer::Span;
    Param {
        name: Ident::new(name, Span::default()),
        type_ann: None,
        default: None,
    }
}

fn make_block(stmts: Vec<Stmt>) -> Block {
    Block {
        stmts,
        span: pine_lexer::Span::default(),
    }
}

fn var_decl(name: &str, init: Expr) -> Stmt {
    use pine_lexer::Span;
    Stmt::VarDecl {
        name: Ident::new(name, Span::default()),
        kind: pine_parser::ast::VarKind::Plain,
        type_ann: None,
        init: Some(init),
        span: Span::default(),
    }
}

fn assign(name: &str, value: Expr) -> Stmt {
    use pine_lexer::Span;
    Stmt::Assign {
        target: pine_parser::ast::AssignTarget::Var(Ident::new(name, Span::default())),
        op: pine_parser::ast::AssignOp::Assign,
        value,
        span: Span::default(),
    }
}

fn ret(value: Expr) -> Stmt {
    use pine_lexer::Span;
    Stmt::Return {
        value: Some(value),
        span: Span::default(),
    }
}

#[test]
fn test_compile_simple_script() {
    // x = 5; y = x + 3; return y
    let script = pine_parser::ast::Script {
        stmts: vec![
            var_decl("x", int_lit(5)),
            var_decl("y", bin_op(BinOp::Add, ident("x"), int_lit(3))),
            ret(ident("y")),
        ],
        span: pine_lexer::Span::default(),
    };

    let compiler = compile_script(&script).expect("Compile failed");
    let chunk = compiler.finish();
    let result = execute_chunk(chunk).expect("Execution failed");

    assert_eq!(result, Some(Value::Int(8)));
}

#[test]
fn test_compile_if_statement() {
    // x = 0; if true then x = 1 else x = 2; return x
    let script = pine_parser::ast::Script {
        stmts: vec![
            var_decl("x", int_lit(0)),
            Stmt::If {
                cond: bool_lit(true),
                then_block: make_block(vec![assign("x", int_lit(1))]),
                elifs: vec![],
                else_block: Some(make_block(vec![assign("x", int_lit(2))])),
                span: pine_lexer::Span::default(),
            },
            ret(ident("x")),
        ],
        span: pine_lexer::Span::default(),
    };

    let compiler = compile_script(&script).expect("Compile failed");
    let chunk = compiler.finish();
    let result = execute_chunk(chunk).expect("Execution failed");

    assert_eq!(result, Some(Value::Int(1)));
}

#[test]
fn test_compile_for_loop() {
    // sum = 0; for i = 1 to 5 { sum = sum + i }; return sum
    let script = pine_parser::ast::Script {
        stmts: vec![
            var_decl("sum", int_lit(0)),
            Stmt::For {
                var: Ident::new("i", pine_lexer::Span::default()),
                from: int_lit(1),
                to: int_lit(5),
                by: None,
                body: make_block(vec![assign(
                    "sum",
                    bin_op(BinOp::Add, ident("sum"), ident("i")),
                )]),
                span: pine_lexer::Span::default(),
            },
            ret(ident("sum")),
        ],
        span: pine_lexer::Span::default(),
    };

    let compiler = compile_script(&script).expect("Compile failed");
    let chunk = compiler.finish();
    let result = execute_chunk(chunk).expect("Execution failed");

    assert_eq!(result, Some(Value::Int(15)));
}

#[test]
fn test_compile_while_loop() {
    // sum = 0; i = 1; while i <= 5 { sum = sum + i; i = i + 1 }; return sum
    let script = pine_parser::ast::Script {
        stmts: vec![
            var_decl("sum", int_lit(0)),
            var_decl("i", int_lit(1)),
            Stmt::While {
                cond: bin_op(BinOp::Le, ident("i"), int_lit(5)),
                body: make_block(vec![
                    assign("sum", bin_op(BinOp::Add, ident("sum"), ident("i"))),
                    assign("i", bin_op(BinOp::Add, ident("i"), int_lit(1))),
                ]),
                span: pine_lexer::Span::default(),
            },
            ret(ident("sum")),
        ],
        span: pine_lexer::Span::default(),
    };

    let compiler = compile_script(&script).expect("Compile failed");
    let chunk = compiler.finish();
    let result = execute_chunk(chunk).expect("Execution failed");

    assert_eq!(result, Some(Value::Int(15)));
}

#[test]
fn test_compile_udf_expression_and_block_body() {
    let script = pine_parser::ast::Script {
        stmts: vec![
            Stmt::FnDef {
                name: Ident::new("diff", pine_lexer::Span::default()),
                params: vec![param("a"), param("b")],
                ret_type: None,
                body: FnBody::Expr(bin_op(BinOp::Sub, ident("a"), ident("b"))),
                span: pine_lexer::Span::default(),
            },
            Stmt::FnDef {
                name: Ident::new("scale", pine_lexer::Span::default()),
                params: vec![param("src"), param("factor")],
                ret_type: None,
                body: FnBody::Block(make_block(vec![Stmt::Expr(bin_op(
                    BinOp::Add,
                    bin_op(BinOp::Mul, ident("src"), ident("factor")),
                    int_lit(1),
                ))])),
                span: pine_lexer::Span::default(),
            },
            var_decl("x", fn_call("diff", vec![int_lit(10), int_lit(7)])),
            var_decl("y", fn_call("scale", vec![int_lit(10), int_lit(2)])),
            ret(bin_op(BinOp::Add, ident("x"), ident("y"))),
        ],
        span: pine_lexer::Span::default(),
    };

    let compiler = compile_script(&script).expect("Compile failed");
    let chunk = compiler.finish();
    let result = execute_chunk(chunk).expect("Execution failed");

    assert_eq!(result, Some(Value::Int(24)));
}
