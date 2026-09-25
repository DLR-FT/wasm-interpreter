use core::iter::Map;
use dlr_wasm_interpreter::{Export, ExternType, Import, ValidationConfig, ValidationError};

#[derive(Clone, Debug)]
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

impl<'wasm> Module<'wasm> {
    pub fn imports<'a>(
        &'a self,
    ) -> Map<core::slice::Iter<'a, Import>, impl FnMut(&'a Import) -> (&'a str, &'a str, ExternType)>
    {
        self.inner.imports(self.wasm)
    }

    pub fn exports<'a>(
        &'a self,
    ) -> Map<core::slice::Iter<'a, Export>, impl FnMut(&'a Export) -> (&'a str, ExternType)> {
        self.inner.exports(self.wasm)
    }
}
