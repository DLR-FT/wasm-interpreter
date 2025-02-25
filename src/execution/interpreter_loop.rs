//! This module solely contains the actual interpretation loop that matches instructions, interpreting the WASM bytecode
//!
//!
//! # Note to Developer:
//!
//! 1. There must be only imports and one `impl` with one function (`run`) in it.
//! 2. This module must not use the [`Error`](crate::core::error::Error) enum.
//! 3. Instead, only the [`RuntimeError`] enum shall be used
//!    - **not** to be confused with the [`Error`](crate::core::error::Error) enum's
//!      [`Error::RuntimeError`](crate::Error::RuntimeError) variant, which as per 2., we don not
//!      want

use alloc::vec;
use alloc::vec::Vec;

use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{DataIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, TableIdx, TypeIdx},
        reader::{
            span::Span,
            types::{memarg::MemArg, BlockType},
            WasmReadable, WasmReader,
        },
        sidetable::Sidetable,
    },
    locals::Locals,
    store::{DataInst, FuncInst},
    value::{self, FuncAddr, Ref},
    value_stack::Stack,
    Limits, NumType, RefType, RuntimeError, ValType, Value,
};

#[cfg(feature = "hooks")]
use crate::execution::hooks::HookSet;

use super::{execution_info::ExecutionInfo, /* lut::Lut, */ store::Store};

