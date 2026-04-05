//! VM vs Eval script-level parity tests.

use pine_eval::runner::{run_bar_by_bar, BarData};
use pine_eval::EvaluationContext;
use pine_vm::executor::{execute_script_with_vm, PlotOutputs, SeriesData};
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct ScriptParityCase {
    name: String,
    script_path: String,
    golden_path: String,
    plots: Vec<String>,
}

fn workspace_root() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| PathBuf::from(d).join("../.."))
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_manifest() -> Result<Vec<ScriptParityCase>, Box<dyn std::error::Error>> {
    let manifest_path = workspace_root().join("tests/vm_parity_cases.json");
    let content = fs::read_to_string(manifest_path)?;
    Ok(serde_json::from_str(&content)?)
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
    let script = parse_script(&root.join(&case.script_path))?;
    let series = load_series_data(&root.join(&case.golden_path))?;

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

    let expected_plots: BTreeSet<&str> = case.plots.iter().map(String::as_str).collect();
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
    for plot_name in &case.plots {
        let vm_values = vm_outputs
            .get(plot_name)
            .unwrap_or_else(|| panic!("{}: missing VM plot '{}'", case.name, plot_name));
        let eval_values = eval_outputs
            .get(plot_name)
            .unwrap_or_else(|| panic!("{}: missing eval plot '{}'", case.name, plot_name));
        assert_plot_series_close(&case.name, plot_name, vm_values, eval_values, tolerance);
    }

    Ok(())
}

#[test]
fn test_script_parity_manifest_cases() {
    let cases = load_manifest().expect("load VM parity manifest");
    assert_eq!(cases.len(), 44, "unexpected VM parity manifest size");

    for case in &cases {
        assert_script_parity(case)
            .unwrap_or_else(|e| panic!("{} script parity failed: {e}", case.name));
    }
}
