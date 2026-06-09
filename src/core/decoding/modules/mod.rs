use crate::{core::decoding::reader::WasmDecoder, DecodingError};

pub mod custom_section;
pub mod indices;
pub mod section_header;

/// Decodes an elemkind: <https://webassembly.github.io/spec/core/binary/modules.html#element-section>
/// # Returns
/// - `Ok(elemkind)` if parsing is successful, Err(_) otherwise
pub fn decode_elemkind(wasm: &mut WasmDecoder) -> Result<u8, DecodingError> {
    let et = wasm.decode_u8()?;
    if et != 0x00 {
        Err(DecodingError::MalformedElemKindDiscriminator(et))
    } else {
        Ok(et)
    }
}
