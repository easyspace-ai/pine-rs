//! VM Golden Tests - Execute golden scripts using VM and compare outputs.

use pine_vm::executor::{execute_script_with_vm, SeriesData};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct GoldenTestCase {
    name: String,
    golden_path: String,
    script_path: String,
}

fn workspace_root() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| PathBuf::from(d).join("../.."))
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_manifest() -> Result<Vec<GoldenTestCase>, Box<dyn std::error::Error>> {
    let manifest_path = workspace_root().join("tests/vm_parity_cases.json");
    let content = fs::read_to_string(manifest_path)?;
    Ok(serde_json::from_str(&content)?)
}

fn load_golden_csv(
    path: &Path,
) -> Result<(SeriesData, HashMap<String, Vec<Option<f64>>>), Box<dyn std::error::Error>> {
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
    let output_start = 6;

    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let mut time = Vec::new();
    let mut expected: HashMap<String, Vec<Option<f64>>> = HashMap::new();

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 2 {
            continue;
        }

        time.push(fields[time_idx].parse()?);
        let close_value: f64 = fields[close_idx].parse()?;
        close.push(close_value);

        open.push(
            open_idx
                .and_then(|i| fields.get(i))
                .and_then(|v| v.parse().ok())
                .unwrap_or(close_value),
        );
        high.push(
            high_idx
                .and_then(|i| fields.get(i))
                .and_then(|v| v.parse().ok())
                .unwrap_or(close_value),
        );
        low.push(
            low_idx
                .and_then(|i| fields.get(i))
                .and_then(|v| v.parse().ok())
                .unwrap_or(close_value),
        );
        volume.push(
            volume_idx
                .and_then(|i| fields.get(i))
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0),
        );

        for (col_idx, col_name) in columns.iter().enumerate() {
            if col_idx >= output_start {
                let value =
                    fields
                        .get(col_idx)
                        .and_then(|f| if f.is_empty() { None } else { f.parse().ok() });
                expected
                    .entry(col_name.to_string())
                    .or_default()
                    .push(value);
            }
        }
    }

    Ok((
        SeriesData::new(open, high, low, close, volume, time),
        expected,
    ))
}

fn parse_script(path: &Path) -> Result<pine_parser::ast::Script, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let tokens = pine_lexer::Lexer::lex_with_indentation(&content)
        .map_err(|e| format!("Lex error: {:?}", e))?;
    let ast = pine_parser::parser::parse(tokens).map_err(|e| format!("Parse error: {:?}", e))?;
    Ok(ast)
}

fn compare_outputs(
    plot_outputs: &pine_vm::executor::PlotOutputs,
    expected: &HashMap<String, Vec<Option<f64>>>,
    tolerance: f64,
) -> (bool, f64) {
    let mut all_match = true;
    let mut max_error = 0.0f64;

    for (plot_name, expected_values) in expected {
        if let Some(actual_values) = plot_outputs.get_plot(plot_name) {
            for (i, (expected, actual)) in
                expected_values.iter().zip(actual_values.iter()).enumerate()
            {
                match (expected, actual) {
                    (Some(exp), Some(act)) => {
                        let error = (exp - act).abs();
                        max_error = max_error.max(error);
                        if error > tolerance {
                            eprintln!(
                                "Mismatch at bar {} for '{}': expected {}, got {} (error: {})",
                                i, plot_name, exp, act, error
                            );
                            all_match = false;
                        }
                    }
                    (None, None) => {}
                    (Some(exp), None) => {
                        eprintln!(
                            "Mismatch at bar {} for '{}': expected {}, got NA",
                            i, plot_name, exp
                        );
                        all_match = false;
                    }
                    (None, Some(act)) => {
                        eprintln!(
                            "Mismatch at bar {} for '{}': expected NA, got {}",
                            i, plot_name, act
                        );
                        all_match = false;
                    }
                }
            }
        } else {
            eprintln!("Plot '{}' not found in VM outputs", plot_name);
            all_match = false;
        }
    }

    (all_match, max_error)
}

fn run_golden_test(test: &GoldenTestCase) -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root();
    let golden_path = root.join(&test.golden_path);
    let script_path = root.join(&test.script_path);

    if !golden_path.exists() {
        return Err(format!(
            "Golden file not found: {} (looked in {})",
            test.golden_path,
            golden_path.display()
        )
        .into());
    }
    if !script_path.exists() {
        return Err(format!(
            "Script file not found: {} (looked in {})",
            test.script_path,
            script_path.display()
        )
        .into());
    }

    let (series_data, expected) = load_golden_csv(&golden_path)?;
    let script = parse_script(&script_path)?;
    let result = execute_script_with_vm(&script, &series_data)?;

    if !result.success {
        return Err(format!("VM execution failed for {}", test.name).into());
    }

    if result.bars_processed != series_data.len() {
        return Err(format!(
            "Bars processed mismatch for {}: expected {}, got {}",
            test.name,
            series_data.len(),
            result.bars_processed
        )
        .into());
    }

    let tolerance = 1e-8;
    let (matches, max_error) = compare_outputs(&result.plot_outputs, &expected, tolerance);

    if matches {
        println!(
            "✓ {}: {} bars, max error {:.2e} (tolerance {:.0e})",
            test.name, result.bars_processed, max_error, tolerance
        );
        Ok(())
    } else {
        Err(format!(
            "Output mismatch for {} (max error: {:.2e})",
            test.name, max_error
        )
        .into())
    }
}

#[test]
fn test_vm_golden_manifest_cases() {
    let cases = load_manifest().expect("load VM parity manifest");
    assert_eq!(cases.len(), 59, "unexpected VM parity manifest size");

    for case in &cases {
        run_golden_test(case)
            .unwrap_or_else(|e| panic!("{} VM golden test failed: {e}", case.name));
    }
}
