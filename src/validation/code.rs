use alloc::vec::Vec;
use core::iter;

use crate::core::indices::{FuncIdx, GlobalIdx, LocalIdx};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::memarg::MemArg;
use crate::core::reader::types::{FuncType, NumType, ResultType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::{Error, Result};

pub fn validate_code_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
    globals: &[Global],
) -> Result<Vec<Span>> {
    assert_eq!(section_header.ty, SectionTy::Code);

    let code_block_spans = wasm.read_vec_enumerated(|wasm, idx| {
        let ty_idx = type_idx_of_fn[idx];
        let func_ty = fn_types[ty_idx].clone();

        let func_size = wasm.read_var_u32()?;
        let func_block = wasm.make_span(func_size as usize)?;

        let locals = {
            let params = func_ty.params.valtypes.iter().cloned();
            let declared_locals = read_declared_locals(wasm)?;
            params.chain(declared_locals).collect::<Vec<ValType>>()
        };

        validate_value_stack(func_ty.returns, |value_stack| {
            read_instructions(
                idx,
                wasm,
                value_stack,
                &locals,
                globals,
                fn_types,
                type_idx_of_fn,
            )
        })?;

        Ok(func_block)
    })?;

    trace!(
        "Read code section. Found {} code blocks",
        code_block_spans.len()
    );

    Ok(code_block_spans)
}

pub fn read_declared_locals(wasm: &mut WasmReader) -> Result<Vec<ValType>> {
    let locals = wasm.read_vec(|wasm| {
        let n = wasm.read_var_u32()? as usize;
        let valtype = ValType::read(wasm)?;

        Ok((n, valtype))
    })?;

    // Flatten local types for easier representation where n > 1
    let locals = locals
        .into_iter()
        .flat_map(|entry| iter::repeat(entry.1).take(entry.0))
        .collect::<Vec<ValType>>();

    Ok(locals)
}

