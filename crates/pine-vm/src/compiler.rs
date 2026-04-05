//! Bytecode compiler for Pine Script VM
//!
//! This module compiles typed HIR (High-Level Intermediate Representation)
//! to bytecode for efficient VM execution.

use crate::opcode::OpCode;
use crate::VmError;
use pine_runtime::value::Value;
use std::collections::HashSet;

/// Sentinel offset used to encode external function calls in bytecode.
pub const EXTERNAL_FUNCTION_BASE: usize = usize::MAX / 2;

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
#[derive(Debug, Default, Clone)]
pub struct BytecodeChunk {
    /// Instructions
    pub instructions: Vec<Instruction>,
    /// Constant pool
    pub constants: Vec<Value>,
    /// Series name table (index -> series name)
    pub series_names: Vec<String>,
    /// Function address table (index -> instruction address)
    ///
    /// The Call opcode uses function indices into this table.
    pub function_addresses: Vec<usize>,
    /// Function name table (function name -> function index)
    pub function_names: std::collections::HashMap<String, usize>,
    /// External function table (function name -> external index)
    /// External function indices start after user function indices
    pub external_functions: Vec<String>,
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

    /// Add a string constant to the pool and return its index
    pub fn add_string_constant(&mut self, value: String) -> usize {
        let index = self.constants.len();
        self.constants.push(Value::String(value.into()));
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
    ///
    /// Stores the absolute target address for the jump instruction.
    pub fn patch_jump(&mut self, pos: usize, target: usize) {
        if pos < self.instructions.len() {
            // Store the absolute target address
            if let Some(inst) = self.instructions.get_mut(pos) {
                if !inst.operands.is_empty() {
                    inst.operands[0] = target;
                }
            }
        }
    }
}

/// Variable scope for local variable management
#[derive(Debug)]
pub struct Scope {
    /// Variable name -> slot index mapping
    variables: std::collections::HashMap<String, usize>,
    /// Next available slot index
    next_slot: usize,
}

impl Scope {
    fn new() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
            next_slot: 0,
        }
    }

    fn with_parent(parent: &Scope) -> Self {
        Self {
            variables: std::collections::HashMap::new(),
            next_slot: parent.next_slot,
        }
    }

    fn declare_var(&mut self, name: impl Into<String>) -> usize {
        let name = name.into();
        let slot = self.next_slot;
        self.variables.insert(name, slot);
        self.next_slot += 1;
        slot
    }

    fn get_var(&self, name: &str) -> Option<usize> {
        self.variables.get(name).copied()
    }
}

