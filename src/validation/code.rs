use alloc::vec::Vec;
use core::iter;

use crate::core::indices::{DataIdx, FuncIdx, GlobalIdx, LocalIdx, MemIdx};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::memarg::MemArg;
use crate::core::reader::types::{FuncType, MemType, NumType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::validation_stack::ValidationStack;
use crate::{Error, Result};

pub fn validate_code_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
    globals: &[Global],
    memories: &[MemType],
    data_count: &Option<u32>,
) -> Result<Vec<Span>> {
    assert_eq!(section_header.ty, SectionTy::Code);

    let code_block_spans = wasm.read_vec_enumerated(|wasm, idx| {
        let ty_idx = type_idx_of_fn[idx];
        let func_ty = fn_types[ty_idx].clone();

        let func_size = wasm.read_var_u32()?;
        let func_block = wasm.make_span(func_size as usize)?;
        let previous_pc = wasm.pc;

        let locals = {
            let params = func_ty.params.valtypes.iter().cloned();
            let declared_locals = read_declared_locals(wasm)?;
            params.chain(declared_locals).collect::<Vec<ValType>>()
        };

        let mut stack = ValidationStack::new();

        read_instructions(
            idx,
            wasm,
            &mut stack,
            &locals,
            globals,
            fn_types,
            type_idx_of_fn,
            memories,
            data_count,
        )?;

        // Check if there were unread trailing instructions after the last END
        if previous_pc + func_size as usize != wasm.pc {
            todo!(
                "throw error because there are trailing unreachable instructions in the code block"
            )
        }

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

#[allow(clippy::too_many_arguments)]
fn read_instructions(
    this_function_idx: usize,
    wasm: &mut WasmReader,
    stack: &mut ValidationStack,
    locals: &[ValType],
    globals: &[Global],
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
    memories: &[MemType],
    data_count: &Option<u32>,
) -> Result<()> {
    // TODO we must terminate only if both we saw the final `end` and when we consumed all of the code span
    loop {
        let Ok(first_instr_byte) = wasm.read_u8() else {
            // TODO only do this if EOF
            return Err(Error::ExprMissingEnd);
        };
        trace!("Read instruction byte {first_instr_byte:#04X?} ({first_instr_byte}) at wasm_binary[{}]", wasm.pc);

        use crate::core::reader::types::opcode::*;
        match first_instr_byte {
            // nop
            NOP => {}
            // end
            END => {
                // TODO check if there are labels on the stack.
                // If there are none (i.e. this is the implicit end of the function and not a jump to the end of a function), the stack must only contain the valid return values, no other junk.
                //
                // Else, anything may remain on the stack, as long as the top of the stack matche the current blocks return value.

                if stack.has_remaining_label() {
                    // This is the END of a block.

                    // We check the valtypes on top of the stack

                    // TODO remove the ugly hack for the todo!(..)!
                    #[allow(clippy::diverging_sub_expression)]
                    {
                        let _block_return_ty: &[ValType] =
                            todo!("get return types for current block");
                    }
                    // stack.assert_val_types_on_top(block_return_ty)?;

                    // Clear the stack until the next label
                    // stack.clear_until_next_label();

                    // And push the blocks return types onto the stack again
                    // for valtype in block_return_ty {
                    // stack.push_valtype(*valtype);
                    // }
                } else {
                    // This is the last end of a function

                    // The stack must only contain the function's return valtypes
                    let this_func_ty = &fn_types[type_idx_of_fn[this_function_idx]];
                    stack.assert_val_types(&this_func_ty.returns.valtypes)?;
                    return Ok(());
                }
            }
            RETURN => {
                let this_func_ty = &fn_types[type_idx_of_fn[this_function_idx]];

                stack
                    .assert_val_types_on_top(&this_func_ty.returns.valtypes)
                    .map_err(|_| Error::EndInvalidValueStack)?;

                stack.make_unspecified();

                // TODO(george-cosma): a `return Ok(());` should probably be introduced here, but since we don't have
                // controls flows implemented, the only way to test `return` is to place it at the end of function.
                // However, an `end` is introduced after it, which is invalid. Compilation for this test case should
                // probably fail.

                // TODO(wucke13) I believe we must not drain the validation stack here; only if we
                // know this return is actually taken during execution we may drain the stack. This
                // could however be a conditional return (return in an `if`), and the other side
                // past the `else` might need the values on the `ValidationStack` that do belong
                // to the current function (but not the current block), so draining would make
                // continued validation of the current function impossible. We should most
                // definitely not `return Ok(())` here, because there might be still more of the
                // current function to validate.
            }
            // call [t1*] -> [t2*]
            CALL => {
                let func_to_call_idx = wasm.read_var_u32()? as FuncIdx;
                let func_ty = &fn_types[type_idx_of_fn[func_to_call_idx]];

                for typ in func_ty.params.valtypes.iter().rev() {
                    stack.assert_pop_val_type(*typ)?;
                }

                for typ in func_ty.returns.valtypes.iter() {
                    stack.push_valtype(*typ);
                }
            }
            // unreachable: [t1*] -> [t2*]
            UNREACHABLE => {
                stack.make_unspecified();
            }
            DROP => {
                stack.drop_val()?;
            }
            // local.get: [] -> [t]
            LOCAL_GET => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                stack.push_valtype(*local_ty);
            }
            // local.set [t] -> []
            LOCAL_SET => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                stack.assert_pop_val_type(*local_ty)?;
            }
            // local.set [t] -> [t]
            LOCAL_TEE => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                stack.assert_pop_val_type(*local_ty)?;
            }
            // global.get [] -> [t]
            GLOBAL_GET => {
                let global_idx = wasm.read_var_u32()? as GlobalIdx;
                let global = globals
                    .get(global_idx)
                    .ok_or(Error::InvalidGlobalIdx(global_idx))?;

                stack.push_valtype(global.ty.ty);
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

                stack.assert_pop_val_type(global.ty.ty)?;
            }
            I32_LOAD => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 4 {
                    return Err(Error::ErroneousAlignment(memarg.align, 4));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_LOAD => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 8 {
                    return Err(Error::ErroneousAlignment(memarg.align, 8));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            F32_LOAD => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 4 {
                    return Err(Error::ErroneousAlignment(memarg.align, 4));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::F32));
            }
            F64_LOAD => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 8 {
                    return Err(Error::ErroneousAlignment(memarg.align, 8));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::F64));
            }
            I32_LOAD8_S => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 1 {
                    return Err(Error::ErroneousAlignment(memarg.align, 1));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I32_LOAD8_U => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 1 {
                    return Err(Error::ErroneousAlignment(memarg.align, 1));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I32_LOAD16_S => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 2 {
                    return Err(Error::ErroneousAlignment(memarg.align, 2));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I32_LOAD16_U => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 2 {
                    return Err(Error::ErroneousAlignment(memarg.align, 2));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_LOAD8_S => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 1 {
                    return Err(Error::ErroneousAlignment(memarg.align, 1));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I64_LOAD8_U => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 1 {
                    return Err(Error::ErroneousAlignment(memarg.align, 1));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I64_LOAD16_S => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 2 {
                    return Err(Error::ErroneousAlignment(memarg.align, 2));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I64_LOAD16_U => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 2 {
                    return Err(Error::ErroneousAlignment(memarg.align, 2));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I64_LOAD32_S => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 4 {
                    return Err(Error::ErroneousAlignment(memarg.align, 4));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I64_LOAD32_U => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 4 {
                    return Err(Error::ErroneousAlignment(memarg.align, 4));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I32_STORE => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align >= 4 {
                    return Err(Error::ErroneousAlignment(memarg.align, 4));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            I64_STORE => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 8 {
                    return Err(Error::ErroneousAlignment(memarg.align, 8));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            F32_STORE => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align >= 4 {
                    return Err(Error::ErroneousAlignment(memarg.align, 4));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            F64_STORE => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 8 {
                    return Err(Error::ErroneousAlignment(memarg.align, 8));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            I32_STORE8 => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 1 {
                    return Err(Error::ErroneousAlignment(memarg.align, 1));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            I32_STORE16 => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 2 {
                    return Err(Error::ErroneousAlignment(memarg.align, 2));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            I64_STORE8 => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 1 {
                    return Err(Error::ErroneousAlignment(memarg.align, 1));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            I64_STORE16 => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 2 {
                    return Err(Error::ErroneousAlignment(memarg.align, 2));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            I64_STORE32 => {
                if memories.is_empty() {
                    return Err(Error::MemoryIsNotDefined(0));
                }
                let memarg = MemArg::read(wasm)?;
                if memarg.align > 4 {
                    return Err(Error::ErroneousAlignment(memarg.align, 4));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            MEMORY_SIZE => {
                let mem_idx = wasm.read_u8()? as MemIdx;
                if memories.len() <= mem_idx {
                    return Err(Error::MemoryIsNotDefined(mem_idx));
                }
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            MEMORY_GROW => {
                let mem_idx = wasm.read_u8()? as MemIdx;
                if memories.len() <= mem_idx {
                    return Err(Error::MemoryIsNotDefined(mem_idx));
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            // i32.const: [] -> [i32]
            I32_CONST => {
                let _num = wasm.read_var_i32()?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_CONST => {
                let _num = wasm.read_var_i64()?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            F32_CONST => {
                let _num = wasm.read_var_f32()?;
                stack.push_valtype(ValType::NumType(NumType::F32));
            }
            F64_CONST => {
                let _num = wasm.read_var_f64()?;
                stack.push_valtype(ValType::NumType(NumType::F64));
            }
            I32_EQZ => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I32_EQ | I32_NE | I32_LT_S | I32_LT_U | I32_GT_S | I32_GT_U | I32_LE_S | I32_LE_U
            | I32_GE_S | I32_GE_U => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_EQZ => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_EQ | I64_NE | I64_LT_S | I64_LT_U | I64_GT_S | I64_GT_U | I64_LE_S | I64_LE_U
            | I64_GE_S | I64_GE_U => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            F32_EQ | F32_NE | F32_LT | F32_GT | F32_LE | F32_GE => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            F64_EQ | F64_NE | F64_LT | F64_GT | F64_LE | F64_GE => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            F32_ABS | F32_NEG | F32_CEIL | F32_FLOOR | F32_TRUNC | F32_NEAREST | F32_SQRT => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;

                stack.push_valtype(ValType::NumType(NumType::F32));
            }
            F32_ADD | F32_SUB | F32_MUL | F32_DIV | F32_MIN | F32_MAX | F32_COPYSIGN => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;

                stack.push_valtype(ValType::NumType(NumType::F32));
            }
            F64_ABS | F64_NEG | F64_CEIL | F64_FLOOR | F64_TRUNC | F64_NEAREST | F64_SQRT => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;

                stack.push_valtype(ValType::NumType(NumType::F64));
            }
            F64_ADD | F64_SUB | F64_MUL | F64_DIV | F64_MIN | F64_MAX | F64_COPYSIGN => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;

                stack.push_valtype(ValType::NumType(NumType::F64));
            }
            I32_ADD | I32_SUB | I32_MUL | I32_DIV_S | I32_DIV_U | I32_REM_S => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            // i32.clz: [i32] -> [i32]
            I32_CLZ | I32_CTZ | I32_POPCNT => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I32_REM_U | I32_AND | I32_OR | I32_XOR | I32_SHL | I32_SHR_S | I32_SHR_U | I32_ROTL
            | I32_ROTR => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_CLZ | I64_CTZ | I64_POPCNT => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }

            I64_ADD | I64_SUB | I64_MUL | I64_DIV_S | I64_DIV_U | I64_REM_S | I64_REM_U
            | I64_AND | I64_OR | I64_XOR | I64_SHL | I64_SHR_S | I64_SHR_U | I64_ROTL
            | I64_ROTR => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }

            I32_WRAP_I64 => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }

            I32_TRUNC_F32_S | I32_TRUNC_F32_U | I32_REINTERPRET_F32 => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }

            I32_TRUNC_F64_S | I32_TRUNC_F64_U => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }

            I64_EXTEND_I32_S | I64_EXTEND_I32_U => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }

            I64_TRUNC_F32_S | I64_TRUNC_F32_U => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }

            I64_TRUNC_F64_S | I64_TRUNC_F64_U | I64_REINTERPRET_F64 => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }

            F32_CONVERT_I32_S | F32_CONVERT_I32_U | F32_REINTERPRET_I32 => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::F32));
            }

            F32_CONVERT_I64_S | F32_CONVERT_I64_U => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::F32));
            }

            F32_DEMOTE_F64 => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;

                stack.push_valtype(ValType::NumType(NumType::F32));
            }

            F64_CONVERT_I32_S | F64_CONVERT_I32_U => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::F64));
            }

            F64_CONVERT_I64_S | F64_CONVERT_I64_U | F64_REINTERPRET_I64 => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::F64));
            }

            F64_PROMOTE_F32 => {
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;

                stack.push_valtype(ValType::NumType(NumType::F64));
            }
            

            FC_EXTENSIONS => {
                let Ok(second_instr_byte) = wasm.read_u8() else {
                    // TODO only do this if EOF
                    return Err(Error::ExprMissingEnd);
                };
                trace!("Read instruction byte {second_instr_byte:#04X?} ({second_instr_byte}) at wasm_binary[{}]", wasm.pc);

                use crate::core::reader::types::opcode::fc_extensions::*;
                match second_instr_byte {
                    I32_TRUNC_SAT_F32_S => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                        stack.push_valtype(ValType::NumType(NumType::I32));
                    }
                    I32_TRUNC_SAT_F32_U => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                        stack.push_valtype(ValType::NumType(NumType::I32));
                    }
                    I32_TRUNC_SAT_F64_S => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                        stack.push_valtype(ValType::NumType(NumType::I32));
                    }
                    I32_TRUNC_SAT_F64_U => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                        stack.push_valtype(ValType::NumType(NumType::I32));
                    }
                    I64_TRUNC_SAT_F32_S => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                        stack.push_valtype(ValType::NumType(NumType::I64));
                    }
                    I64_TRUNC_SAT_F32_U => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                        stack.push_valtype(ValType::NumType(NumType::I64));
                    }
                    I64_TRUNC_SAT_F64_S => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                        stack.push_valtype(ValType::NumType(NumType::I64));
                    }
                    I64_TRUNC_SAT_F64_U => {
                        stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                        stack.push_valtype(ValType::NumType(NumType::I64));
                    }
                    MEMORY_INIT => {
                        let data_idx = wasm.read_var_u32()? as DataIdx;
                        let mem_idx = wasm.read_u8()? as MemIdx;
                        if memories.len() <= mem_idx {
                            return Err(Error::MemoryIsNotDefined(mem_idx));
                        }
                        if data_count.is_none() {
                            return Err(Error::NoDataSegments);
                        }
                        if data_count.unwrap() as usize <= data_idx {
                            return Err(Error::DataSegmentNotFound(data_idx));
                        }
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                    }
                    DATA_DROP => {
                        if data_count.is_none() {
                            return Err(Error::NoDataSegments);
                        }
                        let data_idx = wasm.read_var_u32()? as DataIdx;
                        if data_count.unwrap() as usize <= data_idx {
                            return Err(Error::DataSegmentNotFound(data_idx));
                        }
                    }
                    MEMORY_COPY => {
                        let (dst, src) = (wasm.read_u8()? as usize, wasm.read_u8()? as usize);
                        assert!(dst == 0 && src == 0);
                        if memories.is_empty() {
                            return Err(Error::MemoryIsNotDefined(0));
                        }
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                    }
                    MEMORY_FILL => {
                        let mem_idx = wasm.read_u8()? as MemIdx;
                        if memories.len() <= mem_idx {
                            return Err(Error::MemoryIsNotDefined(mem_idx));
                        }
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                    }
                    _ => {
                        return Err(Error::InvalidMultiByteInstr(
                            first_instr_byte,
                            second_instr_byte,
                        ))
                    }
                }
            }
            _ => return Err(Error::InvalidInstr(first_instr_byte)),
        }
    }
}