/// Interprets a functions. Parameters and return values are passed on the stack.
pub(super) fn run<H: HookSet>(
    // modules: &mut [ExecutionInfo],
    current_module_idx: &mut usize,
    // lut: &Lut,
    stack: &mut Stack,
    mut hooks: H,
    store: &mut Store,
) -> Result<(), RuntimeError> {
    let func_inst = store
        .functions
        .get(stack.current_stackframe().func_idx)
        .unwrap_validated()
        .try_into_local()
        .unwrap_validated();

    // Start reading the function's instructions
    let mut current_wasm_index = *current_module_idx;
    // let mut wasm = &mut modules[*current_module_idx].wasm_reader;

    // let mut wasm = &mut store.modules[*current_module_idx].wasm_reader;

    // the sidetable and stp for this function, stp will reset to 0 every call
    // since function instances have their own sidetable.
    let mut current_sidetable: &Sidetable = &func_inst.sidetable;
    let mut stp = 0;

    // unwrap is sound, because the validation assures that the function points to valid subslice of the WASM binary
    store.modules[current_wasm_index]
        .wasm_reader
        .move_start_to(func_inst.code_expr)
        .unwrap();

    use crate::core::reader::types::opcode::*;
    loop {
        // call the instruction hook
        #[cfg(feature = "hooks")]
        // hooks.instruction_hook(modules[*current_module_idx].wasm_bytecode, wasm.pc);
        hooks.instruction_hook(
            store.modules[*current_module_idx].wasm_bytecode,
            store.modules[current_wasm_index].wasm_reader.pc,
        );

        let first_instr_byte = store.modules[current_wasm_index]
            .wasm_reader
            .read_u8()
            .unwrap_validated();

        #[cfg(debug_assertions)]
        trace!(
            "Executing instruction {}",
            opcode_byte_to_str(first_instr_byte)
        );

        match first_instr_byte {
            NOP => {
                trace!("Instruction: NOP");
            }
            END => {
                // if this is not the very last instruction in the function
                // just skip because it is a delimiter of a ctrl block

                // TODO there is definitely a better to write this
                let current_func_span = store
                    .functions
                    .get(stack.current_stackframe().func_idx)
                    .unwrap_validated()
                    .try_into_local()
                    .unwrap_validated()
                    .code_expr;

                // There might be multiple ENDs in a single function. We want to
                // exit only when the outermost block (aka function block) ends.
                if store.modules[*current_module_idx].wasm_reader.pc
                    != current_func_span.from() + current_func_span.len()
                {
                    continue;
                }

                let (return_module, maybe_return_address, maybe_return_stp) =
                    stack.pop_stackframe();

                // We finished this entire invocation if there is no stackframe left. If there are
                // one or more stack frames, we need to continue from where the callee was called
                // fromn.
                if stack.callframe_count() == 0 {
                    break;
                }

                trace!("end of function reached, returning to previous stack frame");
                current_wasm_index = return_module;

                // wasm = &mut modules[return_module].wasm_reader;
                store.modules[*current_module_idx].wasm_reader.pc = maybe_return_address;
                stp = maybe_return_stp;

                current_sidetable = &store
                    .functions
                    .get(stack.current_stackframe().func_idx)
                    .unwrap_validated()
                    .try_into_local()
                    .unwrap_validated()
                    .sidetable;

                *current_module_idx = return_module;
            }
            IF => {
                store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated();

                let test_val: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                if test_val != 0 {
                    stp += 1;
                } else {
                    do_sidetable_control_transfer(
                        &mut store.modules[current_wasm_index].wasm_reader,
                        stack,
                        &mut stp,
                        current_sidetable,
                    );
                }
            }
            ELSE => {
                do_sidetable_control_transfer(
                    &mut store.modules[current_wasm_index].wasm_reader,
                    stack,
                    &mut stp,
                    current_sidetable,
                );
            }
            BR_IF => {
                store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated();

                let test_val: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                if test_val != 0 {
                    do_sidetable_control_transfer(
                        &mut store.modules[current_wasm_index].wasm_reader,
                        stack,
                        &mut stp,
                        current_sidetable,
                    );
                } else {
                    stp += 1;
                }
            }
            BR_TABLE => {
                let wasm_reader = &mut store.modules[current_wasm_index].wasm_reader;
                let label_vec = wasm_reader
                    .read_vec(|wasm| wasm.read_var_u32().map(|v| v as LabelIdx))
                    .unwrap_validated();
                store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated();

                // TODO is this correct?
                let case_val_i32: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let case_val = case_val_i32 as usize;

                if case_val >= label_vec.len() {
                    stp += label_vec.len();
                } else {
                    stp += case_val;
                }

                do_sidetable_control_transfer(
                    &mut store.modules[current_wasm_index].wasm_reader,
                    stack,
                    &mut stp,
                    current_sidetable,
                );
            }
            BR => {
                //skip n of BR n
                store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated();
                do_sidetable_control_transfer(
                    &mut store.modules[current_wasm_index].wasm_reader,
                    stack,
                    &mut stp,
                    current_sidetable,
                );
            }
            BLOCK | LOOP => {
                BlockType::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
            }
            RETURN => {
                //same as BR, except no need to skip n of BR n
                do_sidetable_control_transfer(
                    &mut store.modules[current_wasm_index].wasm_reader,
                    stack,
                    &mut stp,
                    current_sidetable,
                );
            }
            CALL => {
                let func_to_call_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as FuncIdx;

                let func_to_call_global_idx =
                    store.modules[current_wasm_index].functions[func_to_call_idx];

                let func_to_call_inst = store
                    .functions
                    .get(func_to_call_global_idx)
                    .unwrap_validated();
                let func_to_call_ty = func_to_call_inst.ty();

                // let func_to_call_inst = modules[*current_module_idx]
                //     .store
                //     .funcs
                //     .get(func_to_call_idx)
                //     .unwrap_validated();
                // let func_to_call_ty = modules[*current_module_idx]
                //     .fn_types
                //     .get(func_to_call_inst.ty())
                //     .unwrap_validated();

                let params = stack.pop_tail_iter(func_to_call_ty.params.valtypes.len());

                trace!("Instruction: call [{func_to_call_idx:?}]");

                match func_to_call_inst {
                    FuncInst::Local(local_func_inst) => {
                        // MIGHT BE IMPORTED
                        let func_module_idx = store.get_module_idx(func_to_call_global_idx);
                        match func_module_idx == *current_module_idx {
                            true => {
                                // local function
                                let remaining_locals = local_func_inst.locals.iter().cloned();
                                let locals = Locals::new(params, remaining_locals);

                                stack.push_stackframe(
                                    *current_module_idx,
                                    func_to_call_idx,
                                    &func_to_call_ty,
                                    locals,
                                    store.modules[current_wasm_index].wasm_reader.pc,
                                    stp,
                                );

                                store.modules[current_wasm_index]
                                    .wasm_reader
                                    .move_start_to(local_func_inst.code_expr)
                                    .unwrap_validated();

                                stp = 0;
                                current_sidetable = &local_func_inst.sidetable;
                            }
                            false => {
                                let (next_module, next_func_idx) =
                                    (func_module_idx, func_to_call_global_idx);

                                let local_func_inst =
                                    store.functions[next_func_idx].try_into_local().unwrap();

                                let remaining_locals = local_func_inst.locals.iter().cloned();
                                let locals = Locals::new(params, remaining_locals);

                                stack.push_stackframe(
                                    *current_module_idx,
                                    func_to_call_idx,
                                    &func_to_call_ty,
                                    locals,
                                    store.modules[current_wasm_index].wasm_reader.pc,
                                    stp,
                                );

                                current_wasm_index = next_module;
                                // wasm = &mut modules[next_module].wasm_reader;
                                *current_module_idx = next_module;

                                store.modules[current_wasm_index]
                                    .wasm_reader
                                    .move_start_to(local_func_inst.code_expr)
                                    .unwrap_validated();

                                stp = 0;
                                current_sidetable = &local_func_inst.sidetable;
                            }
                        };
                    }
                    FuncInst::Imported(_imported_func_inst) => {
                        unreachable!()
                    }
                }
            }
            CALL_INDIRECT => {
                let given_type_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as TypeIdx;
                let table_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as TableIdx;

                let tab = &store.tables[store.modules[*current_module_idx].tables[table_idx]];

                let func_ty = store.modules[*current_module_idx]
                    .function_types
                    .get(given_type_idx)
                    .unwrap_validated();

                let i: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let r = tab
                    .elem
                    .get(i as usize)
                    .ok_or(RuntimeError::UndefinedTableIndex)
                    .and_then(|r| {
                        if r.is_null() {
                            trace!("table_idx ({table_idx}) --- element index in table ({i})");
                            Err(RuntimeError::UninitializedElement)
                        } else {
                            Ok(r)
                        }
                    })?;

                let func_addr = match *r {
                    Ref::Func(func_addr) => func_addr.addr.unwrap_validated(),
                    Ref::Extern(_) => unreachable!(),
                };

                let func_idx = store.modules[*current_module_idx].functions[func_addr];
                let func_to_call_inst = &store.functions[func_idx];

                let actual_ty = func_to_call_inst.ty();

                if *func_ty != actual_ty {
                    return Err(RuntimeError::SignatureMismatch);
                }

                match func_to_call_inst {
                    FuncInst::Local(local_func_inst) => {
                        // MIGHT BE IMPORTED
                        let func_module_idx = store.get_module_idx(func_idx);
                        match func_module_idx == *current_module_idx {
                            true => {
                                // local function
                                let params = stack.pop_tail_iter(func_ty.params.valtypes.len());
                                let remaining_locals = local_func_inst.locals.iter().cloned();

                                trace!("Instruction: call_indirect [{func_addr:?}]");
                                let locals = Locals::new(params, remaining_locals);
                                stack.push_stackframe(
                                    *current_module_idx,
                                    func_addr,
                                    func_ty,
                                    locals,
                                    store.modules[current_wasm_index].wasm_reader.pc,
                                    stp,
                                );

                                store.modules[current_wasm_index]
                                    .wasm_reader
                                    .move_start_to(local_func_inst.code_expr)
                                    .unwrap_validated();

                                stp = 0;
                                current_sidetable = &local_func_inst.sidetable;
                            }
                            false => {
                                let (next_module, next_func_idx) = (func_module_idx, func_idx);

                                let local_func_inst =
                                    func_to_call_inst.try_into_local().unwrap_validated();

                                let params = stack.pop_tail_iter(func_ty.params.valtypes.len());
                                let remaining_locals = local_func_inst.locals.iter().cloned();

                                trace!("Instruction: call_indirect [{func_addr:?}]");
                                let locals = Locals::new(params, remaining_locals);
                                stack.push_stackframe(
                                    *current_module_idx,
                                    func_addr,
                                    func_ty,
                                    locals,
                                    store.modules[current_wasm_index].wasm_reader.pc,
                                    stp,
                                );

                                current_wasm_index = next_module;
                                // wasm = &mut modules[next_module].wasm_reader;
                                *current_module_idx = next_module;

                                store.modules[current_wasm_index]
                                    .wasm_reader
                                    .move_start_to(local_func_inst.code_expr)
                                    .unwrap_validated();

                                stp = 0;
                                current_sidetable = &local_func_inst.sidetable;
                            }
                        }
                    }
                    FuncInst::Imported(_imported_func_inst) => {
                        unreachable!()
                    }
                }
            }
            DROP => {
                stack.drop_value();
            }
            SELECT => {
                let test_val: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let val2 = stack.pop_value_with_unknown_type();
                let val1 = stack.pop_value_with_unknown_type();
                if test_val != 0 {
                    stack.push_value(val1);
                } else {
                    stack.push_value(val2);
                }
            }
            SELECT_T => {
                let type_vec = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_vec(ValType::read)
                    .unwrap_validated();
                let test_val: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let val2 = stack.pop_value(type_vec[0]);
                let val1 = stack.pop_value(type_vec[0]);
                if test_val != 0 {
                    stack.push_value(val1);
                } else {
                    stack.push_value(val2);
                }
            }
            LOCAL_GET => {
                let local_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as LocalIdx;
                stack.get_local(local_idx);
                trace!("Instruction: local.get {} [] -> [t]", local_idx);
            }
            LOCAL_SET => stack.set_local(
                store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as LocalIdx,
            ),
            LOCAL_TEE => stack.tee_local(
                store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as LocalIdx,
            ),
            GLOBAL_GET => {
                let global_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as GlobalIdx;

                let global = &store.globals[store.modules[*current_module_idx].globals[global_idx]];

                stack.push_value(global.value);
            }
            GLOBAL_SET => {
                let global_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as GlobalIdx;
                let global =
                    &mut store.globals[store.modules[*current_module_idx].globals[global_idx]];
                global.value = stack.pop_value(global.global.ty.ty);
            }
            TABLE_GET => {
                let table_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as TableIdx;

                let tab = &store.tables[store.modules[*current_module_idx].tables[table_idx]];

                let i: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let val = tab
                    .elem
                    .get(i as usize)
                    .ok_or(RuntimeError::TableAccessOutOfBounds)?;

                stack.push_value((*val).into());
                trace!(
                    "Instruction: table.get '{}' [{}] -> [{}]",
                    table_idx,
                    i,
                    val
                );
            }
            TABLE_SET => {
                let table_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as TableIdx;

                let mut tab =
                    &mut store.tables[store.modules[*current_module_idx].tables[table_idx]];

                let val: Ref = stack.pop_value(ValType::RefType(tab.ty.et)).into();
                let i: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                tab.elem
                    .get_mut(i as usize)
                    .ok_or(RuntimeError::TableAccessOutOfBounds)
                    .map(|r| *r = val)?;
                trace!(
                    "Instruction: table.set '{}' [{} {}] -> []",
                    table_idx,
                    i,
                    val
                )
            }
            I32_LOAD => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: u32 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data.get(address..(address + 4))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    let data: [u8; 4] = data.try_into().expect("this to be exactly 4 bytes");
                    u32::from_le_bytes(data)
                };

                stack.push_value(Value::I32(data));
                trace!("Instruction: i32.load [{relative_address}] -> [{data}]");
            }
            I64_LOAD => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: u64 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 8))
                                .map(|slice| slice.try_into().expect("this to be exactly 8 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u64::from_le_bytes(data)
                };

                stack.push_value(Value::I64(data));
                trace!("Instruction: i64.load [{relative_address}] -> [{data}]");
            }
            F32_LOAD => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: f32 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 4))
                                .map(|slice| slice.try_into().expect("this to be exactly 4 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;
                    f32::from_le_bytes(data)
                };

                stack.push_value(Value::F32(value::F32(data)));
                trace!("Instruction: f32.load [{relative_address}] -> [{data}]");
            }
            F64_LOAD => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: f64 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 8))
                                .map(|slice| slice.try_into().expect("this to be exactly 8 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    f64::from_le_bytes(data)
                };

                stack.push_value(Value::F64(value::F64(data)));
                trace!("Instruction: f64.load [{relative_address}] -> [{data}]");
            }
            I32_LOAD8_S => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: i8 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 1))
                                .map(|slice| slice.try_into().expect("this to be exactly 1 byte"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    // let data: [u8; 1] = data.try_into().expect("this to be exactly 1 byte");
                    u8::from_le_bytes(data) as i8
                };

                stack.push_value(Value::I32(data as u32));
                trace!("Instruction: i32.load8_s [{relative_address}] -> [{data}]");
            }
            I32_LOAD8_U => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: u8 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 1))
                                .map(|slice| slice.try_into().expect("this to be exactly 1 byte"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u8::from_le_bytes(data)
                };

                stack.push_value(Value::I32(data as u32));
                trace!("Instruction: i32.load8_u [{relative_address}] -> [{data}]");
            }
            I32_LOAD16_S => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: i16 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 2))
                                .map(|slice| slice.try_into().expect("this to be exactly 2 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u16::from_le_bytes(data) as i16
                };

                stack.push_value(Value::I32(data as u32));
                trace!("Instruction: i32.load16_s [{relative_address}] -> [{data}]");
            }
            I32_LOAD16_U => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: u16 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 2))
                                .map(|slice| slice.try_into().expect("this to be exactly 2 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u16::from_le_bytes(data)
                };

                stack.push_value(Value::I32(data as u32));
                trace!("Instruction: i32.load16_u [{relative_address}] -> [{data}]");
            }
            I64_LOAD8_S => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: i8 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 1))
                                .map(|slice| slice.try_into().expect("this to be exactly 1 byte"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    // let data: [u8; 1] = data.try_into().expect("this to be exactly 1 byte");
                    u8::from_le_bytes(data) as i8
                };

                stack.push_value(Value::I64(data as u64));
                trace!("Instruction: i64.load8_s [{relative_address}] -> [{data}]");
            }
            I64_LOAD8_U => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: u8 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 1))
                                .map(|slice| slice.try_into().expect("this to be exactly 1 byte"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    // let data: [u8; 1] = data.try_into().expect("this to be exactly 1 byte");
                    u8::from_le_bytes(data)
                };

                stack.push_value(Value::I64(data as u64));
                trace!("Instruction: i64.load8_u [{relative_address}] -> [{data}]");
            }
            I64_LOAD16_S => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: i16 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 2))
                                .map(|slice| slice.try_into().expect("this to be exactly 2 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u16::from_le_bytes(data) as i16
                };

                stack.push_value(Value::I64(data as u64));
                trace!("Instruction: i64.load16_s [{relative_address}] -> [{data}]");
            }
            I64_LOAD16_U => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: u16 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 2))
                                .map(|slice| slice.try_into().expect("this to be exactly 2 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u16::from_le_bytes(data)
                };

                stack.push_value(Value::I64(data as u64));
                trace!("Instruction: i64.load16_u [{relative_address}] -> [{data}]");
            }
            I64_LOAD32_S => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: i32 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 4))
                                .map(|slice| slice.try_into().expect("this to be exactly 4 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u32::from_le_bytes(data) as i32
                };

                stack.push_value(Value::I64(data as u64));
                trace!("Instruction: i64.load32_s [{relative_address}] -> [{data}]");
            }
            I64_LOAD32_U => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                let data: u32 = {
                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let _address = memarg.offset.checked_add(relative_address);
                    let data = memarg
                        .offset
                        .checked_add(relative_address)
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data
                                .get(address..(address + 4))
                                .map(|slice| slice.try_into().expect("this to be exactly 4 bytes"))
                        })
                        .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                    u32::from_le_bytes(data)
                };

                stack.push_value(Value::I64(data as u64));
                trace!("Instruction: i64.load32_u [{relative_address}] -> [{data}]");
            }
            I32_STORE => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                let address = memarg.offset.checked_add(relative_address);
                let memory_location = address
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 4))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                trace!("Instruction: i32.store [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: u64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                let address = memarg.offset.checked_add(relative_address);
                let memory_location = address
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 8))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                trace!("Instruction: i64.store [{relative_address} {data_to_store}] -> []");
            }
            F32_STORE => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: f32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                let address = memarg.offset.checked_add(relative_address);
                let memory_location = address
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 4))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                trace!("Instruction: f32.store [{relative_address} {data_to_store}] -> []");
            }
            F64_STORE => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: f64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]]; // there is only one memory allowed as of now

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                let address = memarg.offset.checked_add(relative_address);
                let memory_location = address
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 8))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                trace!("Instruction: f64.store [{relative_address} {data_to_store}] -> []");
            }
            I32_STORE8 => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]];

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                // ea => effective address
                let ea = memarg.offset.checked_add(relative_address);
                let memory_location = ea
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 1))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes()[0..1]);
                trace!("Instruction: i32.store8 [{relative_address} {data_to_store}] -> []");
            }
            I32_STORE16 => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]];

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                // ea => effective address
                let ea = memarg.offset.checked_add(relative_address);
                let memory_location = ea
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 2))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes()[0..2]);
                trace!("Instruction: i32.store16 [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE8 => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]];

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                // ea => effective address
                let ea = memarg.offset.checked_add(relative_address);
                let memory_location = ea
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 1))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes()[0..1]);
                trace!("Instruction: i64.store8 [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE16 => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]];

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                // ea => effective address
                let ea = memarg.offset.checked_add(relative_address);
                let memory_location = ea
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 2))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes()[0..2]);
                trace!("Instruction: i64.store16 [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE32 => {
                let memarg =
                    MemArg::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                let data_to_store: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[0]];

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                // ea => effective address
                let ea = memarg.offset.checked_add(relative_address);
                let memory_location = ea
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 4))
                    })
                    .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                memory_location.copy_from_slice(&data_to_store.to_le_bytes()[0..4]);
                trace!("Instruction: i64.store32 [{relative_address} {data_to_store}] -> []");
            }
            MEMORY_SIZE => {
                let mem_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_u8()
                    .unwrap_validated() as usize;
                let mem = &mut store.memories[store.modules[*current_module_idx].memories[mem_idx]];
                let size = mem.size() as u32;
                stack.push_value(Value::I32(size));
                trace!("Instruction: memory.size [] -> [{}]", size);
            }
            MEMORY_GROW => {
                let mem_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_u8()
                    .unwrap_validated() as usize;

                let mem = &mut store.memories[store.modules[*current_module_idx].memories[mem_idx]];
                let delta: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let upper_limit = mem.ty.limits.max.unwrap_or(Limits::MAX_MEM_BYTES);
                let pushed_value = if delta < 0 || delta as u32 + mem.size() as u32 > upper_limit {
                    stack.push_value((-1).into());
                    -1
                } else {
                    let previous_size: i32 = mem.size() as i32;
                    mem.grow(delta as usize);
                    stack.push_value(previous_size.into());
                    previous_size
                };
                trace!("Instruction: memory.grow [{}] -> [{}]", delta, pushed_value);
            }
            I32_CONST => {
                let constant = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_i32()
                    .unwrap_validated();
                trace!("Instruction: i32.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            F32_CONST => {
                let constant = f32::from_bits(
                    store.modules[current_wasm_index]
                        .wasm_reader
                        .read_var_f32()
                        .unwrap_validated(),
                );
                trace!("Instruction: f32.const [] -> [{constant:.7}]");
                stack.push_value(constant.into());
            }
            I32_EQZ => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if v1 == 0 { 1 } else { 0 };

                trace!("Instruction: i32.eqz [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_EQ => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: i32.eq [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_NE => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: i32.ne [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_LT_S => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: i32.lt_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }

            I32_LT_U => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if (v1 as u32) < (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.lt_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_GT_S => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: i32.gt_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_GT_U => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if (v1 as u32) > (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.gt_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_LE_S => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: i32.le_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_LE_U => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if (v1 as u32) <= (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.le_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_GE_S => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: i32.ge_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_GE_U => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = if (v1 as u32) >= (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.ge_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_EQZ => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if v1 == 0 { 1 } else { 0 };

                trace!("Instruction: i64.eqz [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_EQ => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: i64.eq [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_NE => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: i64.ne [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_LT_S => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: i64.lt_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }

            I64_LT_U => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if (v1 as u64) < (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.lt_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_GT_S => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: i64.gt_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_GT_U => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if (v1 as u64) > (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.gt_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_LE_S => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: i64.le_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_LE_U => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if (v1 as u64) <= (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.le_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_GE_S => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: i64.ge_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_GE_U => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = if (v1 as u64) >= (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.ge_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_EQ => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: f32.eq [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_NE => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: f32.ne [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_LT => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: f32.lt [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_GT => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: f32.gt [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_LE => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: f32.le [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_GE => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: f32.ge [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }

            F64_EQ => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: f64.eq [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_NE => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: f64.ne [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_LT => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: f64.lt [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_GT => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: f64.gt [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_LE => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: f64.le [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_GE => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: f64.ge [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }

            I32_CLZ => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.leading_zeros() as i32;

                trace!("Instruction: i32.clz [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_CTZ => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.trailing_zeros() as i32;

                trace!("Instruction: i32.ctz [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_POPCNT => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.count_ones() as i32;

                trace!("Instruction: i32.popcnt [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_CONST => {
                let constant = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_i64()
                    .unwrap_validated();
                trace!("Instruction: i64.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            F64_CONST => {
                let constant = f64::from_bits(
                    store.modules[current_wasm_index]
                        .wasm_reader
                        .read_var_f64()
                        .unwrap_validated(),
                );
                trace!("Instruction: f64.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            I32_ADD => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.wrapping_add(v2);

                trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_SUB => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.wrapping_sub(v2);

                trace!("Instruction: i32.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_MUL => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.wrapping_mul(v2);

                trace!("Instruction: i32.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_DIV_S => {
                let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }
                if divisor == i32::MIN && dividend == -1 {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res = divisor / dividend;

                trace!("Instruction: i32.div_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_DIV_U => {
                let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let dividend = dividend as u32;
                let divisor = divisor as u32;

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }

                let res = (divisor / dividend) as i32;

                trace!("Instruction: i32.div_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_REM_S => {
                let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }

                let res = divisor.checked_rem(dividend);
                let res = res.unwrap_or_default();

                trace!("Instruction: i32.rem_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_CLZ => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.leading_zeros() as i64;

                trace!("Instruction: i64.clz [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_CTZ => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.trailing_zeros() as i64;

                trace!("Instruction: i64.ctz [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_POPCNT => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.count_ones() as i64;

                trace!("Instruction: i64.popcnt [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_ADD => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.wrapping_add(v2);

                trace!("Instruction: i64.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_SUB => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.wrapping_sub(v2);

                trace!("Instruction: i64.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_MUL => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.wrapping_mul(v2);

                trace!("Instruction: i64.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_DIV_S => {
                let dividend: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let divisor: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }
                if divisor == i64::MIN && dividend == -1 {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res = divisor / dividend;

                trace!("Instruction: i64.div_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_DIV_U => {
                let dividend: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let divisor: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let dividend = dividend as u64;
                let divisor = divisor as u64;

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }

                let res = (divisor / dividend) as i64;

                trace!("Instruction: i64.div_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_REM_S => {
                let dividend: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let divisor: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }

                let res = divisor.checked_rem(dividend);
                let res = res.unwrap_or_default();

                trace!("Instruction: i64.rem_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_REM_U => {
                let dividend: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let divisor: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let dividend = dividend as u64;
                let divisor = divisor as u64;

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }

                let res = (divisor % dividend) as i64;

                trace!("Instruction: i64.rem_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_AND => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = v1 & v2;

                trace!("Instruction: i64.and [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_OR => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = v1 | v2;

                trace!("Instruction: i64.or [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_XOR => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = v1 ^ v2;

                trace!("Instruction: i64.xor [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_SHL => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = v1.wrapping_shl((v2 & 63) as u32);

                trace!("Instruction: i64.shl [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_SHR_S => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = v1.wrapping_shr((v2 & 63) as u32);

                trace!("Instruction: i64.shr_s [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_SHR_U => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = (v1 as u64).wrapping_shr((v2 & 63) as u32);

                trace!("Instruction: i64.shr_u [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_ROTL => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = v1.rotate_left((v2 & 63) as u32);

                trace!("Instruction: i64.rotl [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_ROTR => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();

                let res = v1.rotate_right((v2 & 63) as u32);

                trace!("Instruction: i64.rotr [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_REM_U => {
                let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let dividend = dividend as u32;
                let divisor = divisor as u32;

                if dividend == 0 {
                    return Err(RuntimeError::DivideBy0);
                }

                let res = divisor.checked_rem(dividend);
                let res = res.unwrap_or_default() as i32;

                trace!("Instruction: i32.rem_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_AND => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1 & v2;

                trace!("Instruction: i32.and [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_OR => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1 | v2;

                trace!("Instruction: i32.or [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_XOR => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1 ^ v2;

                trace!("Instruction: i32.xor [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_SHL => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v2.wrapping_shl(v1 as u32);

                trace!("Instruction: i32.shl [{v2} {v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_SHR_S => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = v2.wrapping_shr(v1 as u32);

                trace!("Instruction: i32.shr_s [{v2} {v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_SHR_U => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = (v2 as u32).wrapping_shr(v1 as u32) as i32;

                trace!("Instruction: i32.shr_u [{v2} {v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_ROTL => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = v2.rotate_left(v1 as u32);

                trace!("Instruction: i32.rotl [{v2} {v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_ROTR => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res = v2.rotate_right(v1 as u32);

                trace!("Instruction: i32.rotr [{v2} {v1}] -> [{res}]");
                stack.push_value(res.into());
            }

            F32_ABS => {
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.abs();

                trace!("Instruction: f32.abs [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_NEG => {
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.neg();

                trace!("Instruction: f32.neg [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_CEIL => {
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.ceil();

                trace!("Instruction: f32.ceil [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_FLOOR => {
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.floor();

                trace!("Instruction: f32.floor [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_TRUNC => {
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.trunc();

                trace!("Instruction: f32.trunc [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_NEAREST => {
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.nearest();

                trace!("Instruction: f32.nearest [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_SQRT => {
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.sqrt();

                trace!("Instruction: f32.sqrt [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_ADD => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 + v2;

                trace!("Instruction: f32.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_SUB => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 - v2;

                trace!("Instruction: f32.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_MUL => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 * v2;

                trace!("Instruction: f32.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_DIV => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 / v2;

                trace!("Instruction: f32.div [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_MIN => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.min(v2);

                trace!("Instruction: f32.min [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_MAX => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.max(v2);

                trace!("Instruction: f32.max [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_COPYSIGN => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1.copysign(v2);

                trace!("Instruction: f32.copysign [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }

            F64_ABS => {
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.abs();

                trace!("Instruction: f64.abs [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_NEG => {
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.neg();

                trace!("Instruction: f64.neg [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_CEIL => {
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.ceil();

                trace!("Instruction: f64.ceil [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_FLOOR => {
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.floor();

                trace!("Instruction: f64.floor [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_TRUNC => {
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.trunc();

                trace!("Instruction: f64.trunc [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_NEAREST => {
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.nearest();

                trace!("Instruction: f64.nearest [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_SQRT => {
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.sqrt();

                trace!("Instruction: f64.sqrt [{v1}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_ADD => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 + v2;

                trace!("Instruction: f64.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_SUB => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 - v2;

                trace!("Instruction: f64.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_MUL => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 * v2;

                trace!("Instruction: f64.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_DIV => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 / v2;

                trace!("Instruction: f64.div [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_MIN => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.min(v2);

                trace!("Instruction: f64.min [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_MAX => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.max(v2);

                trace!("Instruction: f64.max [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_COPYSIGN => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1.copysign(v2);

                trace!("Instruction: f64.copysign [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_WRAP_I64 => {
                let v: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res: i32 = v as i32;

                trace!("Instruction: i32.wrap_i64 [{v}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_TRUNC_F32_S => {
                let v: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F32(2147483648.0) || v <= value::F32(-2147483904.0) {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i32 = v.as_i32();

                trace!("Instruction: i32.trunc_f32_s [{v:.7}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_TRUNC_F32_U => {
                let v: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F32(4294967296.0) || v <= value::F32(-1.0) {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i32 = v.as_u32() as i32;

                trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
                stack.push_value(res.into());
            }

            I32_TRUNC_F64_S => {
                let v: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F64(2147483648.0) || v <= value::F64(-2147483649.0) {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i32 = v.as_i32();

                trace!("Instruction: i32.trunc_f64_s [{v:.7}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_TRUNC_F64_U => {
                let v: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F64(4294967296.0) || v <= value::F64(-1.0) {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i32 = v.as_u32() as i32;

                trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
                stack.push_value(res.into());
            }

            I64_EXTEND_I32_S => {
                let v: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res: i64 = v as i64;

                trace!("Instruction: i64.extend_i32_s [{v}] -> [{res}]");
                stack.push_value(res.into());
            }

            I64_EXTEND_I32_U => {
                let v: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let res: i64 = v as u32 as i64;

                trace!("Instruction: i64.extend_i32_u [{v}] -> [{res}]");
                stack.push_value(res.into());
            }

            I64_TRUNC_F32_S => {
                let v: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F32(9223372036854775808.0) || v <= value::F32(-9223373136366403584.0)
                {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i64 = v.as_i64();

                trace!("Instruction: i64.trunc_f32_s [{v:.7}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_TRUNC_F32_U => {
                let v: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F32(18446744073709551616.0) || v <= value::F32(-1.0) {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i64 = v.as_u64() as i64;

                trace!("Instruction: i64.trunc_f32_u [{v:.7}] -> [{res}]");
                stack.push_value(res.into());
            }

            I64_TRUNC_F64_S => {
                let v: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F64(9223372036854775808.0) || v <= value::F64(-9223372036854777856.0)
                {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i64 = v.as_i64();

                trace!("Instruction: i64.trunc_f64_s [{v:.17}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_TRUNC_F64_U => {
                let v: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                if v.is_infinity() {
                    return Err(RuntimeError::UnrepresentableResult);
                }
                if v.is_nan() {
                    return Err(RuntimeError::BadConversionToInteger);
                }
                if v >= value::F64(18446744073709551616.0) || v <= value::F64(-1.0) {
                    return Err(RuntimeError::UnrepresentableResult);
                }

                let res: i64 = v.as_u64() as i64;

                trace!("Instruction: i64.trunc_f64_u [{v:.17}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_CONVERT_I32_S => {
                let v: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res: value::F32 = value::F32(v as f32);

                trace!("Instruction: f32.convert_i32_s [{v}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_CONVERT_I32_U => {
                let v: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res: value::F32 = value::F32(v as u32 as f32);

                trace!("Instruction: f32.convert_i32_u [{v}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_CONVERT_I64_S => {
                let v: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res: value::F32 = value::F32(v as f32);

                trace!("Instruction: f32.convert_i64_s [{v}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_CONVERT_I64_U => {
                let v: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res: value::F32 = value::F32(v as u64 as f32);

                trace!("Instruction: f32.convert_i64_u [{v}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_DEMOTE_F64 => {
                let v: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F32 = v.as_f32();

                trace!("Instruction: f32.demote_f64 [{v:.17}] -> [{res:.7}]");
                stack.push_value(res.into());
            }
            F64_CONVERT_I32_S => {
                let v: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res: value::F64 = value::F64(v as f64);

                trace!("Instruction: f64.convert_i32_s [{v}] -> [{res:.17}]");
                stack.push_value(res.into());
            }
            F64_CONVERT_I32_U => {
                let v: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res: value::F64 = value::F64(v as u32 as f64);

                trace!("Instruction: f64.convert_i32_u [{v}] -> [{res:.17}]");
                stack.push_value(res.into());
            }
            F64_CONVERT_I64_S => {
                let v: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res: value::F64 = value::F64(v as f64);

                trace!("Instruction: f64.convert_i64_s [{v}] -> [{res:.17}]");
                stack.push_value(res.into());
            }
            F64_CONVERT_I64_U => {
                let v: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res: value::F64 = value::F64(v as u64 as f64);

                trace!("Instruction: f64.convert_i64_u [{v}] -> [{res:.17}]");
                stack.push_value(res.into());
            }
            F64_PROMOTE_F32 => {
                let v: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F64 = v.as_f32();

                trace!("Instruction: f64.promote_f32 [{v:.7}] -> [{res:.17}]");
                stack.push_value(res.into());
            }
            I32_REINTERPRET_F32 => {
                let v: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: i32 = v.reinterpret_as_i32();

                trace!("Instruction: i32.reinterpret_f32 [{v:.7}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_REINTERPRET_F64 => {
                let v: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: i64 = v.reinterpret_as_i64();

                trace!("Instruction: i64.reinterpret_f64 [{v:.17}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_REINTERPRET_I32 => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res: value::F32 = value::F32::from_bits(v1 as u32);

                trace!("Instruction: f32.reinterpret_i32 [{v1}] -> [{res:.7}]");
                stack.push_value(res.into());
            }
            F64_REINTERPRET_I64 => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res: value::F64 = value::F64::from_bits(v1 as u64);

                trace!("Instruction: f64.reinterpret_i64 [{v1}] -> [{res:.17}]");
                stack.push_value(res.into());
            }
            REF_NULL => {
                let reftype =
                    RefType::read_unvalidated(&mut store.modules[current_wasm_index].wasm_reader);

                stack.push_value(Value::Ref(reftype.to_null_ref()));
                trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
            }
            REF_IS_NULL => {
                let rref = stack.pop_unknown_ref();
                let is_null = match rref {
                    Ref::Extern(rref) => rref.addr.is_none(),
                    Ref::Func(rref) => rref.addr.is_none(),
                };

                let res = if is_null { 1 } else { 0 };
                trace!("Instruction: ref.is_null [{}] -> [{}]", rref, res);
                stack.push_value(Value::I32(res));
            }
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
            REF_FUNC => {
                let func_idx = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_var_u32()
                    .unwrap_validated() as FuncIdx;
                stack.push_value(Value::Ref(Ref::Func(FuncAddr::new(Some(func_idx)))));
            }
            FC_EXTENSIONS => {
                // Should we call instruction hook here as well? Multibyte instruction
                let second_instr_byte = store.modules[current_wasm_index]
                    .wasm_reader
                    .read_u8()
                    .unwrap_validated();

                use crate::core::reader::types::opcode::fc_extensions::*;
                match second_instr_byte {
                    I32_TRUNC_SAT_F32_S => {
                        let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                        let res = {
                            if v1.is_nan() {
                                0
                            } else if v1.is_negative_infinity() {
                                i32::MIN
                            } else if v1.is_infinity() {
                                i32::MAX
                            } else {
                                v1.as_i32()
                            }
                        };

                        trace!("Instruction: i32.trunc_sat_f32_s [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    I32_TRUNC_SAT_F32_U => {
                        let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                        let res = {
                            if v1.is_nan() || v1.is_negative_infinity() {
                                0
                            } else if v1.is_infinity() {
                                u32::MAX as i32
                            } else {
                                v1.as_u32() as i32
                            }
                        };

                        trace!("Instruction: i32.trunc_sat_f32_u [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    I32_TRUNC_SAT_F64_S => {
                        let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                        let res = {
                            if v1.is_nan() {
                                0
                            } else if v1.is_negative_infinity() {
                                i32::MIN
                            } else if v1.is_infinity() {
                                i32::MAX
                            } else {
                                v1.as_i32()
                            }
                        };

                        trace!("Instruction: i32.trunc_sat_f64_s [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    I32_TRUNC_SAT_F64_U => {
                        let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                        let res = {
                            if v1.is_nan() || v1.is_negative_infinity() {
                                0
                            } else if v1.is_infinity() {
                                u32::MAX as i32
                            } else {
                                v1.as_u32() as i32
                            }
                        };

                        trace!("Instruction: i32.trunc_sat_f64_u [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    I64_TRUNC_SAT_F32_S => {
                        let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                        let res = {
                            if v1.is_nan() {
                                0
                            } else if v1.is_negative_infinity() {
                                i64::MIN
                            } else if v1.is_infinity() {
                                i64::MAX
                            } else {
                                v1.as_i64()
                            }
                        };

                        trace!("Instruction: i64.trunc_sat_f32_s [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    I64_TRUNC_SAT_F32_U => {
                        let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                        let res = {
                            if v1.is_nan() || v1.is_negative_infinity() {
                                0
                            } else if v1.is_infinity() {
                                u64::MAX as i64
                            } else {
                                v1.as_u64() as i64
                            }
                        };

                        trace!("Instruction: i64.trunc_sat_f32_u [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    I64_TRUNC_SAT_F64_S => {
                        let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                        let res = {
                            if v1.is_nan() {
                                0
                            } else if v1.is_negative_infinity() {
                                i64::MIN
                            } else if v1.is_infinity() {
                                i64::MAX
                            } else {
                                v1.as_i64()
                            }
                        };

                        trace!("Instruction: i64.trunc_sat_f64_s [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    I64_TRUNC_SAT_F64_U => {
                        let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                        let res = {
                            if v1.is_nan() || v1.is_negative_infinity() {
                                0
                            } else if v1.is_infinity() {
                                u64::MAX as i64
                            } else {
                                v1.as_u64() as i64
                            }
                        };

                        trace!("Instruction: i64.trunc_sat_f64_u [{v1}] -> [{res}]");
                        stack.push_value(res.into());
                    }
                    // See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x
                    // Copy a region from a data segment into memory
                    MEMORY_INIT => {
                        //  mappings:
                        //      n => number of bytes to copy
                        //      s => starting pointer in the data segment
                        //      d => destination address to copy to
                        let data_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as DataIdx;

                        let data_init_len = store.data
                            [store.modules[*current_module_idx].data[data_idx]]
                            .data
                            .len();
                        let mem_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_u8()
                            .unwrap_validated() as usize;

                        let mem =
                            &store.memories[store.modules[*current_module_idx].memories[mem_idx]];
                        let n: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                        let s: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                        let d: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                        let final_src_offset = (n as usize)
                            .checked_add(s as usize)
                            .filter(|&res| res <= data_init_len)
                            .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                        let final_dst_offset = (n as usize)
                            .checked_add(d as usize)
                            .filter(|&res| res <= mem.data.len())
                            .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                        let data = &store.data[store.modules[*current_module_idx].data[data_idx]]
                            .data[(s as usize)..final_src_offset];

                        store.memories[store.modules[*current_module_idx].memories[mem_idx]].data
                            [d as usize..final_dst_offset]
                            .copy_from_slice(data);

                        trace!("Instruction: memory.init");
                    }
                    DATA_DROP => {
                        // Here is debatable
                        // If we were to be on par with the spec we'd have to use a DataInst struct
                        // But since memory.init is specifically made for Passive data segments
                        // I thought that using DataMode would be better because we can see if the
                        // data segment is passive or active

                        // Also, we should set data to null here (empty), which we do using an empty init vec
                        let data_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as DataIdx;

                        store.data[store.modules[*current_module_idx].data[data_idx]] =
                            DataInst { data: Vec::new() };
                    }
                    // See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy
                    MEMORY_COPY => {
                        //  mappings:
                        //      n => number of bytes to copy
                        //      s => source address to copy from
                        //      d => destination address to copy to
                        let (dst, src) = (
                            store.modules[current_wasm_index]
                                .wasm_reader
                                .read_u8()
                                .unwrap_validated() as usize,
                            store.modules[current_wasm_index]
                                .wasm_reader
                                .read_u8()
                                .unwrap_validated() as usize,
                        );
                        let n: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                        let s: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                        let d: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                        let len_src = store.memories
                            [store.modules[*current_module_idx].memories[src]]
                            .data
                            .len();

                        let final_src_offset = (n as usize)
                            .checked_add(s as usize)
                            .filter(|&res| res <= len_src)
                            .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                        let len_dest = store.memories
                            [store.modules[*current_module_idx].memories[dst]]
                            .data
                            .len();

                        // let final_dst_offset =
                        (n as usize)
                            .checked_add(d as usize)
                            .filter(|&res| res <= len_dest)
                            .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                        if dst == src {
                            // we copy from memory X to memory X
                            let mem = &mut store.memories
                                [store.modules[*current_module_idx].memories[src]];
                            // let mem = modules[*current_module_idx]
                            //     .store
                            //     .mems
                            //     .get_mut(src)
                            //     .unwrap_validated();
                            mem.data
                                .copy_within(s as usize..final_src_offset, d as usize);
                        } else {
                            // we copy from one memory to another
                            let src = store.modules[*current_module_idx].memories[src];
                            let dst = store.modules[*current_module_idx].memories[dst];

                            use core::cmp::Ordering::*;
                            let (src_mem, dst_mem) = match dst.cmp(&src) {
                                Greater => {
                                    let (left, right) = store.memories.split_at_mut(dst);
                                    (&left[src], &mut right[0])
                                }
                                Less => {
                                    let (left, right) = store.memories.split_at_mut(src);
                                    (&right[0], &mut left[dst])
                                }
                                Equal => unreachable!(),
                            };
                            dst_mem.data[d as usize..(d + n) as usize]
                                .copy_from_slice(&src_mem.data[s as usize..(s + n) as usize]);
                        }

                        trace!("Instruction: memory.copy");
                    }
                    // See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill
                    MEMORY_FILL => {
                        //  mappings:
                        //      n => number of bytes to update
                        //      val => the value to set each byte to (must be < 256)
                        //      d => the pointer to the region to update
                        let mem_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_u8()
                            .unwrap_validated() as usize;

                        let mem = &mut store.memories
                            [store.modules[*current_module_idx].memories[mem_idx]];
                        let n: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                        let val: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                        if !(0..=255).contains(&val) {
                            warn!("Value for memory.fill does not fit in a byte ({val})");
                        }

                        let d: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                        // let final_dst_offset =
                        (n as usize)
                            .checked_add(d as usize)
                            .filter(|&res| res <= mem.data.len())
                            .ok_or(RuntimeError::MemoryAccessOutOfBounds)?;

                        store.memories[store.modules[*current_module_idx].memories[mem_idx]].data
                            [d as usize..(d as usize + n as usize)]
                            .fill(val as u8);

                        trace!("Instruction: memory.fill");
                    }
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-init-x-y
                    // https://webassembly.github.io/spec/core/binary/instructions.html#table-instructions
                    // in binary format it seems that elemidx is first ???????
                    // this is ONLY for passive elements
                    TABLE_INIT => {
                        let elem_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;
                        let table_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;

                        let n: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // size
                        let s: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // offset
                        let d: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // dst

                        let tab_len = store.tables
                            [store.modules[*current_module_idx].tables[table_idx]]
                            .len();

                        let tab =
                            &mut store.tables[store.modules[*current_module_idx].tables[table_idx]];

                        let elem_len = if store.modules[*current_module_idx]
                            .passive_element_indexes
                            .contains(&elem_idx)
                        {
                            store.elements[store.modules[*current_module_idx].elements[elem_idx]]
                                .len()
                        } else {
                            0
                        };

                        trace!(
                            "Instruction: table.init '{}' '{}' [{} {} {}] -> []",
                            elem_idx,
                            table_idx,
                            d,
                            s,
                            n
                        );

                        let final_src_offset = (s as usize)
                            .checked_add(n as usize)
                            .filter(|&res| res <= elem_len)
                            .ok_or(RuntimeError::TableAccessOutOfBounds)?;

                        (d as usize)
                            .checked_add(n as usize)
                            .filter(|&res| res <= tab_len)
                            .ok_or(RuntimeError::TableAccessOutOfBounds)?;

                        let elem = &mut store.elements
                            [store.modules[*current_module_idx].elements[elem_idx]];

                        let dest = &mut tab.elem[d as usize..];
                        let src = &elem.references[s as usize..final_src_offset];
                        dest[..src.len()].copy_from_slice(src);
                    }
                    ELEM_DROP => {
                        let elem_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;

                        // WARN: i'm not sure if this is okay or not
                        store.elements[store.modules[*current_module_idx].elements[elem_idx]]
                            .references = vec![];
                    }
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-copy-x-y
                    TABLE_COPY => {
                        let table_x_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;
                        let table_y_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;

                        let tab_x_elem_len = store.tables
                            [store.modules[*current_module_idx].tables[table_x_idx]]
                            .elem
                            .len();
                        let tab_y_elem_len = store.tables
                            [store.modules[*current_module_idx].tables[table_y_idx]]
                            .elem
                            .len();

                        let n: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // size
                        let s: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // source
                        let d: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // destination

                        let src_res = match s.checked_add(n) {
                            Some(res) => {
                                if res > tab_y_elem_len as u32 {
                                    return Err(RuntimeError::TableAccessOutOfBounds);
                                } else {
                                    res as usize
                                }
                            }
                            _ => return Err(RuntimeError::TableAccessOutOfBounds),
                        };

                        let dst_res = match d.checked_add(n) {
                            Some(res) => {
                                if res > tab_x_elem_len as u32 {
                                    return Err(RuntimeError::TableAccessOutOfBounds);
                                } else {
                                    res as usize
                                }
                            }
                            _ => return Err(RuntimeError::TableAccessOutOfBounds),
                        };

                        let dst = table_x_idx;
                        let src = table_y_idx;

                        if table_x_idx == table_y_idx {
                            store.tables[store.modules[*current_module_idx].tables[table_x_idx]]
                                .elem
                                .copy_within(s as usize..src_res, d as usize); // }
                        } else {
                            use core::cmp::Ordering::*;
                            let src = store.modules[*current_module_idx].tables[src];
                            let dst = store.modules[*current_module_idx].tables[dst];

                            let (src_table, dst_table) = match dst.cmp(&src) {
                                Greater => {
                                    let (left, right) = store.tables.split_at_mut(dst);
                                    (&left[src], &mut right[0])
                                }
                                Less => {
                                    let (left, right) = store.tables.split_at_mut(src);
                                    (&right[0], &mut left[dst])
                                }
                                Equal => unreachable!(),
                            };

                            dst_table.elem[d as usize..dst_res]
                                .copy_from_slice(&src_table.elem[s as usize..src_res]);
                        }

                        trace!(
                            "Instruction: table.copy '{}' '{}' [{} {} {}] -> []",
                            table_x_idx,
                            table_y_idx,
                            d,
                            s,
                            n
                        );
                    }
                    TABLE_GROW => {
                        let table_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;

                        let tab =
                            &mut store.tables[store.modules[*current_module_idx].tables[table_idx]];

                        let sz = tab.elem.len() as u32;

                        let n: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                        let val = stack.pop_unknown_ref();

                        let max = tab.ty.lim.max.unwrap();

                        let final_size = sz.checked_add(n);

                        match final_size {
                            Some(final_size) => {
                                if final_size > max {
                                    stack.push_value(Value::I32(u32::MAX))
                                } else {
                                    tab.elem.extend(vec![val; n as usize]);

                                    stack.push_value(Value::I32(sz));
                                }
                            }
                            _ => stack.push_value(Value::I32(u32::MAX)),
                        }
                    }
                    TABLE_SIZE => {
                        let table_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;

                        let tab =
                            &mut store.tables[store.modules[*current_module_idx].tables[table_idx]];

                        let sz = tab.elem.len() as u32;

                        stack.push_value(Value::I32(sz));

                        trace!("Instruction: table.size '{}' [] -> [{}]", table_idx, sz);
                    }
                    TABLE_FILL => {
                        let table_idx = store.modules[current_wasm_index]
                            .wasm_reader
                            .read_var_u32()
                            .unwrap_validated() as usize;

                        let tab =
                            &mut store.tables[store.modules[*current_module_idx].tables[table_idx]];
                        let ty = tab.ty.et;

                        let n: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // len
                        let val: Ref = stack.pop_value(ValType::RefType(ty)).into();
                        let i: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into(); // dst

                        let end = (i as usize)
                            .checked_add(n as usize)
                            .ok_or(RuntimeError::TableAccessOutOfBounds)?;

                        tab.elem
                            .get_mut(i as usize..end)
                            .ok_or(RuntimeError::TableAccessOutOfBounds)?
                            .fill(val);

                        trace!(
                            "Instruction table.fill '{}' [{} {} {}] -> []",
                            table_idx,
                            i,
                            val,
                            n
                        )
                    }
                    _ => unreachable!(),
                }
            }
            other => {
                trace!("Unknown instruction {other:#x}, skipping..");
            }
        }
    }
    Ok(())
}

//helper function for avoiding code duplication at intraprocedural jumps
fn do_sidetable_control_transfer(
    wasm: &mut WasmReader,
    stack: &mut Stack,
    current_stp: &mut usize,
    current_sidetable: &Sidetable,
) {
    let sidetable_entry = &current_sidetable[*current_stp];

    // TODO fix this corner cutting implementation
    let jump_vals = stack
        .pop_tail_iter(sidetable_entry.valcnt)
        .collect::<Vec<_>>();
    stack.pop_n_values(sidetable_entry.popcnt);

    for val in jump_vals {
        stack.push_value(val);
    }

    *current_stp = (*current_stp as isize + sidetable_entry.delta_stp) as usize;
    wasm.pc = (wasm.pc as isize + sidetable_entry.delta_pc) as usize;
}
