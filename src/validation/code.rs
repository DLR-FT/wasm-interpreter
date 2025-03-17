use alloc::collections::btree_set::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use core::iter;

use crate::core::indices::{
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, MemIdx, TableIdx, TypeIdx,
};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::element::ElemType;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::memarg::MemArg;
use crate::core::reader::types::{BlockType, FuncType, MemType, NumType, TableType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::sidetable::{Sidetable, SidetableEntry};
use crate::validation_stack::{LabelInfo, ValidationStack};
use crate::{Error, RefType, Result};

#[allow(clippy::too_many_arguments)]
pub fn validate_code_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
    num_imported_funcs: usize,
    globals: &[Global],
    memories: &[MemType],
    data_count: &Option<u32>,
    tables: &[TableType],
    elements: &[ElemType],
    referenced_functions: &BTreeSet<u32>,
) -> Result<Vec<(Span, Sidetable)>> {
    assert_eq!(section_header.ty, SectionTy::Code);

    // TODO replace with single sidetable per module
    let code_block_spans_sidetables = wasm.read_vec_enumerated(|wasm, idx| {
        // We need to offset the index by the number of functions that were
        // imported. Imported functions always live at the start of the index
        // space.
        let ty_idx = type_idx_of_fn[idx + num_imported_funcs];
        let func_ty = fn_types[ty_idx].clone();

        let func_size = wasm.read_var_u32()?;
        let func_block = wasm.make_span(func_size as usize)?;
        let previous_pc = wasm.pc;

        let locals = {
            let params = func_ty.params.valtypes.iter().cloned();
            let declared_locals = read_declared_locals(wasm)?;
            params.chain(declared_locals).collect::<Vec<ValType>>()
        };

        let mut stack = ValidationStack::new_for_func(func_ty);
        let mut sidetable: Sidetable = Sidetable::default();

        read_instructions(
            wasm,
            &mut stack,
            &mut sidetable,
            &locals,
            globals,
            fn_types,
            type_idx_of_fn,
            memories,
            data_count,
            tables,
            elements,
            referenced_functions,
        )?;

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
        code_block_spans_sidetables.len()
    );

    Ok(code_block_spans_sidetables)
}

pub fn read_declared_locals(wasm: &mut WasmReader) -> Result<Vec<ValType>> {
    let locals = wasm.read_vec(|wasm| {
        let n = wasm.read_var_u32()? as usize;
        let valtype = ValType::read(wasm)?;

        Ok((n, valtype))
    })?;

    // these checks are related to the official test suite binary.wast file, the first 2 assert_malformed's starting at line 350
    // we check to not have more than 2^32-1 locals, and if that number is okay, we then get to instantiate them all
    // this is because the flat_map and collect take an insane amount of time
    // in total, these 2 tests take more than 240s
    let mut total_no_of_locals: usize = 0;
    for local in &locals {
        let temp = local.0;
        if temp > i32::MAX as usize {
            return Err(Error::TooManyLocals(total_no_of_locals));
        };
        total_no_of_locals = match total_no_of_locals.checked_add(temp) {
            None => return Err(Error::TooManyLocals(total_no_of_locals)),
            Some(n) => n,
        }
    }

    if total_no_of_locals > i32::MAX as usize {
        return Err(Error::TooManyLocals(total_no_of_locals));
    }

    // Flatten local types for easier representation where n > 1
    let locals = locals
        .into_iter()
        .flat_map(|entry| iter::repeat(entry.1).take(entry.0))
        .collect::<Vec<ValType>>();

    Ok(locals)
}

//helper function to avoid code duplication in jump validations
//the entries, except for the loop label, need to be correctly backpatched later
//the temporary values of fields (delta_pc, delta_stp) of the entries are the (ip, stp) of the relevant label
//the label is also updated with the additional information of the index of this sidetable
//entry itself so that the entry can be backpatched when the end instruction of the label
//is hit.
fn generate_unbackpatched_sidetable_entry(
    wasm: &WasmReader,
    sidetable: &mut Sidetable,
    valcnt: usize,
    popcnt: usize,
    label_info: &mut LabelInfo,
) {
    let stp_here = sidetable.len();

    sidetable.push(SidetableEntry {
        delta_pc: wasm.pc as isize,
        delta_stp: stp_here as isize,
        popcnt,
        valcnt,
    });

    match label_info {
        LabelInfo::Block { stps_to_backpatch } => stps_to_backpatch.push(stp_here),
        LabelInfo::Loop { ip, stp } => {
            //we already know where to jump to for loops
            sidetable[stp_here].delta_pc = *ip as isize - wasm.pc as isize;
            sidetable[stp_here].delta_stp = *stp as isize - stp_here as isize;
        }
        LabelInfo::If {
            stps_to_backpatch, ..
        } => stps_to_backpatch.push(stp_here),
        LabelInfo::Func { stps_to_backpatch } => stps_to_backpatch.push(stp_here),
        LabelInfo::Untyped => {
            unreachable!("this label is for untyped wasm sequences")
        }
    }
}

//helper function to avoid code duplication for common stuff in br, br_if, return
fn validate_intrablock_jump_and_generate_sidetable_entry(
    wasm: &WasmReader,
    label_idx: usize,
    stack: &mut ValidationStack,
    sidetable: &mut Sidetable,
) -> Result<()> {
    let ctrl_stack_len = stack.ctrl_stack.len();

    stack.assert_val_types_of_label_jump_types_on_top(label_idx)?;

    let targeted_ctrl_block_entry = stack
        .ctrl_stack
        .get(ctrl_stack_len - label_idx - 1)
        .ok_or(Error::InvalidLabelIdx(label_idx))?;

    let valcnt = targeted_ctrl_block_entry.label_types().len();
    let popcnt = stack.len() - targeted_ctrl_block_entry.height - valcnt;

    let label_info = &mut stack
        .ctrl_stack
        .get_mut(ctrl_stack_len - label_idx - 1)
        .unwrap()
        .label_info;

    generate_unbackpatched_sidetable_entry(wasm, sidetable, valcnt, popcnt, label_info);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn read_instructions(
    wasm: &mut WasmReader,
    stack: &mut ValidationStack,
    sidetable: &mut Sidetable,
    locals: &[ValType],
    globals: &[Global],
    fn_types: &[FuncType],
    type_idx_of_fn: &[usize],
    memories: &[MemType],
    data_count: &Option<u32>,
    tables: &[TableType],
    elements: &[ElemType],
    referenced_functions: &BTreeSet<u32>,
) -> Result<()> {
    loop {
        let Ok(first_instr_byte) = wasm.read_u8() else {
            // TODO only do this if EOF
            return Err(Error::ExprMissingEnd);
        };

        #[cfg(debug_assertions)]
        crate::core::utils::print_beautiful_instruction_name_1_byte(first_instr_byte, wasm.pc);

        #[cfg(not(debug_assertions))]
        trace!("Read instruction byte {first_instr_byte:#04X?} ({first_instr_byte}) at wasm_binary[{}]", wasm.pc);

        use crate::core::reader::types::opcode::*;
        match first_instr_byte {
            // nop: [] -> []
            NOP => {}
            // block: [] -> [t*2]
            BLOCK => {
                let block_ty = BlockType::read(wasm)?.as_func_type(fn_types)?;
                let label_info = LabelInfo::Block {
                    stps_to_backpatch: Vec::new(),
                };
                stack.assert_push_ctrl(label_info, block_ty)?;
            }
            LOOP => {
                let block_ty = BlockType::read(wasm)?.as_func_type(fn_types)?;
                let label_info = LabelInfo::Loop {
                    ip: wasm.pc,
                    stp: sidetable.len(),
                };
                stack.assert_push_ctrl(label_info, block_ty)?;
            }
            IF => {
                let block_ty = BlockType::read(wasm)?.as_func_type(fn_types)?;

                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                let stp_here = sidetable.len();
                sidetable.push(SidetableEntry {
                    delta_pc: wasm.pc as isize,
                    delta_stp: stp_here as isize,
                    popcnt: 0,
                    valcnt: block_ty.params.valtypes.len(),
                });

                let label_info = LabelInfo::If {
                    stp: stp_here,
                    stps_to_backpatch: Vec::new(),
                };
                stack.assert_push_ctrl(label_info, block_ty)?;
            }
            ELSE => {
                let (mut label_info, block_ty) = stack.assert_pop_ctrl()?;
                if let LabelInfo::If {
                    stp,
                    stps_to_backpatch,
                } = &mut label_info
                {
                    if *stp == usize::MAX {
                        //this If was previously matched with an else already, it is already backpatched!
                        return Err(Error::ElseWithoutMatchingIf);
                    }
                    let stp_here = sidetable.len();
                    sidetable.push(SidetableEntry {
                        delta_pc: wasm.pc as isize,
                        delta_stp: stp_here as isize,
                        popcnt: 0,
                        valcnt: block_ty.returns.valtypes.len(),
                    });
                    stps_to_backpatch.push(stp_here);

                    sidetable[*stp].delta_pc = wasm.pc as isize - sidetable[*stp].delta_pc;
                    sidetable[*stp].delta_stp =
                        sidetable.len() as isize - sidetable[*stp].delta_stp;

                    *stp = usize::MAX; // mark this If as backpatched

                    for valtype in block_ty.returns.valtypes.iter().rev() {
                        stack.assert_pop_val_type(*valtype)?;
                    }

                    for valtype in block_ty.params.valtypes.iter() {
                        stack.push_valtype(*valtype);
                    }

                    stack.assert_push_ctrl(label_info, block_ty)?;
                } else {
                    return Err(Error::ElseWithoutMatchingIf);
                }
            }
            BR => {
                let label_idx = wasm.read_var_u32()? as LabelIdx;
                validate_intrablock_jump_and_generate_sidetable_entry(
                    wasm, label_idx, stack, sidetable,
                )?;
                stack.make_unspecified()?;
            }
            BR_IF => {
                let label_idx = wasm.read_var_u32()? as LabelIdx;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                validate_intrablock_jump_and_generate_sidetable_entry(
                    wasm, label_idx, stack, sidetable,
                )?;
            }
            BR_TABLE => {
                let label_vec = wasm.read_vec(|wasm| wasm.read_var_u32().map(|v| v as LabelIdx))?;
                let max_label_idx = wasm.read_var_u32()? as LabelIdx;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                for label_idx in &label_vec {
                    validate_intrablock_jump_and_generate_sidetable_entry(
                        wasm, *label_idx, stack, sidetable,
                    )?;
                }

                validate_intrablock_jump_and_generate_sidetable_entry(
                    wasm,
                    max_label_idx,
                    stack,
                    sidetable,
                )?;

                // The label arity of the branches must be explicitly checked against each other further
                // if their arities are the same, then they must unify, as they unify against the stack variables already
                // If the following check is not made, the algorithm incorrectly unifies label types with different arities
                // in which the smaller arity type is a suffix in the label type list of the larger arity function

                // stack includes all labels, that check is made in the above fn already
                let max_label_arity = stack
                    .ctrl_stack
                    .get(stack.ctrl_stack.len() - max_label_idx - 1)
                    .unwrap()
                    .label_types()
                    .len();
                for label_idx in &label_vec {
                    let label_arity = stack
                        .ctrl_stack
                        .get(stack.ctrl_stack.len() - *label_idx - 1)
                        .unwrap()
                        .label_types()
                        .len();
                    if max_label_arity != label_arity {
                        return Err(Error::InvalidLabelIdx(*label_idx));
                    }
                }

                stack.make_unspecified()?;
            }
            END => {
                let (label_info, block_ty) = stack.assert_pop_ctrl()?;
                let stp_here = sidetable.len();

                match label_info {
                    LabelInfo::Block { stps_to_backpatch } => {
                        stps_to_backpatch.iter().for_each(|i| {
                            sidetable[*i].delta_pc = (wasm.pc as isize) - sidetable[*i].delta_pc;
                            sidetable[*i].delta_stp = (stp_here as isize) - sidetable[*i].delta_stp;
                        });
                    }
                    LabelInfo::If {
                        stp,
                        stps_to_backpatch,
                    } => {
                        if stp != usize::MAX {
                            //This If is still not backpatched, meaning it does not have a corresponding
                            //ELSE. This is only allowed when the corresponding If block has the same input
                            //types as its output types (an untyped ELSE block with no instruction is valid
                            //if and only if it is of this type)
                            if !(block_ty.params == block_ty.returns) {
                                return Err(Error::IfWithoutMatchingElse);
                            }

                            //This If is still not backpatched, meaning it does not have a corresponding
                            //ELSE. Therefore if its condition fails, it jumps after END.
                            sidetable[stp].delta_pc = (wasm.pc as isize) - sidetable[stp].delta_pc;
                            sidetable[stp].delta_stp =
                                (stp_here as isize) - sidetable[stp].delta_stp;
                        }
                        stps_to_backpatch.iter().for_each(|i| {
                            sidetable[*i].delta_pc = (wasm.pc as isize) - sidetable[*i].delta_pc;
                            sidetable[*i].delta_stp = (stp_here as isize) - sidetable[*i].delta_stp;
                        });
                    }
                    LabelInfo::Loop { .. } => (),
                    LabelInfo::Func { stps_to_backpatch } => {
                        // same as blocks, except jump just before the end instr, not after it
                        // the last end instruction will handle the return to callee during execution
                        stps_to_backpatch.iter().for_each(|i| {
                            sidetable[*i].delta_pc =
                                (wasm.pc as isize) - sidetable[*i].delta_pc - 1; // minus 1 is important! TODO: Why?
                            sidetable[*i].delta_stp = (stp_here as isize) - sidetable[*i].delta_stp;
                        });
                    }
                    LabelInfo::Untyped => unreachable!("this label is for untyped wasm sequences"),
                }

                if stack.ctrl_stack.is_empty() {
                    return Ok(());
                }
            }
            RETURN => {
                let label_idx = stack.ctrl_stack.len() - 1; // return behaves the same as br <most_outer>
                validate_intrablock_jump_and_generate_sidetable_entry(
                    wasm, label_idx, stack, sidetable,
                )?;
                stack.make_unspecified()?;
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
            CALL_INDIRECT => {
                let type_idx = wasm.read_var_u32()? as TypeIdx;

                let table_idx = wasm.read_var_u32()? as TableIdx;

                if tables.len() <= table_idx {
                    return Err(Error::TableIsNotDefined(table_idx));
                }

                let tab = &tables[table_idx];

                if tab.et != RefType::FuncRef {
                    return Err(Error::WrongRefTypeForInteropValue(tab.et, RefType::FuncRef));
                }

                if type_idx >= fn_types.len() {
                    return Err(Error::FunctionTypeIsNotDefined(type_idx));
                }

                let func_ty = &fn_types[type_idx];

                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                for typ in func_ty.params.valtypes.iter().rev() {
                    stack.assert_pop_val_type(*typ)?;
                }

                for typ in func_ty.returns.valtypes.iter() {
                    stack.push_valtype(*typ);
                }
            }
            // unreachable: [t1*] -> [t2*]
            UNREACHABLE => {
                stack.make_unspecified()?;
            }
            DROP => {
                stack.drop_val()?;
            }
            SELECT => {
                stack.validate_polymorphic_select()?;
            }
            SELECT_T => {
                let type_vec = wasm.read_vec(ValType::read)?;
                if type_vec.len() != 1 {
                    return Err(Error::InvalidSelectTypeVector);
                }
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(type_vec[0])?;
                stack.assert_pop_val_type(type_vec[0])?;
                stack.push_valtype(type_vec[0]);
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
                stack.assert_val_types_on_top(&[*local_ty])?;
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
            TABLE_GET => {
                let table_idx = wasm.read_var_u32()? as TableIdx;

                if tables.len() <= table_idx {
                    return Err(Error::TableIsNotDefined(table_idx));
                }

                let t = tables.get(table_idx).unwrap().et;

                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.push_valtype(ValType::RefType(t));
            }
            TABLE_SET => {
                let table_idx = wasm.read_var_u32()? as TableIdx;

                if tables.len() <= table_idx {
                    return Err(Error::TableIsNotDefined(table_idx));
                }

                let t = tables.get(table_idx).unwrap().et;

                stack.assert_pop_ref_type(Some(t))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
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
                if memarg.align > 4 {
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
                if memarg.align > 4 {
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
                if mem_idx != 0 {
                    return Err(Error::UnsupportedProposal(
                        crate::core::error::Proposal::MultipleMemories,
                    ));
                }
                if memories.len() <= mem_idx {
                    return Err(Error::MemoryIsNotDefined(mem_idx));
                }
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            MEMORY_GROW => {
                let mem_idx = wasm.read_u8()? as MemIdx;
                if mem_idx != 0 {
                    return Err(Error::UnsupportedProposal(
                        crate::core::error::Proposal::MultipleMemories,
                    ));
                }
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

            REF_NULL => {
                let reftype = RefType::read(wasm)?;
                // at validation-time we don't really care if it's null or not
                stack.push_valtype(ValType::RefType(reftype));
            }

            REF_IS_NULL => {
                stack.assert_pop_ref_type(None)?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }

            // TODO finish this
            // https://webassembly.github.io/spec/core/valid/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
            REF_FUNC => {
                // We will be making use of fn_types to check for length of possible functions
                // Is this okay?
                // I don't know
                let funcs: Vec<()> = vec![(); fn_types.len()];
                let func_idx = wasm.read_var_u32()? as FuncIdx;
                if func_idx >= funcs.len() {
                    return Err(Error::FunctionIsNotDefined(func_idx));
                }

                if !referenced_functions.contains(&(func_idx as u32)) {
                    return Err(Error::ReferencingAnUnreferencedFunction(func_idx));
                }

                stack.push_valtype(ValType::RefType(RefType::FuncRef));
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
                        if mem_idx != 0 {
                            return Err(Error::UnsupportedProposal(
                                crate::core::error::Proposal::MultipleMemories,
                            ));
                        }
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
                        if dst != 0 || src != 0 {
                            return Err(Error::UnsupportedProposal(
                                crate::core::error::Proposal::MultipleMemories,
                            ));
                        }
                        if memories.is_empty() {
                            return Err(Error::MemoryIsNotDefined(0));
                        }
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                    }
                    MEMORY_FILL => {
                        let mem_idx = wasm.read_u8()? as MemIdx;
                        if mem_idx != 0 {
                            return Err(Error::UnsupportedProposal(
                                crate::core::error::Proposal::MultipleMemories,
                            ));
                        }
                        if memories.len() <= mem_idx {
                            return Err(Error::MemoryIsNotDefined(mem_idx));
                        }
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                    }
                    TABLE_INIT => {
                        let elem_idx = wasm.read_var_u32()? as ElemIdx;
                        let table_idx = wasm.read_var_u32()? as TableIdx;

                        if tables.len() <= table_idx {
                            return Err(Error::TableIsNotDefined(table_idx));
                        }

                        let t1 = tables[table_idx].et;

                        if elements.len() <= elem_idx {
                            return Err(Error::ElementIsNotDefined(elem_idx));
                        }

                        let t2 = elements[elem_idx].to_ref_type();

                        if t1 != t2 {
                            return Err(Error::DifferentRefTypes(t1, t2));
                        }
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        // INFO: wasmtime checks for this value to be an index in the tables array, interesting
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                    }
                    ELEM_DROP => {
                        let elem_idx = wasm.read_var_u32()? as ElemIdx;

                        if elements.len() <= elem_idx {
                            return Err(Error::ElementIsNotDefined(elem_idx));
                        }
                    }
                    TABLE_COPY => {
                        let table_x_idx = wasm.read_var_u32()? as TableIdx;
                        let table_y_idx = wasm.read_var_u32()? as TableIdx;

                        if tables.len() <= table_x_idx {
                            return Err(Error::TableIsNotDefined(table_x_idx));
                        }

                        if tables.len() <= table_y_idx {
                            return Err(Error::TableIsNotDefined(table_y_idx));
                        }

                        let t1 = tables[table_x_idx].et;
                        let t2 = tables[table_y_idx].et;

                        if t1 != t2 {
                            return Err(Error::DifferentRefTypes(t1, t2));
                        }

                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                    }
                    TABLE_GROW => {
                        let table_idx = wasm.read_var_u32()? as TableIdx;

                        if tables.len() <= table_idx {
                            return Err(Error::TableIsNotDefined(table_idx));
                        }

                        let t = tables[table_idx].et;

                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_ref_type(Some(t))?;

                        stack.push_valtype(ValType::NumType(NumType::I32));
                    }
                    TABLE_SIZE => {
                        let table_idx = wasm.read_var_u32()? as TableIdx;

                        if tables.len() <= table_idx {
                            return Err(Error::TableIsNotDefined(table_idx));
                        }

                        stack.push_valtype(ValType::NumType(NumType::I32));
                    }
                    TABLE_FILL => {
                        let table_idx = wasm.read_var_u32()? as TableIdx;

                        if tables.len() <= table_idx {
                            return Err(Error::TableIsNotDefined(table_idx));
                        }

                        let t = tables[table_idx].et;

                        stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                        stack.assert_pop_ref_type(Some(t))?;
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

            I32_EXTEND8_S => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I32_EXTEND16_S => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_EXTEND8_S => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I64_EXTEND16_S => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I64_EXTEND32_S => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }

            _ => return Err(Error::InvalidInstr(first_instr_byte)),
        }
    }
}
