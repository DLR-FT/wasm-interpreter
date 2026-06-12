use crate::{
    core::{
        decoding::decoder::WasmDecoder,
        structure::{
            modules::{
                imports::{Import, ImportDesc},
                indices::{IdxVec, TypeIdx},
            },
            types::ExternTypeRef,
        },
    },
    DecodingError, ExternType, FuncType, GlobalType, MemType, Module, TableType, ValidationError,
};

impl<'wasm> Import<'wasm> {
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder<'wasm>,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        let module_name = wasm.decode_name()?;
        let name = wasm.decode_name()?;
        let desc = ImportDesc::decode_and_validate(wasm, c_types)?;

        Ok(Self {
            module_name,
            name,
            desc,
        })
    }
}

impl ImportDesc {
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        let desc = match wasm.decode_u8()? {
            0x00 => Self::Func(TypeIdx::decode_and_validate(wasm, c_types)?),
            // https://webassembly.github.io/spec/core/binary/types.html#table-types
            0x01 => Self::Table(TableType::decode_and_validate(wasm)?),
            0x02 => Self::Mem(MemType::decode_and_validate(wasm)?),
            0x03 => Self::Global(GlobalType::decode(wasm)?),
            other => return Err(DecodingError::MalformedImportDescDiscriminator(other).into()),
        };

        Ok(desc)
    }

    /// returns the external type of `self` according to typing relation,
    /// taking `module` as validation context C
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` comes from the same
    /// [`Module`] that is passed as an argument here.
    pub unsafe fn extern_type_owned(&self, module: &Module) -> ExternType {
        // SAFETY: The caller makes the same safety guarantees as required for extern_type_ref.
        unsafe { self.extern_type(module) }.to_owned()
    }

    /// returns the external type of `self` according to typing relation,
    /// taking `validation_info` as validation context C
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` comes from the same
    /// [`Module`] that is passed as an argument here.
    pub unsafe fn extern_type<'module>(&self, module: &'module Module) -> ExternTypeRef<'module> {
        match self {
            ImportDesc::Func(type_idx) => {
                // unlike ExportDescs, these directly refer to the types section
                // since a corresponding function entry in function section or body
                // in code section does not exist for these

                // SAFETY: The caller ensures that the current `ImportDesc` comes from the same
                // `Module`. Because all type indices contained by a `Module` must always be valid,
                // this is safe.
                let func_type = unsafe { module.types.get(*type_idx) };
                ExternTypeRef::Func(func_type)
            }
            ImportDesc::Table(ty) => ExternTypeRef::Table(*ty),
            ImportDesc::Mem(ty) => ExternTypeRef::Mem(*ty),
            ImportDesc::Global(ty) => ExternTypeRef::Global(*ty),
        }
    }
}
