use crate::core::reader::span::Span;
use crate::core::reader::types::ValType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::ValidationError;

#[derive(Debug, Copy, Clone)]
pub struct Global {
    pub ty: GlobalType,
    // TODO validate init_expr during validation and execute during instantiation
    pub init_expr: Span,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub ty: ValType,
    pub is_mut: bool,
}

impl WasmReadable for GlobalType {
    fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let ty = ValType::read(wasm)?;
        let is_mut = match wasm.read_u8()? {
            0x00 => false,
            0x01 => true,
            other => return Err(ValidationError::InvalidMutType(other)),
        };
        Ok(Self { ty, is_mut })
    }
}
