use crate::{
    core::structure::modules::indices::TypeIdx, ExternType, GlobalType, MemType, TableType,
    ValidationInfo,
};

#[derive(Debug, Clone)]
pub struct Import<'wasm> {
    pub module_name: &'wasm str,
    pub name: &'wasm str,
    pub desc: ImportDesc,
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl ImportDesc {
    /// returns the external type of `self` according to typing relation,
    /// taking `validation_info` as validation context C
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` comes from the same
    /// [`ValidationInfo`] that is passed as an argument here.
    pub unsafe fn extern_type(&self, validation_info: &ValidationInfo) -> ExternType {
        match self {
            ImportDesc::Func(type_idx) => {
                // unlike ExportDescs, these directly refer to the types section
                // since a corresponding function entry in function section or body
                // in code section does not exist for these

                // SAFETY: The caller ensures that the current `ImportDesc`
                // comes from the same `ValidationInfo`. Because all type
                // indices contained by a `ValidationInfo` must always be valid,
                // this is safe.
                let func_type = unsafe { validation_info.types.get(*type_idx) };
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            ImportDesc::Table(ty) => ExternType::Table(*ty),
            ImportDesc::Mem(ty) => ExternType::Mem(*ty),
            ImportDesc::Global(ty) => ExternType::Global(*ty),
        }
    }
}
