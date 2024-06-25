use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;

pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}

impl WasmReadable for MemArg {
    fn read(wasm: &mut WasmReader) -> crate::Result<Self> {
        let offset = wasm.read_var_u32()?;
        #[allow(dead_code)]
        let align = wasm.read_var_u32()?;
        Ok(Self { offset, align })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let offset = wasm.read_var_u32().unwrap_validated();
        let align = wasm.read_var_u32().unwrap_validated();
        Self { offset, align }
    }
}
