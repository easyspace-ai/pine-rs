//! VM vs Eval script-level parity tests.

use pine_eval::runner::{run_bar_by_bar, BarData};
use pine_eval::EvaluationContext;
use pine_vm::executor::{execute_script_with_vm, PlotOutputs, SeriesData};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ScriptParityCase {
    name: &'static str,
    script_path: &'static str,
    golden_path: &'static str,
    plots: &'static [&'static str],
}

fn workspace_root() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| PathBuf::from(d).join("../.."))
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn parse_script(path: &Path) -> Result<pine_parser::ast::Script, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let tokens = pine_lexer::Lexer::lex_with_indentation(&content)
        .map_err(|e| format!("Lex error: {e:?}"))?;
    let ast = pine_parser::parser::parse(tokens).map_err(|e| format!("Parse error: {e:?}"))?;
    Ok(ast)
}

fn load_series_data(path: &Path) -> Result<SeriesData, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.lines();
    let header = lines.next().ok_or("Empty CSV")?;
    let columns: Vec<&str> = header.split(',').collect();

    let get_idx = |name: &str| columns.iter().position(|&c| c == name);

    let time_idx = get_idx("time").unwrap_or(0);
    let open_idx = get_idx("open");
    let high_idx = get_idx("high");
    let low_idx = get_idx("low");
    let close_idx = get_idx("close").unwrap_or(1);
    let volume_idx = get_idx("volume");

    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let mut time = Vec::new();

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 2 {
            continue;
        }

        let close_value: f64 = fields[close_idx].parse()?;
        close.push(close_value);
        time.push(fields[time_idx].parse()?);

        open.push(
            open_idx
                .and_then(|idx| fields.get(idx))
                .and_then(|value| value.parse().ok())
                .unwrap_or(close_value),
        );
        high.push(
            high_idx
                .and_then(|idx| fields.get(idx))
                .and_then(|value| value.parse().ok())
                .unwrap_or(close_value),
        );
        low.push(
            low_idx
                .and_then(|idx| fields.get(idx))
                .and_then(|value| value.parse().ok())
                .unwrap_or(close_value),
        );
        volume.push(
            volume_idx
                .and_then(|idx| fields.get(idx))
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
        );
    }

    Ok(SeriesData::new(open, high, low, close, volume, time))
}

fn to_bar_data(series: &SeriesData) -> Vec<BarData> {
    series
        .open
        .iter()
        .zip(&series.high)
        .zip(&series.low)
        .zip(&series.close)
        .zip(&series.volume)
        .zip(&series.time)
        .map(|(((((open, high), low), close), volume), time)| {
            BarData::new(*open, *high, *low, *close, *volume, *time)
        })
        .collect()
}

fn plot_outputs_to_map(outputs: &PlotOutputs) -> HashMap<String, Vec<Option<f64>>> {
    outputs
        .get_plots()
        .iter()
        .map(|(name, values)| (name.clone(), values.clone()))
        .collect()
}

fn eval_outputs_for_script(
    script: &pine_parser::ast::Script,
    series: &SeriesData,
) -> Result<HashMap<String, Vec<Option<f64>>>, Box<dyn std::error::Error>> {
    let bars = to_bar_data(series);
    let mut ctx = EvaluationContext::new();
    run_bar_by_bar(script, &bars, &mut ctx).map_err(|e| format!("Eval failed: {e:?}"))?;
    Ok(ctx
        .plot_outputs
        .get_plots()
        .iter()
        .map(|(name, values)| (name.clone(), values.clone()))
        .collect())
}

fn assert_plot_series_close(
    case_name: &str,
    plot_name: &str,
    vm_values: &[Option<f64>],
    eval_values: &[Option<f64>],
    tolerance: f64,
) {
    assert_eq!(
        vm_values.len(),
        eval_values.len(),
        "{case_name}: plot '{plot_name}' length mismatch"
    );

    for (idx, (vm_value, eval_value)) in vm_values.iter().zip(eval_values.iter()).enumerate() {
        match (vm_value, eval_value) {
            (Some(vm), Some(eval)) => {
                let error = (vm - eval).abs();
                assert!(
                    error <= tolerance,
                    "{case_name}: plot '{plot_name}' mismatch at bar {idx}: vm={vm}, eval={eval}, error={error}"
                );
            }
            (None, None) => {}
            _ => panic!(
                "{case_name}: plot '{plot_name}' NA mismatch at bar {idx}: vm={vm_value:?}, eval={eval_value:?}"
            ),
        }
    }
}

