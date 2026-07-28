#![expect(
    clippy::missing_safety_doc,
    reason = "see `instructions::State` for more information"
)]

use core::ops::ControlFlow;

use crate::{
    core::structure::modules::indices::{GlobalIdx, LocalIdx},
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction, InterpreterLoopOutcome, State},
    },
    trace, RuntimeError,
};

define_instruction!(
    super::local_get,
    local_get_mod,
    fuel_check = flat(LOCAL_GET)
);
#[inline(always)]
pub unsafe fn local_get(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid local index
    // next.
    let local_idx = unsafe { LocalIdx::decode_unchecked(state.wasm) };
    let value = *state.resumable.stack.get_local(local_idx);
    state.resumable.stack.push_value(value)?;
    trace!("Instruction: local.get {} [] -> [t]", local_idx);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::local_set,
    local_set_mod,
    fuel_check = flat(LOCAL_SET)
);
#[inline(always)]
pub unsafe fn local_set(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid local index
    // next.
    let local_idx = unsafe { LocalIdx::decode_unchecked(state.wasm) };
    let value = state.resumable.stack.pop_value();
    *state.resumable.stack.get_local_mut(local_idx) = value;
    trace!("Instruction: local.set {} [t] -> []", local_idx);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::local_tee,
    local_tee_mod,
    fuel_check = flat(LOCAL_TEE)
);
#[inline(always)]
pub unsafe fn local_tee(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid local index
    // next.
    let local_idx = unsafe { LocalIdx::decode_unchecked(state.wasm) };
    let value = state.resumable.stack.peek_value().unwrap_validated();
    *state.resumable.stack.get_local_mut(local_idx) = value;
    trace!("Instruction: local.tee {} [t] -> [t]", local_idx);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::global_get,
    global_get_mod,
    fuel_check = flat(GLOBAL_GET)
);
#[inline(always)]
pub unsafe fn global_get(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid global
    // index next.
    let global_idx = unsafe { GlobalIdx::decode_unchecked(state.wasm) };
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the global index to be valid in
    // the current module.
    let global_addr = *unsafe { module.global_addrs.get(global_idx) };
    // SAFETY: This global address was just read from the current
    // store. Therefore, it is valid in the current store.
    let global = unsafe { state.store_inner.globals.get(global_addr) };

    state.resumable.stack.push_value(global.value)?;

    trace!(
        "Instruction: global.get '{}' [<GLOBAL>] -> [{:?}]",
        global_idx,
        global.value
    );
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::global_set,
    global_set_mod,
    fuel_check = flat(GLOBAL_SET)
);
#[inline(always)]
pub unsafe fn global_set(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid global
    // index next.
    let global_idx = unsafe { GlobalIdx::decode_unchecked(state.wasm) };
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };
    // SAFETY: Validation guarantees the global index to be valid in
    // the current module.
    let global_addr = *unsafe { module.global_addrs.get(global_idx) };
    // SAFETY: This global address was just read from the current
    // store. Therefore, it is valid in the current store.
    let global = unsafe { state.store_inner.globals.get_mut(global_addr) };

    global.value = state.resumable.stack.pop_value();
    trace!("Instruction: GLOBAL_SET");
    Ok(ControlFlow::Continue(()))
}
