use core::ops::ControlFlow;

use crate::{
    core::{
        decoding::decoder_ptr::decode_label_idx_unchecked,
        structure::{
            instructions,
            modules::indices::{FuncIdx, TableIdx, TypeIdx},
            types::BlockType,
        },
        utils::ToUsizeExt,
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{
            define_instruction_fn, do_sidetable_control_transfer, Args, InterpreterLoopOutcome,
        },
        runtime_structure::function_instances::FuncInst,
    },
    trace, unreachable_validated, DecodingError, Ref, TrapError,
};

define_instruction_fn! {nop, fuel_check = flat(instructions::NOP), |_args| Ok(
    ControlFlow::Continue(())
)}

define_instruction_fn! {
    unreachable,
    fuel_check = flat(instructions::UNREACHABLE),
    |Args { .. }| { Err(TrapError::ReachedUnreachable.into()) }
}

define_instruction_fn! {
    block,
    fuel_check = flat(instructions::BLOCK),
    |Args { wasm, .. }| {
        // SAFETY: Validation guarantess there to be a valid block type
        // next.
        let _ = unsafe { BlockType::decode_unchecked_ptr(wasm) };
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    end,
    fuel_check = flat(instructions::END),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         current_function_end_marker,
         current_sidetable,
         ..
     }| {
         trace!("handling end. marker={current_function_end_marker:?}");
        // There might be multiple ENDs in a single function. We want to
        // exit only when the outermost block (aka function block) ends.
        if wasm.0 != *current_function_end_marker {
            return Ok(ControlFlow::Continue(()));
        }

        let Some((maybe_return_func_addr, maybe_return_address, maybe_return_stp)) =
            resumable.stack.pop_call_frame()
        else {
            // We finished this entire invocation if this was the base call frame.
            return Ok(ControlFlow::Break(
                InterpreterLoopOutcome::ExecutionReturned,
            ));
        };
        // If there are one or more call frames, we need to continue
        // from where the callee was called from.

        trace!("end of function reached, returning to previous call frame");
        resumable.current_func_addr = maybe_return_func_addr;

        // SAFETY: The current function address must come from the given
        // resumable or the current store, because these are the only
        // parameters to this function. The resumable, including its
        // function address, is guaranteed to be valid in the current
        // store by the caller, and the store can only contain addresses
        // that are valid within itself.
        let current_function = unsafe { store_inner.functions.get(resumable.current_func_addr) };
        let FuncInst::WasmFunc(current_wasm_func_inst) = current_function else {
            unreachable!(
                "function addresses on the stack always correspond to native wasm functions"
            )
        };
        *current_module = current_wasm_func_inst.module_addr;

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        wasm.0 = module.wasm_bytecode.as_ptr().wrapping_add(maybe_return_address);
        resumable.stp = maybe_return_stp;

        *current_sidetable = &module.sidetable;

        *current_function_end_marker =
            module.wasm_bytecode.as_ptr().wrapping_add(current_wasm_func_inst.code_expr.from() + current_wasm_func_inst.code_expr.len());

        trace!("Instruction: END");

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    r#loop,
    fuel_check = flat(instructions::LOOP),
    |Args { wasm, .. }| {
        // SAFETY: Validation guarantees there to be a valid block type
        // next.
        let _ = unsafe { BlockType::decode_unchecked_ptr(wasm) };
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    r#if,
    fuel_check = flat(instructions::IF),
    |Args {
         resumable,
         wasm,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid block type
        // next.
        let _block_type = unsafe { BlockType::decode_unchecked_ptr(wasm) };

        let test_val: i32 = resumable.stack.pop_value().try_into().unwrap_validated();

        if test_val != 0 {
            resumable.stp += 1;
        } else {
            do_sidetable_control_transfer(
                wasm,
                &mut resumable.stack,
                &mut resumable.stp,
                current_sidetable,
            )?;
        }
        trace!("Instruction: IF");

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    r#else,
    fuel_check = flat(instructions::ELSE),
    |Args {
         wasm,
         resumable,
         current_sidetable,
         ..
     }| {
        do_sidetable_control_transfer(
            wasm,
            &mut resumable.stack,
            &mut resumable.stp,
            current_sidetable,
        )?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    br,
    fuel_check = flat(instructions::BR),
    |Args {
         resumable,
         wasm,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid label index
        // next.
        let _label_idx = unsafe { decode_label_idx_unchecked(wasm) };
        do_sidetable_control_transfer(
            wasm,
            &mut resumable.stack,
            &mut resumable.stp,
            current_sidetable,
        )?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    br_if,
    fuel_check = flat(instructions::BR_IF),
    |Args {
         resumable,
         wasm,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid label index
        // next.
        let _label_idx = unsafe { decode_label_idx_unchecked(wasm) };

        let test_val: i32 = resumable.stack.pop_value().try_into().unwrap_validated();

        if test_val != 0 {
            do_sidetable_control_transfer(
                wasm,
                &mut resumable.stack,
                &mut resumable.stp,
                current_sidetable,
            )?;
        } else {
            resumable.stp += 1;
        }
        trace!("Instruction: BR_IF");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    br_table,
    fuel_check = flat(instructions::BR_TABLE),
    |Args {
         resumable,
         wasm,
         current_sidetable,
         ..
     }| {
        let label_vec = wasm
            .decode_vec(|wasm| {
                // SAFETY: Validation guarantees that there is a
                // valid vec of label indices.
                unsafe { decode_label_idx_unchecked(wasm) }
            });

        // SAFETY: Validation guarantees there to be another label index
        // for the default case.
        let _default_label_idx = unsafe { decode_label_idx_unchecked(wasm) };

        // TODO is this correct?
        let case_val_i32: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let case_val = case_val_i32.cast_unsigned().into_usize();

        if case_val >= label_vec.len() {
            resumable.stp += label_vec.len();
        } else {
            resumable.stp += case_val;
        }

        do_sidetable_control_transfer(
            wasm,
            &mut resumable.stack,
            &mut resumable.stp,
            current_sidetable,
        )?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    r#return,
    fuel_check = flat(instructions::RETURN),
    |Args {
         resumable,
         wasm,
         current_sidetable,
         ..
     }| {
        // same as BR
        do_sidetable_control_transfer(
            wasm,
            &mut resumable.stack,
            &mut resumable.stp,
            current_sidetable,
        )?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    call,
    fuel_check = flat(instructions::CALL),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         current_function_end_marker,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid function
        // index next.
        let func_idx = unsafe { FuncIdx::decode_unchecked_ptr(wasm) };

        // SAFETY: The current function address must come from the given
        // resumable or the current store, because these are the only
        // parameters to this function. The resumable, including its
        // function address, is guaranteed to be valid in the current
        // store by the caller, and the store can only contain addresses
        // that are valid within itself.
        let FuncInst::WasmFunc(current_wasm_func_inst) =
            (unsafe { store_inner.functions.get(resumable.current_func_addr) })
        else {
            unreachable!()
        };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let current_module_inst = unsafe { modules.get(current_wasm_func_inst.module_addr) };

        // SAFETY: Validation guarantees the function index to be
        // valid in the current module.
        let func_to_call_addr = unsafe { current_module_inst.func_addrs.get(func_idx) };

        // SAFETY: This function address just came from the current
        // store. Therefore, it must be valid in the current store.
        let func_to_call_inst = unsafe { store_inner.functions.get(*func_to_call_addr) };

        trace!("Instruction: call [{func_to_call_addr:?}]");

        match func_to_call_inst {
            FuncInst::HostFunc(host_func_to_call_inst) => {
                let params = resumable
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

                let pc = unsafe { wasm.0.offset_from_unsigned(current_module_inst.wasm_bytecode.as_ptr()) };

                resumable.stack.push_call_frame::<T>(
                    resumable.current_func_addr,
                    &wasm_func_to_call_inst.function_type,
                    remaining_locals,
                    pc,
                    resumable.stp,
                )?;

                resumable.current_func_addr = *func_to_call_addr;
                *current_module = wasm_func_to_call_inst.module_addr;

                // SAFETY: The current module address was just set to an
                // address that came from the current store. Therefore,
                // this address must automatically be valid in the
                // current store.
                let module = unsafe { modules.get(*current_module) };

                wasm.0 = module.wasm_bytecode.as_ptr().wrapping_add(wasm_func_to_call_inst.code_expr.from);

                resumable.stp = wasm_func_to_call_inst.stp;
                *current_sidetable = &module.sidetable;
                *current_function_end_marker = module.wasm_bytecode.as_ptr().wrapping_add(wasm_func_to_call_inst.code_expr.from + wasm_func_to_call_inst.code_expr.len);
            }
        }
        trace!("Instruction: CALL");

        Ok(ControlFlow::Continue(()))
    }
}

// TODO: fix push_call_frame, because the func idx that you get from the table is global func idx
define_instruction_fn! {
    call_indirect,
    fuel_check = flat(instructions::CALL_INDIRECT),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         current_function_end_marker,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid type index
        // next.
        let given_type_idx = unsafe { TypeIdx::decode_unchecked_ptr(wasm) };
        // SAFETY: Validation guarantees there to be a valid table index
        // next.
        let table_idx = unsafe { TableIdx::decode_unchecked_ptr(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be valid in
        // the current module.
        let table_addr = unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the current
        // store. Therefore, it is valid in the current store.
        let tab = unsafe { store_inner.tables.get(*table_addr) };
        // SAFETY: Validation guarantees the type index to be valid in
        // the current module.
        let func_ty = unsafe { module.types.get(given_type_idx) };

        let i: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
        let func_to_call_inst = unsafe { store_inner.functions.get(func_to_call_addr) };

        if func_ty != func_to_call_inst.ty() {
            return Err(TrapError::SignatureMismatch.into());
        }

        trace!("Instruction: call [{func_to_call_addr:?}]");

        match func_to_call_inst {
            FuncInst::HostFunc(host_func_to_call_inst) => {
                let params = resumable
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

                let pc = unsafe { wasm.0.offset_from_unsigned(module.wasm_bytecode.as_ptr()) };

                resumable.stack.push_call_frame::<T>(
                    resumable.current_func_addr,
                    &wasm_func_to_call_inst.function_type,
                    remaining_locals,
                    pc,
                    resumable.stp,
                )?;

                resumable.current_func_addr = func_to_call_addr;
                *current_module = wasm_func_to_call_inst.module_addr;

                // SAFETY: The current module address was just set to an
                // address that came from the current store. Therefore,
                // this address must automatically be valid in the
                // current store.
                let module = unsafe { modules.get(*current_module) };
                wasm.0 = module.wasm_bytecode.as_ptr().wrapping_add(wasm_func_to_call_inst.code_expr.from);

                resumable.stp = wasm_func_to_call_inst.stp;
                *current_sidetable = &module.sidetable;
                *current_function_end_marker = module.wasm_bytecode.as_ptr().wrapping_add(wasm_func_to_call_inst.code_expr.from + wasm_func_to_call_inst.code_expr.len);


            }
        }
        trace!("Instruction: CALL_INDIRECT");
        Ok(ControlFlow::Continue(()))
    }
}
