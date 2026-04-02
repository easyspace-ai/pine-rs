//! Pine Script v6 Virtual Machine
//!
//! This crate provides a bytecode VM for efficient Pine Script execution,
//! including a compiler from typed HIR to bytecode and a stack-based VM.

#![warn(missing_docs)]

pub mod ast_compiler;
pub mod compiler;
pub(crate) mod debug;
pub mod executor;
pub mod opcode;
pub mod vm;

use miette::Diagnostic;
use thiserror::Error;

/// VM errors
#[derive(Debug, Error, Diagnostic)]
pub enum VmError {
    /// Feature not yet implemented
    #[error("VM not yet implemented: {0}")]
    NotImplemented(String),

    /// Invalid opcode
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),

    /// Stack underflow
    #[error("Stack underflow")]
    StackUnderflow,

    /// Invalid constant index
    #[error("Invalid constant index: {0}")]
    InvalidConstant(usize),

    /// Invalid series index
    #[error("Invalid series index: {0}")]
    InvalidSeries(usize),

    /// Invalid function index
    #[error("Invalid function index: {0}")]
    InvalidFunction(usize),

    /// No bytecode loaded
    #[error("No bytecode loaded")]
    NoBytecode,

    /// Compilation error
    #[error("Compilation error: {0}")]
    CompileError(String),
}

impl From<ast_compiler::CompileError> for VmError {
    fn from(err: ast_compiler::CompileError) -> Self {
        VmError::CompileError(err.to_string())
    }
}

/// Result type for VM operations
pub type Result<T> = std::result::Result<T, VmError>;

/// Compile and execute Pine Script using the VM
pub fn run() -> Result<()> {
    // TODO: Implement VM execution
    Ok(())
}

/// Execute a Pine Script AST using the VM
///
/// This is a convenience function for parity testing. It:
/// 1. Compiles the AST to bytecode
/// 2. Sets up the VM with standard library functions
/// 3. Executes the bytecode
/// 4. Returns the result
///
/// # Arguments
/// * `script` - The parsed Pine Script AST
///
/// # Returns
/// The value left on top of the stack after execution, or an error
pub fn execute_script_ast(
    script: &pine_parser::ast::Script,
) -> Result<Option<pine_runtime::value::Value>> {
    use crate::ast_compiler::compile_script;
    use crate::vm::VM;

    // Compile AST to bytecode
    let compiler = compile_script(script)?;
    let chunk = compiler.finish();

    // Create VM and register external functions
    let mut vm = VM::new();

    // Register all external functions used in the script
    for func_name in chunk.external_functions.iter() {
        vm.register_external_function(func_name);
    }

    // Load and execute
    vm.load_chunk(chunk);
    vm.execute()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert!(run().is_ok());
    }
}
