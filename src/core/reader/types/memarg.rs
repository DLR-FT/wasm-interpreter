use core::fmt::Debug;

use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;

#[derive(Debug)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}

impl WasmReadable for MemArg {
    fn read(wasm: &mut WasmReader) -> crate::Result<Self> {
        let align = wasm.read_var_u32()?;
        let offset = wasm.read_var_u32()?;
        Ok(Self { offset, align })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let align = wasm.read_var_u32().unwrap_validated();
        let offset = wasm.read_var_u32().unwrap_validated();
        Self { offset, align }
    }
}
