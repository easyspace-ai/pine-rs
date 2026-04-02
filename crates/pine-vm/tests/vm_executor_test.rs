//! Tests for VM executor with series data and plot output

use pine_lexer::Span;
use pine_parser::ast::{Expr, Script, Stmt};
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

fn parse_script(source: &str) -> Script {
    let tokens = pine_lexer::Lexer::lex_with_indentation(source).expect("Lex failed");
    pine_parser::parser::parse(tokens).expect("Parse failed")
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

#[test]
fn test_executor_tracks_named_series_expression_history() {
    let script = parse_script(
        r#"
indicator("named series")
src = close + open
avg = ta.sma(src, 2)
plot(avg, title="avg")
"#,
    );

    let series_data = SeriesData::new(
        vec![1.0, 2.0, 3.0, 4.0], // open
        vec![1.0, 2.0, 3.0, 4.0], // high
        vec![1.0, 2.0, 3.0, 4.0], // low
        vec![1.0, 2.0, 3.0, 4.0], // close
        vec![0.0; 4],
        vec![1, 2, 3, 4],
    );

    let result = execute_script_with_vm(&script, &series_data).expect("Execution failed");
    let avg = result
        .plot_outputs
        .get_plot("avg")
        .expect("Missing avg plot");

    assert_eq!(avg, &vec![None, Some(3.0), Some(5.0), Some(7.0)]);
}

#[test]
fn test_executor_accumulates_inline_series_expression_history() {
    let script = parse_script(
        r#"
indicator("inline series")
avg = ta.sma(close + open, 2)
plot(avg, title="avg")
"#,
    );

    let series_data = SeriesData::new(
        vec![1.0, 2.0, 3.0, 4.0], // open
        vec![1.0, 2.0, 3.0, 4.0], // high
        vec![1.0, 2.0, 3.0, 4.0], // low
        vec![1.0, 2.0, 3.0, 4.0], // close
        vec![0.0; 4],
        vec![1, 2, 3, 4],
    );

    let result = execute_script_with_vm(&script, &series_data).expect("Execution failed");
    let avg = result
        .plot_outputs
        .get_plot("avg")
        .expect("Missing avg plot");

    assert_eq!(avg, &vec![None, Some(3.0), Some(5.0), Some(7.0)]);
}

#[test]
fn test_executor_compound_series_expression_history() {
    let script = parse_script(
        r#"
//@version=6
indicator("compound series history")
x = close + open
prev = x[1]
plot(x, title="x", display=display.none)
plot(prev, title="prev", display=display.none)
"#,
    );

    let series_data = SeriesData::new(
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![10.0, 20.0, 30.0],
        vec![100.0, 100.0, 100.0],
        vec![1, 2, 3],
    );

    let result = execute_script_with_vm(&script, &series_data).expect("Execution failed");
    assert!(result.success);
    assert_eq!(
        result.plot_outputs.get_plot("x"),
        Some(&vec![Some(11.0), Some(22.0), Some(33.0)])
    );
    assert_eq!(
        result.plot_outputs.get_plot("prev"),
        Some(&vec![None, Some(11.0), Some(22.0)])
    );
}

#[test]
fn test_executor_series_expression_overwrites_within_bar() {
    let script = parse_script(
        r#"
//@version=6
indicator("series overwrite")
x = close
x := x + open
prev = x[1]
plot(x, title="x", display=display.none)
plot(prev, title="prev", display=display.none)
"#,
    );

    let series_data = SeriesData::new(
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![10.0, 20.0, 30.0],
        vec![100.0, 100.0, 100.0],
        vec![1, 2, 3],
    );

    let result = execute_script_with_vm(&script, &series_data).expect("Execution failed");
    assert!(result.success);
    assert_eq!(
        result.plot_outputs.get_plot("x"),
        Some(&vec![Some(11.0), Some(22.0), Some(33.0)])
    );
    assert_eq!(
        result.plot_outputs.get_plot("prev"),
        Some(&vec![None, Some(11.0), Some(22.0)])
    );
}

#[test]
fn test_executor_registers_time_series() {
    let script = parse_script(
        r#"
//@version=6
indicator("time series")
plot(time, title="time", display=display.none)
"#,
    );

    let series_data = SeriesData::new(
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![10.0, 20.0, 30.0],
        vec![100.0, 100.0, 100.0],
        vec![1000, 2000, 3000],
    );

    let result = execute_script_with_vm(&script, &series_data).expect("Execution failed");
    assert!(result.success);
    assert_eq!(
        result.plot_outputs.get_plot("time"),
        Some(&vec![Some(1000.0), Some(2000.0), Some(3000.0)])
    );
}
