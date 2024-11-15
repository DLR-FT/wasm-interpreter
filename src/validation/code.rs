use alloc::vec::Vec;
use core::iter;

use crate::core::indices::{FuncIdx, GlobalIdx, LabelIdx, LocalIdx};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::memarg::MemArg;
use crate::core::reader::types::{BlockType, FuncType, NumType, ResultType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::sidetable::{self, IncompleteSidetableEntry, Sidetable, SidetableBuilder};
use crate::validation_stack::{LabelInfo, ValidationStack};
use crate::{Error, Result};

pub fn validate_code_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
    globals: &[Global],
) -> Result<Vec<(Span, Sidetable)>> {
    assert_eq!(section_header.ty, SectionTy::Code);

    let code_block_spans = wasm.read_vec_enumerated(|wasm, idx| {
        let ty_idx = type_idx_of_fn[idx];
        let func_ty = fn_types[ty_idx].clone();

        debug!("{:x?}", wasm.full_wasm_binary);

        let func_size = wasm.read_var_u32()?;
        let func_block = wasm.make_span(func_size as usize)?;
        let previous_pc = wasm.pc;

        let locals = {
            let params = func_ty.params.valtypes.iter().cloned();
            let declared_locals = read_declared_locals(wasm)?;
            params.chain(declared_locals).collect::<Vec<ValType>>()
        };

        let mut stack = ValidationStack::new();

        let mut sidetable_builder = SidetableBuilder::new();

        read_instructions(
            idx,
            wasm,
            &mut stack,
            &locals,
            globals,
            fn_types,
            type_idx_of_fn,
            &mut sidetable_builder,
        )?;

        let sidetable = sidetable_builder.into_sidetable();

        debug!("{:?}", &sidetable);

        // Check if there were unread trailing instructions after the last END
        if previous_pc + func_size as usize != wasm.pc {
            todo!(
                "throw error because there are trailing unreachable instructions in the code block"
            )
        }

        Ok((func_block, sidetable))
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
    stack: &mut ValidationStack,
    locals: &[ValType],
    globals: &[Global],
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
    sidetable_builder: &mut SidetableBuilder,
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
            // unreachable: [t1*] -> [t2*]
            UNREACHABLE => {
                stack.make_unspecified();
            }
            // nop: [] -> []
            NOP => {}
            // block: [] -> [t*2]
            BLOCK => {
                let func_type: FuncType = BlockType::read(wasm)?.as_func_type(&fn_types)?;

                stack.push_label(LabelInfo::Block {
                    func_type,
                    sidetable_branch_indices: Vec::new(),
                    num_values_on_stack_before: stack.len_without_labels(),
                });
            }
            BR => {
                let label_idx = wasm.read_var_u32()? as LabelIdx;

                let label_info: &LabelInfo = stack
                    .find_nth_label_from_top(label_idx)
                    .ok_or(Error::InvalidLabelIdx)?;

                match label_info {
                    LabelInfo::Block {
                        func_type,
                        num_values_on_stack_before,
                        ..
                    } => {
                        let block_return_types = &func_type.returns.valtypes;
                        // todo!("do branches require the top of the stack or the entire stack to be correct?");
                        stack.assert_val_types_on_top(block_return_types)?;

                        let Some(stack_len) = stack.len_without_labels() else {
                            // The stack contains an `Unspecified` element, so this instruction is practically unreachable.
                            // It does not matter if the Unspecified was before this Block/If/Loop started or after.
                            //
                            // We don't need to generate a sidetable entry and the validation of the top-most stack values has already be done, so we can just continue with the next instruction.
                            continue;
                        };

                        let num_values_on_stack_before = num_values_on_stack_before.expect("this to always be Some. We already checked through `stack.len_without_label` whether the stack contains Unspecified. If so, we ignored this instruction. Because we are in a block right now, the stack values below the block label cannot have changed, so any Unspecified entries below the block label are impossible.");

                        let val_count = block_return_types.len();
                        let pop_count = stack_len - num_values_on_stack_before - val_count;

                        error!("val: {}, pop: {}", val_count, pop_count);

                        // FIXME Now we actually need to modifiy the label info, so we have to borrow it again
                        let Some(LabelInfo::Block {
                            sidetable_branch_indices,
                            ..
                        }) = stack.find_nth_label_from_top_mut(label_idx)
                        else {
                            unreachable!("We just found this block")
                        };

                        sidetable_builder.0.push(IncompleteSidetableEntry {
                            ip: wasm.pc, // TODO maybe - 1?
                            delta_ip: None,
                            delta_stp: None,
                            val_count,
                            pop_count,
                        });

                        // Store index of new sidetable entry so it can be completed, when the end of this block is reached.
                        sidetable_branch_indices.push(sidetable_builder.0.len() - 1);
                    }
                    _ => todo!("handle branches for loops and ifs/elses"),
                }
            }
            BR_IF => {
                let label_idx = wasm.read_var_u32()? as LabelIdx;

                // condition for if
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                trace!("read br_if condition");

                let label_info: &LabelInfo = stack
                    .find_nth_label_from_top(label_idx)
                    .ok_or(Error::InvalidLabelIdx)?;

                match label_info {
                    LabelInfo::Block {
                        func_type,
                        num_values_on_stack_before,
                        ..
                    } => {
                        let block_return_types = &func_type.returns.valtypes;
                        // todo!("do branches require the top of the stack or the entire stack to be correct?");
                        stack.assert_val_types_on_top(block_return_types)?;

                        let Some(stack_len) = stack.len_without_labels() else {
                            // The stack contains an `Unspecified` element, so this instruction is practically unreachable.
                            // It does not matter if the Unspecified was before this Block/If/Loop started or after.
                            //
                            // We don't need to generate a sidetable entry and the validation of the top-most stack values has already be done, so we can just continue with the next instruction.
                            continue;
                        };

                        let num_values_on_stack_before = num_values_on_stack_before.expect("this to always be Some. We already checked through `stack.len_without_label` whether the stack contains Unspecified. If so, we ignored this instruction. Because we are in a block right now, the stack values below the block label cannot have changed, so any Unspecified entries below the block label are impossible.");

                        let val_count = block_return_types.len();
                        let pop_count = stack_len - num_values_on_stack_before - val_count;

                        error!("val: {}, pop: {}", val_count, pop_count);

                        // FIXME Now we actually need to modifiy the label info, so we have to borrow it again
                        let Some(LabelInfo::Block {
                            sidetable_branch_indices,
                            ..
                        }) = stack.find_nth_label_from_top_mut(label_idx)
                        else {
                            unreachable!("We just found this block")
                        };

                        sidetable_builder.0.push(IncompleteSidetableEntry {
                            ip: wasm.pc, // TODO maybe - 1?
                            delta_ip: None,
                            delta_stp: None,
                            val_count,
                            pop_count,
                        });

                        // Store index of new sidetable entry so it can be completed, when the end of this block is reached.
                        sidetable_branch_indices.push(sidetable_builder.0.len() - 1);
                    }
                    _ => todo!("handle branches for loops and ifs/elses"),
                }
            }
            LOOP | IF => {
                let _block_type: FuncType = BlockType::read(wasm)?.as_func_type(&fn_types)?;

                todo!("handle loop and if instructions")
                // todo!(
                // "{}, {}",
                // "add incomplete entry to sidetable",
                // "verify from now on that only the top-most block_ty.params are accessed"
                // );
            }
            // end
            END => {
                // This is the END of a block, loop or if
                match stack.find_nth_label_from_top(0) {
                    Some(LabelInfo::Block { func_type, .. }) => {
                        // Before we can actually pop the label and valtypes from the stack, we need to validate the valtypes on top of the stack
                        stack.assert_val_types_on_top(&func_type.returns.valtypes)?;

                        // FIXME It's not very pretty to shadow `func_type` with a newer variable that should be exactly the same
                        // Clear the stack until the next label
                        let LabelInfo::Block {
                            func_type,
                            sidetable_branch_indices,
                            num_values_on_stack_before,
                        } = stack
                            .pop_label_and_above()
                            .expect("this to find find the topmost label, which we just found")
                        else {
                            unreachable!(
                                "We just checked that the topmost label has to be a Block"
                            );
                        };

                        // And push the blocks return types onto the stack again
                        for valtype in &func_type.returns.valtypes {
                            stack.push_valtype(valtype.clone());
                        }

                        let sidetable_len = sidetable_builder.0.len();

                        for idx in sidetable_branch_indices {
                            let incomplete_entry = sidetable_builder
                                .0
                                .get_mut(idx)
                                .expect("index into sidetable to always be valid");

                            incomplete_entry.delta_ip =
                                Some(wasm.pc as isize - incomplete_entry.ip as isize);
                            incomplete_entry.delta_stp =
                                Some(sidetable_len as isize - idx as isize);
                        }
                    }
                    Some(_) => todo!("handle end for loops and ifs/elses"),
                    None => {
                        // This is the last end of a function

                        // The stack must only contain the function's return valtypes
                        let this_func_ty = &fn_types[type_idx_of_fn[this_function_idx]];
                        stack.assert_val_types(&this_func_ty.returns.valtypes)?;
                        return Ok(());
                    }
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
                // if let Some(popped_control_entry) = control_stack.pop() {
                //     todo!("Complete sidetable entries that jumped to this entry's label");
                // } else {
                //     return Ok(());
                // }
                // todo!("check if there is a pending sidetable entry to be completed")
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
            // local.tee [t] -> [t]
            LOCAL_TEE => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                stack.assert_pop_val_type(*local_ty)?;
                stack.push_valtype(*local_ty);
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
            DROP => {
                stack.pop_valtype()?;
            }
            // i32.load [i32] -> [i32]
            I32_LOAD => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // TODO check correct `memarg.align`
                // TODO check if memory[0] exists

                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            // f32.load [f32] -> [f32]
            F32_LOAD => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Check for I32 because that's the address where we find our value
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::F32));
            }
            // f32.load [f32] -> [f32]
            F64_LOAD => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Check for I32 because that's the address where we find our value
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::F64));
            }
            // i32.store [i32] -> [i32]
            I32_STORE => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // TODO check correct `memarg.align`
                // TODO check if memory[0] exists

                // Value to store
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                // Address
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            // f32.store [f32] -> [f32]
            F32_STORE => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Value to store
                stack.assert_pop_val_type(ValType::NumType(NumType::F32))?;
                // Address
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
            }
            F64_STORE => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // Value to store
                stack.assert_pop_val_type(ValType::NumType(NumType::F64))?;
                // Address
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
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
            _ => return Err(Error::InvalidInstr(first_instr_byte)),
        }
    }
}
