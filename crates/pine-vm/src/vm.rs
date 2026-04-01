//! Stack-based VM execution engine for Pine Script
//!
//! This module provides a stack-based virtual machine that executes
//! bytecode compiled from Pine Script source.

use crate::compiler::BytecodeChunk;
use crate::opcode::OpCode;
use crate::VmError;
use pine_runtime::context::ExecutionContext;
use pine_runtime::na_ops;
use pine_runtime::value::Value;

/// VM execution result
pub type VmResult<T> = Result<T, VmError>;

/// VM stack for operand storage
///
/// Uses a fixed-size array for performance, with bounds checking.
pub struct VmStack {
    /// Stack storage
    data: Vec<Value>,
    /// Current stack pointer (next free position)
    sp: usize,
}

impl VmStack {
    /// Create a new stack with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            sp: 0,
        }
    }

    /// Push a value onto the stack
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.data.push(value);
        self.sp += 1;
    }

    /// Pop a value from the stack
    #[inline]
    pub fn pop(&mut self) -> Option<Value> {
        self.sp = self.sp.saturating_sub(1);
        self.data.pop()
    }

    /// Peek at the top value without removing it
    #[inline]
    pub fn peek(&self) -> Option<&Value> {
        self.data.last()
    }

    /// Peek at the top value mutably
    #[inline]
    pub fn peek_mut(&mut self) -> Option<&mut Value> {
        self.data.last_mut()
    }

    /// Get the value at a specific index from the top (0 = top)
    #[inline]
    pub fn peek_at(&self, offset: usize) -> Option<&Value> {
        let idx = self.sp.saturating_sub(1 + offset);
        self.data.get(idx)
    }

    /// Get the current stack size
    #[inline]
    pub fn len(&self) -> usize {
        self.sp
    }

    /// Check if the stack is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sp == 0
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.data.clear();
        self.sp = 0;
    }

    /// Duplicate the top value
    pub fn dup(&mut self) {
        if let Some(top) = self.peek().cloned() {
            self.push(top);
        }
    }

    /// Swap the top two values
    pub fn swap(&mut self) {
        if self.sp >= 2 {
            let len = self.data.len();
            self.data.swap(len - 1, len - 2);
        }
    }
}

/// VM execution frame
///
/// Represents a function call frame with its own local variable storage.
pub struct CallFrame {
    /// Program counter (instruction pointer)
    pub pc: usize,
    /// Stack base pointer (where this frame's locals start)
    pub bp: usize,
    /// Return address (for when this frame returns)
    pub return_pc: usize,
}

impl CallFrame {
    /// Create a new call frame
    pub fn new(pc: usize, bp: usize, return_pc: usize) -> Self {
        Self { pc, bp, return_pc }
    }
}

/// Stack-based VM for Pine Script execution
pub struct VM {
    /// Operand stack
    stack: VmStack,
    /// Call stack (for function calls)
    call_stack: Vec<CallFrame>,
    /// Current execution context
    context: ExecutionContext,
    /// Current bytecode chunk
    chunk: Option<BytecodeChunk>,
    /// Program counter
    pc: usize,
}

impl VM {
    /// Create a new VM with default configuration
    pub fn new() -> Self {
        Self::with_context(ExecutionContext::default_with_config())
    }

    /// Create a new VM with the given execution context
    pub fn with_context(context: ExecutionContext) -> Self {
        Self {
            stack: VmStack::with_capacity(256),
            call_stack: Vec::with_capacity(64),
            context,
            chunk: None,
            pc: 0,
        }
    }

    /// Load a bytecode chunk for execution
    pub fn load_chunk(&mut self, chunk: BytecodeChunk) {
        self.chunk = Some(chunk);
        self.pc = 0;
    }

    /// Execute the loaded bytecode
    ///
    /// Returns the value left on top of the stack (if any).
    pub fn execute(&mut self) -> VmResult<Option<Value>> {
        // Main execution loop
        loop {
            // Check if we have a chunk and a valid PC
            let chunk = match self.chunk.as_ref() {
                Some(c) => c,
                None => return Err(VmError::NotImplemented),
            };

            if self.pc >= chunk.instructions.len() {
                break;
            }

            let instruction = &chunk.instructions[self.pc];
            let opcode = instruction.opcode;

            match opcode {
                OpCode::Halt => break,
                OpCode::PushConst => {
                    let idx = instruction.operands.first().copied().unwrap_or(0);
                    if let Some(value) = chunk.constants.get(idx) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(VmError::NotImplemented);
                    }
                    self.pc += 1;
                }
                OpCode::LoadSlot => {
                    let slot = instruction.operands.first().copied().unwrap_or(0);
                    if let Some(value) = self.context.get_slot(slot) {
                        self.stack.push(value.clone());
                    } else {
                        self.stack.push(Value::Na);
                    }
                    self.pc += 1;
                }
                OpCode::StoreSlot => {
                    let slot = instruction.operands.first().copied().unwrap_or(0);
                    if let Some(value) = self.stack.pop() {
                        self.context.set_slot(slot, value);
                    }
                    self.pc += 1;
                }
                OpCode::Pop => self.op_pop(),
                OpCode::Dup => self.op_dup(),
                OpCode::Swap => self.op_swap(),
                OpCode::Add => self.op_add()?,
                OpCode::Sub => self.op_sub()?,
                OpCode::Mul => self.op_mul()?,
                OpCode::Div => self.op_div()?,
                OpCode::Mod => self.op_mod()?,
                OpCode::Neg => self.op_neg()?,
                OpCode::Eq => self.op_eq()?,
                OpCode::Ne => self.op_ne()?,
                OpCode::Lt => self.op_lt()?,
                OpCode::Le => self.op_le()?,
                OpCode::Gt => self.op_gt()?,
                OpCode::Ge => self.op_ge()?,
                OpCode::And => self.op_and()?,
                OpCode::Or => self.op_or()?,
                OpCode::Not => self.op_not()?,
                OpCode::IsNa => self.op_is_na()?,
                OpCode::Coalesce => self.op_coalesce()?,
                OpCode::Jump => {
                    let offset = instruction.operands.first().copied().unwrap_or(0);
                    self.pc += offset;
                }
                OpCode::JumpIfFalse => {
                    let offset = instruction.operands.first().copied().unwrap_or(0);
                    let condition = self.stack.peek().unwrap_or(&Value::Na);
                    if !condition.is_truthy() {
                        self.pc += offset;
                    } else {
                        self.pc += 1;
                    }
                }
                OpCode::JumpIfTrue => {
                    let offset = instruction.operands.first().copied().unwrap_or(0);
                    let condition = self.stack.peek().unwrap_or(&Value::Na);
                    if condition.is_truthy() {
                        self.pc += offset;
                    } else {
                        self.pc += 1;
                    }
                }
                OpCode::Return => {
                    // Pop return value and restore frame
                    let result = self.stack.pop();
                    if let Some(frame) = self.call_stack.pop() {
                        self.pc = frame.return_pc;
                    } else {
                        // Top-level return - exit execution
                        return Ok(result);
                    }
                }
                _ => {
                    // Other opcodes not yet implemented
                    return Err(VmError::NotImplemented);
                }
            }
        }

