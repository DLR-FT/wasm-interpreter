use crate::{
    core::{
        decoding::reader::WasmDecoder,
        structure::modules::{
            exports::{Export, ExportDesc},
            globals::Global,
            indices::{FuncIdx, GlobalIdx, IdxVec, MemIdx, TableIdx, TypeIdx},
        },
    },
    validation::validation_config::ValidationConfig,
    DecodingError, ExternType, MemType, Module, TableType, ValidationError,
};

impl<'wasm> Export<'wasm> {
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder<'wasm>,
        c_funcs: &IdxVec<FuncIdx, TypeIdx>,
        c_tables: &IdxVec<TableIdx, TableType>,
        c_mems: &IdxVec<MemIdx, MemType>,
        c_globals: &IdxVec<GlobalIdx, Global>,
    ) -> Result<Self, ValidationError> {
        let name = wasm.decode_name()?;
        let desc = ExportDesc::decode_and_validate(wasm, c_funcs, c_tables, c_mems, c_globals)?;
        Ok(Export { name, desc })
    }
}

impl ExportDesc {
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        c_functions: &IdxVec<FuncIdx, TypeIdx>,
        c_tables: &IdxVec<TableIdx, TableType>,
        c_mems: &IdxVec<MemIdx, MemType>,
        c_globals: &IdxVec<GlobalIdx, Global>,
    ) -> Result<Self, ValidationError> {
        let desc_id = wasm.decode_u8()?;

        let desc = match desc_id {
            0x00 => ExportDesc::Func(FuncIdx::decode_and_validate(wasm, c_functions)?),
            0x01 => ExportDesc::Table(TableIdx::decode_and_validate(wasm, c_tables)?),
            0x02 => ExportDesc::Mem(MemIdx::decode_and_validate(wasm, c_mems)?),
            0x03 => ExportDesc::Global(GlobalIdx::decode_and_validate(wasm, c_globals)?),
            other => return Err(DecodingError::MalformedExportDescDiscriminator(other).into()),
        };
        Ok(desc)
    }

    /// returns the external type of `self` according to typing relation,
    /// taking `validation_info` as validation context C
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` comes from the same
    /// [`Module`] that is passed as an argument here.
    #[allow(unused)] // reason = "this function is analogous to ImportDesc::extern_type, however it is not yet clear if it is needed in the future"
    pub unsafe fn extern_type<T: ValidationConfig>(
        &self,
        validation_info: &Module<T>,
    ) -> ExternType {
        // TODO clean up logic for checking if an exported definition is an
        // import
        match self {
            ExportDesc::Func(func_idx) => {
                // SAFETY: The caller ensures that the current `ExportDesc`
                // comes from the same `Module` that is passed into the
                // current function. Therefore, the function index stored in
                // `self` must be valid in the given `Module`.
                let type_idx = unsafe { validation_info.functions.inner().get(*func_idx) };
                // SAFETY: The type index was just read from the passed
                // `Module`.  Because the `Module` struct
                // guarantees that all indices contained in it are valid for all
                // other `IdxVec` vectors in it, this is sound.
                let func_type = unsafe { validation_info.types.get(*type_idx) };
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            ExportDesc::Table(table_idx) => {
                // SAFETY: The caller ensures that the current `ExportDesc`
                // comes from the same `Module` that is passed into the
                // current function. Therefore, the table index stored in `self`
                // must be valid in the given `Module`.
                let table_type = unsafe { validation_info.tables.inner().get(*table_idx) };

                ExternType::Table(*table_type)
            }
            ExportDesc::Mem(mem_idx) => {
                // SAFETY: The caller ensures that the current `ExportDesc`
                // comes from the same `Module` that is passed into the
                // current function. Therefore, the memory index stored in
                // `self` must be valid in the given `Module`.
                let mem_type = unsafe { validation_info.memories.inner().get(*mem_idx) };

                ExternType::Mem(*mem_type)
            }
            ExportDesc::Global(global_idx) => {
                // SAFETY: The caller ensures that the current `ExportDesc`
                // comes from the same `Module` that is passed into the
                // current function. Therefore, the global index stored in
                // `self` must be valid in the given `Module`.
                let global = unsafe { validation_info.globals.inner().get(*global_idx) };

                ExternType::Global(global.ty)
            }
        }
    }
}