fn assert_script_parity(case: &ScriptParityCase) -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root();
    let script = parse_script(&root.join(case.script_path))?;
    let series = load_series_data(&root.join(case.golden_path))?;

    let vm_result = execute_script_with_vm(&script, &series)?;
    assert!(
        vm_result.success,
        "{}: VM execution was not successful",
        case.name
    );
    assert_eq!(
        vm_result.bars_processed,
        series.close.len(),
        "{}: VM bars processed mismatch",
        case.name
    );

    let vm_outputs = plot_outputs_to_map(&vm_result.plot_outputs);
    let eval_outputs = eval_outputs_for_script(&script, &series)?;

    let expected_plots: BTreeSet<&str> = case.plots.iter().copied().collect();
    let vm_plot_names: BTreeSet<&str> = vm_outputs.keys().map(String::as_str).collect();
    let eval_plot_names: BTreeSet<&str> = eval_outputs.keys().map(String::as_str).collect();

    assert_eq!(
        vm_plot_names, expected_plots,
        "{}: VM plot names mismatch",
        case.name
    );
    assert_eq!(
        eval_plot_names, expected_plots,
        "{}: eval plot names mismatch",
        case.name
    );

    let tolerance = 1e-8;
    for plot_name in case.plots {
        let vm_values = vm_outputs
            .get(*plot_name)
            .unwrap_or_else(|| panic!("{}: missing VM plot '{}'", case.name, plot_name));
        let eval_values = eval_outputs
            .get(*plot_name)
            .unwrap_or_else(|| panic!("{}: missing eval plot '{}'", case.name, plot_name));
        assert_plot_series_close(case.name, plot_name, vm_values, eval_values, tolerance);
    }

    Ok(())
}

#[test]
fn test_script_parity_sma_manual() {
    assert_script_parity(&ScriptParityCase {
        name: "sma_manual",
        script_path: "tests/scripts/series/sma_manual.pine",
        golden_path: "tests/golden/sma_manual.csv",
        plots: &["Manual SMA", "Close", "manualSma"],
    })
    .expect("sma_manual script parity failed");
}

#[test]
fn test_script_parity_cross_events() {
    assert_script_parity(&ScriptParityCase {
        name: "cross_events",
        script_path: "tests/scripts/stdlib/ta/cross_events.pine",
        golden_path: "tests/golden/cross_events.csv",
        plots: &["Cross Up", "Cross Down", "Bars Since Up"],
    })
    .expect("cross_events script parity failed");
}

#[test]
fn test_script_parity_macd() {
    assert_script_parity(&ScriptParityCase {
        name: "macd_12_26_9",
        script_path: "tests/scripts/stdlib/ta/macd_12_26_9.pine",
        golden_path: "tests/golden/macd_12_26_9.csv",
        plots: &["MACD Line", "Signal Line", "Histogram"],
    })
    .expect("macd_12_26_9 script parity failed");
}

#[test]
fn test_script_parity_bbands() {
    assert_script_parity(&ScriptParityCase {
        name: "bbands_20_2",
        script_path: "tests/scripts/stdlib/ta/bbands_20_2.pine",
        golden_path: "tests/golden/bbands_20_2.csv",
        plots: &["Basis (SMA 20)", "Upper Band", "Lower Band"],
    })
    .expect("bbands_20_2 script parity failed");
}

#[test]
fn test_script_parity_rsi() {
    assert_script_parity(&ScriptParityCase {
        name: "rsi_14",
        script_path: "tests/scripts/stdlib/ta/rsi_14.pine",
        golden_path: "tests/golden/rsi_14.csv",
        plots: &["RSI 14"],
    })
    .expect("rsi_14 script parity failed");
}

