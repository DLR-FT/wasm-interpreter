use alloc::collections::btree_set::BTreeSet;

use crate::{
    core::{
        decoding::reader::WasmDecoder,
        structure::modules::{
            globals::Global,
            indices::{FuncIdx, IdxVec, TypeIdx},
        },
    },
    validation::{
        instructions::constant_expressions::decode_and_validate_constant_expression,
        validation_stack::ValidationStack,
    },
    GlobalType, ValidationError,
};

impl Global {
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        imported_global_types: &[GlobalType],
        validation_context_refs: &mut BTreeSet<FuncIdx>,
        c_funcs: &IdxVec<FuncIdx, TypeIdx>,
    ) -> Result<Self, ValidationError> {
        let ty = GlobalType::decode(wasm)?;
        let stack = &mut ValidationStack::new();
        let (init_expr, seen_func_idxs) =
            decode_and_validate_constant_expression(wasm, stack, imported_global_types, c_funcs)?;

        stack.assert_val_types(&[ty.ty], true)?;
        validation_context_refs.extend(seen_func_idxs);

        Ok(Global { ty, init_expr })
    }
}