fn read_instructions(
    this_function_idx: usize,
    wasm: &mut WasmReader,
    value_stack: &mut Vec<ValType>,
    locals: &[ValType],
    globals: &[Global],
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
) -> Result<()> {
    let assert_pop_value_stack = |value_stack: &mut Vec<ValType>, expected_ty: ValType| {
        value_stack
            .pop()
            .ok_or(Error::InvalidValueStackType(None))
            .and_then(|ty| {
                (ty == expected_ty)
                    .then_some(())
                    .ok_or(Error::InvalidValueStackType(Some(ty)))
            })
    };

    loop {
        let Ok(first_instr_byte) = wasm.read_u8() else {
            return Err(Error::ExprMissingEnd);
        };
        trace!("Read instruction byte {first_instr_byte:#X?} ({first_instr_byte})");

        use crate::core::reader::types::opcode::*;
        match first_instr_byte {
            // nop
            NOP => {}
            // end
            END => {
                return Ok(());
            }
            RETURN => {
                let this_func_ty = &fn_types[type_idx_of_fn[this_function_idx]];

                if value_stack.len() < this_func_ty.returns.valtypes.len() {
                    return Err(Error::EndInvalidValueStack);
                }

                let ret_vals_start = value_stack.len() - this_func_ty.returns.valtypes.len();
                let _remaining_locals = value_stack.drain(..ret_vals_start);

                // TODO(george-cosma): a `return Ok(());` should probably be introduced here, but since we don't have
                // controls flows implemented, the only way to test `return` is to place it at the end of function.
                // However, an `end` is introduced after it, which is invalid. Compilation for this test case should
                // probably fail.
            }
            // call [t1*] -> [t2*]
            CALL => {
                let func_to_call_idx = wasm.read_var_u32()? as FuncIdx;
                let func_ty = &fn_types[type_idx_of_fn[func_to_call_idx]];

                for typ in func_ty.params.valtypes.iter().rev() {
                    assert_pop_value_stack(value_stack, *typ)?;
                }

                for typ in func_ty.returns.valtypes.iter() {
                    value_stack.push(*typ);
                }
            }
            // local.get: [] -> [t]
            LOCAL_GET => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                value_stack.push(*local_ty);
            }
            // local.set [t] -> [0]
            LOCAL_SET => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                let popped = value_stack.pop();
                if popped.as_ref() != Some(local_ty) {
                    return Err(Error::InvalidValueStackType(popped));
                }
            }
            // local.set [t] -> [t]
            LOCAL_TEE => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                let back = value_stack.last();
                if back != Some(local_ty) {
                    return Err(Error::InvalidValueStackType(back.cloned()));
                }
            }
            // global.get [] -> [t]
            GLOBAL_GET => {
                let global_idx = wasm.read_var_u32()? as GlobalIdx;
                let global = globals
                    .get(global_idx)
                    .ok_or(Error::InvalidGlobalIdx(global_idx))?;

                value_stack.push(global.ty.ty);
            }
            // global.set [t] -> []
            GLOBAL_SET => {
                let global_idx = wasm.read_var_u32()? as GlobalIdx;
                let global = globals
                    .get(global_idx)
                    .ok_or(Error::InvalidGlobalIdx(global_idx))?;

                if !global.ty.is_mut {
                    return Err(Error::GlobalIsConst);
                }

                let ty_on_stack = value_stack
                    .pop()
                    .ok_or(Error::InvalidValueStackType(None))?;

                if ty_on_stack != global.ty.ty {
                    return Err(Error::InvalidValueStackType(Some(ty_on_stack)));
                }
            }
            // i32.load [i32] -> [i32]
            I32_LOAD => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // TODO check correct `memarg.align`
                // TODO check if memory[0] exists

                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            // f32.load [f32] -> [f32]
            F32_LOAD => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Check for I32 because that's the address where we find our value
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::F32));
            }
            // f32.load [f32] -> [f32]
            F64_LOAD => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Check for I32 because that's the address where we find our value
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::F64));
            }
            // i32.store [i32] -> [i32]
            I32_STORE => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // TODO check correct `memarg.align`
                // TODO check if memory[0] exists

                // Value to store
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                // Address
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
            }
            // f32.store [f32] -> [f32]
            F32_STORE => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Value to store
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F32))?;
                // Address
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
            }
            F64_STORE => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Value to store
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F64))?;
                // Address
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
            }
            // i32.const: [] -> [i32]
            I32_CONST => {
                let _num = wasm.read_var_i32()?;
                value_stack.push(ValType::NumType(NumType::I32));
            }
            I64_CONST => {
                let _num = wasm.read_var_i64()?;
                value_stack.push(ValType::NumType(NumType::I64));
            }
            F32_CONST => {
                let _num = wasm.read_var_f32()?;
                value_stack.push(ValType::NumType(NumType::F32));
            }
            F64_CONST => {
                let _num = wasm.read_var_f64()?;
                value_stack.push(ValType::NumType(NumType::F64));
            }
            I32_EQZ => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            I32_EQ | I32_NE | I32_LT_S | I32_LT_U | I32_GT_S | I32_GT_U | I32_LE_S | I32_LE_U
            | I32_GE_S | I32_GE_U => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            I32_EQZ => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push_back(ValType::NumType(NumType::I32));
            }
            I32_EQ | I32_NE | I32_LT_S | I32_LT_U | I32_GT_S | I32_GT_U | I32_LE_S | I32_LE_U
            | I32_GE_S | I32_GE_U => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push_back(ValType::NumType(NumType::I32));
            }
            F32_EQ | F32_NE | F32_LT | F32_GT | F32_LE | F32_GE => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            F64_EQ | F64_NE | F64_LT | F64_GT | F64_LE | F64_GE => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F64))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F64))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            F32_ABS | F32_NEG | F32_CEIL | F32_FLOOR | F32_TRUNC | F32_NEAREST | F32_SQRT => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F32))?;

                value_stack.push(ValType::NumType(NumType::F32));
            }
            F32_ADD | F32_SUB | F32_MUL | F32_DIV | F32_MIN | F32_MAX | F32_COPYSIGN => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F32))?;

                value_stack.push(ValType::NumType(NumType::F32));
            }
            F64_ABS | F64_NEG | F64_CEIL | F64_FLOOR | F64_TRUNC | F64_NEAREST | F64_SQRT => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F64))?;

                value_stack.push(ValType::NumType(NumType::F64));
            }
            F64_ADD | F64_SUB | F64_MUL | F64_DIV | F64_MIN | F64_MAX | F64_COPYSIGN => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F64))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::F64))?;

                value_stack.push(ValType::NumType(NumType::F64));
            }
            I32_ADD | I32_SUB | I32_MUL | I32_DIV_S | I32_DIV_U | I32_REM_S => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            // i32.clz: [i32] -> [i32]
            I32_CLZ | I32_CTZ | I32_POPCNT => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            I32_REM_U | I32_AND | I32_OR | I32_XOR | I32_SHL | I32_SHR_S | I32_SHR_U | I32_ROTL
            | I32_ROTR => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            I64_CLZ | I64_CTZ | I64_POPCNT => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I64))?;

                value_stack.push(ValType::NumType(NumType::I64));
            }

            I64_ADD | I64_SUB | I64_MUL | I64_DIV_S | I64_DIV_U | I64_REM_S | I64_REM_U
            | I64_AND | I64_OR | I64_XOR | I64_SHL | I64_SHR_S | I64_SHR_U | I64_ROTL
            | I64_ROTR => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I64))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I64))?;

                value_stack.push(ValType::NumType(NumType::I64));
            }

            F32_CONVERT_I32_S | F32_CONVERT_I32_U | F32_REINTERPRET_I32 => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::F32));
            }

            F32_CONVERT_I64_S | F32_CONVERT_I64_U => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I64))?;

                value_stack.push(ValType::NumType(NumType::F32));
            }
            _ => return Err(Error::InvalidInstr(first_instr_byte)),
        }
    }
}

fn validate_value_stack<F>(return_ty: ResultType, f: F) -> Result<()>
where
    F: FnOnce(&mut Vec<ValType>) -> Result<()>,
{
    let mut value_stack: Vec<ValType> = Vec::new();

    f(&mut value_stack)?;

    // TODO also check here if correct order
    if value_stack != return_ty.valtypes {
        error!(
            "Expected types {:?} on stack, got {:?}",
            return_ty.valtypes, value_stack
        );
        return Err(Error::EndInvalidValueStack);
    }
    Ok(())
}
