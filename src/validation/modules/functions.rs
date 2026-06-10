use alloc::{collections::btree_set::BTreeSet, vec::Vec};

use crate::{
    core::{
        decoding::{
            modules::code_section::decode_locals,
            reader::{span::Span, WasmDecoder},
        },
        sidetable::Sidetable,
        structure::{
            modules::{
                element_segments::ElemType,
                globals::Global,
                indices::{
                    ElemIdx, ExtendedIdxVec, FuncIdx, GlobalIdx, IdxVec, MemIdx, TableIdx, TypeIdx,
                },
            },
            types::{FuncType, MemType, TableType, ValType},
        },
        utils::ToUsizeExt,
    },
    validation::{
        instructions::expressions::decode_and_validate_expr, validation_stack::ValidationStack,
    },
    ValidationError,
};

/// Decodes a code section[^binary-format] and validates it[^validation].
///
/// [^binary-format]: [WebAssembly Specification 2.0 - 5.5.13. Code Section](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-codesec).
/// [^validation]: [WebAssembly Specification 2.0 - 3.4.1. Functions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#functions%E2%91%A4).
///
/// # Safety
///
/// The caller must ensure that all index values passed into this function are
/// valid in the relevant `IdxVec`. The following table lists all index types
/// and their respective `IdxVec` types:
///
/// | Index | `IdxVec` |
/// | ----- | -------- |
/// | [`TypeIdx`] | [`IdxVec<TypeIdx, FuncType>`] |
/// | [`FuncIdx`] | [`IdxVec<FuncIdx, TypeIdx>`] contained in [`ExtendedIdxVec<FuncIdx, TypeIdx>`] |
/// | [`TableIdx`] | [`IdxVec<TableIdx, TableType>`] |
#[allow(clippy::too_many_arguments)]
pub unsafe fn decode_and_validate_code_section(
    wasm: &mut WasmDecoder,
    fn_types: &IdxVec<TypeIdx, FuncType>,
    c_funcs: &ExtendedIdxVec<FuncIdx, TypeIdx>,
    c_globals: &IdxVec<GlobalIdx, Global>,
    c_mems: &IdxVec<MemIdx, MemType>,
    data_count: Option<u32>,
    c_tables: &IdxVec<TableIdx, TableType>,
    c_elems: &IdxVec<ElemIdx, ElemType>,
    validation_context_refs: &BTreeSet<FuncIdx>,
    sidetable: &mut Sidetable,
) -> Result<Vec<(Span, usize)>, ValidationError> {
    let code_block_spans_stps = wasm.decode_vec_enumerated(|wasm, idx| {
        // We need to offset the index by the number of functions that were
        // imported. Imported functions always live at the start of the index
        // space.
        let ty_idx = c_funcs
            .iter_local_definitions()
            .nth(idx.into_usize())
            .ok_or(ValidationError::FunctionAndCodeSectionsHaveDifferentLengths)?;

        // SAFETY: The caller ensures that all passed `TypeIdx` values,
        // including this one, are valid in this `IdxVec<TypeIdx, FuncType>`.
        let func_ty: FuncType = unsafe { fn_types.get(*ty_idx).clone() };

        let func_size = wasm.decode_var_u32()?;
        let func_block = wasm.make_span(func_size.into_usize())?;
        let previous_pc = wasm.pc;

        let locals = {
            let params = func_ty.params.valtypes.iter().cloned();
            let declared_locals = decode_locals(wasm)?;
            params.chain(declared_locals).collect::<Vec<ValType>>()
        };

        let mut stack = ValidationStack::new_for_func(func_ty);
        let stp = sidetable.len();

        // SAFETY: The caller ensures the same safety requirements for the same
        // unmodified index values.
        unsafe {
            decode_and_validate_expr(
                wasm,
                &mut stack,
                sidetable,
                &locals,
                c_globals,
                fn_types,
                c_funcs.inner(),
                c_mems,
                data_count,
                c_tables,
                c_elems,
                validation_context_refs,
            )
        }?;

        // Check if there were unread trailing instructions after the last END
        if previous_pc + func_size.into_usize() != wasm.pc {
            return Err(ValidationError::CodeExprHasTrailingInstructions);
        }

        Ok((func_block, stp))
    })?;

    trace!(
        "Read code section. Found {} code blocks",
        code_block_spans_stps.len()
    );

    Ok(code_block_spans_stps)
}
