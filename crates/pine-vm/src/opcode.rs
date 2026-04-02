//! Bytecode opcodes for Pine Script VM

/// VM opcodes for stack-based execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    /// Push a constant value (operand: constant index)
    PushConst = 0,
    /// Push a variable from slot (operand: slot index)
    PushSlot,
    /// Pop value from stack
    Pop,
    /// Duplicate top of stack
    Dup,
    /// Swap top two stack values
    Swap,

    // Series operations
    /// Push current series value (operand: series index)
    PushSeries,
    /// Push historical series value (operands: series index, offset)
    PushSeriesAt,
    /// Push historical series value with dynamic offset (operand: series index)
    /// The offset is popped from the stack
    PushSeriesAtDynamic,
    /// Push value to series
    SeriesPush,

    // Arithmetic operations
    /// Add two values
    Add,
    /// Subtract two values
    Sub,
    /// Multiply two values
    Mul,
    /// Divide two values
    Div,
    /// Modulo operation
    Mod,
    /// Negate value
    Neg,

    // Comparison operations
    /// Equal
    Eq,
    /// Not equal
    Ne,
    /// Less than
    Lt,
    /// Less than or equal
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal
    Ge,

    // Logical operations
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical NOT
    Not,

    // Control flow
    /// Jump unconditionally (operand: jump offset)
    Jump,
    /// Jump if false (operand: jump offset)
    JumpIfFalse,
    /// Jump if true (operand: jump offset)
    JumpIfTrue,
    /// Call a function (operands: function index, arg count)
    Call,
    /// Return from function
    Return,
    /// Halt execution
    Halt,

    // Variable operations
    /// Store to slot (operand: slot index)
    StoreSlot,
    /// Load from slot (operand: slot index)
    LoadSlot,

    // NA handling
    /// Check if value is NA
    IsNa,
    /// Coalesce: if top is NA, replace with next
    Coalesce,

    /// Update user-defined series in context (operand: series name index)
    /// Pops value from stack and pushes it to the named series
    UpdateUserSeries,
}

impl OpCode {
    /// Get the opcode from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        use OpCode::*;
        match value {
            0 => Some(PushConst),
            1 => Some(PushSlot),
            2 => Some(Pop),
            3 => Some(Dup),
            4 => Some(Swap),
            5 => Some(PushSeries),
            6 => Some(PushSeriesAt),
            7 => Some(SeriesPush),
            8 => Some(Add),
            9 => Some(Sub),
            10 => Some(Mul),
            11 => Some(Div),
            12 => Some(Mod),
            13 => Some(Neg),
            14 => Some(Eq),
            15 => Some(Ne),
            16 => Some(Lt),
            17 => Some(Le),
            18 => Some(Gt),
            19 => Some(Ge),
            20 => Some(And),
            21 => Some(Or),
            22 => Some(Not),
            23 => Some(Jump),
            24 => Some(JumpIfFalse),
            25 => Some(JumpIfTrue),
            26 => Some(Call),
            27 => Some(Return),
            28 => Some(Halt),
            29 => Some(StoreSlot),
            30 => Some(LoadSlot),
            31 => Some(IsNa),
            32 => Some(Coalesce),
            33 => Some(PushSeriesAtDynamic),
            34 => Some(UpdateUserSeries),
            _ => None,
        }
    }

    /// Get the number of operands for this opcode
    pub fn operand_count(&self) -> usize {
        use OpCode::*;
        match self {
            PushConst | PushSlot | Jump | JumpIfFalse | JumpIfTrue | StoreSlot | LoadSlot
            | PushSeriesAtDynamic => 1,
            PushSeriesAt | Call => 2,
            UpdateUserSeries => 1,
            _ => 0,
        }
    }
}
