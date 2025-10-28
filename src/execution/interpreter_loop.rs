//! This module solely contains the actual interpretation loop that matches instructions, interpreting the WASM bytecode
//!
//!
//! # Note to Developer:
//!
//! 1. There must be only imports and one `impl` with one function (`run`) in it.
//! 2. This module must only use [`RuntimeError`] and never [`Error`](crate::core::error::ValidationError).

use core::{
    num::NonZeroU32,
    {
        array,
        iter::zip,
        ops::{Add, Div, Mul, Neg, Sub},
    },
};

use crate::{
    addrs::{AddrVec, DataAddr, ElemAddr, MemAddr, TableAddr},
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{DataIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, MemIdx, TableIdx, TypeIdx},
        reader::{
            types::{memarg::MemArg, BlockType},
            WasmReadable, WasmReader,
        },
        sidetable::Sidetable,
    },
    resumable::Resumable,
    store::HaltExecutionError,
    unreachable_validated,
    value::{self, Ref, F32, F64},
    value_stack::Stack,
    DataInst, ElemInst, FuncInst, MemInst, ModuleInst, RefType, RuntimeError, TableInst, TrapError,
    ValType, Value,
};

use crate::execution::config::Config;

use super::{little_endian::LittleEndianBytes, store::Store};

/// Interprets wasm native functions. Wasm parameters and Wasm return values are passed on the stack.
/// Returns `Ok(None)` in case execution successfully terminates, `Ok(Some(required_fuel))` if execution
/// terminates due to insufficient fuel, indicating how much fuel is required to resume with `required_fuel`,
/// and `[Error::RuntimeError]` otherwise.
pub(super) fn run<T: Config>(
    resumable: &mut Resumable,
    store: &mut Store<T>,
) -> Result<Option<NonZeroU32>, RuntimeError> {
    let stack = &mut resumable.stack;
    let mut current_func_addr = resumable.current_func_addr;
    let pc = resumable.pc;
    let mut stp = resumable.stp;
    let func_inst = store.functions.get(current_func_addr);
    let FuncInst::WasmFunc(wasm_func_inst) = &func_inst else {
        unreachable!(
            "the interpreter loop shall only be executed with native wasm functions as root call"
        );
    };
    let mut current_module_idx = wasm_func_inst.module_addr;

    // Start reading the function's instructions
    let wasm = &mut WasmReader::new(store.modules[current_module_idx].wasm_bytecode);

    let mut current_sidetable: &Sidetable = &store.modules[current_module_idx].sidetable;

    // local variable for holding where the function code ends (last END instr address + 1) to avoid lookup at every END instr
    let mut current_function_end_marker =
        wasm_func_inst.code_expr.from() + wasm_func_inst.code_expr.len();

    wasm.pc = pc;

    use crate::core::reader::types::opcode::*;
    loop {
        // call the instruction hook
        store
            .user_data
            .instruction_hook(store.modules[current_module_idx].wasm_bytecode, wasm.pc);

        // Fuel mechanism: 1 fuel per instruction
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= 1 {
                *fuel -= 1;
            } else {
                resumable.current_func_addr = current_func_addr;
                resumable.pc = wasm.pc;
                resumable.stp = stp;
                return Ok(NonZeroU32::new(1));
            }
        }

        let first_instr_byte = wasm.read_u8().unwrap_validated();

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
                // There might be multiple ENDs in a single function. We want to
                // exit only when the outermost block (aka function block) ends.
                if wasm.pc != current_function_end_marker {
                    continue;
                }

                let (maybe_return_func_addr, maybe_return_address, maybe_return_stp) =
                    stack.pop_call_frame();

                // We finished this entire invocation if there is no call frame left. If there are
                // one or more call frames, we need to continue from where the callee was called
                // from.
                if stack.call_frame_count() == 0 {
                    break;
                }

                trace!("end of function reached, returning to previous call frame");
                current_func_addr = maybe_return_func_addr;
                let FuncInst::WasmFunc(current_wasm_func_inst) =
                    store.functions.get(current_func_addr)
                else {
                    unreachable!("function addresses on the stack always correspond to native wasm functions")
                };
                current_module_idx = current_wasm_func_inst.module_addr;
                wasm.full_wasm_binary = store.modules[current_module_idx].wasm_bytecode;
                wasm.pc = maybe_return_address;
                stp = maybe_return_stp;

                current_sidetable = &store.modules[current_module_idx].sidetable;

                current_function_end_marker = current_wasm_func_inst.code_expr.from()
                    + current_wasm_func_inst.code_expr.len();

                trace!("Instruction: END");
            }
            IF => {
                wasm.read_var_u32().unwrap_validated();

                let test_val: i32 = stack.pop_value().try_into().unwrap_validated();

                if test_val != 0 {
                    stp += 1;
                } else {
                    do_sidetable_control_transfer(wasm, stack, &mut stp, current_sidetable)?;
                }
                trace!("Instruction: IF");
            }
            ELSE => {
                do_sidetable_control_transfer(wasm, stack, &mut stp, current_sidetable)?;
            }
            BR_IF => {
                wasm.read_var_u32().unwrap_validated();

                let test_val: i32 = stack.pop_value().try_into().unwrap_validated();

                if test_val != 0 {
                    do_sidetable_control_transfer(wasm, stack, &mut stp, current_sidetable)?;
                } else {
                    stp += 1;
                }
                trace!("Instruction: BR_IF");
            }
            BR_TABLE => {
                let label_vec = wasm
                    .read_vec(|wasm| wasm.read_var_u32().map(|v| v as LabelIdx))
                    .unwrap_validated();
                wasm.read_var_u32().unwrap_validated();

                // TODO is this correct?
                let case_val_i32: i32 = stack.pop_value().try_into().unwrap_validated();
                let case_val = case_val_i32 as usize;

                if case_val >= label_vec.len() {
                    stp += label_vec.len();
                } else {
                    stp += case_val;
                }

                do_sidetable_control_transfer(wasm, stack, &mut stp, current_sidetable)?;
            }
            BR => {
                //skip n of BR n
                wasm.read_var_u32().unwrap_validated();
                do_sidetable_control_transfer(wasm, stack, &mut stp, current_sidetable)?;
            }
            BLOCK | LOOP => {
                BlockType::read(wasm).unwrap_validated();
            }
            RETURN => {
                //same as BR, except no need to skip n of BR n
                do_sidetable_control_transfer(wasm, stack, &mut stp, current_sidetable)?;
            }
            CALL => {
                let local_func_idx = wasm.read_var_u32().unwrap_validated() as FuncIdx;
                let FuncInst::WasmFunc(current_wasm_func_inst) =
                    store.functions.get(current_func_addr)
                else {
                    unreachable!()
                };

                let func_to_call_addr =
                    store.modules[current_wasm_func_inst.module_addr].func_addrs[local_func_idx];

                let func_to_call_ty = store.functions.get(func_to_call_addr).ty();

                trace!("Instruction: call [{func_to_call_addr:?}]");

                match store.functions.get(func_to_call_addr) {
                    FuncInst::HostFunc(host_func_to_call_inst) => {
                        let params = stack
                            .pop_tail_iter(func_to_call_ty.params.valtypes.len())
                            .collect();
                        let returns =
                            (host_func_to_call_inst.hostcode)(&mut store.user_data, params);

                        let returns = returns.map_err(|HaltExecutionError| {
                            RuntimeError::HostFunctionHaltedExecution
                        })?;

                        // Verify that the return parameters match the host function parameters
                        // since we have no validation guarantees for host functions
                        if returns.len() != func_to_call_ty.returns.valtypes.len() {
                            return Err(RuntimeError::HostFunctionSignatureMismatch);
                        }
                        for (value, ty) in zip(returns, func_to_call_ty.returns.valtypes) {
                            if value.to_ty() != ty {
                                return Err(RuntimeError::HostFunctionSignatureMismatch);
                            }
                            stack.push_value::<T>(value)?;
                        }
                    }
                    FuncInst::WasmFunc(wasm_func_to_call_inst) => {
                        let remaining_locals = &wasm_func_to_call_inst.locals;

                        stack.push_call_frame::<T>(
                            current_func_addr,
                            &func_to_call_ty,
                            remaining_locals,
                            wasm.pc,
                            stp,
                        )?;

                        current_func_addr = func_to_call_addr;
                        current_module_idx = wasm_func_to_call_inst.module_addr;
                        wasm.full_wasm_binary = store.modules[current_module_idx].wasm_bytecode;
                        wasm.move_start_to(wasm_func_to_call_inst.code_expr)
                            .expect("code expression spans to always be valid");

                        stp = wasm_func_to_call_inst.stp;
                        current_sidetable = &store.modules[current_module_idx].sidetable;
                        current_function_end_marker = wasm_func_to_call_inst.code_expr.from()
                            + wasm_func_to_call_inst.code_expr.len();
                    }
                }
                trace!("Instruction: CALL");
            }

            // TODO: fix push_call_frame, because the func idx that you get from the table is global func idx
            CALL_INDIRECT => {
                let given_type_idx = wasm.read_var_u32().unwrap_validated() as TypeIdx;
                let table_idx = wasm.read_var_u32().unwrap_validated() as TableIdx;

                let table_addr = *store.modules[current_module_idx]
                    .table_addrs
                    .get(table_idx)
                    .unwrap_validated();
                let tab = store.tables.get(table_addr);
                let func_ty = store.modules[current_module_idx]
                    .types
                    .get(given_type_idx)
                    .unwrap_validated();

                let i: u32 = stack.pop_value().try_into().unwrap_validated();

                let r = tab
                    .elem
                    .get(i as usize)
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

                let func_to_call_ty = store.functions.get(func_to_call_addr).ty();
                if *func_ty != func_to_call_ty {
                    return Err(TrapError::SignatureMismatch.into());
                }

                trace!("Instruction: call [{func_to_call_addr:?}]");

                match store.functions.get(func_to_call_addr) {
                    FuncInst::HostFunc(host_func_to_call_inst) => {
                        let params = stack
                            .pop_tail_iter(func_to_call_ty.params.valtypes.len())
                            .collect();
                        let returns =
                            (host_func_to_call_inst.hostcode)(&mut store.user_data, params);

                        let returns = returns.map_err(|HaltExecutionError| {
                            RuntimeError::HostFunctionHaltedExecution
                        })?;

                        // Verify that the return parameters match the host function parameters
                        // since we have no validation guarantees for host functions
                        if returns.len() != func_to_call_ty.returns.valtypes.len() {
                            return Err(RuntimeError::HostFunctionSignatureMismatch);
                        }
                        for (value, ty) in zip(returns, func_to_call_ty.returns.valtypes) {
                            if value.to_ty() != ty {
                                return Err(RuntimeError::HostFunctionSignatureMismatch);
                            }
                            stack.push_value::<T>(value)?;
                        }
                    }
                    FuncInst::WasmFunc(wasm_func_to_call_inst) => {
                        let remaining_locals = &wasm_func_to_call_inst.locals;

                        stack.push_call_frame::<T>(
                            current_func_addr,
                            &func_to_call_ty,
                            remaining_locals,
                            wasm.pc,
                            stp,
                        )?;

                        current_func_addr = func_to_call_addr;
                        current_module_idx = wasm_func_to_call_inst.module_addr;
                        wasm.full_wasm_binary = store.modules[current_module_idx].wasm_bytecode;
                        wasm.move_start_to(wasm_func_to_call_inst.code_expr)
                            .expect("code expression spans to always be valid");

                        stp = wasm_func_to_call_inst.stp;
                        current_sidetable = &store.modules[current_module_idx].sidetable;
                        current_function_end_marker = wasm_func_to_call_inst.code_expr.from()
                            + wasm_func_to_call_inst.code_expr.len();
                    }
                }
                trace!("Instruction: CALL_INDIRECT");
            }
            DROP => {
                stack.pop_value();
                trace!("Instruction: DROP");
            }
            SELECT => {
                let test_val: i32 = stack.pop_value().try_into().unwrap_validated();
                let val2 = stack.pop_value();
                let val1 = stack.pop_value();
                if test_val != 0 {
                    stack.push_value::<T>(val1)?;
                } else {
                    stack.push_value::<T>(val2)?;
                }
                trace!("Instruction: SELECT");
            }
            SELECT_T => {
                let _type_vec = wasm.read_vec(ValType::read).unwrap_validated();
                let test_val: i32 = stack.pop_value().try_into().unwrap_validated();
                let val2 = stack.pop_value();
                let val1 = stack.pop_value();
                if test_val != 0 {
                    stack.push_value::<T>(val1)?;
                } else {
                    stack.push_value::<T>(val2)?;
                }
                trace!("Instruction: SELECT_T");
            }
            LOCAL_GET => {
                let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                let value = *stack.get_local(local_idx);
                stack.push_value::<T>(value)?;
                trace!("Instruction: local.get {} [] -> [t]", local_idx);
            }
            LOCAL_SET => {
                let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                let value = stack.pop_value();
                *stack.get_local_mut(local_idx) = value;
                trace!("Instruction: local.set {} [t] -> []", local_idx);
            }
            LOCAL_TEE => {
                let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                let value = stack.peek_value().unwrap_validated();
                *stack.get_local_mut(local_idx) = value;
                trace!("Instruction: local.tee {} [t] -> [t]", local_idx);
            }
            GLOBAL_GET => {
                let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                let global_addr = *store.modules[current_module_idx]
                    .global_addrs
                    .get(global_idx)
                    .unwrap_validated();
                let global = store.globals.get(global_addr);

                stack.push_value::<T>(global.value)?;

                trace!(
                    "Instruction: global.get '{}' [<GLOBAL>] -> [{:?}]",
                    global_idx,
                    global.value
                );
            }
            GLOBAL_SET => {
                let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                let global_addr = *store.modules[current_module_idx]
                    .global_addrs
                    .get(global_idx)
                    .unwrap_validated();
                let global = store.globals.get_mut(global_addr);
                global.value = stack.pop_value();
                trace!("Instruction: GLOBAL_SET");
            }
            TABLE_GET => {
                let table_idx = wasm.read_var_u32().unwrap_validated() as TableIdx;
                let table_addr = *store.modules[current_module_idx]
                    .table_addrs
                    .get(table_idx)
                    .unwrap_validated();
                let tab = store.tables.get(table_addr);

                let i: i32 = stack.pop_value().try_into().unwrap_validated();

                let val = tab
                    .elem
                    .get(i as usize)
                    .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

                stack.push_value::<T>((*val).into())?;
                trace!(
                    "Instruction: table.get '{}' [{}] -> [{}]",
                    table_idx,
                    i,
                    val
                );
            }
            TABLE_SET => {
                let table_idx = wasm.read_var_u32().unwrap_validated() as TableIdx;
                let table_addr = *store.modules[current_module_idx]
                    .table_addrs
                    .get(table_idx)
                    .unwrap_validated();
                let tab = store.tables.get_mut(table_addr);

                let val: Ref = stack.pop_value().try_into().unwrap_validated();
                let i: i32 = stack.pop_value().try_into().unwrap_validated();

                tab.elem
                    .get_mut(i as usize)
                    .ok_or(TrapError::TableOrElementAccessOutOfBounds)
                    .map(|r| *r = val)?;
                trace!(
                    "Instruction: table.set '{}' [{} {}] -> []",
                    table_idx,
                    i,
                    val
                )
            }
            UNREACHABLE => {
                return Err(TrapError::ReachedUnreachable.into());
            }
            I32_LOAD => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem_inst = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data = mem_inst.mem.load(idx)?;

                stack.push_value::<T>(Value::I32(data))?;
                trace!("Instruction: i32.load [{relative_address}] -> [{data}]");
            }
            I64_LOAD => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I64(data))?;
                trace!("Instruction: i64.load [{relative_address}] -> [{data}]");
            }
            F32_LOAD => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::F32(data))?;
                trace!("Instruction: f32.load [{relative_address}] -> [{data}]");
            }
            F64_LOAD => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::F64(data))?;
                trace!("Instruction: f64.load [{relative_address}] -> [{data}]");
            }
            I32_LOAD8_S => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: i8 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I32(data as u32))?;
                trace!("Instruction: i32.load8_s [{relative_address}] -> [{data}]");
            }
            I32_LOAD8_U => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: u8 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I32(data as u32))?;
                trace!("Instruction: i32.load8_u [{relative_address}] -> [{data}]");
            }
            I32_LOAD16_S => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: i16 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I32(data as u32))?;
                trace!("Instruction: i32.load16_s [{relative_address}] -> [{data}]");
            }
            I32_LOAD16_U => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: u16 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I32(data as u32))?;
                trace!("Instruction: i32.load16_u [{relative_address}] -> [{data}]");
            }
            I64_LOAD8_S => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: i8 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I64(data as u64))?;
                trace!("Instruction: i64.load8_s [{relative_address}] -> [{data}]");
            }
            I64_LOAD8_U => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: u8 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I64(data as u64))?;
                trace!("Instruction: i64.load8_u [{relative_address}] -> [{data}]");
            }
            I64_LOAD16_S => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: i16 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I64(data as u64))?;
                trace!("Instruction: i64.load16_s [{relative_address}] -> [{data}]");
            }
            I64_LOAD16_U => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: u16 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I64(data as u64))?;
                trace!("Instruction: i64.load16_u [{relative_address}] -> [{data}]");
            }
            I64_LOAD32_S => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: i32 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I64(data as u64))?;
                trace!("Instruction: i64.load32_s [{relative_address}] -> [{data}]");
            }
            I64_LOAD32_U => {
                let memarg = MemArg::read(wasm).unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                let data: u32 = mem.mem.load(idx)?;

                stack.push_value::<T>(Value::I64(data as u64))?;
                trace!("Instruction: i64.load32_u [{relative_address}] -> [{data}]");
            }
            I32_STORE => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: u32 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, data_to_store)?;

                trace!("Instruction: i32.store [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: u64 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, data_to_store)?;

                trace!("Instruction: i64.store [{relative_address} {data_to_store}] -> []");
            }
            F32_STORE => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: F32 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, data_to_store)?;

                trace!("Instruction: f32.store [{relative_address} {data_to_store}] -> []");
            }
            F64_STORE => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: F64 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, data_to_store)?;

                trace!("Instruction: f64.store [{relative_address} {data_to_store}] -> []");
            }
            I32_STORE8 => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: i32 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let wrapped_data = data_to_store as i8;

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, wrapped_data)?;

                trace!("Instruction: i32.store8 [{relative_address} {wrapped_data}] -> []");
            }
            I32_STORE16 => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: i32 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let wrapped_data = data_to_store as i16;

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, wrapped_data)?;

                trace!("Instruction: i32.store16 [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE8 => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: i64 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let wrapped_data = data_to_store as i8;

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, wrapped_data)?;

                trace!("Instruction: i64.store8 [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE16 => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: i64 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let wrapped_data = data_to_store as i16;

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, wrapped_data)?;

                trace!("Instruction: i64.store16 [{relative_address} {data_to_store}] -> []");
            }
            I64_STORE32 => {
                let memarg = MemArg::read(wasm).unwrap_validated();

                let data_to_store: i64 = stack.pop_value().try_into().unwrap_validated();
                let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

                let wrapped_data = data_to_store as i32;

                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .first()
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);

                let idx = calculate_mem_address(&memarg, relative_address)?;
                mem.mem.store(idx, wrapped_data)?;

                trace!("Instruction: i64.store32 [{relative_address} {data_to_store}] -> []");
            }
            MEMORY_SIZE => {
                let mem_idx = wasm.read_u8().unwrap_validated() as usize;
                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .get(mem_idx)
                    .unwrap_validated();
                let mem = store.memories.get(mem_addr);
                let size = mem.size() as u32;
                stack.push_value::<T>(Value::I32(size))?;
                trace!("Instruction: memory.size [] -> [{}]", size);
            }
            MEMORY_GROW => {
                let mem_idx = wasm.read_u8().unwrap_validated() as usize;
                let mem_addr = *store.modules[current_module_idx]
                    .mem_addrs
                    .get(mem_idx)
                    .unwrap_validated();
                let mem = store.memories.get_mut(mem_addr);
                let sz: u32 = mem.size() as u32;

                let n: u32 = stack.pop_value().try_into().unwrap_validated();

                // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
                // for now we execute it always according to the following match expr.
                // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the stack per spec
                let pushed_value = match mem.grow(n) {
                    Ok(_) => sz,
                    Err(_) => u32::MAX,
                };
                stack.push_value::<T>(Value::I32(pushed_value))?;
                trace!("Instruction: memory.grow [{}] -> [{}]", n, pushed_value);
            }
            I32_CONST => {
                let constant = wasm.read_var_i32().unwrap_validated();
                trace!("Instruction: i32.const [] -> [{constant}]");
                stack.push_value::<T>(constant.into())?;
            }
            F32_CONST => {
                let constant = F32::from_bits(wasm.read_f32().unwrap_validated());
                trace!("Instruction: f32.const [] -> [{constant:.7}]");
                stack.push_value::<T>(constant.into())?;
            }
            I32_EQZ => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 == 0 { 1 } else { 0 };

                trace!("Instruction: i32.eqz [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_EQ => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: i32.eq [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_NE => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: i32.ne [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_LT_S => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: i32.lt_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I32_LT_U => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u32) < (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.lt_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_GT_S => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: i32.gt_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_GT_U => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u32) > (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.gt_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_LE_S => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: i32.le_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_LE_U => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u32) <= (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.le_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_GE_S => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: i32.ge_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_GE_U => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u32) >= (v2 as u32) { 1 } else { 0 };

                trace!("Instruction: i32.ge_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_EQZ => {
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 == 0 { 1 } else { 0 };

                trace!("Instruction: i64.eqz [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_EQ => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: i64.eq [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_NE => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: i64.ne [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_LT_S => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: i64.lt_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I64_LT_U => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u64) < (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.lt_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_GT_S => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: i64.gt_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_GT_U => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u64) > (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.gt_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_LE_S => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: i64.le_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_LE_U => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u64) <= (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.le_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_GE_S => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: i64.ge_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_GE_U => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = if (v1 as u64) >= (v2 as u64) { 1 } else { 0 };

                trace!("Instruction: i64.ge_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_EQ => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: f32.eq [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_NE => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: f32.ne [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_LT => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: f32.lt [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_GT => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: f32.gt [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_LE => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: f32.le [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_GE => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: f32.ge [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            F64_EQ => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 == v2 { 1 } else { 0 };

                trace!("Instruction: f64.eq [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_NE => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 != v2 { 1 } else { 0 };

                trace!("Instruction: f64.ne [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_LT => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 < v2 { 1 } else { 0 };

                trace!("Instruction: f64.lt [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_GT => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 > v2 { 1 } else { 0 };

                trace!("Instruction: f64.gt [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_LE => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 <= v2 { 1 } else { 0 };

                trace!("Instruction: f64.le [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_GE => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

                let res = if v1 >= v2 { 1 } else { 0 };

                trace!("Instruction: f64.ge [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I32_CLZ => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.leading_zeros() as i32;

                trace!("Instruction: i32.clz [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_CTZ => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.trailing_zeros() as i32;

                trace!("Instruction: i32.ctz [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_POPCNT => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.count_ones() as i32;

                trace!("Instruction: i32.popcnt [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_CONST => {
                let constant = wasm.read_var_i64().unwrap_validated();
                trace!("Instruction: i64.const [] -> [{constant}]");
                stack.push_value::<T>(constant.into())?;
            }
            F64_CONST => {
                let constant = F64::from_bits(wasm.read_f64().unwrap_validated());
                trace!("Instruction: f64.const [] -> [{constant}]");
                stack.push_value::<T>(constant.into())?;
            }
            I32_ADD => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.wrapping_add(v2);

                trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_SUB => {
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.wrapping_sub(v2);

                trace!("Instruction: i32.sub [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_MUL => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.wrapping_mul(v2);

                trace!("Instruction: i32.mul [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_DIV_S => {
                let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }
                if divisor == i32::MIN && dividend == -1 {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res = divisor / dividend;

                trace!("Instruction: i32.div_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_DIV_U => {
                let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

                let dividend = dividend as u32;
                let divisor = divisor as u32;

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }

                let res = (divisor / dividend) as i32;

                trace!("Instruction: i32.div_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_REM_S => {
                let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }

                let res = divisor.checked_rem(dividend);
                let res = res.unwrap_or_default();

                trace!("Instruction: i32.rem_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_CLZ => {
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.leading_zeros() as i64;

                trace!("Instruction: i64.clz [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_CTZ => {
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.trailing_zeros() as i64;

                trace!("Instruction: i64.ctz [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_POPCNT => {
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.count_ones() as i64;

                trace!("Instruction: i64.popcnt [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_ADD => {
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.wrapping_add(v2);

                trace!("Instruction: i64.add [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_SUB => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.wrapping_sub(v2);

                trace!("Instruction: i64.sub [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_MUL => {
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let res = v1.wrapping_mul(v2);

                trace!("Instruction: i64.mul [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_DIV_S => {
                let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }
                if divisor == i64::MIN && dividend == -1 {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res = divisor / dividend;

                trace!("Instruction: i64.div_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_DIV_U => {
                let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

                let dividend = dividend as u64;
                let divisor = divisor as u64;

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }

                let res = (divisor / dividend) as i64;

                trace!("Instruction: i64.div_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_REM_S => {
                let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }

                let res = divisor.checked_rem(dividend);
                let res = res.unwrap_or_default();

                trace!("Instruction: i64.rem_s [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_REM_U => {
                let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

                let dividend = dividend as u64;
                let divisor = divisor as u64;

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }

                let res = (divisor % dividend) as i64;

                trace!("Instruction: i64.rem_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_AND => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = v1 & v2;

                trace!("Instruction: i64.and [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_OR => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = v1 | v2;

                trace!("Instruction: i64.or [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_XOR => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = v1 ^ v2;

                trace!("Instruction: i64.xor [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_SHL => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = v1.wrapping_shl((v2 & 63) as u32);

                trace!("Instruction: i64.shl [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_SHR_S => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = v1.wrapping_shr((v2 & 63) as u32);

                trace!("Instruction: i64.shr_s [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_SHR_U => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = (v1 as u64).wrapping_shr((v2 & 63) as u32);

                trace!("Instruction: i64.shr_u [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_ROTL => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = v1.rotate_left((v2 & 63) as u32);

                trace!("Instruction: i64.rotl [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_ROTR => {
                let v2: i64 = stack.pop_value().try_into().unwrap_validated();
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();

                let res = v1.rotate_right((v2 & 63) as u32);

                trace!("Instruction: i64.rotr [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_REM_U => {
                let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
                let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

                let dividend = dividend as u32;
                let divisor = divisor as u32;

                if dividend == 0 {
                    return Err(TrapError::DivideBy0.into());
                }

                let res = divisor.checked_rem(dividend);
                let res = res.unwrap_or_default() as i32;

                trace!("Instruction: i32.rem_u [{divisor} {dividend}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_AND => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1 & v2;

                trace!("Instruction: i32.and [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_OR => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1 | v2;

                trace!("Instruction: i32.or [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_XOR => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v1 ^ v2;

                trace!("Instruction: i32.xor [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_SHL => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();
                let res = v2.wrapping_shl(v1 as u32);

                trace!("Instruction: i32.shl [{v2} {v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_SHR_S => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = v2.wrapping_shr(v1 as u32);

                trace!("Instruction: i32.shr_s [{v2} {v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_SHR_U => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = (v2 as u32).wrapping_shr(v1 as u32) as i32;

                trace!("Instruction: i32.shr_u [{v2} {v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_ROTL => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = v2.rotate_left(v1 as u32);

                trace!("Instruction: i32.rotl [{v2} {v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_ROTR => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let v2: i32 = stack.pop_value().try_into().unwrap_validated();

                let res = v2.rotate_right(v1 as u32);

                trace!("Instruction: i32.rotr [{v2} {v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            F32_ABS => {
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.abs();

                trace!("Instruction: f32.abs [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_NEG => {
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.neg();

                trace!("Instruction: f32.neg [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_CEIL => {
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.ceil();

                trace!("Instruction: f32.ceil [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_FLOOR => {
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.floor();

                trace!("Instruction: f32.floor [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_TRUNC => {
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.trunc();

                trace!("Instruction: f32.trunc [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_NEAREST => {
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.nearest();

                trace!("Instruction: f32.nearest [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_SQRT => {
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.sqrt();

                trace!("Instruction: f32.sqrt [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_ADD => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1 + v2;

                trace!("Instruction: f32.add [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_SUB => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1 - v2;

                trace!("Instruction: f32.sub [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_MUL => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1 * v2;

                trace!("Instruction: f32.mul [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_DIV => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1 / v2;

                trace!("Instruction: f32.div [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_MIN => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.min(v2);

                trace!("Instruction: f32.min [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_MAX => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.max(v2);

                trace!("Instruction: f32.max [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_COPYSIGN => {
                let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v1.copysign(v2);

                trace!("Instruction: f32.copysign [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            F64_ABS => {
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.abs();

                trace!("Instruction: f64.abs [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_NEG => {
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.neg();

                trace!("Instruction: f64.neg [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_CEIL => {
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.ceil();

                trace!("Instruction: f64.ceil [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_FLOOR => {
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.floor();

                trace!("Instruction: f64.floor [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_TRUNC => {
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.trunc();

                trace!("Instruction: f64.trunc [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_NEAREST => {
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.nearest();

                trace!("Instruction: f64.nearest [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_SQRT => {
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.sqrt();

                trace!("Instruction: f64.sqrt [{v1}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_ADD => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1 + v2;

                trace!("Instruction: f64.add [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_SUB => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1 - v2;

                trace!("Instruction: f64.sub [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_MUL => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1 * v2;

                trace!("Instruction: f64.mul [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_DIV => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1 / v2;

                trace!("Instruction: f64.div [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_MIN => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.min(v2);

                trace!("Instruction: f64.min [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_MAX => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.max(v2);

                trace!("Instruction: f64.max [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_COPYSIGN => {
                let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v1.copysign(v2);

                trace!("Instruction: f64.copysign [{v1} {v2}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_WRAP_I64 => {
                let v: i64 = stack.pop_value().try_into().unwrap_validated();
                let res: i32 = v as i32;

                trace!("Instruction: i32.wrap_i64 [{v}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_TRUNC_F32_S => {
                let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F32(2147483648.0) || v <= value::F32(-2147483904.0) {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i32 = v.as_i32();

                trace!("Instruction: i32.trunc_f32_s [{v:.7}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_TRUNC_F32_U => {
                let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F32(4294967296.0) || v <= value::F32(-1.0) {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i32 = v.as_u32() as i32;

                trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I32_TRUNC_F64_S => {
                let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F64(2147483648.0) || v <= value::F64(-2147483649.0) {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i32 = v.as_i32();

                trace!("Instruction: i32.trunc_f64_s [{v:.7}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_TRUNC_F64_U => {
                let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F64(4294967296.0) || v <= value::F64(-1.0) {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i32 = v.as_u32() as i32;

                trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I64_EXTEND_I32_S => {
                let v: i32 = stack.pop_value().try_into().unwrap_validated();

                let res: i64 = v as i64;

                trace!("Instruction: i64.extend_i32_s [{v}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I64_EXTEND_I32_U => {
                let v: i32 = stack.pop_value().try_into().unwrap_validated();

                let res: i64 = v as u32 as i64;

                trace!("Instruction: i64.extend_i32_u [{v}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I64_TRUNC_F32_S => {
                let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F32(9223372036854775808.0) || v <= value::F32(-9223373136366403584.0)
                {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i64 = v.as_i64();

                trace!("Instruction: i64.trunc_f32_s [{v:.7}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_TRUNC_F32_U => {
                let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F32(18446744073709551616.0) || v <= value::F32(-1.0) {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i64 = v.as_u64() as i64;

                trace!("Instruction: i64.trunc_f32_u [{v:.7}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }

            I64_TRUNC_F64_S => {
                let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F64(9223372036854775808.0) || v <= value::F64(-9223372036854777856.0)
                {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i64 = v.as_i64();

                trace!("Instruction: i64.trunc_f64_s [{v:.17}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_TRUNC_F64_U => {
                let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
                if v.is_infinity() {
                    return Err(TrapError::UnrepresentableResult.into());
                }
                if v.is_nan() {
                    return Err(TrapError::BadConversionToInteger.into());
                }
                if v >= value::F64(18446744073709551616.0) || v <= value::F64(-1.0) {
                    return Err(TrapError::UnrepresentableResult.into());
                }

                let res: i64 = v.as_u64() as i64;

                trace!("Instruction: i64.trunc_f64_u [{v:.17}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_CONVERT_I32_S => {
                let v: i32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = value::F32(v as f32);

                trace!("Instruction: f32.convert_i32_s [{v}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_CONVERT_I32_U => {
                let v: i32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = value::F32(v as u32 as f32);

                trace!("Instruction: f32.convert_i32_u [{v}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_CONVERT_I64_S => {
                let v: i64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = value::F32(v as f32);

                trace!("Instruction: f32.convert_i64_s [{v}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_CONVERT_I64_U => {
                let v: i64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = value::F32(v as u64 as f32);

                trace!("Instruction: f32.convert_i64_u [{v}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_DEMOTE_F64 => {
                let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = v.as_f32();

                trace!("Instruction: f32.demote_f64 [{v:.17}] -> [{res:.7}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_CONVERT_I32_S => {
                let v: i32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = value::F64(v as f64);

                trace!("Instruction: f64.convert_i32_s [{v}] -> [{res:.17}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_CONVERT_I32_U => {
                let v: i32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = value::F64(v as u32 as f64);

                trace!("Instruction: f64.convert_i32_u [{v}] -> [{res:.17}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_CONVERT_I64_S => {
                let v: i64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = value::F64(v as f64);

                trace!("Instruction: f64.convert_i64_s [{v}] -> [{res:.17}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_CONVERT_I64_U => {
                let v: i64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = value::F64(v as u64 as f64);

                trace!("Instruction: f64.convert_i64_u [{v}] -> [{res:.17}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_PROMOTE_F32 => {
                let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = v.as_f64();

                trace!("Instruction: f64.promote_f32 [{v:.7}] -> [{res:.17}]");
                stack.push_value::<T>(res.into())?;
            }
            I32_REINTERPRET_F32 => {
                let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
                let res: i32 = v.reinterpret_as_i32();

                trace!("Instruction: i32.reinterpret_f32 [{v:.7}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            I64_REINTERPRET_F64 => {
                let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
                let res: i64 = v.reinterpret_as_i64();

                trace!("Instruction: i64.reinterpret_f64 [{v:.17}] -> [{res}]");
                stack.push_value::<T>(res.into())?;
            }
            F32_REINTERPRET_I32 => {
                let v1: i32 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F32 = value::F32::from_bits(v1 as u32);

                trace!("Instruction: f32.reinterpret_i32 [{v1}] -> [{res:.7}]");
                stack.push_value::<T>(res.into())?;
            }
            F64_REINTERPRET_I64 => {
                let v1: i64 = stack.pop_value().try_into().unwrap_validated();
                let res: value::F64 = value::F64::from_bits(v1 as u64);

                trace!("Instruction: f64.reinterpret_i64 [{v1}] -> [{res:.17}]");
                stack.push_value::<T>(res.into())?;
            }
            REF_NULL => {
                let reftype = RefType::read(wasm).unwrap_validated();

                stack.push_value::<T>(Value::Ref(Ref::Null(reftype)))?;
                trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
            }
            REF_IS_NULL => {
                let rref: Ref = stack.pop_value().try_into().unwrap_validated();
                let is_null = matches!(rref, Ref::Null(_));

                let res = if is_null { 1 } else { 0 };
                trace!("Instruction: ref.is_null [{}] -> [{}]", rref, res);
                stack.push_value::<T>(Value::I32(res))?;
            }
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
            REF_FUNC => {
                let func_idx = wasm.read_var_u32().unwrap_validated() as FuncIdx;
                let func_addr = *store.modules[current_module_idx]
                    .func_addrs
                    .get(func_idx)
                    .unwrap_validated();
                stack.push_value::<T>(Value::Ref(Ref::Func(func_addr)))?;
            }
            FC_EXTENSIONS => {
                // Should we call instruction hook here as well? Multibyte instruction
                let second_instr = wasm.read_var_u32().unwrap_validated();

                use crate::core::reader::types::opcode::fc_extensions::*;
                match second_instr {
                    I32_TRUNC_SAT_F32_S => {
                        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    I32_TRUNC_SAT_F32_U => {
                        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    I32_TRUNC_SAT_F64_S => {
                        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    I32_TRUNC_SAT_F64_U => {
                        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    I64_TRUNC_SAT_F32_S => {
                        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    I64_TRUNC_SAT_F32_U => {
                        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    I64_TRUNC_SAT_F64_S => {
                        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    I64_TRUNC_SAT_F64_U => {
                        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
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
                        stack.push_value::<T>(res.into())?;
                    }
                    // See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x
                    // Copy a region from a data segment into memory
                    MEMORY_INIT => {
                        //  mappings:
                        //      n => number of bytes to copy
                        //      s => starting pointer in the data segment
                        //      d => destination address to copy to
                        let data_idx = wasm.read_var_u32().unwrap_validated() as DataIdx;
                        let mem_idx = wasm.read_u8().unwrap_validated() as usize;

                        let n: i32 = stack.pop_value().try_into().unwrap_validated();
                        let s: i32 = stack.pop_value().try_into().unwrap_validated();
                        let d: i32 = stack.pop_value().try_into().unwrap_validated();

                        memory_init(
                            &store.modules,
                            &mut store.memories,
                            &store.data,
                            current_module_idx,
                            data_idx,
                            mem_idx,
                            n,
                            s,
                            d,
                        )?;
                    }
                    DATA_DROP => {
                        let data_idx = wasm.read_var_u32().unwrap_validated() as DataIdx;
                        data_drop(
                            &store.modules,
                            &mut store.data,
                            current_module_idx,
                            data_idx,
                        )?;
                    }
                    // See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy
                    MEMORY_COPY => {
                        //  mappings:
                        //      n => number of bytes to copy
                        //      s => source address to copy from
                        //      d => destination address to copy to
                        let (dst_idx, src_idx) = (
                            wasm.read_u8().unwrap_validated() as usize,
                            wasm.read_u8().unwrap_validated() as usize,
                        );
                        let n: i32 = stack.pop_value().try_into().unwrap_validated();
                        let s: i32 = stack.pop_value().try_into().unwrap_validated();
                        let d: i32 = stack.pop_value().try_into().unwrap_validated();

                        let src_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .get(src_idx)
                            .unwrap_validated();
                        let dst_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .get(dst_idx)
                            .unwrap_validated();

                        let src_mem = store.memories.get(src_addr);
                        let dest_mem = store.memories.get(dst_addr);

                        dest_mem
                            .mem
                            .copy(d as MemIdx, &src_mem.mem, s as MemIdx, n as MemIdx)?;
                        trace!("Instruction: memory.copy");
                    }
                    // See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill
                    MEMORY_FILL => {
                        //  mappings:
                        //      n => number of bytes to update
                        //      val => the value to set each byte to (must be < 256)
                        //      d => the pointer to the region to update
                        let mem_idx = wasm.read_u8().unwrap_validated() as usize;
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .get(mem_idx)
                            .unwrap_validated();
                        let mem = store.memories.get(mem_addr);
                        let n: i32 = stack.pop_value().try_into().unwrap_validated();
                        let val: i32 = stack.pop_value().try_into().unwrap_validated();

                        if !(0..=255).contains(&val) {
                            warn!("Value for memory.fill does not fit in a byte ({val})");
                        }

                        let d: i32 = stack.pop_value().try_into().unwrap_validated();

                        mem.mem.fill(d as usize, val as u8, n as usize)?;

                        trace!("Instruction: memory.fill");
                    }
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-init-x-y
                    // https://webassembly.github.io/spec/core/binary/instructions.html#table-instructions
                    // in binary format it seems that elemidx is first ???????
                    // this is ONLY for passive elements
                    TABLE_INIT => {
                        let elem_idx = wasm.read_var_u32().unwrap_validated() as usize;
                        let table_idx = wasm.read_var_u32().unwrap_validated() as usize;

                        let n: i32 = stack.pop_value().try_into().unwrap_validated(); // size
                        let s: i32 = stack.pop_value().try_into().unwrap_validated(); // offset
                        let d: i32 = stack.pop_value().try_into().unwrap_validated(); // dst

                        table_init(
                            &store.modules,
                            &mut store.tables,
                            &store.elements,
                            current_module_idx,
                            elem_idx,
                            table_idx,
                            n,
                            s,
                            d,
                        )?;
                    }
                    ELEM_DROP => {
                        let elem_idx = wasm.read_var_u32().unwrap_validated() as usize;

                        elem_drop(
                            &store.modules,
                            &mut store.elements,
                            current_module_idx,
                            elem_idx,
                        )?;
                    }
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-copy-x-y
                    TABLE_COPY => {
                        let table_x_idx = wasm.read_var_u32().unwrap_validated() as usize;
                        let table_y_idx = wasm.read_var_u32().unwrap_validated() as usize;

                        let tab_x_elem_len = store
                            .tables
                            .get(store.modules[current_module_idx].table_addrs[table_x_idx])
                            .elem
                            .len();
                        let tab_y_elem_len = store
                            .tables
                            .get(store.modules[current_module_idx].table_addrs[table_y_idx])
                            .elem
                            .len();

                        let n: u32 = stack.pop_value().try_into().unwrap_validated(); // size
                        let s: u32 = stack.pop_value().try_into().unwrap_validated(); // source
                        let d: u32 = stack.pop_value().try_into().unwrap_validated(); // destination

                        let src_res = match s.checked_add(n) {
                            Some(res) => {
                                if res > tab_y_elem_len as u32 {
                                    return Err(TrapError::TableOrElementAccessOutOfBounds.into());
                                } else {
                                    res as usize
                                }
                            }
                            _ => return Err(TrapError::TableOrElementAccessOutOfBounds.into()),
                        };

                        let dst_res = match d.checked_add(n) {
                            Some(res) => {
                                if res > tab_x_elem_len as u32 {
                                    return Err(TrapError::TableOrElementAccessOutOfBounds.into());
                                } else {
                                    res as usize
                                }
                            }
                            _ => return Err(TrapError::TableOrElementAccessOutOfBounds.into()),
                        };

                        let dst = table_x_idx;
                        let src = table_y_idx;

                        if table_x_idx == table_y_idx {
                            let table_addr = *store.modules[current_module_idx]
                                .table_addrs
                                .get(table_x_idx)
                                .unwrap_validated();
                            let table = store.tables.get_mut(table_addr);
                            table.elem.copy_within(s as usize..src_res, d as usize);
                        } else {
                            let src_addr = *store.modules[current_module_idx]
                                .table_addrs
                                .get(src)
                                .unwrap_validated();
                            let dst_addr = *store.modules[current_module_idx]
                                .table_addrs
                                .get(dst)
                                .unwrap_validated();

                            let (src_table, dst_table) = store
                                .tables
                                .get_two_mut(src_addr, dst_addr)
                                .expect("both addrs to never be equal");

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
                        let table_idx = wasm.read_var_u32().unwrap_validated() as usize;
                        let table_addr = *store.modules[current_module_idx]
                            .table_addrs
                            .get(table_idx)
                            .unwrap_validated();
                        let tab = &mut store.tables.get_mut(table_addr);

                        let sz = tab.elem.len() as u32;

                        let n: u32 = stack.pop_value().try_into().unwrap_validated();
                        let val: Ref = stack.pop_value().try_into().unwrap_validated();

                        // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
                        // for now we execute it always according to the following match expr.
                        // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the stack per spec
                        match tab.grow(n, val) {
                            Ok(_) => {
                                stack.push_value::<T>(Value::I32(sz))?;
                            }
                            Err(_) => {
                                stack.push_value::<T>(Value::I32(u32::MAX))?;
                            }
                        }
                    }
                    TABLE_SIZE => {
                        let table_idx = wasm.read_var_u32().unwrap_validated() as usize;
                        let table_addr = *store.modules[current_module_idx]
                            .table_addrs
                            .get(table_idx)
                            .unwrap_validated();
                        let tab = store.tables.get(table_addr);

                        let sz = tab.elem.len() as u32;

                        stack.push_value::<T>(Value::I32(sz))?;

                        trace!("Instruction: table.size '{}' [] -> [{}]", table_idx, sz);
                    }
                    TABLE_FILL => {
                        let table_idx = wasm.read_var_u32().unwrap_validated() as usize;
                        let table_addr = *store.modules[current_module_idx]
                            .table_addrs
                            .get(table_idx)
                            .unwrap_validated();
                        let tab = store.tables.get_mut(table_addr);

                        let len: u32 = stack.pop_value().try_into().unwrap_validated();
                        let val: Ref = stack.pop_value().try_into().unwrap_validated();
                        let dst: u32 = stack.pop_value().try_into().unwrap_validated();

                        let end = (dst as usize)
                            .checked_add(len as usize)
                            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

                        tab.elem
                            .get_mut(dst as usize..end)
                            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?
                            .fill(val);

                        trace!(
                            "Instruction table.fill '{}' [{} {} {}] -> []",
                            table_idx,
                            dst,
                            val,
                            len
                        )
                    }
                    _ => unreachable!(),
                }
            }

            I32_EXTEND8_S => {
                let mut v: u32 = stack.pop_value().try_into().unwrap_validated();

                if v | 0xFF != 0xFF {
                    trace!("Number v ({}) not contained in 8 bits, truncating", v);
                    v &= 0xFF;
                }

                let res = if v | 0x7F != 0x7F { v | 0xFFFFFF00 } else { v };

                stack.push_value::<T>(res.into())?;

                trace!("Instruction i32.extend8_s [{}] -> [{}]", v, res);
            }
            I32_EXTEND16_S => {
                let mut v: u32 = stack.pop_value().try_into().unwrap_validated();

                if v | 0xFFFF != 0xFFFF {
                    trace!("Number v ({}) not contained in 16 bits, truncating", v);
                    v &= 0xFFFF;
                }

                let res = if v | 0x7FFF != 0x7FFF {
                    v | 0xFFFF0000
                } else {
                    v
                };

                stack.push_value::<T>(res.into())?;

                trace!("Instruction i32.extend16_s [{}] -> [{}]", v, res);
            }
            I64_EXTEND8_S => {
                let mut v: u64 = stack.pop_value().try_into().unwrap_validated();

                if v | 0xFF != 0xFF {
                    trace!("Number v ({}) not contained in 8 bits, truncating", v);
                    v &= 0xFF;
                }

                let res = if v | 0x7F != 0x7F {
                    v | 0xFFFFFFFF_FFFFFF00
                } else {
                    v
                };

                stack.push_value::<T>(res.into())?;

                trace!("Instruction i64.extend8_s [{}] -> [{}]", v, res);
            }
            I64_EXTEND16_S => {
                let mut v: u64 = stack.pop_value().try_into().unwrap_validated();

                if v | 0xFFFF != 0xFFFF {
                    trace!("Number v ({}) not contained in 16 bits, truncating", v);
                    v &= 0xFFFF;
                }

                let res = if v | 0x7FFF != 0x7FFF {
                    v | 0xFFFFFFFF_FFFF0000
                } else {
                    v
                };

                stack.push_value::<T>(res.into())?;

                trace!("Instruction i64.extend16_s [{}] -> [{}]", v, res);
            }
            I64_EXTEND32_S => {
                let mut v: u64 = stack.pop_value().try_into().unwrap_validated();

                if v | 0xFFFF_FFFF != 0xFFFF_FFFF {
                    trace!("Number v ({}) not contained in 32 bits, truncating", v);
                    v &= 0xFFFF_FFFF;
                }

                let res = if v | 0x7FFF_FFFF != 0x7FFF_FFFF {
                    v | 0xFFFFFFFF_00000000
                } else {
                    v
                };

                stack.push_value::<T>(res.into())?;

                trace!("Instruction i64.extend32_s [{}] -> [{}]", v, res);
            }
            FD_EXTENSIONS => {
                // Should we call instruction hook here as well? Multibyte instruction
                let second_instr = wasm.read_var_u32().unwrap_validated();

                use crate::core::reader::types::opcode::fd_extensions::*;
                match second_instr {
                    V128_LOAD => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let data: u128 = memory.mem.load(idx)?;
                        stack.push_value::<T>(data.to_le_bytes().into())?;
                    }
                    V128_STORE => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        memory.mem.store(idx, u128::from_le_bytes(data))?;
                    }

                    // v128.loadNxM_sx
                    V128_LOAD8X8_S => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

                        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
                        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
                        let half_lanes: [i8; 8] =
                            to_lanes::<1, 16, i8>(data)[..8].try_into().unwrap();

                        let extended_lanes = half_lanes.map(|lane| lane as i16);

                        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
                    }
                    V128_LOAD8X8_U => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

                        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
                        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
                        let half_lanes: [u8; 8] =
                            to_lanes::<1, 16, u8>(data)[..8].try_into().unwrap();

                        let extended_lanes = half_lanes.map(|lane| lane as u16);

                        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
                    }
                    V128_LOAD16X4_S => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

                        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
                        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
                        let half_lanes: [i16; 4] =
                            to_lanes::<2, 8, i16>(data)[..4].try_into().unwrap();

                        let extended_lanes = half_lanes.map(|lane| lane as i32);

                        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
                    }
                    V128_LOAD16X4_U => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

                        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
                        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
                        let half_lanes: [u16; 4] =
                            to_lanes::<2, 8, u16>(data)[..4].try_into().unwrap();

                        let extended_lanes = half_lanes.map(|lane| lane as u32);

                        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
                    }
                    V128_LOAD32X2_S => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

                        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
                        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
                        let half_lanes: [i32; 2] =
                            to_lanes::<4, 4, i32>(data)[..2].try_into().unwrap();

                        let extended_lanes = half_lanes.map(|lane| lane as i64);

                        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
                    }
                    V128_LOAD32X2_U => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

                        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
                        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
                        let half_lanes: [u32; 2] =
                            to_lanes::<4, 4, u32>(data)[..2].try_into().unwrap();

                        let extended_lanes = half_lanes.map(|lane| lane as u64);

                        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
                    }

                    // v128.loadN_splat
                    V128_LOAD8_SPLAT => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let lane = memory.mem.load::<1, u8>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes([lane; 16])))?;
                    }
                    V128_LOAD16_SPLAT => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let lane = memory.mem.load::<2, u16>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes([lane; 8])))?;
                    }
                    V128_LOAD32_SPLAT => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let lane = memory.mem.load::<4, u32>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes([lane; 4])))?;
                    }
                    V128_LOAD64_SPLAT => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let lane = memory.mem.load::<8, u64>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes([lane; 2])))?;
                    }

                    // v128.loadN_zero
                    V128_LOAD32_ZERO => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let data = memory.mem.load::<4, u32>(idx)? as u128;
                        stack.push_value::<T>(Value::V128(data.to_le_bytes()))?;
                    }
                    V128_LOAD64_ZERO => {
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);

                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let idx = calculate_mem_address(&memarg, relative_address)?;

                        let data = memory.mem.load::<8, u64>(idx)? as u128;
                        stack.push_value::<T>(Value::V128(data.to_le_bytes()))?;
                    }

                    // v128.loadN_lane
                    V128_LOAD8_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let mut lanes: [u8; 16] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() =
                            memory.mem.load::<1, u8>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }

                    V128_LOAD16_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let mut lanes: [u16; 8] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() =
                            memory.mem.load::<2, u16>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }
                    V128_LOAD32_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let mut lanes: [u32; 4] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() =
                            memory.mem.load::<4, u32>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }
                    V128_LOAD64_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let mut lanes: [u64; 2] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() =
                            memory.mem.load::<8, u64>(idx)?;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }

                    // v128.storeN_lane
                    V128_STORE8_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;

                        let lane = *to_lanes::<1, 16, u8>(data).get(lane_idx).unwrap_validated();

                        memory.mem.store::<1, u8>(idx, lane)?;
                    }
                    V128_STORE16_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;

                        let lane = *to_lanes::<2, 8, u16>(data).get(lane_idx).unwrap_validated();

                        memory.mem.store::<2, u16>(idx, lane)?;
                    }
                    V128_STORE32_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;

                        let lane = *to_lanes::<4, 4, u32>(data).get(lane_idx).unwrap_validated();

                        memory.mem.store::<4, u32>(idx, lane)?;
                    }
                    V128_STORE64_LANE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
                        let memarg = MemArg::read(wasm).unwrap_validated();
                        let mem_addr = *store.modules[current_module_idx]
                            .mem_addrs
                            .first()
                            .unwrap_validated();
                        let memory = store.memories.get(mem_addr);
                        let idx = calculate_mem_address(&memarg, relative_address)?;
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;

                        let lane = *to_lanes::<8, 2, u64>(data).get(lane_idx).unwrap_validated();

                        memory.mem.store::<8, u64>(idx, lane)?;
                    }

                    V128_CONST => {
                        let mut data = [0; 16];
                        for byte_ref in &mut data {
                            *byte_ref = wasm.read_u8().unwrap_validated();
                        }

                        stack.push_value::<T>(Value::V128(data))?;
                    }

                    // vvunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvunop>
                    V128_NOT => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        stack.push_value::<T>(Value::V128(data.map(|byte| !byte)))?;
                    }

                    // vvbinop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvbinop>
                    V128_AND => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let result = array::from_fn(|i| data1[i] & data2[i]);
                        stack.push_value::<T>(Value::V128(result))?;
                    }
                    V128_ANDNOT => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let result = array::from_fn(|i| data1[i] & !data2[i]);
                        stack.push_value::<T>(Value::V128(result))?;
                    }
                    V128_OR => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let result = array::from_fn(|i| data1[i] | data2[i]);
                        stack.push_value::<T>(Value::V128(result))?;
                    }
                    V128_XOR => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let result = array::from_fn(|i| data1[i] ^ data2[i]);
                        stack.push_value::<T>(Value::V128(result))?;
                    }

                    // vvternop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvternop>
                    V128_BITSELECT => {
                        let data3: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let result =
                            array::from_fn(|i| (data1[i] & data3[i]) | (data2[i] & !data3[i]));
                        stack.push_value::<T>(Value::V128(result))?;
                    }

                    // vvtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvtestop>
                    V128_ANY_TRUE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let any_true = data.into_iter().any(|byte| byte > 0);
                        stack.push_value::<T>(Value::I32(any_true as u32))?;
                    }

                    I8X16_SWIZZLE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let result =
                            array::from_fn(|i| *data1.get(data2[i] as usize).unwrap_or(&0));
                        stack.push_value::<T>(Value::V128(result))?;
                    }

                    I8X16_SHUFFLE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();

                        let lane_selector_indices: [u8; 16] =
                            array::from_fn(|_| wasm.read_u8().unwrap_validated());

                        let result = lane_selector_indices.map(|i| {
                            *data1
                                .get(i as usize)
                                .or_else(|| data2.get(i as usize - 16))
                                .unwrap_validated()
                        });

                        stack.push_value::<T>(Value::V128(result))?;
                    }

                    // shape.splat
                    I8X16_SPLAT => {
                        let value: u32 = stack.pop_value().try_into().unwrap_validated();
                        let lane = value as u8;
                        let data = from_lanes([lane; 16]);
                        stack.push_value::<T>(Value::V128(data))?;
                    }
                    I16X8_SPLAT => {
                        let value: u32 = stack.pop_value().try_into().unwrap_validated();
                        let lane = value as u16;
                        let data = from_lanes([lane; 8]);
                        stack.push_value::<T>(Value::V128(data))?;
                    }
                    I32X4_SPLAT => {
                        let lane: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data = from_lanes([lane; 4]);
                        stack.push_value::<T>(Value::V128(data))?;
                    }
                    I64X2_SPLAT => {
                        let lane: u64 = stack.pop_value().try_into().unwrap_validated();
                        let data = from_lanes([lane; 2]);
                        stack.push_value::<T>(Value::V128(data))?;
                    }
                    F32X4_SPLAT => {
                        let lane: F32 = stack.pop_value().try_into().unwrap_validated();
                        let data = from_lanes([lane; 4]);
                        stack.push_value::<T>(Value::V128(data))?;
                    }
                    F64X2_SPLAT => {
                        let lane: F64 = stack.pop_value().try_into().unwrap_validated();
                        let data = from_lanes([lane; 2]);
                        stack.push_value::<T>(Value::V128(data))?;
                    }

                    // shape.extract_lane
                    I8X16_EXTRACT_LANE_S => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::I32(lane as u32))?;
                    }
                    I8X16_EXTRACT_LANE_U => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::I32(lane as u32))?;
                    }
                    I16X8_EXTRACT_LANE_S => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::I32(lane as u32))?;
                    }
                    I16X8_EXTRACT_LANE_U => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u16; 8] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::I32(lane as u32))?;
                    }
                    I32X4_EXTRACT_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::I32(lane))?;
                    }
                    I64X2_EXTRACT_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u64; 2] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::I64(lane))?;
                    }
                    F32X4_EXTRACT_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::F32(lane))?;
                    }
                    F64X2_EXTRACT_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let lane = *lanes.get(lane_idx).unwrap_validated();
                        stack.push_value::<T>(Value::F64(lane))?;
                    }

                    // shape.replace_lane
                    I8X16_REPLACE_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let value: u32 = stack.pop_value().try_into().unwrap_validated();
                        let new_lane = value as u8;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let mut lanes: [u8; 16] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }
                    I16X8_REPLACE_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let value: u32 = stack.pop_value().try_into().unwrap_validated();
                        let new_lane = value as u16;
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let mut lanes: [u16; 8] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }
                    I32X4_REPLACE_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let new_lane: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let mut lanes: [u32; 4] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }
                    I64X2_REPLACE_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let new_lane: u64 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let mut lanes: [u64; 2] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }
                    F32X4_REPLACE_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let new_lane: F32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let mut lanes: [F32; 4] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }
                    F64X2_REPLACE_LANE => {
                        let lane_idx = wasm.read_u8().unwrap_validated() as usize;
                        let new_lane: F64 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let mut lanes: [F64; 2] = to_lanes(data);
                        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
                        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
                    }

                    // Group vunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vunop>
                    // viunop
                    I8X16_ABS => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let result: [i8; 16] = lanes.map(i8::wrapping_abs);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_ABS => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let result: [i16; 8] = lanes.map(i16::wrapping_abs);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_ABS => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let result: [i32; 4] = lanes.map(i32::wrapping_abs);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_ABS => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i64; 2] = to_lanes(data);
                        let result: [i64; 2] = lanes.map(i64::wrapping_abs);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_NEG => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let result: [i8; 16] = lanes.map(i8::wrapping_neg);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_NEG => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let result: [i16; 8] = lanes.map(i16::wrapping_neg);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_NEG => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let result: [i32; 4] = lanes.map(i32::wrapping_neg);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_NEG => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i64; 2] = to_lanes(data);
                        let result: [i64; 2] = lanes.map(i64::wrapping_neg);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    // vfunop
                    F32X4_ABS => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| lane.abs());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_ABS => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result: [F64; 2] = lanes.map(|lane| lane.abs());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_NEG => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| lane.neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_NEG => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result: [F64; 2] = lanes.map(|lane| lane.neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_SQRT => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| lane.sqrt());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_SQRT => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result: [F64; 2] = lanes.map(|lane| lane.sqrt());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_CEIL => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| lane.ceil());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_CEIL => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result: [F64; 2] = lanes.map(|lane| lane.ceil());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_FLOOR => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| lane.floor());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_FLOOR => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result: [F64; 2] = lanes.map(|lane| lane.floor());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_TRUNC => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| lane.trunc());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_TRUNC => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result: [F64; 2] = lanes.map(|lane| lane.trunc());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_NEAREST => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| lane.nearest());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_NEAREST => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result: [F64; 2] = lanes.map(|lane| lane.nearest());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    // others
                    I8X16_POPCNT => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let result: [u8; 16] = lanes.map(|lane| lane.count_ones() as u8);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    // Group vbinop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vbinop>
                    // vibinop
                    I8X16_ADD => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [u8; 16] =
                            array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_ADD => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] =
                            array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_ADD => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [u32; 4] =
                            array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_ADD => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u64; 2] = to_lanes(data2);
                        let lanes1: [u64; 2] = to_lanes(data1);
                        let result: [u64; 2] =
                            array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_SUB => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [u8; 16] =
                            array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_SUB => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] =
                            array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_SUB => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [u32; 4] =
                            array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_SUB => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u64; 2] = to_lanes(data2);
                        let lanes1: [u64; 2] = to_lanes(data1);
                        let result: [u64; 2] =
                            array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    // vfbinop
                    F32X4_ADD => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_ADD => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_SUB => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_SUB => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_MUL => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_MUL => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_DIV => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_DIV => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_MIN => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_MIN => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_MAX => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_MAX => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_PMIN => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| {
                            let v1 = lanes1[i];
                            let v2 = lanes2[i];
                            if v2 < v1 {
                                v2
                            } else {
                                v1
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_PMIN => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| {
                            let v1 = lanes1[i];
                            let v2 = lanes2[i];
                            if v2 < v1 {
                                v2
                            } else {
                                v1
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_PMAX => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [F32; 4] = array::from_fn(|i| {
                            let v1 = lanes1[i];
                            let v2 = lanes2[i];
                            if v1 < v2 {
                                v2
                            } else {
                                v1
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_PMAX => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [F64; 2] = array::from_fn(|i| {
                            let v1 = lanes1[i];
                            let v2 = lanes2[i];
                            if v1 < v2 {
                                v2
                            } else {
                                v1
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    // viminmaxop
                    I8X16_MIN_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_MIN_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_MIN_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let result: [i32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_MIN_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [u8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_MIN_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_MIN_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [u32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_MAX_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_MAX_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_MAX_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let result: [i32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_MAX_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [u8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_MAX_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_MAX_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [u32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    // visatbinop
                    I8X16_ADD_SAT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_ADD_SAT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_ADD_SAT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [u8; 16] =
                            array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_ADD_SAT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] =
                            array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_SUB_SAT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_SUB_SAT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_SUB_SAT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [u8; 16] =
                            array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_SUB_SAT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] =
                            array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    // others
                    I16X8_MUL => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] =
                            array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_MUL => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [u32; 4] =
                            array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_MUL => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u64; 2] = to_lanes(data2);
                        let lanes1: [u64; 2] = to_lanes(data1);
                        let result: [u64; 2] =
                            array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_AVGR_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [u8; 16] = array::from_fn(|i| {
                            (lanes1[i] as u16 + lanes2[i] as u16).div_ceil(2) as u8
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_AVGR_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [u16; 8] = array::from_fn(|i| {
                            (lanes1[i] as u32 + lanes2[i] as u32).div_ceil(2) as u16
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_Q15MULRSAT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] = array::from_fn(|i| {
                            (((lanes1[i] as i64).mul(lanes2[i] as i64) + 2i64.pow(14)) >> 15i64)
                                .clamp(i16::MIN as i64, i16::MAX as i64)
                                as i16
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    // Group vrelop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vrelop>
                    // virelop
                    I8X16_EQ => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_EQ => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_EQ => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_EQ => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u64; 2] = to_lanes(data2);
                        let lanes1: [u64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_NE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_NE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_NE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_NE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u64; 2] = to_lanes(data2);
                        let lanes1: [u64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_LT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_LT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_LT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_LT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i64; 2] = to_lanes(data2);
                        let lanes1: [i64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_LT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_LT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_LT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_GT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_GT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_GT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_GT_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i64; 2] = to_lanes(data2);
                        let lanes1: [i64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_GT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_GT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_GT_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_LE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_LE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_LE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_LE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i64; 2] = to_lanes(data2);
                        let lanes1: [i64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_LE_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_LE_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_LE_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    I8X16_GE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_GE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_GE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_GE_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i64; 2] = to_lanes(data2);
                        let lanes1: [i64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_GE_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let result: [i8; 16] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_GE_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let result: [i16; 8] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_GE_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    // vfrelop
                    F32X4_EQ => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_EQ => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_NE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_NE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_LT => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_LT => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_GT => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_GT => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_LE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_LE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_GE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F32; 4] = to_lanes(data2);
                        let lanes1: [F32; 4] = to_lanes(data1);
                        let result: [i32; 4] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_GE => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [F64; 2] = to_lanes(data2);
                        let lanes1: [F64; 2] = to_lanes(data1);
                        let result: [i64; 2] =
                            array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    // Group vishiftop
                    I8X16_SHL => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shl(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_SHL => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u16; 8] = to_lanes(data);
                        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shl(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_SHL => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shl(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_SHL => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u64; 2] = to_lanes(data);
                        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shl(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_SHR_S => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let result: [i8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_SHR_U => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_SHR_S => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let result: [i16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_SHR_U => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u16; 8] = to_lanes(data);
                        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_SHR_S => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let result: [i32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_SHR_U => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_SHR_S => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i64; 2] = to_lanes(data);
                        let result: [i64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_SHR_U => {
                        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u64; 2] = to_lanes(data);
                        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    // Group vtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vtestop>
                    // vitestop
                    I8X16_ALL_TRUE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let all_true = lanes.into_iter().all(|lane| lane != 0);
                        stack.push_value::<T>(Value::I32(all_true as u32))?;
                    }
                    I16X8_ALL_TRUE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u16; 8] = to_lanes(data);
                        let all_true = lanes.into_iter().all(|lane| lane != 0);
                        stack.push_value::<T>(Value::I32(all_true as u32))?;
                    }
                    I32X4_ALL_TRUE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let all_true = lanes.into_iter().all(|lane| lane != 0);
                        stack.push_value::<T>(Value::I32(all_true as u32))?;
                    }
                    I64X2_ALL_TRUE => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u64; 2] = to_lanes(data);
                        let all_true = lanes.into_iter().all(|lane| lane != 0);
                        stack.push_value::<T>(Value::I32(all_true as u32))?;
                    }

                    // Group vcvtop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vcvtop>
                    I16X8_EXTEND_HIGH_I8X16_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let high_lanes: [i8; 8] = lanes[8..].try_into().unwrap();
                        let result = high_lanes.map(|lane| lane as i16);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_EXTEND_HIGH_I8X16_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let high_lanes: [u8; 8] = lanes[8..].try_into().unwrap();
                        let result = high_lanes.map(|lane| lane as u16);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_EXTEND_LOW_I8X16_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let low_lanes: [i8; 8] = lanes[..8].try_into().unwrap();
                        let result = low_lanes.map(|lane| lane as i16);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_EXTEND_LOW_I8X16_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let low_lanes: [u8; 8] = lanes[..8].try_into().unwrap();
                        let result = low_lanes.map(|lane| lane as u16);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_EXTEND_HIGH_I16X8_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let high_lanes: [i16; 4] = lanes[4..].try_into().unwrap();
                        let result = high_lanes.map(|lane| lane as i32);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_EXTEND_HIGH_I16X8_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u16; 8] = to_lanes(data);
                        let high_lanes: [u16; 4] = lanes[4..].try_into().unwrap();
                        let result = high_lanes.map(|lane| lane as u32);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_EXTEND_LOW_I16X8_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let low_lanes: [i16; 4] = lanes[..4].try_into().unwrap();
                        let result = low_lanes.map(|lane| lane as i32);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_EXTEND_LOW_I16X8_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u16; 8] = to_lanes(data);
                        let low_lanes: [u16; 4] = lanes[..4].try_into().unwrap();
                        let result = low_lanes.map(|lane| lane as u32);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_EXTEND_HIGH_I32X4_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let high_lanes: [i32; 2] = lanes[2..].try_into().unwrap();
                        let result = high_lanes.map(|lane| lane as i64);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_EXTEND_HIGH_I32X4_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let high_lanes: [u32; 2] = lanes[2..].try_into().unwrap();
                        let result = high_lanes.map(|lane| lane as u64);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_EXTEND_LOW_I32X4_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
                        let result = low_lanes.map(|lane| lane as i64);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I64X2_EXTEND_LOW_I32X4_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
                        let result = low_lanes.map(|lane| lane as u64);
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_TRUNC_SAT_F32X4_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result = lanes.map(|lane| {
                            if lane.is_nan() {
                                0
                            } else if lane.is_negative_infinity() {
                                i32::MIN
                            } else if lane.is_infinity() {
                                i32::MAX
                            } else {
                                lane.as_i32()
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_TRUNC_SAT_F32X4_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let result = lanes.map(|lane| {
                            if lane.is_nan() {
                                0
                            } else if lane.is_negative_infinity() {
                                u32::MIN
                            } else if lane.is_infinity() {
                                u32::MAX
                            } else {
                                lane.as_u32()
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I32X4_TRUNC_SAT_F64X2_S_ZERO => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result = lanes.map(|lane| {
                            if lane.is_nan() {
                                0
                            } else if lane.is_negative_infinity() {
                                i32::MIN
                            } else if lane.is_infinity() {
                                i32::MAX
                            } else {
                                lane.as_i32()
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes([
                            result[0], result[1], 0, 0,
                        ])))?;
                    }
                    I32X4_TRUNC_SAT_F64X2_U_ZERO => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F64; 2] = to_lanes(data);
                        let result = lanes.map(|lane| {
                            if lane.is_nan() {
                                0
                            } else if lane.is_negative_infinity() {
                                u32::MIN
                            } else if lane.is_infinity() {
                                u32::MAX
                            } else {
                                lane.as_u32()
                            }
                        });
                        stack.push_value::<T>(Value::V128(from_lanes([
                            result[0], result[1], 0, 0,
                        ])))?;
                    }
                    F32X4_CONVERT_I32X4_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_CONVERT_I32X4_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_CONVERT_LOW_I32X4_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
                        let result = low_lanes.map(|lane| F64(lane as f64));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_CONVERT_LOW_I32X4_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u32; 4] = to_lanes(data);
                        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
                        let result = low_lanes.map(|lane| F64(lane as f64));
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F32X4_DEMOTE_F64X2_ZERO => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes = to_lanes::<8, 2, F64>(data);
                        let half_lanes = lanes.map(|lane| lane.as_f32());
                        let result = [half_lanes[0], half_lanes[1], F32(0.0), F32(0.0)];
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    F64X2_PROMOTE_LOW_F32X4 => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [F32; 4] = to_lanes(data);
                        let half_lanes: [F32; 2] = lanes[..2].try_into().unwrap();
                        let result = half_lanes.map(|lane| lane.as_f64());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    // ishape.narrow_ishape_sx
                    I8X16_NARROW_I16X8_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let mut concatenated_narrowed_lanes = lanes1
                            .into_iter()
                            .chain(lanes2)
                            .map(|lane| lane.clamp(i8::MIN as i16, i8::MAX as i16) as i8);
                        let result: [i8; 16] =
                            array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I8X16_NARROW_I16X8_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let mut concatenated_narrowed_lanes = lanes1
                            .into_iter()
                            .chain(lanes2)
                            .map(|lane| lane.clamp(u8::MIN as i16, u8::MAX as i16) as u8);
                        let result: [u8; 16] =
                            array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_NARROW_I32X4_S => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let mut concatenated_narrowed_lanes = lanes1
                            .into_iter()
                            .chain(lanes2)
                            .map(|lane| lane.clamp(i16::MIN as i32, i16::MAX as i32) as i16);
                        let result: [i16; 8] =
                            array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }
                    I16X8_NARROW_I32X4_U => {
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let mut concatenated_narrowed_lanes = lanes1
                            .into_iter()
                            .chain(lanes2)
                            .map(|lane| lane.clamp(u16::MIN as i32, u16::MAX as i32) as u16);
                        let result: [u16; 8] =
                            array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
                        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
                    }

                    // ishape.bitmask
                    I8X16_BITMASK => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let bits = lanes.map(|lane| lane < 0);
                        let bitmask = bits
                            .into_iter()
                            .enumerate()
                            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
                        stack.push_value::<T>(Value::I32(bitmask))?;
                    }
                    I16X8_BITMASK => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let bits = lanes.map(|lane| lane < 0);
                        let bitmask = bits
                            .into_iter()
                            .enumerate()
                            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
                        stack.push_value::<T>(Value::I32(bitmask))?;
                    }
                    I32X4_BITMASK => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i32; 4] = to_lanes(data);
                        let bits = lanes.map(|lane| lane < 0);
                        let bitmask = bits
                            .into_iter()
                            .enumerate()
                            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
                        stack.push_value::<T>(Value::I32(bitmask))?;
                    }
                    I64X2_BITMASK => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i64; 2] = to_lanes(data);
                        let bits = lanes.map(|lane| lane < 0);
                        let bitmask = bits
                            .into_iter()
                            .enumerate()
                            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
                        stack.push_value::<T>(Value::I32(bitmask))?;
                    }

                    // ishape.dot_ishape_s
                    I32X4_DOT_I16X8_S => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let multiplied: [i32; 8] = array::from_fn(|i| {
                            let v1 = lanes1[i] as i32;
                            let v2 = lanes2[i] as i32;
                            v1.wrapping_mul(v2)
                        });
                        let added: [i32; 4] = array::from_fn(|i| {
                            let v1 = multiplied[2 * i];
                            let v2 = multiplied[2 * i + 1];
                            v1.wrapping_add(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(added)))?;
                    }

                    // ishape.extmul_half_ishape_sx
                    I16X8_EXTMUL_HIGH_I8X16_S => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let high_lanes1: [i8; 8] = lanes1[8..].try_into().unwrap();
                        let high_lanes2: [i8; 8] = lanes2[8..].try_into().unwrap();
                        let multiplied: [i16; 8] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as i16;
                            let v2 = high_lanes2[i] as i16;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I16X8_EXTMUL_HIGH_I8X16_U => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let high_lanes1: [u8; 8] = lanes1[8..].try_into().unwrap();
                        let high_lanes2: [u8; 8] = lanes2[8..].try_into().unwrap();
                        let multiplied: [u16; 8] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as u16;
                            let v2 = high_lanes2[i] as u16;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I16X8_EXTMUL_LOW_I8X16_S => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [i8; 16] = to_lanes(data1);
                        let lanes2: [i8; 16] = to_lanes(data2);
                        let high_lanes1: [i8; 8] = lanes1[..8].try_into().unwrap();
                        let high_lanes2: [i8; 8] = lanes2[..8].try_into().unwrap();
                        let multiplied: [i16; 8] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as i16;
                            let v2 = high_lanes2[i] as i16;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I16X8_EXTMUL_LOW_I8X16_U => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [u8; 16] = to_lanes(data1);
                        let lanes2: [u8; 16] = to_lanes(data2);
                        let high_lanes1: [u8; 8] = lanes1[..8].try_into().unwrap();
                        let high_lanes2: [u8; 8] = lanes2[..8].try_into().unwrap();
                        let multiplied: [u16; 8] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as u16;
                            let v2 = high_lanes2[i] as u16;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I32X4_EXTMUL_HIGH_I16X8_S => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let high_lanes1: [i16; 4] = lanes1[4..].try_into().unwrap();
                        let high_lanes2: [i16; 4] = lanes2[4..].try_into().unwrap();
                        let multiplied: [i32; 4] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as i32;
                            let v2 = high_lanes2[i] as i32;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I32X4_EXTMUL_HIGH_I16X8_U => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let high_lanes1: [u16; 4] = lanes1[4..].try_into().unwrap();
                        let high_lanes2: [u16; 4] = lanes2[4..].try_into().unwrap();
                        let multiplied: [u32; 4] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as u32;
                            let v2 = high_lanes2[i] as u32;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I32X4_EXTMUL_LOW_I16X8_S => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [i16; 8] = to_lanes(data1);
                        let lanes2: [i16; 8] = to_lanes(data2);
                        let high_lanes1: [i16; 4] = lanes1[..4].try_into().unwrap();
                        let high_lanes2: [i16; 4] = lanes2[..4].try_into().unwrap();
                        let multiplied: [i32; 4] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as i32;
                            let v2 = high_lanes2[i] as i32;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I32X4_EXTMUL_LOW_I16X8_U => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [u16; 8] = to_lanes(data1);
                        let lanes2: [u16; 8] = to_lanes(data2);
                        let high_lanes1: [u16; 4] = lanes1[..4].try_into().unwrap();
                        let high_lanes2: [u16; 4] = lanes2[..4].try_into().unwrap();
                        let multiplied: [u32; 4] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as u32;
                            let v2 = high_lanes2[i] as u32;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I64X2_EXTMUL_HIGH_I32X4_S => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let high_lanes1: [i32; 2] = lanes1[2..].try_into().unwrap();
                        let high_lanes2: [i32; 2] = lanes2[2..].try_into().unwrap();
                        let multiplied: [i64; 2] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as i64;
                            let v2 = high_lanes2[i] as i64;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I64X2_EXTMUL_HIGH_I32X4_U => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let high_lanes1: [u32; 2] = lanes1[2..].try_into().unwrap();
                        let high_lanes2: [u32; 2] = lanes2[2..].try_into().unwrap();
                        let multiplied: [u64; 2] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as u64;
                            let v2 = high_lanes2[i] as u64;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I64X2_EXTMUL_LOW_I32X4_S => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [i32; 4] = to_lanes(data1);
                        let lanes2: [i32; 4] = to_lanes(data2);
                        let high_lanes1: [i32; 2] = lanes1[..2].try_into().unwrap();
                        let high_lanes2: [i32; 2] = lanes2[..2].try_into().unwrap();
                        let multiplied: [i64; 2] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as i64;
                            let v2 = high_lanes2[i] as i64;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }
                    I64X2_EXTMUL_LOW_I32X4_U => {
                        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes1: [u32; 4] = to_lanes(data1);
                        let lanes2: [u32; 4] = to_lanes(data2);
                        let high_lanes1: [u32; 2] = lanes1[..2].try_into().unwrap();
                        let high_lanes2: [u32; 2] = lanes2[..2].try_into().unwrap();
                        let multiplied: [u64; 2] = array::from_fn(|i| {
                            let v1 = high_lanes1[i] as u64;
                            let v2 = high_lanes2[i] as u64;
                            v1.wrapping_mul(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
                    }

                    // ishape.extadd_pairwise_ishape_sx
                    I16X8_EXTADD_PAIRWISE_I8X16_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i8; 16] = to_lanes(data);
                        let added_pairwise: [i16; 8] = array::from_fn(|i| {
                            let v1 = lanes[2 * i] as i16;
                            let v2 = lanes[2 * i + 1] as i16;
                            v1.wrapping_add(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
                    }
                    I16X8_EXTADD_PAIRWISE_I8X16_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u8; 16] = to_lanes(data);
                        let added_pairwise: [u16; 8] = array::from_fn(|i| {
                            let v1 = lanes[2 * i] as u16;
                            let v2 = lanes[2 * i + 1] as u16;
                            v1.wrapping_add(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
                    }
                    I32X4_EXTADD_PAIRWISE_I16X8_S => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [i16; 8] = to_lanes(data);
                        let added_pairwise: [i32; 4] = array::from_fn(|i| {
                            let v1 = lanes[2 * i] as i32;
                            let v2 = lanes[2 * i + 1] as i32;
                            v1.wrapping_add(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
                    }
                    I32X4_EXTADD_PAIRWISE_I16X8_U => {
                        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
                        let lanes: [u16; 8] = to_lanes(data);
                        let added_pairwise: [u32; 4] = array::from_fn(|i| {
                            let v1 = lanes[2 * i] as u32;
                            let v2 = lanes[2 * i + 1] as u32;
                            v1.wrapping_add(v2)
                        });
                        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
                    }

                    // Unimplemented or invalid instructions
                    F32X4_RELAXED_MADD
                    | F32X4_RELAXED_MAX
                    | F32X4_RELAXED_MIN
                    | F32X4_RELAXED_NMADD
                    | F64X2_RELAXED_MADD
                    | F64X2_RELAXED_MAX
                    | F64X2_RELAXED_MIN
                    | F64X2_RELAXED_NMADD
                    | I16X8_RELAXED_LANESELECT
                    | I32X4_RELAXED_LANESELECT
                    | I32X4_RELAXED_TRUNC_F32X4_S
                    | I32X4_RELAXED_TRUNC_F32X4_U
                    | I32X4_RELAXED_TRUNC_F64X2_S_ZERO
                    | I32X4_RELAXED_TRUNC_F64X2_U_ZERO
                    | I64X2_RELAXED_LANESELECT
                    | I8X16_RELAXED_LANESELECT
                    | I8X16_RELAXED_SWIZZLE
                    | 154
                    | 187
                    | 194
                    | 256.. => unreachable_validated!(),
                }
            }

            // Unimplemented or invalid instructions
            0x06..=0x0A
            | 0x12..=0x19
            | 0x1C..=0x1F
            | 0x25..=0x27
            | 0xC0..=0xFA
            | 0xFB
            | 0xFE
            | 0xFF => {
                unreachable_validated!();
            }
        }
    }
    Ok(None)
}

//helper function for avoiding code duplication at intraprocedural jumps
fn do_sidetable_control_transfer(
    wasm: &mut WasmReader,
    stack: &mut Stack,
    current_stp: &mut usize,
    current_sidetable: &Sidetable,
) -> Result<(), RuntimeError> {
    let sidetable_entry = &current_sidetable[*current_stp];

    stack.remove_in_between(sidetable_entry.popcnt, sidetable_entry.valcnt);

    *current_stp = (*current_stp as isize + sidetable_entry.delta_stp) as usize;
    wasm.pc = (wasm.pc as isize + sidetable_entry.delta_pc) as usize;

    Ok(())
}

#[inline(always)]
fn calculate_mem_address(memarg: &MemArg, relative_address: u32) -> Result<usize, RuntimeError> {
    memarg
        .offset
        // The spec states that this should be a 33 bit integer, e.g. it is not legal to wrap if the
        // sum of offset and relative_address exceeds u32::MAX. To emulate this behavior, we use a
        // checked addition.
        // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
        .checked_add(relative_address)
        .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?
        .try_into()
        .map_err(|_| TrapError::MemoryOrDataAccessOutOfBounds.into())
}

//helpers for avoiding code duplication during module instantiation
#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub(super) fn table_init(
    store_modules: &[ModuleInst],
    store_tables: &mut AddrVec<TableAddr, TableInst>,
    store_elements: &AddrVec<ElemAddr, ElemInst>,
    current_module_idx: usize,
    elem_idx: usize,
    table_idx: usize,
    n: i32,
    s: i32,
    d: i32,
) -> Result<(), RuntimeError> {
    let tab_addr = *store_modules[current_module_idx]
        .table_addrs
        .get(table_idx)
        .unwrap_validated();
    let elem_addr = *store_modules[current_module_idx]
        .elem_addrs
        .get(elem_idx)
        .unwrap_validated();

    let tab = store_tables.get_mut(tab_addr);

    let elem = store_elements.get(elem_addr);

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
        .filter(|&res| res <= elem.len())
        .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

    if (d as usize)
        .checked_add(n as usize)
        .filter(|&res| res <= tab.len())
        .is_none()
    {
        return Err(TrapError::TableOrElementAccessOutOfBounds.into());
    }

    let dest = &mut tab.elem[d as usize..];
    let src = &elem.references[s as usize..final_src_offset];
    dest[..src.len()].copy_from_slice(src);
    Ok(())
}

#[inline(always)]
pub(super) fn elem_drop(
    store_modules: &[ModuleInst],
    store_elements: &mut AddrVec<ElemAddr, ElemInst>,
    current_module_idx: usize,
    elem_idx: usize,
) -> Result<(), RuntimeError> {
    // WARN: i'm not sure if this is okay or not
    let elem_addr = *store_modules[current_module_idx]
        .elem_addrs
        .get(elem_idx)
        .unwrap_validated();
    store_elements.get_mut(elem_addr).references.clear();
    Ok(())
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub(super) fn memory_init(
    store_modules: &[ModuleInst],
    store_memories: &mut AddrVec<MemAddr, MemInst>,
    store_data: &AddrVec<DataAddr, DataInst>,
    current_module_idx: usize,
    data_idx: usize,
    mem_idx: usize,
    n: i32,
    s: i32,
    d: i32,
) -> Result<(), RuntimeError> {
    let mem_addr = *store_modules[current_module_idx]
        .mem_addrs
        .get(mem_idx)
        .unwrap_validated();
    let mem = store_memories.get(mem_addr);

    mem.mem.init(
        d as MemIdx,
        &store_data
            .get(store_modules[current_module_idx].data_addrs[data_idx])
            .data,
        s as MemIdx,
        n as MemIdx,
    )?;

    trace!("Instruction: memory.init");
    Ok(())
}

#[inline(always)]
pub(super) fn data_drop(
    store_modules: &[ModuleInst],
    store_data: &mut AddrVec<DataAddr, DataInst>,
    current_module_idx: usize,
    data_idx: usize,
) -> Result<(), RuntimeError> {
    // Here is debatable
    // If we were to be on par with the spec we'd have to use a DataInst struct
    // But since memory.init is specifically made for Passive data segments
    // I thought that using DataMode would be better because we can see if the
    // data segment is passive or active

    // Also, we should set data to null here (empty), which we do by clearing it
    let data_addr = *store_modules[current_module_idx]
        .data_addrs
        .get(data_idx)
        .unwrap_validated();
    store_data.get_mut(data_addr).data.clear();
    Ok(())
}

#[inline(always)]
fn to_lanes<const M: usize, const N: usize, T: LittleEndianBytes<M>>(data: [u8; 16]) -> [T; N] {
    assert_eq!(M * N, 16);

    let mut lanes = data
        .chunks(M)
        .map(|chunk| T::from_le_bytes(chunk.try_into().unwrap()));
    array::from_fn(|_| lanes.next().unwrap())
}

#[inline(always)]
fn from_lanes<const M: usize, const N: usize, T: LittleEndianBytes<M>>(lanes: [T; N]) -> [u8; 16] {
    assert_eq!(M * N, 16);

    let mut bytes = lanes.into_iter().flat_map(T::to_le_bytes);
    array::from_fn(|_| bytes.next().unwrap())
}
