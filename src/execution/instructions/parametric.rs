#![expect(
    clippy::missing_safety_doc,
    reason = "see `instructions::State` for more information"
)]

use core::ops::ControlFlow;

use crate::{
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{InterpreterLoopOutcome, State},
    },
    RuntimeError, ValType,
};

#[inline(always)]
pub unsafe fn drop(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let _ = unsafe { state.resumable.stack.pop_value() };

    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn select(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let test_val: i32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let val2 = unsafe { state.resumable.stack.pop_value() };
    // SAFETY: Validation guarantees that there is a value on the stack.
    let val1 = unsafe { state.resumable.stack.pop_value() };
    if test_val != 0 {
        state.resumable.stack.push_value(val1)?;
    } else {
        state.resumable.stack.push_value(val2)?;
    }
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn select_t(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // skip past type vec
    state
        .wasm
        .decode_vec_map(ValType::decode)
        .unwrap_validated()
        .for_each(|_| {});
    // SAFETY: Validation guarantees that there is a value on the stack.
    let test_val: i32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let val2 = unsafe { state.resumable.stack.pop_value() };
    // SAFETY: Validation guarantees that there is a value on the stack.
    let val1 = unsafe { state.resumable.stack.pop_value() };
    if test_val != 0 {
        state.resumable.stack.push_value(val1)?;
    } else {
        state.resumable.stack.push_value(val2)?;
    }
    Ok(ControlFlow::Continue(()))
}
