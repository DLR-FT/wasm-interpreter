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
        reader::{types::memarg::MemArg, WasmReadable, WasmReader},
    },
    execution::{pop_callframe, push_callframe},
    hooks::HookSet,
    NumType, RunResult, Runner, RuntimeError, ValType, Value,
};

use super::RuntimeInstance;

impl<'a, 'bytecode, H> Runner<'a, 'bytecode, H>
where
    H: HookSet,
{
    /// Interprets a functions. Parameters and return values are passed on the stack.
    pub(super) fn run(&mut self) -> Result<RunResult, RuntimeError> {
        let Self {
            wasm_reader,
            hook_set,
            ref mut state,
            fuel,
            runtime_instance,
            value_stack,
            stack,
            ..
        } = self;

        let call_stack = match state {
            crate::RunnerState::Uninitialized => {
                error!("Runner::run() was called on an uninitialized runner");
                return Err(RuntimeError::UnexpectedRunnerState);
            }
            crate::RunnerState::ReadyToRun { func_idx } => {
                // TODO run push_callstack here?

                let func = runtime_instance
                    .store
                    .funcs
                    .get(*func_idx)
                    .unwrap_validated();

                // Pop parameters from stack
                let func_type = runtime_instance.types.get(func.ty).unwrap_validated();
                let mut params: Vec<Value> = func_type
                    .params
                    .valtypes
                    .iter()
                    .map(|ty| stack.pop_value(*ty))
                    .collect();
                params.reverse();

                // Start reading the function's instructions
                let mut wasm = WasmReader::new(runtime_instance.wasm_bytecode);

                // unwrap is sound, because the validation assures that the function points to valid subslice of the WASM binary
                wasm.move_start_to(func.code_expr).unwrap();
                todo!()
            }
            crate::RunnerState::InProgress {
                call_stack,
                func_idx,
            } => call_stack, // TODO remove
            crate::RunnerState::ReturnValueAvailable { func_idx } => return Ok(RunResult::Done),
        };

        use crate::core::reader::types::opcode::*;
        loop {
            // TODO
            // call the instruction hook
            // #[cfg(feature = "hooks")]
            // H::instruction_hook(self);

            // I had to move these inside  the loop since the hook needs to borrow self as mutable, and these lines also borrow self as mutable
            let call_frame_ref = call_stack.last_mut().expect("todo why we can expect this");
            let locals = &mut call_frame_ref.locals;
            let wasm = &mut call_frame_ref.reader;

            let first_instr_byte = wasm.read_u8().unwrap_validated();

            match first_instr_byte {
                END => {
                    // TODO: check if this was the outermost block of the function, if so, pop its CallFrame and continue
                    // TODO: otherwise, consult the sidetable
                    pop_callframe(call_stack);
                    if call_stack.is_empty() {
                        return Ok(RunResult::Done);
                    }
                }
                CALL => {
                    let func_idx = wasm.read_var_u32().unwrap_validated() as FuncIdx;
                    push_callframe(call_stack, runtime_instance, func_idx, stack)?;
                }
                RETURN => {
                    pop_callframe(call_stack);
                    if call_stack.is_empty() {
                        return Ok(RunResult::Done);
                    }
                }
                LOCAL_GET => {
                    let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                    let local = locals.get(local_idx);
                    trace!("Instruction: local.get [] -> [{local:?}]");
                    stack.push_value(local.clone());
                }
                LOCAL_SET => {
                    let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                    let local = locals.get_mut(local_idx);
                    let value = stack.pop_value(local.to_ty());
                    trace!("Instruction: local.set [{local:?}] -> []");
                    *local = value;
                }
                GLOBAL_GET => {
                    let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                    let global = runtime_instance
                        .store
                        .globals
                        .get(global_idx)
                        .unwrap_validated();

                    stack.push_value(global.value.clone());
                }
                GLOBAL_SET => {
                    let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                    let global = runtime_instance
                        .store
                        .globals
                        .get_mut(global_idx)
                        .unwrap_validated();

                    global.value = stack.pop_value(global.global.ty.ty)
                }
                I32_LOAD => {
                    let memarg = MemArg::read_unvalidated(wasm);
                    let relative_address: u32 =
                        stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let mem = runtime_instance.store.mems.first().unwrap_validated(); // there is only one memory allowed as of now

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
                I32_STORE => {
                    let memarg = MemArg::read_unvalidated(wasm);

                    let data_to_store: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let relative_address: u32 =
                        stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let mem = runtime_instance.store.mems.get_mut(0).unwrap_validated(); // there is only one memory allowed as of now

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
                I32_CONST => {
                    let constant = wasm.read_var_i32().unwrap_validated();
                    trace!("Instruction: i32.const [] -> [{constant}]");
                    stack.push_value(constant.into());
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
                I32_ADD => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1.wrapping_add(v2);

                    trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
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
                other => {
                    trace!("Unknown instruction {other:#x}, skipping..");
                }
            }
        }
    }
}
