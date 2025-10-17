use core::fmt::Debug;

use crate::{core::reader::WasmReader, ValidationError};

#[derive(Debug)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}

impl MemArg {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let align = wasm.read_var_u32()?;
        let offset = wasm.read_var_u32()?;
        Ok(Self { offset, align })
    }
}
