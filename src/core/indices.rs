/// This module defines index types for each Wasm index space class that exists.

// All index types with the exception of [`LocalIdx`]s and [`LabelIdx`] share
// one index space for the entire Wasm module.  Thus these types can provide a
// method to read indices from Wasm bytecode while validating them at the same
// time.  Note that these index types do not provide access to the inner
// integer index value as a safety mechanism to prevent the interpreter from
// creating invalid indices.

// TODO check whether is is clever to internally use usize instead of u32; potential problems are:
// - unsound on architectures where `usize` < `u32`
// - wasteful in memory on architectures where `usize` > `u32`

use crate::ValidationError;

use super::reader::{types::FuncType, WasmReader};

/// An index for a `functype` that is defined in the type section of the current Wasm module.
pub struct TypeIdx(u32);

impl TypeIdx {
    pub fn read_validate(
        wasm: &mut WasmReader,
        c_types: &[FuncType],
    ) -> Result<Self, ValidationError> {
        let idx = wasm.read_var_u32()?;
        if idx as usize >= c_types.len() {
            return Err(ValidationError::InvalidTypeIdx(idx));
        }
        Ok(Self(idx))
    }
}

/// An index for a `func` (function)
/// Spec: v2.0 - 2.5.1 - funcidx
pub type FuncIdx = usize;
pub type TableIdx = usize;
pub type MemIdx = usize;
pub type GlobalIdx = usize;
#[allow(dead_code)]
pub type ElemIdx = usize;
pub type DataIdx = usize;

// pub type LocalIdx = usize;
// #[allow(dead_code)]
// pub type LabelIdx = usize;
