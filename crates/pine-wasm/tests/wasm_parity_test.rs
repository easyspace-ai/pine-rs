//! WASM bridge parity tests: verify VM and eval produce identical results
//! via the native helper API (runs on any target, no browser needed).

use pine_eval::runner::BarData;
use pine_vm::executor::SeriesData;
use pine_wasm::native;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct VmParityCase {
    name: String,
    script_path: String,
    golden_path: String,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_vm_parity_cases() -> Vec<VmParityCase> {
    let root = workspace_root();
    let content = fs::read_to_string(root.join("tests/vm_parity_cases.json"))
        .expect("read VM parity manifest");
    serde_json::from_str(&content).expect("parse VM parity manifest")
}

fn load_bars_from_csv(path: &str) -> (Vec<BarData>, SeriesData) {
    let root = workspace_root();
    let content = fs::read_to_string(root.join(path)).expect("read golden CSV");
    let mut lines = content.lines();
    let _header = lines.next().expect("CSV header");

    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let mut time = Vec::new();

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 {
            continue;
        }
        let t: i64 = fields[0].parse().unwrap_or(0);
        let o: f64 = fields[1].parse().unwrap_or(0.0);
        let h: f64 = fields[2].parse().unwrap_or(0.0);
        let l: f64 = fields[3].parse().unwrap_or(0.0);
        let c: f64 = fields[4].parse().unwrap_or(0.0);
        let v: f64 = fields[5].parse().unwrap_or(0.0);
        time.push(t);
        open.push(o);
        high.push(h);
        low.push(l);
        close.push(c);
        volume.push(v);
    }

    let bars: Vec<BarData> = (0..close.len())
        .map(|i| BarData::new(open[i], high[i], low[i], close[i], volume[i], time[i]))
        .collect();

    let series = SeriesData::new(
        open.clone(),
        high.clone(),
        low.clone(),
        close.clone(),
        volume.clone(),
        time.clone(),
    );

    (bars, series)
}

fn assert_plot_maps_match(
    eval_plots: &HashMap<String, Vec<Option<f64>>>,
    vm_plots: &HashMap<String, Vec<Option<f64>>>,
    label: &str,
) {
    let mut eval_keys: Vec<_> = eval_plots.keys().cloned().collect();
    let mut vm_keys: Vec<_> = vm_plots.keys().cloned().collect();
    eval_keys.sort();
    vm_keys.sort();
    assert_eq!(eval_keys, vm_keys, "plot key mismatch for {label}");

    for key in eval_keys {
        let eval_vals = &eval_plots[&key];
        let vm_vals = &vm_plots[&key];
        assert_eq!(
            eval_vals.len(),
            vm_vals.len(),
            "length mismatch for {label}:{key}"
        );
        for (i, (e, v)) in eval_vals.iter().zip(vm_vals.iter()).enumerate() {
            match (e, v) {
                (None, None) => {}
                (Some(e), Some(v)) => {
                    if e.is_nan() && v.is_nan() {
                        continue;
                    }
                    assert!(
                        (e - v).abs() <= 1e-8,
                        "value mismatch for {label}:{key} at bar {i}: eval={e}, vm={v}"
                    );
                }
                _ => panic!("na mismatch for {label}:{key} at bar {i}: eval={e:?}, vm={v:?}"),
            }
        }
    }
}

/// Verify that the native eval and VM helpers produce identical output
/// for a representative subset of the VM parity manifest.
#[test]
fn test_wasm_native_vm_eval_parity() {
    let cases = load_vm_parity_cases();
    // Test at least 10 representative cases through the WASM native bridge
    let sample: Vec<_> = cases.iter().take(20).collect();
    assert!(
        sample.len() >= 10,
        "expected at least 10 parity cases, got {}",
        sample.len()
    );

    let root = workspace_root();
    for case in &sample {
        let script = fs::read_to_string(root.join(&case.script_path)).expect("read script");
        let (bars, series) = load_bars_from_csv(&case.golden_path);

        let eval_plots = native::run_eval(&script, &bars)
            .unwrap_or_else(|e| panic!("{}: eval failed: {e}", case.name));
        let vm_plots = native::run_vm(&script, &series)
            .unwrap_or_else(|e| panic!("{}: vm failed: {e}", case.name));

        assert_plot_maps_match(&eval_plots, &vm_plots, &case.name);
    }
}

/// Verify that the native parse function works correctly.
#[test]
fn test_wasm_native_parse() {
    let result = native::parse("//@version=6\nindicator(\"Test\")\nplot(close)");
    assert!(result.is_ok());

    let result = native::parse("//@version=6\nindicator(\"Test\"\nplot(close)");
    assert!(result.is_err());
}
