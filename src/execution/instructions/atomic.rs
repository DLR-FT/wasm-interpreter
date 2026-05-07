use core::ops::ControlFlow;

use crate::{
    execution::instructions::{define_instruction, InterpreterLoopOutcome, State},
    RuntimeError,
};

define_instruction!(
    super::memory_atomic_notify,
    memory_atomic_notify_mod,
    fuel_check = flat_fe(MEMORY_ATOMIC_NOTIFY)
);
#[inline(always)]
pub unsafe fn memory_atomic_notify(
    _: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    todo!()
}

define_instruction!(
    super::memory_atomic_wait32,
    memory_atomic_wait32_mod,
    fuel_check = flat_fe(MEMORY_ATOMIC_WAIT32)
);
#[inline(always)]
pub unsafe fn memory_atomic_wait32(
    _: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    todo!()
}

define_instruction!(
    super::memory_atomic_wait64,
    memory_atomic_wait64_mod,
    fuel_check = flat_fe(MEMORY_ATOMIC_WAIT64)
);
#[inline(always)]
pub unsafe fn memory_atomic_wait64(
    _: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    todo!()
}
