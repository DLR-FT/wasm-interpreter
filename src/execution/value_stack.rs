use alloc::vec::Vec;

use crate::core::indices::{FuncIdx, LocalIdx};
use crate::core::reader::types::ValType;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::value::Value;
use crate::locals::Locals;
use crate::unreachable_validated;

/// The stack at runtime containing
/// 1. Values
/// 2. Labels
/// 3. Activations
///
/// See <https://webassembly.github.io/spec/core/exec/runtime.html#stack>
#[derive(Default)]
pub(crate) struct Stack {
    /// WASM values on the stack, i.e. the actual data that instructions operate on
    values: Vec<Value>,

    /// Stack frames
    ///
    /// Each time a function is called, a new frame is pushed, whenever a function returns, a frame is popped
    frames: Vec<CallFrame>,
}

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pop a value of the given [ValType] from the value stack
    pub fn pop_value(&mut self, ty: ValType) -> Value {
        let popped = self.values.pop().unwrap_validated();
        if popped.to_ty() == ty {
            popped
        } else {
            unreachable_validated!()
        }
    }

    /// Copy a value of the given [ValType] from the value stack without removing it
    pub fn peek_value(&self, ty: ValType) -> Value {
        let value = self.values.last().unwrap_validated();
        if value.to_ty() == ty {
            *value
        } else {
            unreachable_validated!()
        }
    }

    /// Push a value to the value stack
    pub fn push_value(&mut self, value: Value) {
        self.values.push(value);
    }

    /// Copy a local variable to the top of the value stack
    pub fn get_local(&mut self, idx: LocalIdx) {
        let local_value = self.frames.last().unwrap_validated().locals.get(idx);
        self.values.push(*local_value);
    }

    /// Pop value from the top of the value stack, writing it to the given local
    pub fn set_local(&mut self, idx: LocalIdx) {
        let local_ty = self.current_stackframe().locals.get_ty(idx);

        let stack_value = self.pop_value(local_ty);
        *self.current_stackframe_mut().locals.get_mut(idx) = stack_value;
    }

    /// Copy value from top of the value stack to the given local
    pub fn tee_local(&mut self, idx: LocalIdx) {
        let local_ty = self.current_stackframe().locals.get_ty(idx);

        let stack_value = self.peek_value(local_ty);
        *self.current_stackframe_mut().locals.get_mut(idx) = stack_value;
    }

    /// Get a shared reference to the current [CallFrame]
    pub fn current_stackframe(&self) -> &CallFrame {
        self.frames.last().unwrap_validated()
    }

    /// Get a mutable reference to the current [CallFrame]
    pub fn current_stackframe_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap_validated()
    }

    /// Pop a [CallFrame] from the call stack
    // TODO figure out if this has to change the value stack

    pub fn pop_stackframe(&mut self) -> usize {
        // TODO maybe manipulate the value stack
        self.frames.pop().unwrap_validated().return_addr
    }

    /// Pop a stackframe from the call stack
    // TODO figure out if this has to change the value stack
    pub fn push_stackframe(&mut self, func_idx: FuncIdx, locals: Locals, return_addr: usize) {
        // TODO maybe manipulate the value stack
        self.frames.push(CallFrame {
            func_idx,
            locals,
            return_addr,
        })
    }
}

/// The [WASM spec](https://webassembly.github.io/spec/core/exec/runtime.html#stack) calls this `Activations`, however it refers to the call frames of functions.
pub(crate) struct CallFrame {
    /// Index to the function of this CallFrame
    pub func_idx: FuncIdx,

    /// Local varaiables such as parameters for this [CallFrame]'s function
    pub locals: Locals,

    /// Value that the PC has to be set to when this function returns
    pub return_addr: usize,
}
