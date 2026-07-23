#![expect(
    clippy::missing_safety_doc,
    reason = "see `instructions::State` for more information"
)]

use core::ops::ControlFlow;

use crate::{
    core::{
        decoding::modules::indices::decode_label_idx_unchecked,
        structure::{
            modules::indices::{FuncIdx, TableIdx, TypeIdx},
            types::BlockType,
        },
        utils::ToUsizeExt,
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{
            define_instruction, do_sidetable_control_transfer, InterpreterLoopOutcome, State,
        },
        runtime_structure::function_instances::FuncInst,
    },
    trace, unreachable_validated, Config, DecodingError, Ref, RuntimeError, TrapError,
};

define_instruction!(super::nop, nop_mod, fuel_check = flat(NOP));
#[inline(always)]
pub unsafe fn nop(_: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::unreachable,
    unreachable_mod,
    fuel_check = flat(UNREACHABLE)
);
#[inline(always)]
pub unsafe fn unreachable(_: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    Err(TrapError::ReachedUnreachable.into())
}

define_instruction!(super::block, block_mod, fuel_check = flat(BLOCK));
#[inline(always)]
pub unsafe fn block(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantess there to be a valid block type
    // next.
    let _ = unsafe { BlockType::decode_unchecked(state.wasm) };
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::end, end_mod, fuel_check = flat(END));
#[inline(always)]
pub unsafe fn end(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // There might be multiple ENDs in a single function. We want to
    // exit only when the outermost block (aka function block) ends.
    if state.wasm.pc != *state.current_function_end_marker {
        return Ok(ControlFlow::Continue(()));
    }

    let Some((maybe_return_func_addr, maybe_return_address, maybe_return_stp)) =
        state.resumable.stack.pop_call_frame()
    else {
        // We finished this entire invocation if this was the base call frame.
        return Ok(ControlFlow::Break(
            InterpreterLoopOutcome::ExecutionReturned,
        ));
    };
    // If there are one or more call frames, we need to continue
    // from where the callee was called from.

    trace!("end of function reached, returning to previous call frame");
    state.resumable.current_func_addr = maybe_return_func_addr;

    // SAFETY: The current function address must come from the given
    // resumable or the current store, because these are the only
    // parameters to this function. The resumable, including its
    // function address, is guaranteed to be valid in the current
    // store by the caller, and the store can only contain addresses
    // that are valid within itself.
    let current_function = unsafe {
        state
            .store_inner
            .functions
            .get(state.resumable.current_func_addr)
    };
    let FuncInst::WasmFunc(current_wasm_func_inst) = current_function else {
        unreachable!("function addresses on the stack always correspond to native wasm functions")
    };
    *state.current_module = current_wasm_func_inst.module_addr;

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    state.wasm.full_wasm_binary = module.wasm_bytecode;
    state.wasm.pc = maybe_return_address;
    state.resumable.stp = maybe_return_stp;

    *state.current_sidetable = &module.sidetable;

    *state.current_function_end_marker =
        current_wasm_func_inst.code_expr.from() + current_wasm_func_inst.code_expr.len();

    trace!("Instruction: END");

    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::r#loop, r#loop_mod, fuel_check = flat(LOOP));
#[inline(always)]
pub unsafe fn r#loop(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid block type
    // next.
    let _ = unsafe { BlockType::decode_unchecked(state.wasm) };
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::r#if, r#if_mod, fuel_check = flat(IF));
#[inline(always)]
pub unsafe fn r#if(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid block type
    // next.
    let _block_type = unsafe { BlockType::decode_unchecked(state.wasm) };

    let test_val: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if test_val != 0 {
        state.resumable.stp += 1;
    } else {
        do_sidetable_control_transfer(
            state.wasm,
            &mut state.resumable.stack,
            &mut state.resumable.stp,
            state.current_sidetable,
        )?;
    }
    trace!("Instruction: IF");

    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::r#else, r#else_mod, fuel_check = flat(ELSE));
#[inline(always)]
pub unsafe fn r#else(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    do_sidetable_control_transfer(
        state.wasm,
        &mut state.resumable.stack,
        &mut state.resumable.stp,
        state.current_sidetable,
    )?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::br, br_mod, fuel_check = flat(BR));
