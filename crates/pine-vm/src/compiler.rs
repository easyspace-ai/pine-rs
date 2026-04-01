//! Bytecode compiler for Pine Script VM
//!
//! This module compiles typed HIR (High-Level Intermediate Representation)
//! to bytecode for efficient VM execution.

use crate::opcode::OpCode;
use crate::VmError;
use pine_runtime::value::Value;

/// A bytecode instruction
#[derive(Debug, Clone)]
pub struct Instruction {
    /// The opcode
    pub opcode: OpCode,
    /// Optional operands
    pub operands: Vec<usize>,
}

impl Instruction {
    /// Create a new instruction without operands
    pub fn new(opcode: OpCode) -> Self {
        Self {
            opcode,
            operands: Vec::new(),
        }
    }

    /// Create a new instruction with one operand
    pub fn with_operand(opcode: OpCode, operand: usize) -> Self {
        Self {
            opcode,
            operands: vec![operand],
        }
    }

    /// Create a new instruction with two operands
    pub fn with_operands(opcode: OpCode, op1: usize, op2: usize) -> Self {
        Self {
            opcode,
            operands: vec![op1, op2],
        }
    }
}

/// Compiled bytecode chunk
#[derive(Debug, Default)]
pub struct BytecodeChunk {
    /// Instructions
    pub instructions: Vec<Instruction>,
    /// Constant pool
    pub constants: Vec<Value>,
    /// Line numbers for debugging (instruction index -> line)
    pub line_numbers: Vec<usize>,
}

impl BytecodeChunk {
    /// Create a new empty bytecode chunk
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, value: Value) -> usize {
        let index = self.constants.len();
        self.constants.push(value);
        index
    }

    /// Emit an instruction
    pub fn emit(&mut self, instruction: Instruction, line: usize) {
        self.instructions.push(instruction);
        self.line_numbers.push(line);
    }

    /// Emit a simple opcode without operands
    pub fn emit_op(&mut self, opcode: OpCode, line: usize) {
        self.emit(Instruction::new(opcode), line);
    }

    /// Emit an opcode with one operand
    pub fn emit_op1(&mut self, opcode: OpCode, operand: usize, line: usize) {
        self.emit(Instruction::with_operand(opcode, operand), line);
    }

    /// Emit an opcode with two operands
    pub fn emit_op2(&mut self, opcode: OpCode, op1: usize, op2: usize, line: usize) {
        self.emit(Instruction::with_operands(opcode, op1, op2), line);
    }

    /// Get the current instruction position (for backpatching jumps)
    pub fn current_pos(&self) -> usize {
        self.instructions.len()
    }

    /// Patch a jump target at the given position
    pub fn patch_jump(&mut self, pos: usize, target: usize) {
        if pos < self.instructions.len() {
            // Store the jump offset (target - pos)
            let offset = target.saturating_sub(pos);
            if let Some(inst) = self.instructions.get_mut(pos) {
                if !inst.operands.is_empty() {
                    inst.operands[0] = offset;
                }
            }
        }
    }
}

