//! WebAssembly bridge: `check_script` and `run_script_json` / `run_script_json_vm`
//! over [`pine_eval`] and [`pine_vm`].
//!
//! Build: `cargo build -p pine-wasm --target wasm32-unknown-unknown`.

#![allow(missing_docs)]

#[cfg(target_arch = "wasm32")]
mod wasm_bridge;

#[cfg(target_arch = "wasm32")]
pub use wasm_bridge::{check_script, run_script_json, run_script_json_vm};

/// Native (non-wasm) helpers for testing VM↔eval parity without a browser.
///
/// These mirror the wasm_bridge functions but work on any target.
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use pine_eval::runner::{run_bar_by_bar, BarData};
    use pine_eval::EvaluationContext;
    use pine_vm::executor::{execute_script_with_vm, SeriesData};
    use std::collections::HashMap;

    /// Parse a Pine Script source into an AST.
    pub fn parse(source: &str) -> Result<pine_parser::ast::Script, String> {
        let tokens =
            pine_lexer::Lexer::lex_with_indentation(source).map_err(|e| format!("{e:?}"))?;
        pine_parser::parser::parse(tokens).map_err(|e| format!("{e:?}"))
    }

    /// Run script with eval engine; returns plot map.
    pub fn run_eval(
        source: &str,
        bars: &[BarData],
    ) -> Result<HashMap<String, Vec<Option<f64>>>, String> {
        let script = parse(source)?;
        let mut ctx = EvaluationContext::new();
        run_bar_by_bar(&script, bars, &mut ctx).map_err(|e| format!("{e}"))?;
        Ok(ctx.plot_outputs.get_plots().clone())
    }

    /// Run script with VM engine; returns plot map.
    pub fn run_vm(
        source: &str,
        series: &SeriesData,
    ) -> Result<HashMap<String, Vec<Option<f64>>>, String> {
        let script = parse(source)?;
        let result = execute_script_with_vm(&script, series).map_err(|e| format!("{e:?}"))?;
        Ok(result.plot_outputs.get_plots().clone())
    }
}
