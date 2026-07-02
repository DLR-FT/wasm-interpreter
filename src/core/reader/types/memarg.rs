use core::fmt::Debug;

use crate::core::{error::DecodingError, reader::WasmReader};

#[derive(Debug)]
pub(crate) struct MemArg {
    pub offset: u32,
    pub align: u32,
}

impl MemArg {
    pub(crate) fn read(wasm: &mut WasmReader) -> Result<Self, DecodingError> {
        let align = wasm.read_var_u32()?;
        let offset = wasm.read_var_u32()?;
        Ok(Self { offset, align })
    }
}
