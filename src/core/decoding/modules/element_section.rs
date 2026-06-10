use crate::{DecodingError, core::decoding::reader::WasmDecoder};

pub struct ElemKind;

impl ElemKind {
    /// Decodes an element kind
    ///
    /// See: [WebAssembly Specification 2.0 - 5.5.12. Element Section](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-elemkind).
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        let byte = wasm.decode_u8()?;

        if byte != 0x00 {
            return Err(DecodingError::MalformedElemKindDiscriminator(byte));
        }

        Ok(ElemKind)
    }
}