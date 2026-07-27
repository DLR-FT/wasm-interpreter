use core::ops::ControlFlow;

use crate::{
    core::structure::modules::indices::FuncIdx,
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction, Args, InterpreterLoopOutcome},
    },
    trace, Ref, RefType, RuntimeError, Value,
};

define_instruction!(super::ref_null, ref_null_mod, fuel_check = flat(REF_NULL));
#[inline(always)]
pub unsafe fn ref_null(args: Args) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let reftype = RefType::decode(args.wasm).unwrap_validated();

    args.resumable
        .stack
        .push_value(Value::Ref(Ref::Null(reftype)))?;
    trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::ref_is_null,
    ref_is_null_mod,
    fuel_check = flat(REF_IS_NULL)
);
#[inline(always)]
pub unsafe fn ref_is_null(args: Args) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let rref: Ref = args
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let is_null = matches!(rref, Ref::Null(_));

    let res = if is_null { 1 } else { 0 };
    trace!("Instruction: ref.is_null [{}] -> [{}]", rref, res);
    args.resumable.stack.push_value(Value::I32(res))?;
    Ok(ControlFlow::Continue(()))
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
define_instruction!(super::ref_func, ref_func_mod, fuel_check = flat(REF_FUNC));
#[inline(always)]
pub unsafe fn ref_func(args: Args) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees a valid function index to be
    // next.
    let func_idx = unsafe { FuncIdx::decode_unchecked(args.wasm) };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let current_module = unsafe { args.modules.get(*args.current_module) };
    // SAFETY: Validation guarantees the function index to be valid
    // in the current module.
    let func_addr = unsafe { current_module.func_addrs.get(func_idx) };
    args.resumable
        .stack
        .push_value(Value::Ref(Ref::Func(*func_addr)))?;
    Ok(ControlFlow::Continue(()))
}
