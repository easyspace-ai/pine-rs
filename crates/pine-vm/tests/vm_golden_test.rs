//! VM Golden Tests - Execute golden scripts using VM and compare outputs

use pine_vm::executor::{execute_script_with_vm, SeriesData};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Test case for a golden file pair
#[derive(Debug)]
struct GoldenTestCase {
    name: &'static str,
    golden_path: &'static str,
    script_path: &'static str,
}

/// Get workspace root path
fn workspace_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR is set to crates/pine-vm when running tests
    // We need to go up two levels to reach workspace root
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| std::path::PathBuf::from(d).join("../.."))
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// All golden test cases
#[allow(dead_code)]
const GOLDEN_TESTS: &[GoldenTestCase] = &[
    GoldenTestCase {
        name: "sma_14",
        golden_path: "tests/golden/sma_14.csv",
        script_path: "tests/scripts/stdlib/ta/sma_14.pine",
    },
    GoldenTestCase {
        name: "ema_12",
        golden_path: "tests/golden/ema_12.csv",
        script_path: "tests/scripts/stdlib/ta/ema_12.pine",
    },
    GoldenTestCase {
        name: "rsi_14",
        golden_path: "tests/golden/rsi_14.csv",
        script_path: "tests/scripts/stdlib/ta/rsi_14.pine",
    },
    GoldenTestCase {
        name: "macd_12_26_9",
        golden_path: "tests/golden/macd_12_26_9.csv",
        script_path: "tests/scripts/stdlib/ta/macd_12_26_9.pine",
    },
    GoldenTestCase {
        name: "bbands_20_2",
        golden_path: "tests/golden/bbands_20_2.csv",
        script_path: "tests/scripts/stdlib/ta/bbands_20_2.pine",
    },
    GoldenTestCase {
        name: "stoch_14_3_3",
        golden_path: "tests/golden/stoch_14_3_3.csv",
        script_path: "tests/scripts/stdlib/ta/stoch_14_3_3.pine",
    },
    GoldenTestCase {
        name: "atr_14",
        golden_path: "tests/golden/atr_14.csv",
        script_path: "tests/scripts/stdlib/ta/atr_14.pine",
    },
    GoldenTestCase {
        name: "cci_20",
        golden_path: "tests/golden/cci_20.csv",
        script_path: "tests/scripts/stdlib/ta/cci_20.pine",
    },
    GoldenTestCase {
        name: "cross_events",
        golden_path: "tests/golden/cross_events.csv",
        script_path: "tests/scripts/stdlib/ta/cross_events.pine",
    },
    GoldenTestCase {
        name: "highest_10",
        golden_path: "tests/golden/highest_10.csv",
        script_path: "tests/scripts/stdlib/ta/highest_10.pine",
    },
    GoldenTestCase {
        name: "lowest_10",
        golden_path: "tests/golden/lowest_10.csv",
        script_path: "tests/scripts/stdlib/ta/lowest_10.pine",
    },
    GoldenTestCase {
        name: "highestbars_10",
        golden_path: "tests/golden/highestbars_10.csv",
        script_path: "tests/scripts/stdlib/ta/highestbars_10.pine",
    },
    GoldenTestCase {
        name: "lowestbars_10",
        golden_path: "tests/golden/lowestbars_10.csv",
        script_path: "tests/scripts/stdlib/ta/lowestbars_10.pine",
    },
    GoldenTestCase {
        name: "mom_10",
        golden_path: "tests/golden/mom_10.csv",
        script_path: "tests/scripts/stdlib/ta/mom_10.pine",
    },
    GoldenTestCase {
        name: "tr_basic",
        golden_path: "tests/golden/tr_basic.csv",
        script_path: "tests/scripts/stdlib/ta/tr_basic.pine",
    },
    GoldenTestCase {
        name: "sma_manual",
        golden_path: "tests/golden/sma_manual.csv",
        script_path: "tests/scripts/series/sma_manual.pine",
    },
];

