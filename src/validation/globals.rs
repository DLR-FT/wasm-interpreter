use alloc::vec::Vec;

use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::types::global::{Global, GlobalType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::read_constant_expression::read_constant_expression;
use crate::validation_stack::ValidationStack;
use crate::Result;

/// Validate the global section.
///
/// The global section is a vector of global variables. Each [Global] variable is composed of a [GlobalType] and an
/// initialization expression represented by a constant expression.
///
/// See [`read_constant_expression`] for more information.
pub(super) fn validate_global_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<Global>> {
    assert_eq!(section_header.ty, SectionTy::Global);

    wasm.read_vec(|wasm| {
        let ty = GlobalType::read(wasm)?;
        let init_expr = read_constant_expression(
            wasm,
            &mut ValidationStack::new(),
            Some(ty.ty),
            Some(&[/* todo!(imported globals tpyes) */]),
            // we can't refer to any functions
            None,
        )?;

        Ok(Global { ty, init_expr })
    })
}
