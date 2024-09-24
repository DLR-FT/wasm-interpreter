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

use alloc::vec::Vec;

use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{FuncIdx, GlobalIdx, LocalIdx},
        reader::{
            types::{memarg::MemArg, BlockType, FuncType},
            WasmReadable, WasmReader,
        },
        sidetable::{self, Sidetable, SidetableEntry},
    },
    locals::Locals,
    store::Store,
    unreachable_validated, value,
    value_stack::Stack,
    NumType, RuntimeError, ValType, Value,
};

#[cfg(feature = "hooks")]
use crate::execution::hooks::HookSet;

/// Interprets a functions. Parameters and return values are passed on the stack.
pub(super) fn run<H: HookSet>(
    wasm_bytecode: &[u8],
    types: &[FuncType],
    store: &mut Store,
    stack: &mut Stack,
    mut hooks: H,
) -> Result<(), RuntimeError> {
    let func_inst = store
        .funcs
        .get(stack.current_stackframe().func_idx)
        .unwrap_validated();

    let sidetable: &Sidetable = &func_inst.sidetable;
    let mut sidetable_pointer: usize = 0;

    // Start reading the function's instructions
    let mut wasm = WasmReader::new(wasm_bytecode);

    // unwrap is sound, because the validation assures that the function points to valid subslice of the WASM binary
    wasm.move_start_to(func_inst.code_expr).unwrap();

    use crate::core::reader::types::opcode::*;
    loop {
        // call the instruction hook
        #[cfg(feature = "hooks")]
        hooks.instruction_hook(wasm_bytecode, wasm.pc);

        let first_instr_byte = wasm.read_u8().unwrap_validated();

        match first_instr_byte {
            END => {
                let maybe_return_address = stack.pop_stackframe();

                // We finished this entire invocation if there is no stackframe left. If there are
                // one or more stack frames, we need to continue from where the callee was called
                // fromn.
                if stack.callframe_count() == 0 {
                    break;
                }

                trace!("end of function reached, returning to previous stack frame");
                wasm.pc = maybe_return_address;
            }
            RETURN => {
                trace!("returning from function");

                let func_to_call_idx = stack.current_stackframe().func_idx;

                let func_to_call_inst = store.funcs.get(func_to_call_idx).unwrap_validated();
                let func_to_call_ty = types.get(func_to_call_inst.ty).unwrap_validated();

                let ret_vals = stack
                    .pop_tail_iter(func_to_call_ty.returns.valtypes.len())
                    .collect::<Vec<_>>();
                stack.clear_callframe_values();

                for val in ret_vals {
                    stack.push_value(val);
                }

                if stack.callframe_count() == 1 {
                    break;
                }

                trace!("end of function reached, returning to previous stack frame");
                wasm.pc = stack.pop_stackframe();
            }
            CALL => {
                let func_to_call_idx = wasm.read_var_u32().unwrap_validated() as FuncIdx;

                let func_to_call_inst = store.funcs.get(func_to_call_idx).unwrap_validated();
                let func_to_call_ty = types.get(func_to_call_inst.ty).unwrap_validated();

                let params = stack.pop_tail_iter(func_to_call_ty.params.valtypes.len());
                let remaining_locals = func_to_call_inst.locals.iter().cloned();

                trace!("Instruction: call [{func_to_call_idx:?}]");
                let locals = Locals::new(params, remaining_locals);
                stack.push_stackframe(func_to_call_idx, func_to_call_ty, locals, wasm.pc);

                wasm.move_start_to(func_to_call_inst.code_expr)
                    .unwrap_validated();
            }
            BLOCK => {
                let _block_type = BlockType::read_unvalidated(&mut wasm);

                // Nothing else to do. The sidetable is responsible for control flow.
            }
            IF => {
                todo!("execute if instruction, low priority as if can be simulated with br_if and blocks")
            }
            ELSE => {
                do_sidetable_control_transfer(&sidetable, &mut sidetable_pointer, &mut wasm, stack);
            }
            BR => {
                let _target_label = wasm.read_var_u32().unwrap_validated();
                do_sidetable_control_transfer(&sidetable, &mut sidetable_pointer, &mut wasm, stack);
            }
            BR_IF => {
                let _target_label = wasm.read_var_u32().unwrap_validated();

                let condition: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                if condition != 0 {
                    do_sidetable_control_transfer(
                        &sidetable,
                        &mut sidetable_pointer,
                        &mut wasm,
                        stack,
                    );
                } else {
                    sidetable_pointer += 1;
                }
            }
            BR_TABLE => {
                todo!("execute BR_TABLE, Titzer stores multiple entries in the sidetable here, one for each label. See https://arxiv.org/pdf/2205.01183#lstlisting.1");
            }
            LOCAL_GET => {
                stack.get_local(wasm.read_var_u32().unwrap_validated() as LocalIdx);
            }
            LOCAL_SET => stack.set_local(wasm.read_var_u32().unwrap_validated() as LocalIdx),
            LOCAL_TEE => stack.tee_local(wasm.read_var_u32().unwrap_validated() as LocalIdx),
            GLOBAL_GET => {
                let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                let global = store.globals.get(global_idx).unwrap_validated();

                stack.push_value(global.value);
            }
            GLOBAL_SET => {
                let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                let global = store.globals.get_mut(global_idx).unwrap_validated();

                global.value = stack.pop_value(global.global.ty.ty)
            }
            I32_LOAD => {
                let memarg = MemArg::read_unvalidated(&mut wasm);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = store.mems.first().unwrap_validated(); // there is only one memory allowed as of now

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
                        .expect("TODO trap here");

                    let data: [u8; 4] = data.try_into().expect("this to be exactly 4 bytes");
                    u32::from_le_bytes(data)
                };

                stack.push_value(Value::I32(data));
                trace!("Instruction: i32.load [{relative_address}] -> [{data}]");
            }
            F32_LOAD => {
                let memarg = MemArg::read_unvalidated(&mut wasm);
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = store.mems.first().unwrap_validated(); // there is only one memory allowed as of now

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
                        .expect("TODO trap here");
                    f32::from_le_bytes(data)
                };

                stack.push_value(Value::F32(value::F32(data)));
                trace!("Instruction: f32.load [{relative_address}] -> [{data}]");
            }
            I32_STORE => {
                let memarg = MemArg::read_unvalidated(&mut wasm);

                let data_to_store: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = store.mems.get_mut(0).unwrap_validated(); // there is only one memory allowed as of now

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                let address = memarg.offset.checked_add(relative_address);
                let memory_location = address
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 4))
                    })
                    .expect("TODO trap here");

                memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                trace!("Instruction: i32.store [{relative_address} {data_to_store}] -> []");
            }
            F32_STORE => {
                let memarg = MemArg::read_unvalidated(&mut wasm);

                let data_to_store: f32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = store.mems.get_mut(0).unwrap_validated(); // there is only one memory allowed as of now

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                let address = memarg.offset.checked_add(relative_address);
                let memory_location = address
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 4))
                    })
                    .expect("TODO trap here");

                memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                trace!("Instruction: f32.store [{relative_address} {data_to_store}] -> []");
            }
            F64_STORE => {
                let memarg = MemArg::read_unvalidated(&mut wasm);

                let data_to_store: f64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let relative_address: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                let mem = store.mems.get_mut(0).unwrap_validated(); // there is only one memory allowed as of now

                // The spec states that this should be a 33 bit integer
                // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                let address = memarg.offset.checked_add(relative_address);
                let memory_location = address
                    .and_then(|address| {
                        let address = address as usize;
                        mem.data.get_mut(address..(address + 4))
                    })
                    .expect("TODO trap here");

                memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                trace!("Instruction: f64.store [{relative_address} {data_to_store}] -> []");
            }
            I32_CONST => {
                let constant = wasm.read_var_i32().unwrap_validated();
                trace!("Instruction: i32.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            F32_CONST => {
                let constant = f32::from_bits(wasm.read_var_f32().unwrap_validated());
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
                let constant = wasm.read_var_i64().unwrap_validated();
                trace!("Instruction: i64.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            F64_CONST => {
                let constant = f64::from_bits(wasm.read_var_f64().unwrap_validated());
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
                let res: value::F32 = v1.round();

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
                let res: value::F64 = v1.round();

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
            other => {
                trace!("Unknown instruction {other:#x}, skipping..");
            }
        }
    }
    Ok(())
}

fn do_sidetable_control_transfer(
    sidetable: &Sidetable,
    sidetable_pointer: &mut usize,
    wasm: &mut WasmReader,
    stack: &mut Stack,
) {
    let entry = *sidetable
        .get(*sidetable_pointer)
        .expect("sidetable entry to exist");

    wasm.skip(
        entry
            .delta_pc
            .try_into()
            .expect("delta_pc to not be negative for branches"),
    )
    .unwrap_validated();

    *sidetable_pointer +=
        usize::try_from(entry.delta_stp).expect("delta_stp to be negative for branches");
    usize::try_from(entry.delta_stp).expect("delta_stp to be negative for branches");

    stack.unwind(entry.val_count, entry.pop_count);
}
