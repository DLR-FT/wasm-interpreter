#![expect(
    clippy::missing_safety_doc,
    reason = "see `instructions::State` for more information"
)]

use core::ops::ControlFlow;

use crate::{
    core::structure::modules::indices::FuncIdx,
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction, InterpreterLoopOutcome, State},
    },
    Ref, RefType, RuntimeError, Value,
};

define_instruction!(super::ref_null, ref_null_mod, fuel_check = flat(REF_NULL));
#[inline(always)]
pub unsafe fn ref_null(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let reftype = RefType::decode(state.wasm).unwrap_validated();

    state
        .resumable
        .stack
        .push_value(Value::Ref(Ref::Null(reftype)))?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::ref_is_null,
    ref_is_null_mod,
    fuel_check = flat(REF_IS_NULL)
);
#[inline(always)]
pub unsafe fn ref_is_null(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a ref value on the stack.
    let rref: Ref = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let is_null = matches!(rref, Ref::Null(_));

    let res = if is_null { 1 } else { 0 };
    state.resumable.stack.push_value(Value::I32(res))?;
    Ok(ControlFlow::Continue(()))
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
define_instruction!(super::ref_func, ref_func_mod, fuel_check = flat(REF_FUNC));
#[inline(always)]
pub unsafe fn ref_func(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees a valid function index to be
    // next.
    let func_idx = unsafe { FuncIdx::decode_unchecked(state.wasm) };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let current_module = unsafe { state.modules.get(*state.current_module) };
    // SAFETY: Validation guarantees the function index to be valid
    // in the current module.
    let func_addr = unsafe { current_module.func_addrs.get(func_idx) };
    state
        .resumable
        .stack
        .push_value(Value::Ref(Ref::Func(*func_addr)))?;
    Ok(ControlFlow::Continue(()))
}
