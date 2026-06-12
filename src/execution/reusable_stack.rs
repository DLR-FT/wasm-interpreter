//! TODO What is this module?

use alloc::vec::Vec;

use crate::{
    execution::runtime_structure::value_stack::Stack, Config, FuncType, RuntimeError, ValType,
    Value,
};

/// A stack that is in an unknown state, but that can be re-used for future module instantiations
/// and resumables..
// Note: Making the inner stack private is a deliberate decision, as it also stops our crate from
// accessing it without proper initialization.
pub struct ReusableStack(Stack);

impl ReusableStack {
    pub(crate) fn new(stack: Stack) -> Self {
        ReusableStack(stack)
    }
}

pub(crate) fn take_and_reuse_or_init_stack<T: Config>(
    maybe_reusable_stack: &mut Option<ReusableStack>,
    params_to_base_call_frame: Vec<Value>,
    base_call_frame_func_ty: &FuncType,
    base_call_frame_remaining_locals: &[ValType],
) -> Result<Stack, RuntimeError> {
    match maybe_reusable_stack.take() {
        Some(mut existing_stack) => {
            existing_stack.0.clear_and_reinitialize(
                params_to_base_call_frame,
                base_call_frame_func_ty,
                base_call_frame_remaining_locals,
            )?;

            Ok(existing_stack.0)
        }
        None => Stack::new::<T>(
            params_to_base_call_frame,
            base_call_frame_func_ty,
            base_call_frame_remaining_locals,
        ),
    }
}

pub(crate) fn reuse_or_init_stack_mut<'a, T: Config>(
    maybe_reusable_stack: &'a mut Option<ReusableStack>,
    params_to_base_call_frame: Vec<Value>,
    base_call_frame_func_ty: &FuncType,
    base_call_frame_remaining_locals: &[ValType],
) -> Result<&'a mut Stack, RuntimeError> {
    match maybe_reusable_stack {
        Some(existing_stack) => {
            existing_stack.0.clear_and_reinitialize(
                params_to_base_call_frame,
                base_call_frame_func_ty,
                base_call_frame_remaining_locals,
            )?;

            Ok(&mut existing_stack.0)
        }
        empty_option @ None => {
            let new_stack = Stack::new::<T>(
                params_to_base_call_frame,
                base_call_frame_func_ty,
                base_call_frame_remaining_locals,
            )?;
            let inserted_stack = empty_option.insert(ReusableStack(new_stack));

            Ok(&mut inserted_stack.0)
        }
    }
}
