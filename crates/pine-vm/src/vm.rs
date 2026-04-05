//! Stack-based VM execution engine for Pine Script
//!
//! This module provides a stack-based virtual machine that executes
//! bytecode compiled from Pine Script source.

use crate::compiler::BytecodeChunk;
use crate::debug::vm_debug;
use crate::opcode::OpCode;
use crate::VmError;
use pine_runtime::context::ExecutionContext;
use pine_runtime::na_ops;
use pine_runtime::value::Value;
use pine_stdlib::registry::FunctionRegistry;

/// VM execution result
pub type VmResult<T> = Result<T, VmError>;

/// VM stack for operand storage
///
/// Uses a fixed-size array for performance, with bounds checking.
pub struct VmStack {
    /// Stack storage
    pub(crate) data: Vec<Value>,
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

/// Plot record for a single bar
#[derive(Debug, Clone)]
pub struct PlotRecord {
    /// Plot title
    pub title: String,
    /// Plot value
    pub value: Option<f64>,
}

/// Stack-based VM for Pine Script execution
pub struct VM {
    /// Operand stack
    stack: VmStack,
    /// Local variable slots, kept separate from the operand stack.
    slots: Vec<Value>,
    /// Call stack (for function calls)
    call_stack: Vec<CallFrame>,
    /// Current execution context
    context: ExecutionContext,
    /// Current bytecode chunk
    chunk: Option<BytecodeChunk>,
    /// Program counter
    pc: usize,
    /// Base pointer (current frame's base in stack)
    bp: usize,
    /// Function registry for external function calls
    function_registry: FunctionRegistry,
    /// External function table (index -> name mapping for fast dispatch)
    external_functions: Vec<String>,
    /// Plot outputs collected during execution
    plot_outputs: Vec<PlotRecord>,
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
            slots: Vec::with_capacity(64),
            call_stack: Vec::with_capacity(64),
            context,
            chunk: None,
            pc: 0,
            bp: 0,
            function_registry: FunctionRegistry::with_stdlib(),
            external_functions: Vec::new(),
            plot_outputs: Vec::new(),
        }
    }

    /// Register an external function with the VM
    pub fn register_external_function(&mut self, name: impl Into<String>) -> usize {
        let name = name.into();
        let idx = self.external_functions.len();
        self.external_functions.push(name);
        idx
    }

    /// Get the function registry (for inspection/testing)
    pub fn function_registry(&self) -> &FunctionRegistry {
        &self.function_registry
    }

    /// Get the plot outputs collected during execution
    pub fn plot_outputs(&self) -> &[PlotRecord] {
        &self.plot_outputs
    }

    /// Clear plot outputs
    pub fn clear_plot_outputs(&mut self) {
        self.plot_outputs.clear();
    }

    /// Load a bytecode chunk for execution
    pub fn load_chunk(&mut self, chunk: BytecodeChunk) {
        self.chunk = Some(chunk);
        self.pc = 0;
        self.slots.clear();
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
                None => return Err(VmError::NoBytecode),
            };

            if self.pc >= chunk.instructions.len() {
                break;
            }

            let instruction = &chunk.instructions[self.pc];
            let opcode = instruction.opcode;

            // Trace every instruction
            if matches!(opcode, OpCode::Call | OpCode::Not | OpCode::JumpIfFalse) {
                let stack_top = self.stack.peek().cloned().unwrap_or(Value::Na);
                vm_debug!(
                    "DEBUG VM TRACE: pc={}, opcode={:?}, operands={:?}, stack_top={:?}",
                    self.pc,
                    opcode,
                    instruction.operands,
                    stack_top
                );
            }

            match opcode {
                OpCode::Halt => break,
                OpCode::PushConst => {
                    let idx = instruction.operands.first().copied().unwrap_or(0);
                    vm_debug!(
                        "DEBUG PushConst: idx={}, constants.len()={}",
                        idx,
                        chunk.constants.len()
                    );
                    if let Some(value) = chunk.constants.get(idx) {
                        vm_debug!("DEBUG PushConst: pushing {:?}", value);
                        self.stack.push(value.clone());
                    } else {
                        return Err(VmError::InvalidConstant(idx));
                    }
                    self.pc += 1;
                }
                OpCode::LoadSlot => {
                    let slot = instruction.operands.first().copied().unwrap_or(0);
                    // Load from slot storage relative to base pointer.
                    let value = self.slots.get(self.bp + slot).cloned().unwrap_or(Value::Na);
                    self.stack.push(value);
                    self.pc += 1;
                }
                OpCode::StoreSlot => {
                    let slot = instruction.operands.first().copied().unwrap_or(0);
                    if let Some(value) = self.stack.pop() {
                        // Store to slot storage relative to base pointer.
                        let idx = self.bp + slot;
                        if idx < self.slots.len() {
                            self.slots[idx] = value;
                        } else {
                            while self.slots.len() <= idx {
                                self.slots.push(Value::Na);
                            }
                            self.slots[idx] = value;
                        }
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
                OpCode::Le => {
                    let b = self.stack.peek_at(0).cloned().unwrap_or(Value::Na);
                    let a = self.stack.peek_at(1).cloned().unwrap_or(Value::Na);
                    vm_debug!("DEBUG VM Le: a={:?}, b={:?}", a, b);
                    self.op_le()?;
                    let result = self.stack.peek().cloned().unwrap_or(Value::Na);
                    vm_debug!("DEBUG VM Le: result={:?}", result);
                }
                OpCode::Gt => self.op_gt()?,
                OpCode::Ge => self.op_ge()?,
                OpCode::And => self.op_and()?,
                OpCode::Or => self.op_or()?,
                OpCode::Not => self.op_not()?,
                OpCode::IsNa => self.op_is_na()?,
                OpCode::Coalesce => self.op_coalesce()?,
                OpCode::UpdateUserSeries => {
                    let name_idx = instruction.operands.first().copied().unwrap_or(0);
                    let name = chunk
                        .constants
                        .get(name_idx)
                        .and_then(|v| match v {
                            Value::String(s) => Some(s.to_string()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let value = self.stack.pop().unwrap_or(Value::Na);
                    vm_debug!("DEBUG UpdateUserSeries: name={}, value={:?}", name, value);
                    let call_site = ExecutionContext::global_call_site();
                    self.context.push_to_series(&name, call_site, value);
                    let series_len = self
                        .context
                        .get_series(&name, call_site)
                        .map(|series| series.len())
                        .unwrap_or(0);
                    vm_debug!("DEBUG UpdateUserSeries: series.len={}", series_len);
                    self.pc += 1;
                }
                OpCode::Jump => {
                    let target = instruction.operands.first().copied().unwrap_or(0);
                    self.pc = target;
                }
                OpCode::JumpIfFalse => {
                    let target = instruction.operands.first().copied().unwrap_or(0);
                    let condition = self.stack.peek().unwrap_or(&Value::Na);
                    vm_debug!(
                        "DEBUG VM JumpIfFalse: condition={:?}, target={}, is_truthy={}, will_jump={}",
                        condition,
                        target,
                        condition.is_truthy(),
                        !condition.is_truthy()
                    );
                    if !condition.is_truthy() {
                        self.pc = target;
                    } else {
                        self.pc += 1;
                    }
                }
                OpCode::JumpIfTrue => {
                    let target = instruction.operands.first().copied().unwrap_or(0);
                    let condition = self.stack.peek().unwrap_or(&Value::Na);
                    if condition.is_truthy() {
                        self.pc = target;
                    } else {
                        self.pc += 1;
                    }
                }
                OpCode::Return => {
                    // Pop return value
                    let result = self.stack.pop().unwrap_or(Value::Na);

                    if let Some(frame) = self.call_stack.pop() {
                        // Restore base pointer
                        self.bp = frame.bp;
                        // Jump to return address
                        self.pc = frame.return_pc;
                        // Push return value back to stack
                        self.stack.push(result);
                    } else {
                        // Top-level return - exit execution
                        self.stack.push(result);
                        return Ok(self.stack.pop());
                    }
                }
                OpCode::PushSeries => {
                    let series_idx = instruction.operands.first().copied().unwrap_or(0);
                    let series_name = chunk
                        .series_names
                        .get(series_idx)
                        .ok_or(VmError::InvalidSeries(series_idx))?;
                    let call_site = ExecutionContext::global_call_site();
                    let value = self
                        .context
                        .get_series_current(series_name, call_site)
                        .cloned()
                        .unwrap_or(Value::Na);
                    self.stack.push(value);
                    self.pc += 1;
                }
                OpCode::PushSeriesAt => {
                    let series_idx = instruction.operands.first().copied().unwrap_or(0);
                    let offset = instruction.operands.get(1).copied().unwrap_or(0);
                    let series_name = chunk
                        .series_names
                        .get(series_idx)
                        .ok_or(VmError::InvalidSeries(series_idx))?;
                    let call_site = ExecutionContext::global_call_site();
                    let value = self
                        .context
                        .get_series_at(series_name, call_site, offset)
                        .cloned()
                        .unwrap_or(Value::Na);
                    self.stack.push(value);
                    self.pc += 1;
                }
                OpCode::SeriesPush => {
                    let series_idx = instruction.operands.first().copied().unwrap_or(0);
                    let series_name = chunk
                        .series_names
                        .get(series_idx)
                        .ok_or(VmError::InvalidSeries(series_idx))?
                        .clone();
                    let value = self.stack.pop().unwrap_or(Value::Na);
                    let call_site = ExecutionContext::global_call_site();
                    self.context.push_to_series(series_name, call_site, value);
                    self.pc += 1;
                }
                OpCode::PushSeriesAtDynamic => {
                    let series_idx = instruction.operands.first().copied().unwrap_or(0);
                    // Pop offset from stack (must be after series_idx is extracted)
                    let offset_val = self.stack.pop().unwrap_or(Value::Na);
                    let offset = match offset_val {
                        Value::Int(i) => i as usize,
                        Value::Float(f) => f as usize,
                        _ => 0,
                    };
                    let series_name = chunk
                        .series_names
                        .get(series_idx)
                        .ok_or(VmError::InvalidSeries(series_idx))?;
                    let call_site = ExecutionContext::global_call_site();
                    let value = self
                        .context
                        .get_series_at(series_name, call_site, offset)
                        .cloned()
                        .unwrap_or(Value::Na);
                    self.stack.push(value);
                    self.pc += 1;
                }
                OpCode::Call => {
                    let func_idx = instruction.operands.first().copied().unwrap_or(0);
                    let arg_count = instruction.operands.get(1).copied().unwrap_or(0);
                    let user_func_count = chunk.function_addresses.len();
                    vm_debug!(
                        "DEBUG VM: OpCode::Call func_idx={} arg_count={} user_func_count={}",
                        func_idx,
                        arg_count,
                        user_func_count
                    );
                    vm_debug!("DEBUG VM: external_functions={:?}", self.external_functions);

                    // Check if this is an external function call
                    if func_idx >= user_func_count {
                        // External function call
                        let ext_idx = func_idx - user_func_count;
                        if let Some(func_name) = self.external_functions.get(ext_idx) {
                            // Collect arguments from stack
                            let mut args = Vec::with_capacity(arg_count);
                            for _ in 0..arg_count {
                                let arg = self.stack.pop().unwrap_or(Value::Na);
                                // Expand SeriesRef to full series data for series functions
                                let expanded = match &arg {
                                    Value::SeriesRef(name) => {
                                        if is_series_function(func_name) {
                                            // Get full series data from context (oldest first)
                                            let call_site = ExecutionContext::global_call_site();
                                            let series_data: Vec<Value> = self
                                                .context
                                                .get_series_history_oldest_first(name, call_site)
                                                .unwrap_or_default();
                                            Value::Array(series_data)
                                        } else {
                                            // For non-series functions, use current value
                                            let call_site = ExecutionContext::global_call_site();
                                            self.context
                                                .get_series_current(name, call_site)
                                                .cloned()
                                                .unwrap_or(Value::Na)
                                        }
                                    }
                                    _ => arg,
                                };
                                args.push(expanded);
                            }
                            args.reverse(); // Stack is LIFO, so reverse to get correct order

                            // Check for special VM internal functions
                            vm_debug!(
                                "DEBUG VM Call: func_name={}, arg_count={}, args={:?}",
                                func_name,
                                arg_count,
                                args
                            );
                            if func_name == "__series_at" {
                                vm_debug!("DEBUG __series_at: args={:?}", args);
                                // Arguments: series_name (string), offset (int)
                                // After args.reverse(), order is: series_name, offset
                                let series_name = args
                                    .first()
                                    .and_then(|v| match v {
                                        Value::String(s) => Some(s.to_string()),
                                        _ => None,
                                    })
                                    .unwrap_or_default();
                                let offset = args
                                    .get(1)
                                    .map(|v| match v {
                                        Value::Int(i) => *i as usize,
                                        Value::Float(f) => *f as usize,
                                        _ => 0,
                                    })
                                    .unwrap_or(0);
                                let call_site = ExecutionContext::global_call_site();
                                vm_debug!(
                                    "DEBUG __series_at: series_name={}, offset={}, call_site={:?}",
                                    series_name,
                                    offset,
                                    call_site
                                );
                                let series_data = self.context.get_series(&series_name, call_site);
                                vm_debug!("DEBUG __series_at: series_data={:?}", series_data);
                                let value = self
                                    .context
                                    .get_series_at(&series_name, call_site, offset)
                                    .cloned()
                                    .unwrap_or(Value::Na);
                                vm_debug!("DEBUG __series_at: returning {:?}", value);
                                self.stack.push(value);
                                // Fall through to pc increment at the end
                            } else if func_name == "plot" || func_name.starts_with("plot.") {
                                // Extract title and value from args
                                // plot(value, title, ...) - title is usually the second arg
                                let value = args.first().cloned().unwrap_or(Value::Na);
                                let title = args
                                    .get(1)
                                    .and_then(|v| match v {
                                        Value::String(s) => Some(s.to_string()),
                                        _ => None,
                                    })
                                    .unwrap_or_else(|| "Plot".to_string());

                                self.plot_outputs.push(PlotRecord {
                                    title,
                                    value: value.as_float(),
                                });
                                self.stack.push(Value::Na); // plot returns na
                            } else if func_name == "na" {
                                // na(value) - returns true if value is NA, false otherwise
                                vm_debug!("DEBUG VM na: args={:?}", args);
                                let result = if args.is_empty() {
                                    Value::Bool(true)
                                } else {
                                    Value::Bool(matches!(args[0], Value::Na))
                                };
                                vm_debug!("DEBUG VM na: returning {:?}", result);
                                self.stack.push(result);
                            } else if func_name == "__load_series_data" {
                                // Load series data from context by name
                                // args[0] should be the series name (string)
                                let series_name = args
                                    .first()
                                    .and_then(|v| match v {
                                        Value::String(s) => Some(s.to_string()),
                                        _ => None,
                                    })
                                    .unwrap_or_default();
                                let call_site = ExecutionContext::global_call_site();
                                let series_data: Vec<Value> = self
                                    .context
                                    .get_series_history_oldest_first(&series_name, call_site)
                                    .unwrap_or_default();
                                self.stack.push(Value::Array(series_data));
                            } else if func_name == "__array_size" {
                                let arr = args.first().cloned().unwrap_or(Value::Na);
                                let size = match arr {
                                    Value::Array(ref v) => v.len() as i64,
                                    _ => 0,
                                };
                                self.stack.push(Value::Int(size));
                            } else if func_name == "__array_get" {
                                let arr = args.first().cloned().unwrap_or(Value::Na);
                                let index = args
                                    .get(1)
                                    .and_then(|v| match v {
                                        Value::Int(i) => usize::try_from(*i).ok(),
                                        Value::Float(f) if *f >= 0.0 => Some(*f as usize),
                                        _ => None,
                                    })
                                    .unwrap_or(0);
                                let item = match arr {
                                    Value::Array(ref values) => {
                                        values.get(index).cloned().unwrap_or(Value::Na)
                                    }
                                    _ => Value::Na,
                                };
                                self.stack.push(item);
                            } else if func_name == "__tuple_get" {
                                let tuple_value = args.first().cloned().unwrap_or(Value::Na);
                                let index = args
                                    .get(1)
                                    .and_then(|v| match v {
                                        Value::Int(i) => usize::try_from(*i).ok(),
                                        Value::Float(f) if *f >= 0.0 => Some(*f as usize),
                                        _ => None,
                                    })
                                    .unwrap_or(0);

                                let item = match tuple_value {
                                    Value::Tuple(values) => {
                                        values.get(index).cloned().unwrap_or(Value::Na)
                                    }
                                    Value::Array(values) => {
                                        values.get(index).cloned().unwrap_or(Value::Na)
                                    }
                                    _ => Value::Na,
                                };
                                self.stack.push(item);
                            } else {
                                // Dispatch to external function
                                vm_debug!("DEBUG: dispatching {} with args {:?}", func_name, args);
                                if let Some(result) =
                                    self.function_registry.dispatch(func_name, &args)
                                {
                                    vm_debug!("DEBUG: {} returned {:?}", func_name, result);
                                    self.stack.push(result);
                                } else {
                                    vm_debug!("DEBUG: {} not found in registry", func_name);
                                    self.stack.push(Value::Na);
                                }
                            }
                            self.pc += 1; // Increment pc after all external function handlers
                        } else {
                            return Err(VmError::InvalidFunction(func_idx));
                        }
                    } else {
                        // User-defined function call
                        let func_addr = chunk
                            .function_addresses
                            .get(func_idx)
                            .copied()
                            .ok_or(VmError::InvalidFunction(func_idx))?;

                        // Calculate base pointer for new frame (args are already on stack)
                        let new_bp = self.stack.len().saturating_sub(arg_count);

                        for (offset, arg) in self
                            .stack
                            .data
                            .iter()
                            .skip(new_bp)
                            .take(arg_count)
                            .cloned()
                            .enumerate()
                        {
                            let idx = new_bp + offset;
                            if idx < self.slots.len() {
                                self.slots[idx] = arg;
                            } else {
                                while self.slots.len() <= idx {
                                    self.slots.push(Value::Na);
                                }
                                self.slots[idx] = arg;
                            }
                        }

                        // Create new call frame
                        let frame = CallFrame::new(self.pc, self.bp, self.pc + 1);
                        self.call_stack.push(frame);

                        // Set new base pointer and jump to function
                        self.bp = new_bp;
                        self.pc = func_addr;
                    }
                }
                _ => {
                    // Other opcodes not yet implemented
                    return Err(VmError::NotImplemented(format!("{:?}", opcode)));
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

    /// Consume the VM and return the execution context.
    pub fn into_context(self) -> ExecutionContext {
        self.context
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

    /// Resolve a value for arithmetic operations.
    /// SeriesRef is resolved to the current bar's value.
    fn resolve_value(&self, value: &Value) -> Value {
        match value {
            Value::SeriesRef(name) => {
                let call_site = ExecutionContext::global_call_site();
                self.context
                    .get_series_current(name, call_site)
                    .cloned()
                    .unwrap_or(Value::Na)
            }
            _ => value.clone(),
        }
    }

    fn op_add(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        let b = self.resolve_value(&b);
        let a = self.resolve_value(&a);
        self.stack.push(na_ops::add(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_sub(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        let b = self.resolve_value(&b);
        let a = self.resolve_value(&a);
        self.stack.push(na_ops::sub(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_mul(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        let b = self.resolve_value(&b);
        let a = self.resolve_value(&a);
        self.stack.push(na_ops::mul(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_div(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        let b = self.resolve_value(&b);
        let a = self.resolve_value(&a);
        self.stack.push(na_ops::div(&a, &b));
        self.pc += 1;
        Ok(())
    }

    fn op_mod(&mut self) -> VmResult<()> {
        let b = self.stack.pop().unwrap_or(Value::Na);
        let a = self.stack.pop().unwrap_or(Value::Na);
        let b = self.resolve_value(&b);
        let a = self.resolve_value(&a);
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

/// Check if a function is a series function that needs full series data
fn is_series_function(name: &str) -> bool {
    // Check for ta.* functions
    if name.starts_with("ta.") {
        return true;
    }
    false
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

    #[test]
    fn test_vm_series_operations() {
        let mut compiler = Compiler::new();

        // Register a series named "close"
        let series_idx = compiler.register_series("close");
        assert_eq!(series_idx, 0);

        // Push a constant value and store it to the series
        compiler.compile_const(Value::Float(100.0));
        compiler.compile_series_push(series_idx);

        // Push the current series value to stack
        compiler.compile_push_series(series_idx);
        compiler.compile_halt();

        let chunk = compiler.finish();

        // Execute the chunk
        let mut vm = VM::new();
        vm.load_chunk(chunk);

        // Set bar index so series operations work correctly
        vm.context_mut().set_bar_index(0);

        let result = vm.execute().unwrap();

        // The value on top of stack should be 100.0
        assert_eq!(result, Some(Value::Float(100.0)));

        // Verify the series has the value
        let call_site = ExecutionContext::global_call_site();
        assert_eq!(
            vm.context().get_series_current("close", call_site),
            Some(&Value::Float(100.0))
        );
    }

    #[test]
    fn test_vm_series_historical_access() {
        // Test that we can access historical series values
        let mut vm = VM::new();
        let call_site = ExecutionContext::global_call_site();

        // Simulate bar-by-bar execution
        for i in 0..5 {
            vm.context_mut().set_bar_index(i);
            vm.context_mut()
                .push_to_series("close", call_site, Value::Float(100.0 + i as f64));
        }

        // Verify current value
        assert_eq!(
            vm.context().get_series_current("close", call_site),
            Some(&Value::Float(104.0))
        );

        // Verify historical values
        assert_eq!(
            vm.context().get_series_at("close", call_site, 0),
            Some(&Value::Float(104.0))
        );
        assert_eq!(
            vm.context().get_series_at("close", call_site, 1),
            Some(&Value::Float(103.0))
        );
        assert_eq!(
            vm.context().get_series_at("close", call_site, 4),
            Some(&Value::Float(100.0))
        );
    }

    #[test]
    fn test_vm_series_na_for_missing_history() {
        // Test that accessing out-of-bounds history returns NA
        let mut vm = VM::new();
        let call_site = ExecutionContext::global_call_site();

        // Only push 2 values
        vm.context_mut().set_bar_index(0);
        vm.context_mut()
            .push_to_series("close", call_site, Value::Float(100.0));
        vm.context_mut().set_bar_index(1);
        vm.context_mut()
            .push_to_series("close", call_site, Value::Float(101.0));

        // Accessing offset 5 when only 2 values exist should return None
        assert_eq!(vm.context().get_series_at("close", call_site, 5), None);
    }

    #[test]
    fn test_vm_call_return() {
        // Test a simple function: fn add(a, b) => a + b
        let mut compiler = Compiler::new();

        // Main code: push 10, push 32, call add(10, 32), halt
        compiler.compile_const(Value::Int(10));
        compiler.compile_const(Value::Int(32));
        compiler.compile_call(0, 2); // Call function 0 with 2 args
        compiler.compile_halt();

        // Register function after main code
        let func_idx = compiler.register_function();
        assert_eq!(func_idx, 0);

        // Function body: load slot 0 (a), load slot 1 (b), add, return
        compiler.compile_load_slot(0); // Load first argument
        compiler.compile_load_slot(1); // Load second argument
        compiler.compile_binary(BinaryOp::Add);
        compiler.compile_op(crate::opcode::OpCode::Return);

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();

        // Result should be 42 (10 + 32)
        assert_eq!(result, Some(Value::Int(42)));
    }

    #[test]
    fn test_vm_call_with_nested_functions() {
        // Test calling a function that calls another function
        let mut compiler = Compiler::new();

        // Main: quadruple(5) => 20
        compiler.compile_const(Value::Int(5));
        compiler.compile_call(1, 1); // Call function 1 (quadruple) with 1 arg
        compiler.compile_halt();

        // Function 0: double(x) => x + x
        let double_idx = compiler.register_function();
        assert_eq!(double_idx, 0);
        compiler.compile_load_slot(0);
        compiler.compile_load_slot(0);
        compiler.compile_binary(BinaryOp::Add);
        compiler.compile_op(crate::opcode::OpCode::Return);

        // Function 1: quadruple(x) => double(double(x))
        let quad_idx = compiler.register_function();
        assert_eq!(quad_idx, 1);
        compiler.compile_load_slot(0);
        compiler.compile_call(double_idx, 1); // Call double with 1 arg
        compiler.compile_call(double_idx, 1); // Call double again
        compiler.compile_op(crate::opcode::OpCode::Return);

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();

        // 5 * 2 * 2 = 20
        assert_eq!(result, Some(Value::Int(20)));
    }

    #[test]
    fn test_vm_if_statement() {
        // Test: if true then 1 else 2 => 1
        let mut compiler = Compiler::new();

        compiler.compile_if(
            |c| c.compile_const(Value::Bool(true)),
            |c| c.compile_const(Value::Int(1)),
            Some(|c: &mut Compiler| c.compile_const(Value::Int(2))),
        );
        compiler.compile_halt();

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(1)));
    }

    #[test]
    fn test_vm_if_else_statement() {
        // Test: if false then 1 else 2 => 2
        let mut compiler = Compiler::new();

        compiler.compile_if(
            |c| c.compile_const(Value::Bool(false)),
            |c| c.compile_const(Value::Int(1)),
            Some(|c: &mut Compiler| c.compile_const(Value::Int(2))),
        );
        compiler.compile_halt();

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(2)));
    }

    #[test]
    fn test_vm_while_loop_simple() {
        // Simple test: i = 0; while i < 3 { i = i + 1 }; return i
        let mut compiler = Compiler::new();

        // i (slot 0) = 0
        compiler.compile_const(Value::Int(0));
        compiler.compile_store_slot(0);

        // While loop: while i < 3
        compiler.compile_while(
            |c| {
                c.compile_load_slot(0);
                c.compile_const(Value::Int(3));
                c.compile_binary(BinaryOp::Lt);
            },
            |c| {
                // i = i + 1
                c.compile_load_slot(0);
                c.compile_const(Value::Int(1));
                c.compile_binary(BinaryOp::Add);
                c.compile_store_slot(0);
            },
        );

        // Return i
        compiler.compile_load_slot(0);
        compiler.compile_op(crate::opcode::OpCode::Return);

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(3)));
    }

    #[test]
    fn test_vm_while_loop() {
        // Test: sum = 0; i = 1; while i <= 5 { sum = sum + i; i = i + 1 } => sum = 15
        let mut compiler = Compiler::new();

        // Initialize sum (slot 0) = 0, i (slot 1) = 1
        compiler.compile_const(Value::Int(0));
        compiler.compile_store_slot(0);

        compiler.compile_const(Value::Int(1));
        compiler.compile_store_slot(1);

        // While loop: while i <= 5
        compiler.compile_while(
            |c| {
                c.compile_load_slot(1); // Load i
                c.compile_const(Value::Int(5));
                c.compile_binary(BinaryOp::Le);
            },
            |c| {
                // sum = sum + i
                c.compile_load_slot(0);
                c.compile_load_slot(1);
                c.compile_binary(BinaryOp::Add);
                c.compile_store_slot(0);

                // i = i + 1
                c.compile_load_slot(1);
                c.compile_const(Value::Int(1));
                c.compile_binary(BinaryOp::Add);
                c.compile_store_slot(1);
            },
        );

        // Return sum
        compiler.compile_load_slot(0);
        compiler.compile_op(crate::opcode::OpCode::Return);

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(15))); // 1+2+3+4+5 = 15
    }

    #[test]
    fn test_vm_for_loop() {
        // Test: sum = 0; for i = 1 to 5 { sum = sum + i } => sum = 15
        let mut compiler = Compiler::new();

        // Initialize sum (slot 0) = 0
        compiler.compile_const(Value::Int(0));
        compiler.compile_store_slot(0);

        // For loop: for i = 1 to 5 (i uses slot 1)
        compiler.compile_for(
            1,                                  // loop var slot
            |c| c.compile_const(Value::Int(1)), // start
            |c| c.compile_const(Value::Int(5)), // end
            None::<fn(&mut Compiler)>,          // step (default 1)
            |c| {
                // sum = sum + i
                c.compile_load_slot(0);
                c.compile_load_slot(1);
                c.compile_binary(BinaryOp::Add);
                c.compile_store_slot(0);
            },
        );

        // Return sum
        compiler.compile_load_slot(0);
        compiler.compile_op(crate::opcode::OpCode::Return);

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(15)));
    }

    #[test]
    fn test_vm_nested_if() {
        // Test nested if: if true then (if false then 1 else 2) else 3 => 2
        let mut compiler = Compiler::new();

        compiler.compile_if(
            |c| c.compile_const(Value::Bool(true)),
            |c| {
                c.compile_if(
                    |c| c.compile_const(Value::Bool(false)),
                    |c| c.compile_const(Value::Int(1)),
                    Some(|c: &mut Compiler| c.compile_const(Value::Int(2))),
                );
            },
            Some(|c: &mut Compiler| c.compile_const(Value::Int(3))),
        );
        compiler.compile_halt();

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(2)));
    }

    #[test]
    fn test_vm_variable_scope() {
        // Test: x = 5; y = x + 3; return y
        let mut compiler = Compiler::new();

        // x = 5
        compiler.compile_const(Value::Int(5));
        compiler.compile_var_decl("x");

        // y = x + 3
        compiler.compile_load_var("x");
        compiler.compile_const(Value::Int(3));
        compiler.compile_binary(BinaryOp::Add);
        compiler.compile_var_decl("y");

        // return y
        compiler.compile_load_var("y");
        compiler.compile_op(crate::opcode::OpCode::Return);

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(8)));
    }

    #[test]
    fn test_vm_variable_reassignment() {
        // Test: x = 5; x = x + 3; return x
        let mut compiler = Compiler::new();

        // x = 5
        compiler.compile_const(Value::Int(5));
        compiler.compile_var_decl("x");

        // x = x + 3
        compiler.compile_load_var("x");
        compiler.compile_const(Value::Int(3));
        compiler.compile_binary(BinaryOp::Add);
        compiler.compile_store_var("x");

        // return x
        compiler.compile_load_var("x");
        compiler.compile_op(crate::opcode::OpCode::Return);

        let chunk = compiler.finish();
        let result = execute_chunk(chunk).unwrap();
        assert_eq!(result, Some(Value::Int(8)));
    }
}
