use crate::core::reader::types::ValType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::{unreachable_validated, Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Global {
    pub ty: GlobalType,
    // TODO validate init_expr during validation and execute during instantiation
    pub init_expr: (),
}

impl WasmReadable for Global {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        Ok(Self {
            ty: GlobalType::read(wasm)?,
            init_expr: (), /* todo!("read constant expression") */
        })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        Self {
            ty: GlobalType::read_unvalidated(wasm),
            init_expr: (), /* todo!("read constant expression") */
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub ty: ValType,
    pub is_mut: bool,
}

impl WasmReadable for GlobalType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let ty = ValType::read(wasm)?;
        let is_mut = match wasm.read_u8()? {
            0x00 => false,
            0x01 => true,
            other => return Err(Error::InvalidMutType(other)),
        };
        Ok(Self { ty, is_mut })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let ty = ValType::read_unvalidated(wasm);
        let is_mut = match wasm.read_u8().unwrap_validated() {
            0x00 => false,
            0x01 => true,
            _ => unreachable_validated!(),
        };

        Self { ty, is_mut }
    }
}
