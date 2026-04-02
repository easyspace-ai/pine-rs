//! Tests for VM executor with series data and plot output

use pine_lexer::Span;
use pine_parser::ast::{Expr, Ident, Script, Stmt};
use pine_vm::executor::{execute_script_with_vm, SeriesData};

fn create_simple_script() -> Script {
    // Create: return 42
    let stmt = Stmt::Return {
        value: Some(Expr::Literal(
            pine_parser::ast::Lit::Int(42),
            Span::default(),
        )),
        span: Span::default(),
    };

    Script {
        stmts: vec![stmt],
        span: Span::default(),
    }
}

#[test]
fn test_executor_basic() {
    let script = create_simple_script();

    // Create series data with 5 bars
    let series_data = SeriesData::new(
        vec![100.0, 101.0, 102.0, 103.0, 104.0],      // open
        vec![105.0, 106.0, 107.0, 108.0, 109.0],      // high
        vec![99.0, 100.0, 101.0, 102.0, 103.0],       // low
        vec![101.0, 102.0, 103.0, 104.0, 105.0],      // close
        vec![1000.0, 1100.0, 1200.0, 1300.0, 1400.0], // volume
        vec![1, 2, 3, 4, 5],                          // time
    );

    let result = execute_script_with_vm(&script, &series_data);
    assert!(result.is_ok(), "Execution failed: {:?}", result.err());

    let exec_result = result.unwrap();
    assert!(exec_result.success);
    assert_eq!(exec_result.bars_processed, 5);
}

#[test]
fn test_executor_external_function() {
    use pine_parser::ast::{Arg, Expr, Ident};

    // Create: return math.abs(-5)
    let func_expr = Expr::Ident(Ident::new("math.abs", Span::default()));
    let arg = Arg {
        name: None,
        value: Expr::Literal(pine_parser::ast::Lit::Int(-5), Span::default()),
    };
    let call_expr = Expr::FnCall {
        func: Box::new(func_expr),
        args: vec![arg],
        span: Span::default(),
    };

    let stmt = Stmt::Return {
        value: Some(call_expr),
        span: Span::default(),
    };

    let script = Script {
        stmts: vec![stmt],
        span: Span::default(),
    };

    let series_data = SeriesData::new(
        vec![100.0],
        vec![105.0],
        vec![99.0],
        vec![101.0],
        vec![1000.0],
        vec![1],
    );

    let result = execute_script_with_vm(&script, &series_data);
    assert!(result.is_ok(), "Execution failed: {:?}", result.err());

    let exec_result = result.unwrap();
    assert!(exec_result.success);
    assert_eq!(exec_result.bars_processed, 1);
}
