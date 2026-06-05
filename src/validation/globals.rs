use alloc::{collections::btree_set::BTreeSet, vec::Vec};

use crate::{
    core::{
        decoding::{
            modules::section_header::{SectionHeader, SectionTy},
            reader::WasmReader,
        },
        structure::{
            modules::{
                globals::Global,
                indices::{FuncIdx, IdxVec, TypeIdx},
            },
            types::GlobalType,
        },
    },
    validation::{
        read_constant_expression::read_constant_expression, validation_stack::ValidationStack,
    },
    ValidationError,
};

/// Validate the global section.
///
/// The global section is a vector of global variables. Each [Global] variable is composed of a [GlobalType] and an
/// initialization expression represented by a constant expression.
///
/// See [`read_constant_expression`] for more information.
pub(super) fn validate_global_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
    imported_global_types: &[GlobalType],
    validation_context_refs: &mut BTreeSet<FuncIdx>,
    c_funcs: &IdxVec<FuncIdx, TypeIdx>,
) -> Result<Vec<Global>, ValidationError> {
    assert_eq!(section_header.ty, SectionTy::Global);

    wasm.read_vec(|wasm| {
        let ty = GlobalType::read(wasm)?;
        let stack = &mut ValidationStack::new();
        let (init_expr, seen_func_idxs) =
            read_constant_expression(wasm, stack, imported_global_types, c_funcs)?;

        stack.assert_val_types(&[ty.ty], true)?;
        validation_context_refs.extend(seen_func_idxs);

        Ok(Global { ty, init_expr })
    })
}