#[inline(always)]
pub unsafe fn br(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid label index
    // next.
    let _label_idx = unsafe { decode_label_idx_unchecked(state.wasm) };
    do_sidetable_control_transfer(
        state.wasm,
        &mut state.resumable.stack,
        &mut state.resumable.stp,
        state.current_sidetable,
    )?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::br_if, br_if_mod, fuel_check = flat(BR_IF));
#[inline(always)]
pub unsafe fn br_if(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid label index
    // next.
    let _label_idx = unsafe { decode_label_idx_unchecked(state.wasm) };

    let test_val: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if test_val != 0 {
        do_sidetable_control_transfer(
            state.wasm,
            &mut state.resumable.stack,
            &mut state.resumable.stp,
            state.current_sidetable,
        )?;
    } else {
        state.resumable.stp += 1;
    }
    trace!("Instruction: BR_IF");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::br_table, br_table_mod, fuel_check = flat(BR_TABLE));
#[inline(always)]
pub unsafe fn br_table(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let label_vec = state
        .wasm
        .decode_vec::<_, _, DecodingError>(|wasm| {
            // SAFETY: Validation guarantees that there is a
            // valid vec of label indices.
            Ok(unsafe { decode_label_idx_unchecked(wasm) })
        })
        .unwrap();

    // SAFETY: Validation guarantees there to be another label index
    // for the default case.
    let _default_label_idx = unsafe { decode_label_idx_unchecked(state.wasm) };

    // TODO is this correct?
    let case_val_i32: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let case_val = case_val_i32.cast_unsigned().into_usize();

    if case_val >= label_vec.len() {
        state.resumable.stp += label_vec.len();
    } else {
        state.resumable.stp += case_val;
    }

    do_sidetable_control_transfer(
        state.wasm,
        &mut state.resumable.stack,
        &mut state.resumable.stp,
        state.current_sidetable,
    )?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::r#return, r#return_mod, fuel_check = flat(RETURN));
#[inline(always)]
pub unsafe fn r#return(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // same as BR
    do_sidetable_control_transfer(
        state.wasm,
        &mut state.resumable.stack,
        &mut state.resumable.stp,
        state.current_sidetable,
    )?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::call::<T>, call_mod, fuel_check = flat(CALL));