/// Tracks a loop's compilation context for break/continue support.
#[derive(Debug, Clone)]
pub struct LoopContext {
    /// Instruction address of the loop start (for continue)
    pub loop_start: usize,
    /// Positions of Jump instructions emitted for `break` that need patching
    pub break_patches: Vec<usize>,
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
    /// Variable scope stack
    scopes: Vec<Scope>,
    /// User-defined variables that must also be maintained as runtime series
    series_vars: HashSet<String>,
    /// Counter for synthetic series names used by compound series expressions
    synthetic_series_counter: usize,
    /// Stack of active loop contexts for break/continue
    loop_stack: Vec<LoopContext>,
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
            scopes: vec![Scope::new()],
            series_vars: HashSet::new(),
            synthetic_series_counter: 0,
            loop_stack: Vec::new(),
        }
    }

    //========================================================================
    // Variable Scope Management
    //========================================================================

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        let parent = self.scopes.last().expect("No scope available");
        self.scopes.push(Scope::with_parent(parent));
    }

    /// Enter a function-local scope whose slots are relative to the call frame.
    pub fn enter_function_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            let exited = self.scopes.pop().expect("No scope available");
            if let Some(parent) = self.scopes.last_mut() {
                parent.next_slot = parent.next_slot.max(exited.next_slot);
            }
        }
    }

    /// Declare a variable in the current scope
    pub fn declare_var(&mut self, name: impl Into<String>) -> usize {
        let current_scope = self.scopes.last_mut().expect("No scope available");
        current_scope.declare_var(name)
    }

    /// Lookup a variable by name
    ///
    /// Returns the slot index if found in any scope.
    pub fn lookup_var(&self, name: &str) -> Option<usize> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.get_var(name) {
                return Some(slot);
            }
        }
        None
    }

    /// Compile a variable declaration
    ///
    /// Declares the variable in the current scope and stores the value from stack.
    pub fn compile_var_decl(&mut self, name: impl Into<String>) -> usize {
        let slot = self.declare_var(name);
        self.compile_store_slot(slot);
        slot
    }

    /// Compile a variable load by name
    ///
    /// Looks up the variable and emits a LoadSlot instruction.
    pub fn compile_load_var(&mut self, name: &str) -> bool {
        if let Some(slot) = self.lookup_var(name) {
            self.compile_load_slot(slot);
            true
        } else {
            false
        }
    }

    /// Compile a variable store by name
    ///
    /// Looks up the variable and emits a StoreSlot instruction.
    pub fn compile_store_var(&mut self, name: &str) -> bool {
        if let Some(slot) = self.lookup_var(name) {
            self.compile_store_slot(slot);
            true
        } else {
            false
        }
    }

    /// Compile a constant value
    pub fn compile_const(&mut self, value: Value) {
        let idx = self.chunk.add_constant(value);
        self.chunk
            .emit_op1(OpCode::PushConst, idx, self.current_line);
    }

    /// Compile a variable load from slot
    pub fn compile_load_slot(&mut self, slot: usize) {
        self.chunk
            .emit_op1(OpCode::LoadSlot, slot, self.current_line);
    }

    /// Compile a variable store to slot
    pub fn compile_store_slot(&mut self, slot: usize) {
        self.chunk
            .emit_op1(OpCode::StoreSlot, slot, self.current_line);
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
        self.chunk
            .emit_op2(OpCode::Call, func_idx, arg_count, self.current_line);
    }

    /// Register a series name and return its index
    pub fn register_series(&mut self, name: impl Into<String>) -> usize {
        let name = name.into();
        // Check if already registered
        if let Some(idx) = self.chunk.series_names.iter().position(|n| n == &name) {
            return idx;
        }
        let idx = self.chunk.series_names.len();
        self.chunk.series_names.push(name);
        idx
    }

    /// Compile a push series operation (push current value to stack)
    pub fn compile_push_series(&mut self, series_idx: usize) {
        self.chunk
            .emit_op1(OpCode::PushSeries, series_idx, self.current_line);
    }

    /// Compile a push series at offset operation (push historical value to stack)
    pub fn compile_push_series_at(&mut self, series_idx: usize, offset: usize) {
        self.chunk
            .emit_op2(OpCode::PushSeriesAt, series_idx, offset, self.current_line);
    }

    /// Lookup a series index by name
    pub fn lookup_series(&self, name: &str) -> Option<usize> {
        self.chunk.series_names.iter().position(|n| n == name)
    }

    /// Compile a dynamic series access operation
    ///
    /// The offset is popped from the stack, then the series value at that offset
    /// is pushed to the stack.
    pub fn compile_push_series_dynamic(&mut self, series_idx: usize) {
        self.chunk
            .emit_op1(OpCode::PushSeriesAtDynamic, series_idx, self.current_line);
    }

    /// Compile a series push operation (pop value from stack and push to series)
    pub fn compile_series_push(&mut self, series_idx: usize) {
        self.chunk
            .emit_op1(OpCode::SeriesPush, series_idx, self.current_line);
    }

    /// Compile an update to a user-defined series in context
    /// This allows the value to be accessed by ta.* functions that need series history
    pub fn compile_update_user_series(&mut self, name: &str) {
        let name_idx = self.chunk.add_string_constant(name.to_string());
        self.chunk
            .emit_op1(OpCode::UpdateUserSeries, name_idx, self.current_line);
    }

    /// Mark a variable name as a runtime series and ensure it has a series slot.
    pub fn mark_series_var(&mut self, name: &str) {
        self.series_vars.insert(name.to_string());
        self.register_series(name.to_string());
    }

    /// Check whether a name should be treated as a runtime series.
    pub fn is_series_var(&self, name: &str) -> bool {
        self.series_vars.contains(name)
    }

    /// Allocate a synthetic series name for compound expressions.
    pub fn next_synthetic_series_name(&mut self) -> String {
        let name = format!("__expr_series_{}", self.synthetic_series_counter);
        self.synthetic_series_counter += 1;
        self.mark_series_var(&name);
        name
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

    /// Patch a jump to point to a specific position
    pub fn patch_jump_to(&mut self, jump_pos: usize, target: usize) {
        self.chunk.patch_jump(jump_pos, target);
    }

    //========================================================================
    // Control Flow
    //========================================================================

    /// Compile an if statement
    ///
    /// Generates: condition, JumpIfFalse(else), then_body, Jump(end), else_body
    ///
    /// # Arguments
    /// * `compile_condition` - closure to compile the condition expression
    /// * `compile_then` - closure to compile the then body
    /// * `compile_else` - optional closure to compile the else body
    pub fn compile_if(
        &mut self,
        compile_condition: impl FnOnce(&mut Self),
        compile_then: impl FnOnce(&mut Self),
        compile_else: Option<impl FnOnce(&mut Self)>,
    ) {
        // Compile condition
        compile_condition(self);

        // Jump to else (or end) if false
        let else_jump = self.compile_jump(JumpOp::IfFalse);

        // Compile then body
        compile_then(self);

        // Jump over else body (if exists)
        let end_jump = if compile_else.is_some() {
            Some(self.compile_jump(JumpOp::Unconditional))
        } else {
            None
        };

        // Patch else jump to current position
        self.patch_jump(else_jump);

        // Compile else body if present
        if let Some(compile_else_fn) = compile_else {
            compile_else_fn(self);
            // Patch end jump
            if let Some(jump) = end_jump {
                self.patch_jump(jump);
            }
        }
    }

    /// Compile a while loop
    ///
    /// Generates: start, condition, JumpIfFalse(end), body, Jump(start), end
    ///
    /// # Arguments
    /// * `compile_condition` - closure to compile the condition expression
    /// * `compile_body` - closure to compile the loop body
    pub fn compile_while(
        &mut self,
        compile_condition: impl FnOnce(&mut Self),
        compile_body: impl FnOnce(&mut Self),
    ) {
        // Start of loop
        let start_pos = self.chunk.current_pos();
        self.push_loop(start_pos);

        // Compile condition
        compile_condition(self);

        // Jump to end if false
        let end_jump = self.compile_jump(JumpOp::IfFalse);

        // Compile body
        compile_body(self);

        // Jump back to start
        let loop_jump = self.compile_jump(JumpOp::Unconditional);
        self.patch_jump_to(loop_jump, start_pos);

        // Patch end jump
        self.patch_jump(end_jump);

        self.pop_loop();
    }

    /// Compile a for loop (inclusive range: for i = start to end [by step])
    ///
    /// Generates:
    /// - Initialize loop variable with start value
    /// - Loop start: check condition (i <= end), jump to end if false
    /// - Body
    /// - Increment (i = i + step)
    /// - Jump to loop start
    ///
    /// # Arguments
    /// * `loop_var_slot` - slot index for loop variable
    /// * `compile_start` - closure to compile start expression
    /// * `compile_end` - closure to compile end expression
    /// * `compile_step` - optional closure to compile step expression (default: 1)
    /// * `compile_body` - closure to compile the loop body
    pub fn compile_for(
        &mut self,
        loop_var_slot: usize,
        compile_start: impl FnOnce(&mut Self),
        compile_end: impl FnOnce(&mut Self),
        compile_step: Option<impl FnOnce(&mut Self)>,
        compile_body: impl FnOnce(&mut Self),
    ) {
        // Initialize loop variable with start value
        compile_start(self);
        self.compile_store_slot(loop_var_slot);
        // Note: StoreSlot consumes the value from stack, no need to pop

        // Loop start position
        let start_pos = self.chunk.current_pos();
        self.push_loop(start_pos);

        // Condition: loop_var <= end
        self.compile_load_slot(loop_var_slot);
        compile_end(self);
        self.compile_binary(BinaryOp::Le);

        // Jump to end if condition is false
        let end_jump = self.compile_jump(JumpOp::IfFalse);

        // Compile body
        compile_body(self);

        // Increment loop variable
        self.compile_load_slot(loop_var_slot);
        if let Some(compile_step_fn) = compile_step {
            compile_step_fn(self);
        } else {
            self.compile_const(Value::Int(1));
        }
        self.compile_binary(BinaryOp::Add);
        self.compile_store_slot(loop_var_slot);
        // Note: StoreSlot consumes the value from stack, no need to pop

        // Jump back to start
        let loop_jump = self.compile_jump(JumpOp::Unconditional);
        self.patch_jump_to(loop_jump, start_pos);

        // Patch end jump
        self.patch_jump(end_jump);

        self.pop_loop();
    }

    /// Push a new loop context onto the stack. Call at the start of every loop.
    pub fn push_loop(&mut self, loop_start: usize) {
        self.loop_stack.push(LoopContext {
            loop_start,
            break_patches: Vec::new(),
        });
    }

    /// Pop the current loop context and patch all break jumps to `end_pos`.
    pub fn pop_loop(&mut self) {
        if let Some(ctx) = self.loop_stack.pop() {
            let end = self.chunk.current_pos();
            for pos in ctx.break_patches {
                self.chunk.patch_jump(pos, end);
            }
        }
    }

    /// Emit a break: unconditional jump whose target will be patched later.
    pub fn compile_break(&mut self) {
        let pos = self.compile_jump(JumpOp::Unconditional);
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.break_patches.push(pos);
        }
    }

    /// Emit a continue: jump back to the current loop's start address.
    pub fn compile_continue(&mut self) {
        if let Some(ctx) = self.loop_stack.last() {
            let start = ctx.loop_start;
            let pos = self.compile_jump(JumpOp::Unconditional);
            self.patch_jump_to(pos, start);
        }
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

    /// Compile an arbitrary opcode
    pub fn compile_op(&mut self, opcode: OpCode) {
        self.chunk.emit_op(opcode, self.current_line);
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

    //========================================================================
    // Function Management
    //========================================================================

    /// Register a function and return its index
    ///
    /// The function address is set to the current instruction position.
    /// Call this right before emitting function body instructions.
    pub fn register_function(&mut self) -> usize {
        let address = self.chunk.current_pos();
        let index = self.chunk.function_addresses.len();
        self.chunk.function_addresses.push(address);
        index
    }

    /// Get the address of a function by index
    pub fn get_function_address(&self, func_idx: usize) -> Option<usize> {
        self.chunk.function_addresses.get(func_idx).copied()
    }

    /// Reserve a function slot and return its index
    ///
    /// Use this for forward declarations. The address is initially 0
    /// and should be patched later using `patch_function_address`.
    pub fn reserve_function_slot(&mut self) -> usize {
        let index = self.chunk.function_addresses.len();
        self.chunk.function_addresses.push(0);
        index
    }

    /// Patch a function address after the function body is compiled
    pub fn patch_function_address(&mut self, func_idx: usize, address: usize) {
        if func_idx < self.chunk.function_addresses.len() {
            self.chunk.function_addresses[func_idx] = address;
        }
    }

    /// Register a function name to index mapping
    pub fn register_function_name(&mut self, name: impl Into<String>, index: usize) {
        self.chunk.function_names.insert(name.into(), index);
    }

    /// Look up a function index by name
    pub fn lookup_function(&self, name: &str) -> Option<usize> {
        self.chunk.function_names.get(name).copied()
    }

    /// Register an external function and return its index
    /// External function indices live in a dedicated range so later function
    /// declarations do not change already-emitted call targets.
    pub fn register_external_function(&mut self, name: impl Into<String>) -> usize {
        let name = name.into();
        // Check if already registered
        if let Some(idx) = self
            .chunk
            .external_functions
            .iter()
            .position(|n| n == &name)
        {
            return EXTERNAL_FUNCTION_BASE + idx;
        }
        let idx = self.chunk.external_functions.len();
        self.chunk.external_functions.push(name);
        EXTERNAL_FUNCTION_BASE + idx
    }

    /// Get the list of external functions (for VM registration)
    pub fn external_functions(&self) -> &[String] {
        &self.chunk.external_functions
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