/// Load input series data and expected outputs from golden CSV
fn load_golden_csv(
    path: &Path,
) -> Result<(SeriesData, HashMap<String, Vec<Option<f64>>>), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.lines();

    // Parse header
    let header = lines.next().ok_or("Empty CSV")?;
    let columns: Vec<&str> = header.split(',').collect();

    // Find column indices for OHLCV
    let get_idx = |name: &str| columns.iter().position(|&c| c == name);

    let time_idx = get_idx("time").unwrap_or(0);
    let open_idx = get_idx("open");
    let high_idx = get_idx("high");
    let low_idx = get_idx("low");
    let close_idx = get_idx("close").unwrap_or(1);
    let volume_idx = get_idx("volume");

    // Output columns start after volume (index 5) or after close if no volume
    let output_start = 6; // time, open, high, low, close, volume

    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let mut time = Vec::new();

    // Expected outputs: column name -> series of values
    let mut expected: HashMap<String, Vec<Option<f64>>> = HashMap::new();

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 2 {
            continue;
        }

        // Parse time
        let t: i64 = fields[time_idx].parse()?;
        time.push(t);

        // Parse close (required)
        let c: f64 = fields[close_idx].parse()?;
        close.push(c);

        // Parse optional OHLCV fields
        let o = open_idx
            .and_then(|i| fields.get(i))
            .and_then(|v| v.parse().ok())
            .unwrap_or(c);
        open.push(o);

        let h = high_idx
            .and_then(|i| fields.get(i))
            .and_then(|v| v.parse().ok())
            .unwrap_or(c);
        high.push(h);

        let l = low_idx
            .and_then(|i| fields.get(i))
            .and_then(|v| v.parse().ok())
            .unwrap_or(c);
        low.push(l);

        let v = volume_idx
            .and_then(|i| fields.get(i))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        volume.push(v);

        // Parse expected outputs
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

/// Parse a Pine Script file
fn parse_script(path: &Path) -> Result<pine_parser::ast::Script, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;

    let tokens = pine_lexer::Lexer::lex_with_indentation(&content)
        .map_err(|e| format!("Lex error: {:?}", e))?;

    let ast = pine_parser::parser::parse(tokens).map_err(|e| format!("Parse error: {:?}", e))?;

    Ok(ast)
}

/// Compare VM plot outputs with expected values
/// Returns (all_match, max_error)
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
                    (None, None) => {} // Both NA, OK
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

/// Run a single golden test
fn run_golden_test(test: &GoldenTestCase) -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root();
    let golden_path = root.join(test.golden_path);
    let script_path = root.join(test.script_path);

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

    // Load golden data
    let (series_data, expected) = load_golden_csv(&golden_path)?;

    // Parse script
    let script = parse_script(&script_path)?;

    // Execute with VM
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

    // Compare outputs
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
fn test_vm_golden_sma_14() {
    run_golden_test(&GoldenTestCase {
        name: "sma_14",
        golden_path: "tests/golden/sma_14.csv",
        script_path: "tests/scripts/stdlib/ta/sma_14.pine",
    })
    .expect("sma_14 test failed");
}

#[test]
fn test_vm_golden_ema_12() {
    run_golden_test(&GoldenTestCase {
        name: "ema_12",
        golden_path: "tests/golden/ema_12.csv",
        script_path: "tests/scripts/stdlib/ta/ema_12.pine",
    })
    .expect("ema_12 test failed");
}

#[test]
fn test_vm_golden_rsi_14() {
    run_golden_test(&GoldenTestCase {
        name: "rsi_14",
        golden_path: "tests/golden/rsi_14.csv",
        script_path: "tests/scripts/stdlib/ta/rsi_14.pine",
    })
    .expect("rsi_14 test failed");
}

#[test]
fn test_vm_golden_macd() {
    run_golden_test(&GoldenTestCase {
        name: "macd_12_26_9",
        golden_path: "tests/golden/macd_12_26_9.csv",
        script_path: "tests/scripts/stdlib/ta/macd_12_26_9.pine",
    })
    .expect("macd_12_26_9 test failed");
}