        // Return the value on top of the stack (if any)
        Ok(self.stack.pop())
    }

    /// Get a reference to the execution context
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Get a mutable reference to the execution context
    pub fn context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.context
    }

    /// Get a reference to the stack (for inspection)
    pub fn stack(&self) -> &VmStack {
        &self.stack
    }

    //========================================================================
    // Opcode implementations
    //========================================================================

    fn op_pop(&mut self) {
        self.stack.pop();
        self.pc += 1;
    }

    fn op_dup(&mut self) {
        self.stack.dup();
        self.pc += 1;
    }

    fn op_swap(&mut self) {
        self.stack.swap();
        self.pc += 1;
    }

    fn op_add(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::add(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_sub(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::sub(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_mul(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::mul(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_div(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::div(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_mod(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        // Use remainder for now (can be refined later)
        self.stack
            .push(if let (Some(a), Some(b)) = (a.as_float(), b.as_float()) {
                if b == 0.0 {
                    Value::Na
                } else {
                    Value::Float(a % b)
                }
            } else {
                Value::Na
            });
        self.pc += 1;
        Ok(())
    }

    fn op_neg(&mut self) -> VmResult<()> {
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::neg(&a));
        self.pc += 1;
        Ok(())
    }

    fn op_eq(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::eq(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_ne(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::ne(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_lt(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::lt(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_le(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::le(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_gt(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::gt(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_ge(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::ge(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_and(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::and(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_or(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::or(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_not(&mut self) -> VmResult<()> {
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::not(&a));
        self.pc += 1;
        Ok(())
    }

    fn op_is_na(&mut self) -> VmResult<()> {
        let a = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(Value::Bool(matches!(a, Value::Na)));
        self.pc += 1;
        Ok(())
    }

    fn op_coalesce(&mut self) -> VmResult<()> {
        let default = self.stack.pop().unwrap_or(Value::Na);
        let value = self.stack.pop().unwrap_or(Value::Na);
        self.stack.push(na_ops::coalesce(&value, &default));
        self.pc += 1;
        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a bytecode chunk and return the result
pub fn execute_chunk(chunk: BytecodeChunk) -> VmResult<Option<Value>> {
    let mut vm = VM::new();
    vm.load_chunk(chunk);
    vm.execute()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{BinaryOp, Compiler};

    #[test]
    fn test_vm_execute_const() {
        let mut compiler = Compiler::new();
        compiler.compile_const(Value::Int(42));
        compiler.compile_halt();

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();

        assert_eq!(result, Some(Value::Int(42)));
    }

    #[test]
    fn test_vm_execute_add() {
        let mut compiler = Compiler::new();
        compiler.compile_const(Value::Int(10));
        compiler.compile_const(Value::Int(32));
        compiler.compile_binary(BinaryOp::Add);
        compiler.compile_halt();

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();

        // na_ops::add returns Int for Int + Int
        assert_eq!(result, Some(Value::Int(42)));
    }

    #[test]
    fn test_vm_stack_operations() {
        let mut stack = VmStack::with_capacity(10);

        stack.push(Value::Int(1));
        stack.push(Value::Int(2));
        assert_eq!(stack.len(), 2);

        stack.dup();
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.peek(), Some(&Value::Int(2)));

        stack.swap();
        assert_eq!(stack.pop(), Some(Value::Int(2)));
        assert_eq!(stack.pop(), Some(Value::Int(2)));
        assert_eq!(stack.pop(), Some(Value::Int(1)));
    }

    #[test]
    fn test_vm_comparison() {
        let mut compiler = Compiler::new();
        compiler.compile_const(Value::Int(5));
        compiler.compile_const(Value::Int(10));
        compiler.compile_binary(BinaryOp::Lt);
        compiler.compile_halt();

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();

        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_vm_logical() {
        let mut compiler = Compiler::new();
        compiler.compile_const(Value::Bool(true));
        compiler.compile_const(Value::Bool(false));
        compiler.compile_binary(BinaryOp::And);
        compiler.compile_halt();

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();

        assert_eq!(result, Some(Value::Bool(false)));
    }
}
