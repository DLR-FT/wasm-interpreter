use crate::{core::decoding::reader::WasmReader, DecodingError};

pub mod indices;
pub mod section_header;

/// Parse an elemkind: <https://webassembly.github.io/spec/core/binary/modules.html#element-section>
/// # Returns
/// - `Ok(elemkind)` if parsing is successful, Err(_) otherwise
pub fn parse_elemkind(wasm: &mut WasmReader) -> Result<u8, DecodingError> {
    let et = wasm.read_u8()?;
    if et != 0x00 {
        Err(DecodingError::MalformedElemKindDiscriminator(et))
    } else {
        Ok(et)
    }
}
