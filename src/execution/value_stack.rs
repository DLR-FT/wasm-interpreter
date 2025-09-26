use alloc::vec::{Drain, Vec};

use crate::core::indices::LocalIdx;
use crate::core::reader::types::{FuncType, ValType};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::value::Value;
use crate::{unreachable_validated, RuntimeError};

use super::value::Ref;

// TODO make these configurable
const MAX_VALUE_STACK_SIZE: usize = 0xf0000; // 64 Kibi-Values
const MAX_CALL_STACK_SIZE: usize = 0x1000; // 4 Kibi-Functions

/// The stack at runtime containing
/// 1. Values
/// 2. Labels
/// 3. Activations
///
/// See <https://webassembly.github.io/spec/core/exec/runtime.html#stack>
#[derive(Default, Debug)]
pub(crate) struct Stack {
    /// WASM values on the stack, i.e. the actual data that instructions operate on
    values: Vec<Value>,

    /// Call frames
    ///
    /// Each time a function is called, a new frame is pushed, whenever a function returns, a frame is popped
    frames: Vec<CallFrame>,
}

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_values(values: Vec<Value>) -> Self {
        Self {
            values,
            ..Self::default()
        }
    }

    pub(super) fn into_values(self) -> Vec<Value> {
        self.values
    }

    pub fn drop_value(&mut self) {
        // If there is at least one call frame, we shall not pop values past the current
        // call frame. However, there is one legitimate reason to pop when there is **no** current
        // call frame: after the outermost function returns, to extract the final return values of
        // this interpreter invocation.
        debug_assert!(
            if !self.frames.is_empty() {
                self.values.len() > self.current_call_frame().value_stack_base_idx
            } else {
                true
            },
            "can not pop values past the current call frame"
        );

        self.values.pop().unwrap_validated();
    }

    /// Pop a reference of unknown type from the value stack
    pub fn pop_unknown_ref(&mut self) -> Ref {
        // If there is at least one call frame, we shall not pop values past the current
        // call frame. However, there is one legitimate reason to pop when there is **no** current
        // call frame: after the outermost function returns, to extract the final return values of
        // this interpreter invocation.
        debug_assert!(
            if !self.frames.is_empty() {
                self.values.len() > self.current_call_frame().value_stack_base_idx
            } else {
                true
            },
            "can not pop values past the current call frame"
        );

        let popped = self.values.pop().unwrap_validated();
        match popped.to_ty() {
            ValType::RefType(_) => match popped {
                Value::Ref(rref) => rref,
                _ => unreachable!(),
            },
            _ => unreachable_validated!(),
        }
    }

    /// Pop a value of the given [ValType] from the value stack
    pub fn pop_value(&mut self) -> Value {
        // If there is at least one call frame, we shall not pop values past the current
        // call frame. However, there is one legitimate reason to pop when there is **no** current
        // call frame: after the outermost function returns, to extract the final return values of
        // this interpreter invocation.
        debug_assert!(
            if !self.frames.is_empty() {
                self.values.len() > self.current_call_frame().value_stack_base_idx
            } else {
                true
            },
            "can not pop values past the current call frame"
        );

        self.values.pop().unwrap_validated()
    }

    //unfortunately required for polymorphic select
    pub fn pop_value_with_unknown_type(&mut self) -> Value {
        self.values.pop().unwrap_validated()
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

    /// Returns a cloned copy of the top value on the stack, or `None` if the stack is empty
    pub fn peek_unknown_value(&self) -> Option<Value> {
        self.values.last().copied()
    }

    /// Push a value to the value stack
    pub fn push_value(&mut self, value: Value) -> Result<(), RuntimeError> {
        // check for value stack exhaustion
        if self.values.len() > MAX_VALUE_STACK_SIZE {
            return Err(RuntimeError::StackExhaustion);
        }

        // push the value
        self.values.push(value);

        Ok(())
    }

    /// Copy a local variable to the top of the value stack
    pub fn get_local(&mut self, idx: LocalIdx) -> Result<(), RuntimeError> {
        let call_frame_base_idx = self.current_call_frame().call_frame_base_idx;
        let local_value = self
            .values
            .get(call_frame_base_idx + idx)
            .unwrap_validated();
        self.push_value(*local_value)
    }

    /// Pop value from the top of the value stack, writing it to the given local
    pub fn set_local(&mut self, idx: LocalIdx) {
        debug_assert!(
            self.values.len() > self.current_call_frame().value_stack_base_idx,
            "can not pop values past the current call frame"
        );

        let call_frame_base_idx = self.current_call_frame().call_frame_base_idx;
        let stack_value = self.pop_value();

        trace!("Instruction: local.set [{stack_value:?}] -> []");

        *self
            .values
            .get_mut(call_frame_base_idx + idx)
            .unwrap_validated() = stack_value;
    }

    /// Copy value from top of the value stack to the given local
    pub fn tee_local(&mut self, idx: LocalIdx) {
        let call_frame_base_idx = self.current_call_frame().call_frame_base_idx;

        let local_ty = self
            .values
            .get(call_frame_base_idx + idx)
            .unwrap_validated()
            .to_ty();
        let stack_value = self.peek_value(local_ty);

        trace!("Instruction: local.tee [{stack_value:?}] -> []");

        *self
            .values
            .get_mut(call_frame_base_idx + idx)
            .unwrap_validated() = stack_value;
    }

    /// Get a shared reference to the current [`CallFrame`]
    pub fn current_call_frame(&self) -> &CallFrame {
        self.frames.last().unwrap_validated()
    }

    /// Get a mutable reference to the current [`CallFrame`]
    pub fn _current_call_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap_validated()
    }

    /// Pop a [`CallFrame`] from the call stack, returning the caller function store address, return address, and the return stp
    pub fn pop_call_frame(&mut self) -> (usize, usize, usize) {
        let CallFrame {
            return_func_addr,
            return_addr,
            call_frame_base_idx,
            return_value_count,
            return_stp,
            ..
        } = self.frames.pop().unwrap_validated();

        let remove_count = self.values.len() - call_frame_base_idx - return_value_count;

        self.remove_inbetween(remove_count, return_value_count);

        debug_assert_eq!(
            self.values.len(),
            call_frame_base_idx + return_value_count,
            "after a function call finished, the stack must have exactly as many values as it had before calling the function plus the number of function return values"
        );

        (return_func_addr, return_addr, return_stp)
    }

    /// Push a call frame to the call stack
    ///
    /// Takes the current [`Self::values`]'s length as [`CallFrame::value_stack_base_idx`].
    pub fn push_call_frame(
        &mut self,
        return_func_addr: usize,
        func_ty: &FuncType,
        remaining_locals: &[ValType],
        return_addr: usize,
        return_stp: usize,
    ) -> Result<(), RuntimeError> {
        // check for call stack exhaustion
        if self.call_frame_count() > MAX_CALL_STACK_SIZE {
            return Err(RuntimeError::StackExhaustion);
        }

        debug_assert!(
            self.values.len() >= func_ty.params.valtypes.len(),
            "when pushing a new call frame, at least as many values need to be on the stack as required by the new call frames's function"
        );

        // the topmost `param_count` values are transferred into/consumed by this new call frame
        let param_count = func_ty.params.valtypes.len();
        let call_frame_base_idx = self.values.len() - param_count;

        // after the params, put the additional locals
        for local in remaining_locals {
            self.values.push(Value::default_from_ty(*local));
        }

        // now that the locals are all populated, the actual stack section of this call frame begins
        let value_stack_base_idx = self.values.len();

        self.frames.push(CallFrame {
            return_func_addr,
            return_addr,
            value_stack_base_idx,
            call_frame_base_idx,
            return_value_count: func_ty.returns.valtypes.len(),
            return_stp,
        });

        Ok(())
    }

    /// Returns how many call frames are on the stack, in total.
    pub fn call_frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Pop `n` elements from the value stack's tail as an iterator, with the first element being
    /// closest to the **bottom** of the value stack
    ///
    /// Note that this is providing the values in reverse order compared to popping `n` values
    /// (which would yield the element closest to the **top** of the value stack first).
    pub fn pop_tail_iter(&mut self, n: usize) -> Drain<'_, Value> {
        let start = self.values.len() - n;
        self.values.drain(start..)
    }

    /// Remove `remove_count` values from the stack, keeping the topmost `keep_count` values
    ///
    /// From the stack, remove `remove_count` elements, by sliding down the `keep_count` topmost
    /// values `remove_count` positions.
    ///
    /// **Effects**
    ///
    /// - after the operation, [`Stack`] will contain `remove_count` fewer elements
    /// - `keep_count` topmost elements will be identical before and after the operation
    /// - all elements below the `remove_count + keep_count` topmost stack entry remain
    pub fn remove_inbetween(&mut self, remove_count: usize, keep_count: usize) {
        let len = self.values.len();
        self.values
            .copy_within(len - keep_count.., len - keep_count - remove_count);
        self.values.truncate(len - remove_count);
    }
}