#[inline(always)]
pub unsafe fn call<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid function
    // index next.
    let func_idx = unsafe { FuncIdx::decode_unchecked(state.wasm) };

    // SAFETY: The current function address must come from the given
    // resumable or the current store, because these are the only
    // parameters to this function. The resumable, including its
    // function address, is guaranteed to be valid in the current
    // store by the caller, and the store can only contain addresses
    // that are valid within itself.
    let FuncInst::WasmFunc(current_wasm_func_inst) = (unsafe {
        state
            .store_inner
            .functions
            .get(state.resumable.current_func_addr)
    }) else {
        unreachable!()
    };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let current_module_inst = unsafe { state.modules.get(current_wasm_func_inst.module_addr) };

    // SAFETY: Validation guarantees the function index to be
    // valid in the current module.
    let func_to_call_addr = unsafe { current_module_inst.func_addrs.get(func_idx) };

    // SAFETY: This function address just came from the current
    // store. Therefore, it must be valid in the current store.
    let func_to_call_inst = unsafe { state.store_inner.functions.get(*func_to_call_addr) };

    trace!("Instruction: call [{func_to_call_addr:?}]");

    match func_to_call_inst {
        FuncInst::HostFunc(host_func_to_call_inst) => {
            let params = state
                .resumable
                .stack
                .pop_tail_iter(host_func_to_call_inst.function_type.params.valtypes.len());

            return Ok(ControlFlow::Break(InterpreterLoopOutcome::HostCalled {
                params,
                func_addr: *func_to_call_addr,
                hostcode: host_func_to_call_inst.hostcode,
            }));
        }
        FuncInst::WasmFunc(wasm_func_to_call_inst) => {
            let remaining_locals = &wasm_func_to_call_inst.locals;

            state.resumable.stack.push_call_frame::<T>(
                state.resumable.current_func_addr,
                &wasm_func_to_call_inst.function_type,
                remaining_locals,
                state.wasm.pc,
                state.resumable.stp,
            )?;

            state.resumable.current_func_addr = *func_to_call_addr;
            *state.current_module = wasm_func_to_call_inst.module_addr;

            // SAFETY: The current module address was just set to an
            // address that came from the current store. Therefore,
            // this address must automatically be valid in the
            // current store.
            let module = unsafe { state.modules.get(*state.current_module) };

            state.wasm.full_wasm_binary = module.wasm_bytecode;
            state
                .wasm
                .move_start_to(wasm_func_to_call_inst.code_expr)
                .expect("code expression spans to always be valid");

            state.resumable.stp = wasm_func_to_call_inst.stp;
            *state.current_sidetable = &module.sidetable;
            *state.current_function_end_marker =
                wasm_func_to_call_inst.code_expr.from() + wasm_func_to_call_inst.code_expr.len();
        }
    }
    trace!("Instruction: CALL");

    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::call_indirect::<T>,
    call_indirect_mod,
    fuel_check = flat(CALL_INDIRECT)
);
#[inline(always)]
pub unsafe fn call_indirect<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid type index
    // next.
    let given_type_idx = unsafe { TypeIdx::decode_unchecked(state.wasm) };
    // SAFETY: Validation guarantees there to be a valid table index
    // next.
    let table_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the table index to be valid in
    // the current module.
    let table_addr = unsafe { module.table_addrs.get(table_idx) };
    // SAFETY: This table address was just read from the current
    // store. Therefore, it is valid in the current store.
    let tab = unsafe { state.store_inner.tables.get(*table_addr) };
    // SAFETY: Validation guarantees the type index to be valid in
    // the current module.
    let func_ty = unsafe { module.types.get(given_type_idx) };

    let i: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let r = tab
        .elem
        .get(i.into_usize())
        .ok_or(TrapError::TableAccessOutOfBounds)
        .and_then(|r| {
            if matches!(r, Ref::Null(_)) {
                trace!("table_idx ({table_idx}) --- element index in table ({i})");
                Err(TrapError::UninitializedElement)
            } else {
                Ok(r)
            }
        })?;

    let func_to_call_addr = match *r {
        Ref::Func(func_addr) => func_addr,
        Ref::Null(_) => return Err(TrapError::IndirectCallNullFuncRef.into()),
        Ref::Extern(_) => unreachable_validated!(),
    };

    // SAFETY: This function address just came from a table of the
    // current store. Therefore, it must be valid in the current
    // store.
    let func_to_call_inst = unsafe { state.store_inner.functions.get(func_to_call_addr) };

    if func_ty != func_to_call_inst.ty() {
        return Err(TrapError::SignatureMismatch.into());
    }

    trace!("Instruction: call [{func_to_call_addr:?}]");

    match func_to_call_inst {
        FuncInst::HostFunc(host_func_to_call_inst) => {
            let params = state
                .resumable
                .stack
                .pop_tail_iter(host_func_to_call_inst.function_type.params.valtypes.len());

            return Ok(ControlFlow::Break(InterpreterLoopOutcome::HostCalled {
                params,
                func_addr: func_to_call_addr,
                hostcode: host_func_to_call_inst.hostcode,
            }));
        }
        FuncInst::WasmFunc(wasm_func_to_call_inst) => {
            let remaining_locals = &wasm_func_to_call_inst.locals;

            state.resumable.stack.push_call_frame::<T>(
                state.resumable.current_func_addr,
                &wasm_func_to_call_inst.function_type,
                remaining_locals,
                state.wasm.pc,
                state.resumable.stp,
            )?;

            state.resumable.current_func_addr = func_to_call_addr;
            *state.current_module = wasm_func_to_call_inst.module_addr;

            // SAFETY: The current module address was just set to an
            // address that came from the current store. Therefore,
            // this address must automatically be valid in the
            // current store.
            let module = unsafe { state.modules.get(*state.current_module) };
            state.wasm.full_wasm_binary = module.wasm_bytecode;
            state
                .wasm
                .move_start_to(wasm_func_to_call_inst.code_expr)
                .expect("code expression spans to always be valid");

            state.resumable.stp = wasm_func_to_call_inst.stp;
            *state.current_sidetable = &module.sidetable;
            *state.current_function_end_marker =
                wasm_func_to_call_inst.code_expr.from() + wasm_func_to_call_inst.code_expr.len();
        }
    }
    trace!("Instruction: CALL_INDIRECT");
    Ok(ControlFlow::Continue(()))
}
