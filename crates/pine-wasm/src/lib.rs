//! WebAssembly bridge: `check_script` and `run_script_json` over [`pine_eval`].
//!
//! Build: `cargo build -p pine-wasm --target wasm32-unknown-unknown`.

#![allow(missing_docs)]

#[cfg(target_arch = "wasm32")]
mod wasm_bridge;

#[cfg(target_arch = "wasm32")]
pub use wasm_bridge::{check_script, run_script_json};
