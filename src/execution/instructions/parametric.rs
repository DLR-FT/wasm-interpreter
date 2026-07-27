use core::ops::ControlFlow;

use crate::{
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction, Args, InterpreterLoopOutcome},
    },
    trace, RuntimeError, ValType,
};

define_instruction!(super::drop, drop_mod, fuel_check = flat(DROP));
#[inline(always)]
pub unsafe fn drop(args: Args) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    args.resumable.stack.pop_value();
    trace!("Instruction: DROP");

    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::select, select_mod, fuel_check = flat(SELECT));
#[inline(always)]
pub unsafe fn select(args: Args) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let test_val: i32 = args
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let val2 = args.resumable.stack.pop_value();
    let val1 = args.resumable.stack.pop_value();
    if test_val != 0 {
        args.resumable.stack.push_value(val1)?;
    } else {
        args.resumable.stack.push_value(val2)?;
    }
    trace!("Instruction: SELECT");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::select_t, select_t_mod, fuel_check = flat(SELECT_T));
#[inline(always)]
pub unsafe fn select_t(args: Args) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let _type_vec = args.wasm.decode_vec(ValType::decode).unwrap_validated();
    let test_val: i32 = args
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let val2 = args.resumable.stack.pop_value();
    let val1 = args.resumable.stack.pop_value();
    if test_val != 0 {
        args.resumable.stack.push_value(val1)?;
    } else {
        args.resumable.stack.push_value(val2)?;
    }
    trace!("Instruction: SELECT_T");
    Ok(ControlFlow::Continue(()))
}