/// The [WASM spec](https://webassembly.github.io/spec/core/exec/runtime.html#stack) calls this `Activations`, however it refers to the call frames of functions.
#[derive(Debug)]
pub(crate) struct CallFrame {
    /// Store address of the function that called this [`CallFrame`]'s function
    pub return_func_addr: usize,

    /// Value that the PC has to be set to when this function returns
    pub return_addr: usize,

    /// The index to the lowermost value in [`Stack::values`] belonging to this [`CallFrame`]'s
    /// stack
    ///
    /// Values below this may still belong to this [`CallFrame`], but they are locals. Consequently,
    /// this is the lowest index down to which the stack may be popped in this [`CallFrame`].
    /// However, clearing up this [`CallFrame`] may require further popping, down to (and
    /// including!) the index stored in [`Self::call_frame_base_idx`].
    pub value_stack_base_idx: usize,

    /// The index to the lowermost value on [`Stack::values`] that belongs to this [`CallFrame`]
    ///
    /// Clearing this [`CallFrame`] requires popping all elements on [`Stack::values`] down to (and
    /// including!) this index.
    pub call_frame_base_idx: usize,

    /// Number of return values to retain on [`Stack::values`] when unwinding/popping a [`CallFrame`]
    pub return_value_count: usize,

    // Value that the stp has to be set to when this function returns
    pub return_stp: usize,
}