/// Simple expression compiler
///
/// This is a basic compiler that handles arithmetic expressions.
/// It will be expanded to handle full Pine Script semantics.
pub struct Compiler {
    /// The bytecode chunk being built
    chunk: BytecodeChunk,
    /// Current line number for debugging
    current_line: usize,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            chunk: BytecodeChunk::new(),
            current_line: 1,
        }
    }

    /// Compile a constant value
    pub fn compile_const(&mut self, value: Value) {
        let idx = self.chunk.add_constant(value);
        self.chunk.emit_op1(OpCode::PushConst, idx, self.current_line);
    }

    /// Compile a variable load from slot
    pub fn compile_load_slot(&mut self, slot: usize) {
        self.chunk.emit_op1(OpCode::LoadSlot, slot, self.current_line);
    }

    /// Compile a variable store to slot
    pub fn compile_store_slot(&mut self, slot: usize) {
        self.chunk.emit_op1(OpCode::StoreSlot, slot, self.current_line);
    }

    /// Compile binary operation
    pub fn compile_binary(&mut self, op: BinaryOp) {
        let opcode = match op {
            BinaryOp::Add => OpCode::Add,
            BinaryOp::Sub => OpCode::Sub,
            BinaryOp::Mul => OpCode::Mul,
            BinaryOp::Div => OpCode::Div,
            BinaryOp::Mod => OpCode::Mod,
            BinaryOp::Eq => OpCode::Eq,
            BinaryOp::Ne => OpCode::Ne,
            BinaryOp::Lt => OpCode::Lt,
            BinaryOp::Le => OpCode::Le,
            BinaryOp::Gt => OpCode::Gt,
            BinaryOp::Ge => OpCode::Ge,
            BinaryOp::And => OpCode::And,
            BinaryOp::Or => OpCode::Or,
        };
        self.chunk.emit_op(opcode, self.current_line);
    }

    /// Compile unary operation
    pub fn compile_unary(&mut self, op: UnaryOp) {
        let opcode = match op {
            UnaryOp::Neg => OpCode::Neg,
            UnaryOp::Not => OpCode::Not,
        };
        self.chunk.emit_op(opcode, self.current_line);
    }

    /// Compile a function call
    pub fn compile_call(&mut self, func_idx: usize, arg_count: usize) {
        self.chunk.emit_op2(OpCode::Call, func_idx, arg_count, self.current_line);
    }

    /// Compile a jump (placeholder, to be patched later)
    ///
    /// Returns the position of the jump instruction for backpatching.
    pub fn compile_jump(&mut self, jump_op: JumpOp) -> usize {
        let opcode = match jump_op {
            JumpOp::Unconditional => OpCode::Jump,
            JumpOp::IfFalse => OpCode::JumpIfFalse,
            JumpOp::IfTrue => OpCode::JumpIfTrue,
        };
        let pos = self.chunk.current_pos();
        self.chunk.emit_op1(opcode, 0, self.current_line); // 0 = placeholder
        pos
    }

    /// Patch a jump to point to the current position
    pub fn patch_jump(&mut self, jump_pos: usize) {
        let target = self.chunk.current_pos();
        self.chunk.patch_jump(jump_pos, target);
    }

    /// Compile a pop operation
    pub fn compile_pop(&mut self) {
        self.chunk.emit_op(OpCode::Pop, self.current_line);
    }

    /// Compile a dup operation
    pub fn compile_dup(&mut self) {
        self.chunk.emit_op(OpCode::Dup, self.current_line);
    }

    /// Compile halt instruction
    pub fn compile_halt(&mut self) {
        self.chunk.emit_op(OpCode::Halt, self.current_line);
    }

    /// Set the current line number for debugging
    pub fn set_line(&mut self, line: usize) {
        self.current_line = line;
    }

    /// Take the compiled chunk
    pub fn finish(self) -> BytecodeChunk {
        self.chunk
    }

    /// Get a reference to the chunk (for inspection during compilation)
    pub fn chunk(&self) -> &BytecodeChunk {
        &self.chunk
    }
}

/// Binary operations supported by the VM
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Sub,
    /// Multiplication (*)
    Mul,
    /// Division (/)
    Div,
    /// Modulo (%)
    Mod,
    /// Equal (==)
    Eq,
    /// Not equal (!=)
    Ne,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Le,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Ge,
    /// Logical AND
    And,
    /// Logical OR
    Or,
}

/// Unary operations supported by the VM
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    /// Negation (-)
    Neg,
    /// Logical NOT
    Not,
}

/// Jump operation types
#[derive(Debug, Clone, Copy)]
pub enum JumpOp {
    /// Unconditional jump
    Unconditional,
    /// Jump if condition is false
    IfFalse,
    /// Jump if condition is true
    IfTrue,
}

/// Compile typed HIR to bytecode
///
/// This is the main entry point for compilation.
/// Currently returns an empty chunk as HIR integration is pending.
pub fn compile() -> Result<BytecodeChunk, VmError> {
    // TODO: Implement full HIR to bytecode compilation
    // For now, return an empty chunk with just a halt instruction
    let mut compiler = Compiler::new();
    compiler.compile_halt();
    Ok(compiler.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_const() {
        let mut compiler = Compiler::new();
        compiler.compile_const(Value::Int(42));
        compiler.compile_halt();

        let chunk = compiler.finish();
        assert_eq!(chunk.instructions.len(), 2);
        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Int(42));
    }

    #[test]
    fn test_compile_binary() {
        let mut compiler = Compiler::new();
        compiler.compile_const(Value::Int(1));
        compiler.compile_const(Value::Int(2));
        compiler.compile_binary(BinaryOp::Add);
        compiler.compile_halt();

        let chunk = compiler.finish();
        assert_eq!(chunk.instructions.len(), 4);
        assert!(matches!(chunk.instructions[2].opcode, OpCode::Add));
    }

    #[test]
    fn test_compile_jump() {
        let mut compiler = Compiler::new();
        compiler.compile_const(Value::Bool(true));
        let jump_pos = compiler.compile_jump(JumpOp::IfFalse);
        compiler.compile_const(Value::Int(1));
        compiler.patch_jump(jump_pos);
        compiler.compile_const(Value::Int(2));
        compiler.compile_halt();

        let chunk = compiler.finish();
        assert_eq!(chunk.instructions.len(), 5);
        // The jump should be patched to skip the first const
        assert!(chunk.instructions[1].operands[0] > 0);
    }
}