#[test]
fn test_vm_golden_bbands() {
    run_golden_test(&GoldenTestCase {
        name: "bbands_20_2",
        golden_path: "tests/golden/bbands_20_2.csv",
        script_path: "tests/scripts/stdlib/ta/bbands_20_2.pine",
    })
    .expect("bbands_20_2 test failed");
}

#[test]
fn test_vm_golden_stoch() {
    run_golden_test(&GoldenTestCase {
        name: "stoch_14_3_3",
        golden_path: "tests/golden/stoch_14_3_3.csv",
        script_path: "tests/scripts/stdlib/ta/stoch_14_3_3.pine",
    })
    .expect("stoch_14_3_3 test failed");
}

#[test]
fn test_vm_golden_atr_14() {
    run_golden_test(&GoldenTestCase {
        name: "atr_14",
        golden_path: "tests/golden/atr_14.csv",
        script_path: "tests/scripts/stdlib/ta/atr_14.pine",
    })
    .expect("atr_14 test failed");
}

#[test]
fn test_vm_golden_cci_20() {
    run_golden_test(&GoldenTestCase {
        name: "cci_20",
        golden_path: "tests/golden/cci_20.csv",
        script_path: "tests/scripts/stdlib/ta/cci_20.pine",
    })
    .expect("cci_20 test failed");
}

#[test]
fn test_vm_golden_cross_events() {
    run_golden_test(&GoldenTestCase {
        name: "cross_events",
        golden_path: "tests/golden/cross_events.csv",
        script_path: "tests/scripts/stdlib/ta/cross_events.pine",
    })
    .expect("cross_events test failed");
}

#[test]
fn test_vm_golden_highest_10() {
    run_golden_test(&GoldenTestCase {
        name: "highest_10",
        golden_path: "tests/golden/highest_10.csv",
        script_path: "tests/scripts/stdlib/ta/highest_10.pine",
    })
    .expect("highest_10 test failed");
}

#[test]
fn test_vm_golden_lowest_10() {
    run_golden_test(&GoldenTestCase {
        name: "lowest_10",
        golden_path: "tests/golden/lowest_10.csv",
        script_path: "tests/scripts/stdlib/ta/lowest_10.pine",
    })
    .expect("lowest_10 test failed");
}

#[test]
fn test_vm_golden_highestbars_10() {
    run_golden_test(&GoldenTestCase {
        name: "highestbars_10",
        golden_path: "tests/golden/highestbars_10.csv",
        script_path: "tests/scripts/stdlib/ta/highestbars_10.pine",
    })
    .expect("highestbars_10 test failed");
}

#[test]
fn test_vm_golden_lowestbars_10() {
    run_golden_test(&GoldenTestCase {
        name: "lowestbars_10",
        golden_path: "tests/golden/lowestbars_10.csv",
        script_path: "tests/scripts/stdlib/ta/lowestbars_10.pine",
    })
    .expect("lowestbars_10 test failed");
}

#[test]
fn test_vm_golden_mom_10() {
    run_golden_test(&GoldenTestCase {
        name: "mom_10",
        golden_path: "tests/golden/mom_10.csv",
        script_path: "tests/scripts/stdlib/ta/mom_10.pine",
    })
    .expect("mom_10 test failed");
}

#[test]
fn test_vm_golden_tr_basic() {
    run_golden_test(&GoldenTestCase {
        name: "tr_basic",
        golden_path: "tests/golden/tr_basic.csv",
        script_path: "tests/scripts/stdlib/ta/tr_basic.pine",
    })
    .expect("tr_basic test failed");
}

#[test]
fn test_vm_golden_sma_manual() {
    run_golden_test(&GoldenTestCase {
        name: "sma_manual",
        golden_path: "tests/golden/sma_manual.csv",
        script_path: "tests/scripts/series/sma_manual.pine",
    })
    .expect("sma_manual test failed");
}
