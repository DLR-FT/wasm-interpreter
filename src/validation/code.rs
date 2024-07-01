use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::iter;

use crate::core::indices::{GlobalIdx, LocalIdx};
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
    globals: &[Global],
) -> Result<Vec<Span>> {
    assert_eq!(section_header.ty, SectionTy::Code);

    let code_block_spans = wasm.read_vec_enumerated(|wasm, idx| {
        // TODO maybe offset idx by number of imported functions?
        let func_ty = fn_types[idx].clone();

        let func_size = wasm.read_var_u32()?;
        let func_block = wasm.make_span(func_size as usize);

        let locals = {
            let params = func_ty.params.valtypes.iter().cloned();
            let declared_locals = read_declared_locals(wasm)?;
            params.chain(declared_locals).collect::<Vec<ValType>>()
        };

        validate_value_stack(func_ty.returns, |value_stack| {
            read_instructions(wasm, value_stack, &locals, globals)
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
    wasm: &mut WasmReader,
    value_stack: &mut VecDeque<ValType>,
    locals: &[ValType],
    globals: &[Global],
) -> Result<()> {
    let assert_pop_value_stack = |value_stack: &mut VecDeque<ValType>, expected_ty: ValType| {
        value_stack
            .pop_back()
            .ok_or(Error::InvalidValueStackType(None))
            .and_then(|ty| {
                (ty == expected_ty)
                    .then_some(())
                    .ok_or(Error::InvalidValueStackType(Some(ty)))
            })
    };

    loop {
        let Ok(instr) = wasm.read_u8() else {
            return Err(Error::ExprMissingEnd);
        };
        trace!("Read instruction byte {instr:#x?} ({instr})");
        match instr {
            // nop
            0x01 => {}
            // end
            0x0B => {
                return Ok(());
            }
            // local.get: [] -> [t]
            0x20 => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                value_stack.push_back(*local_ty);
            }
            // local.set [t] -> [0]
            0x21 => {
                let local_idx = wasm.read_var_u32()? as LocalIdx;
                let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                let popped = value_stack.pop_back();
                if popped.as_ref() != Some(local_ty) {
                    return Err(Error::InvalidValueStackType(popped));
                }
            }
            // global.get [] -> [t]
            0x23 => {
                let global_idx = wasm.read_var_u32()? as GlobalIdx;
                let global = globals
                    .get(global_idx)
                    .ok_or(Error::InvalidGlobalIdx(global_idx))?;

                value_stack.push_back(global.ty.ty);
            }
            // global.set [t] -> []
            0x24 => {
                let global_idx = wasm.read_var_u32()? as GlobalIdx;
                let global = globals
                    .get(global_idx)
                    .ok_or(Error::InvalidGlobalIdx(global_idx))?;

                if !global.ty.is_mut {
                    return Err(Error::GlobalIsConst);
                }

                let ty_on_stack = value_stack
                    .pop_back()
                    .ok_or(Error::InvalidValueStackType(None))?;

                if ty_on_stack != global.ty.ty {
                    return Err(Error::InvalidValueStackType(Some(ty_on_stack)));
                }
            }
            // i32.load [i32] -> [i32]
            0x28 => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // TODO check correct `memarg.align`
                // TODO check if memory[0] exists

                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push_back(ValType::NumType(NumType::I32));
            }
            // i32.store [i32] -> [i32]
            0x36 => {
                let _memarg = MemArg::read_unvalidated(wasm);

                // TODO check correct `memarg.align`
                // TODO check if memory[0] exists

                // Address
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                // Value to store
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
            }
            // i32.add: [i32 i32] -> [i32]
            0x6A => {
                // First value
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                // Second value
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push_back(ValType::NumType(NumType::I32));
            }
            0x6C => {
                // First value
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                // Second value
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push_back(ValType::NumType(NumType::I32));
            }
            // i32.div_s: [i32 i32] -> [i32]
            0x6D => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push_back(ValType::NumType(NumType::I32));
            }
            // i32.const: [] -> [i32]
            0x41 => {
                let _num = wasm.read_var_i32()?;
                value_stack.push_back(ValType::NumType(NumType::I32));
            }
            other => {
                return Err(Error::InvalidInstr(other));
            }
        }
    }
}

fn validate_value_stack<F>(return_ty: ResultType, f: F) -> Result<()>
where
    F: FnOnce(&mut VecDeque<ValType>) -> Result<()>,
{
    let mut value_stack: VecDeque<ValType> = VecDeque::new();

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