#[test]
fn test_script_parity_stoch() {
    assert_script_parity(&ScriptParityCase {
        name: "stoch_14_3_3",
        script_path: "tests/scripts/stdlib/ta/stoch_14_3_3.pine",
        golden_path: "tests/golden/stoch_14_3_3.csv",
        plots: &["%K"],
    })
    .expect("stoch_14_3_3 script parity failed");
}

#[test]
fn test_script_parity_atr() {
    assert_script_parity(&ScriptParityCase {
        name: "atr_14",
        script_path: "tests/scripts/stdlib/ta/atr_14.pine",
        golden_path: "tests/golden/atr_14.csv",
        plots: &["ATR 14"],
    })
    .expect("atr_14 script parity failed");
}

#[test]
fn test_script_parity_sma_14() {
    assert_script_parity(&ScriptParityCase {
        name: "sma_14",
        script_path: "tests/scripts/stdlib/ta/sma_14.pine",
        golden_path: "tests/golden/sma_14.csv",
        plots: &["SMA 14"],
    })
    .expect("sma_14 script parity failed");
}

#[test]
fn test_script_parity_ema_12() {
    assert_script_parity(&ScriptParityCase {
        name: "ema_12",
        script_path: "tests/scripts/stdlib/ta/ema_12.pine",
        golden_path: "tests/golden/ema_12.csv",
        plots: &["EMA 12"],
    })
    .expect("ema_12 script parity failed");
}

#[test]
fn test_script_parity_highest_10() {
    assert_script_parity(&ScriptParityCase {
        name: "highest_10",
        script_path: "tests/scripts/stdlib/ta/highest_10.pine",
        golden_path: "tests/golden/highest_10.csv",
        plots: &["Highest 10"],
    })
    .expect("highest_10 script parity failed");
}

#[test]
fn test_script_parity_lowest_10() {
    assert_script_parity(&ScriptParityCase {
        name: "lowest_10",
        script_path: "tests/scripts/stdlib/ta/lowest_10.pine",
        golden_path: "tests/golden/lowest_10.csv",
        plots: &["Lowest 10"],
    })
    .expect("lowest_10 script parity failed");
}

#[test]
fn test_script_parity_highestbars_10() {
    assert_script_parity(&ScriptParityCase {
        name: "highestbars_10",
        script_path: "tests/scripts/stdlib/ta/highestbars_10.pine",
        golden_path: "tests/golden/highestbars_10.csv",
        plots: &["Highest Bars 10"],
    })
    .expect("highestbars_10 script parity failed");
}

#[test]
fn test_script_parity_lowestbars_10() {
    assert_script_parity(&ScriptParityCase {
        name: "lowestbars_10",
        script_path: "tests/scripts/stdlib/ta/lowestbars_10.pine",
        golden_path: "tests/golden/lowestbars_10.csv",
        plots: &["Lowest Bars 10"],
    })
    .expect("lowestbars_10 script parity failed");
}

#[test]
fn test_script_parity_mom_10() {
    assert_script_parity(&ScriptParityCase {
        name: "mom_10",
        script_path: "tests/scripts/stdlib/ta/mom_10.pine",
        golden_path: "tests/golden/mom_10.csv",
        plots: &["Momentum 10"],
    })
    .expect("mom_10 script parity failed");
}

#[test]
fn test_script_parity_cci_20() {
    assert_script_parity(&ScriptParityCase {
        name: "cci_20",
        script_path: "tests/scripts/stdlib/ta/cci_20.pine",
        golden_path: "tests/golden/cci_20.csv",
        plots: &["CCI 20"],
    })
    .expect("cci_20 script parity failed");
}

#[test]
fn test_script_parity_tr_basic() {
    assert_script_parity(&ScriptParityCase {
        name: "tr_basic",
        script_path: "tests/scripts/stdlib/ta/tr_basic.pine",
        golden_path: "tests/golden/tr_basic.csv",
        plots: &["TR"],
    })
    .expect("tr_basic script parity failed");
}
