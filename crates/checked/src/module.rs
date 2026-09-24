use dlr_wasm_interpreter::{ValidationConfig, ValidationError};

pub struct Module<'wasm> {
    pub(crate) inner: dlr_wasm_interpreter::Module,
    pub(crate) wasm: &'wasm [u8],
}

pub fn decode_and_validate<'wasm, T: ValidationConfig>(
    wasm: &'wasm [u8],
    user_data: &mut T,
) -> Result<Module<'wasm>, ValidationError> {
    Ok(Module {
        inner: dlr_wasm_interpreter::decode_and_validate(wasm, user_data)?,
        wasm,
    })
}
