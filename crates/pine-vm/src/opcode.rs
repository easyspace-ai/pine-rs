//! Bytecode opcodes

/// VM opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    /// Push a constant value
    PushConst = 0,
    /// Push a series value
    PushSeries,
    /// Pop value from stack
    Pop,
    /// Add two values
    Add,
    /// Subtract two values
    Sub,
    /// Multiply two values
    Mul,
    /// Divide two values
    Div,
    /// Call a function
    Call,
    /// Return from function
    Return,
    /// Jump if false
    JumpIfFalse,
    /// Jump
    Jump,
}
