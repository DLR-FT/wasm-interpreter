use alloc::vec::{Drain, Vec};

use crate::core::indices::{FuncIdx, LocalIdx};
use crate::core::reader::types::{FuncType, ValType};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::value::Value;
use crate::locals::Locals;
use crate::unreachable_validated;

/// The stack at runtime containing values
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
        // If there is at least one stack frame, we shall not pop values past the current
        // stackframe. However, there is one legitimate reason to pop when there is **no** current
        // stackframe: after the outermost function returns, to extract the final return values of
        // this interpreter invocation.
        debug_assert!(
            if !self.frames.is_empty() {
                self.values.len() > self.current_stackframe().value_stack_base_idx
            } else {
                true
            },
            "can not pop values past the current stackframe"
        );

        let popped = self.values.pop().unwrap_validated();
        if popped.to_ty() == ty {
            popped
        } else {
            unreachable_validated!()
        }
    }

    /// This unwinds the stack by popping the topmost `num_values_to_keep` values and storing them temporarily.
    /// Then the next topmost `num_values_to_remove` values are discarded before the previously popped values are pushed back to the stack.
    ///
    /// Example:
    /// ```
    /// BOTTOM                                                  TOP
    /// -----------------------------------------------------------
    /// | ... | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 1 | 2 | 3 | 4 |
    /// ---------------------------------------------------------
    ///           | num_values_to_remove  |   num_values_to_keep  |
    /// ```
    /// becomes
    ///
    /// ```
    /// BOTTOM                          TOP
    /// |----------------------------------
    /// | ... | 1 | 8 | 9 | 1 | 2 | 3 | 4 |
    /// |----------------------------------
    ///           |   num_values_to_keep  |
    /// ```
    ///
    // TODO Eventually this value stack should store raw bytes instead of enums on the stack. Then both `num_values` parameters should instead work with bytes.
    pub fn unwind(&mut self, num_values_to_keep: usize, num_values_to_remove: usize) {
        // FIXME: This is inefficient
        let mut temporary_values = Vec::new();

        for _ in 0..num_values_to_keep {
            temporary_values.push(self.values.pop().unwrap_validated());
        }

        for _ in 0..num_values_to_remove {
            self.values.pop().unwrap_validated();
        }

        // We should not have crossed a callframe boundary
        debug_assert!(
            self.frames
                .last()
                .map_or(true, |last_frame| self.values.len()
                    > last_frame.value_stack_base_idx),
            "can not pop values past the current stackframe"
        );

        for value in temporary_values.into_iter().rev() {
            self.values.push(value);
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
        debug_assert!(
            self.values.len() > self.current_stackframe().value_stack_base_idx,
            "can not pop values past the current stackframe"
        );

        trace!("Instruction: local.set [{stack_value:?}] -> []");
        *self.current_stackframe_mut().locals.get_mut(idx) = stack_value;
    }

    /// Copy value from top of the value stack to the given local
    pub fn tee_local(&mut self, idx: LocalIdx) {
        let local_ty = self.current_stackframe().locals.get_ty(idx);

        let stack_value = self.peek_value(local_ty);
        *self.current_stackframe_mut().locals.get_mut(idx) = stack_value;
    }

    /// Get a shared reference to the current [`CallFrame`]
    pub fn current_stackframe(&self) -> &CallFrame {
        self.frames.last().unwrap_validated()
    }

    /// Get a mutable reference to the current [`CallFrame`]
    pub fn current_stackframe_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap_validated()
    }

    /// Pop a [`CallFrame`] from the call stack, returning the return address
    pub fn pop_stackframe(&mut self) -> usize {
        let CallFrame {
            return_addr,
            value_stack_base_idx,
            return_value_count,
            ..
        } = self.frames.pop().unwrap_validated();

        let truncation_top = self.values.len() - return_value_count;
        let _ = self.values.drain(value_stack_base_idx..truncation_top);

        debug_assert_eq!(
            self.values.len(),
            value_stack_base_idx + return_value_count,
            "after a function call finished, the stack must have exactly as many values as it had before calling the function plus the number of function return values"
        );

        return_addr
    }

    /// Push a stackframe to the call stack
    ///
    /// Takes the current [`Self::values`]'s length as [`CallFrame::value_stack_base_idx`].
    pub fn push_stackframe(
        &mut self,
        func_idx: FuncIdx,
        func_ty: &FuncType,
        locals: Locals,
        return_addr: usize,
    ) {
        self.frames.push(CallFrame {
            func_idx,
            locals,
            return_addr,
            value_stack_base_idx: self.values.len(),
            return_value_count: func_ty.returns.valtypes.len(),
        })
    }

    /// Returns how many stackframes are on the stack, in total.
    pub fn callframe_count(&self) -> usize {
        self.frames.len()
    }

    /// Pop `n` elements from the value stack's tail as an iterator, with the first element being
    /// closest to the **bottom** of the value stack
    ///
    /// Note that this is providing the values in reverse order compared to popping `n` values
    /// (which would yield the element closest to the **top** of the value stack first).
    pub fn pop_tail_iter(&mut self, n: usize) -> Drain<Value> {
        let start = self.values.len() - n;
        self.values.drain(start..)
    }

    /// Clear all of the values pushed to the value stack by the current stack frame
    pub fn clear_callframe_values(&mut self) {
        self.values
            .truncate(self.current_stackframe().value_stack_base_idx);
    }
}

/// The [WASM spec](https://webassembly.github.io/spec/core/exec/runtime.html#stack) calls this `Activations`, however it refers to the call frames of functions.
pub(crate) struct CallFrame {
    /// Index to the function of this [`CallFrame`]
    pub func_idx: FuncIdx,

    /// Local variables such as parameters for this [`CallFrame`]'s function
    pub locals: Locals,

    /// Value that the PC has to be set to when this function returns
    pub return_addr: usize,

    /// The index to the first value on [`Stack::values`] that belongs to this [`CallFrame`]
    pub value_stack_base_idx: usize,

    /// Number of return values to retain on [`Stack::values`] when unwinding/popping a [`CallFrame`]
    pub return_value_count: usize,
}

#[test]
fn test_stack_unwind() {
    fn test_with_ten_example_numbers(num_keep: usize, num_pop: usize, expected: &[u32]) {
        let mut stack = Stack::new();
        for i in 0..10 {
            stack.push_value(Value::I32(i));
        }

        stack.unwind(num_keep, num_pop);

        let expected_values: Vec<Value> = expected.into_iter().copied().map(Value::I32).collect();

        assert_eq!(&stack.values, &expected_values);
    }

    test_with_ten_example_numbers(2, 3, &[0, 1, 2, 3, 4, 8, 9]);

    test_with_ten_example_numbers(0, 2, &[0, 1, 2, 3, 4, 5, 6, 7]);

    test_with_ten_example_numbers(0, 0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    test_with_ten_example_numbers(4, 0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    test_with_ten_example_numbers(10, 0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    test_with_ten_example_numbers(1, 9, &[9]);
}
