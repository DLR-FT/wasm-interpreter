use core::iter::Map;

use alloc::collections::btree_set::{self, BTreeSet};
use alloc::vec::Vec;

use crate::core::indices::{
    DataIdx, ElemIdx, ExtendedIdxVec, FuncIdx, GlobalIdx, IdxVec, IdxVecOverflowError, MemIdx,
    TableIdx, TypeIdx,
};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::data::DataSegment;
use crate::core::reader::types::element::ElemType;
use crate::core::reader::types::export::{Export, ExportDesc};
use crate::core::reader::types::global::{Global, GlobalType};
use crate::core::reader::types::import::{Import, ImportDesc};
use crate::core::reader::types::{ExternType, FuncType, MemType, ResultType, TableType};
use crate::core::reader::WasmReader;
use crate::core::sidetable::Sidetable;
use crate::core::utils::ToUsizeExt;
use crate::custom_section::CustomSection;
use crate::ValidationError;

pub(crate) mod code;
pub(crate) mod custom_section;
pub(crate) mod data;
pub(crate) mod globals;
pub(crate) mod read_constant_expression;
pub(crate) mod validation_stack;

/// Information collected from validating a module.
///
/// This can be used to instantiate a new module instance in some
/// [`Store`](crate::Store) thorugh
/// [`Store::module_instantiate`](crate::Store::module_instantiate)
#[derive(Clone, Debug)]
pub struct ValidationInfo<'bytecode> {
    pub(crate) wasm: &'bytecode [u8],
    pub(crate) types: IdxVec<TypeIdx, FuncType>,
    pub(crate) imports: Vec<Import<'bytecode>>,
    pub(crate) functions: ExtendedIdxVec<FuncIdx, TypeIdx>,
    pub(crate) tables: ExtendedIdxVec<TableIdx, TableType>,
    pub(crate) memories: ExtendedIdxVec<MemIdx, MemType>,
    pub(crate) globals: ExtendedIdxVec<GlobalIdx, Global>,
    pub(crate) exports: Vec<Export<'bytecode>>,
    pub(crate) elements: IdxVec<ElemIdx, ElemType>,
    pub(crate) data: IdxVec<DataIdx, DataSegment>,
    /// Each block contains the validated code section and the stp corresponding to
    /// the beginning of that code section
    pub(crate) func_blocks_stps: Vec<(Span, usize)>,
    pub(crate) sidetable: Sidetable,
    /// The start function which is automatically executed during instantiation
    pub(crate) start: Option<FuncIdx>,
    pub(crate) custom_sections: Vec<CustomSection<'bytecode>>,
    // pub(crate) exports_length: Exported,
}

fn validate_no_duplicate_exports(validation_info: &ValidationInfo) -> Result<(), ValidationError> {
    let mut found_export_names: btree_set::BTreeSet<&str> = btree_set::BTreeSet::new();
    for export in &validation_info.exports {
        if found_export_names.contains(export.name) {
            return Err(ValidationError::DuplicateExportName);
        }
        found_export_names.insert(export.name);
    }
    Ok(())
}

pub fn validate(wasm: &[u8]) -> Result<ValidationInfo<'_>, ValidationError> {
    let mut wasm = WasmReader::new(wasm);

    // represents C.refs in https://webassembly.github.io/spec/core/valid/conventions.html#context
    // A func.ref instruction is onlv valid if it has an immediate that is a member of C.refs.
    // this list holds all the func_idx's occurring in the module, except in its functions or start function.
    // I make an exception here by not including func_idx's occuring within data segments in C.refs as well, so that single pass validation is possible.
    // If there is a func_idx within the data segment, this would ultimately mean that data segment cannot be validated,
    // therefore this hack is acceptable.
    // https://webassembly.github.io/spec/core/valid/modules.html#data-segments
    // https://webassembly.github.io/spec/core/valid/modules.html#valid-module

    let mut validation_context_refs: BTreeSet<FuncIdx> = BTreeSet::new();

    trace!("Starting validation of bytecode");

    trace!("Validating magic value");
    let [0x00, 0x61, 0x73, 0x6d] = wasm.strip_bytes::<4>()? else {
        return Err(ValidationError::InvalidMagic);
    };

    trace!("Validating version number");
    let [0x01, 0x00, 0x00, 0x00] = wasm.strip_bytes::<4>()? else {
        return Err(ValidationError::InvalidBinaryFormatVersion);
    };
    debug!("Header ok");

    let mut header = None;
    read_next_header(&mut wasm, &mut header)?;

    let mut custom_sections = Vec::new();
    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let types = handle_section(&mut wasm, &mut header, SectionTy::Type, |wasm, _| {
        wasm.read_vec(FuncType::read).map(|types| IdxVec::new(types).expect("that index space creation never fails because the length of the types vector is encoded as a 32-bit integer in the bytecode"))
    })?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let imports = handle_section(&mut wasm, &mut header, SectionTy::Import, |wasm, _| {
        wasm.read_vec(|wasm| Import::read_and_validate(wasm, &types))
    })?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    // The `Function` section only covers module-level (or "local") functions.
    // Imported functions have their types known in the `import` section. Both
    // local and imported functions share the same index space.
    //
    // Imported functions are given priority and have the first indicies, and
    // only after that do the local functions get assigned their indices.
    let local_functions =
        handle_section(&mut wasm, &mut header, SectionTy::Function, |wasm, _| {
            wasm.read_vec(|wasm| TypeIdx::read_and_validate(wasm, &types))
        })?
        .unwrap_or_default();

    let imported_functions = imports.iter().filter_map(|import| match &import.desc {
        ImportDesc::Func(type_idx) => Some(*type_idx),
        _ => None,
    });

    let functions = ExtendedIdxVec::new(imported_functions.collect(), local_functions)
        .map_err(|IdxVecOverflowError| ValidationError::TooManyFunctions)?;

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let imported_tables = imports.iter().filter_map(|m| match m.desc {
        ImportDesc::Table(table) => Some(table),
        _ => None,
    });
    let local_tables = handle_section(&mut wasm, &mut header, SectionTy::Table, |wasm, _| {
        wasm.read_vec(TableType::read)
    })?
    .unwrap_or_default();

    let tables = ExtendedIdxVec::new(imported_tables.collect(), local_tables)
        .map_err(|IdxVecOverflowError| ValidationError::TooManyTables)?;

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let imported_memories = imports.iter().filter_map(|m| match m.desc {
        ImportDesc::Mem(mem) => Some(mem),
        _ => None,
    });
    // let imported_memories_length = imported_memories.len();
    let local_memories = handle_section(&mut wasm, &mut header, SectionTy::Memory, |wasm, _| {
        wasm.read_vec(MemType::read)
    })?
    .unwrap_or_default();

    let memories = ExtendedIdxVec::new(imported_memories.collect(), local_memories)
        .map_err(|IdxVecOverflowError| ValidationError::TooManyMemories)?;

    if memories.inner().len() > 1 {
        return Err(ValidationError::UnsupportedMultipleMemoriesProposal);
    }

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let imported_global_types: Vec<GlobalType> = imports
        .iter()
        .filter_map(|m| match m.desc {
            ImportDesc::Global(global) => Some(global),
            _ => None,
        })
        .collect();
    let local_globals = handle_section(&mut wasm, &mut header, SectionTy::Global, |wasm, h| {
        globals::validate_global_section(
            wasm,
            h,
            &imported_global_types,
            &mut validation_context_refs,
            functions.inner(),
        )
    })?
    .unwrap_or_default();

    let imported_globals = imported_global_types.iter().map(|ty| Global {
        // TODO using a default MAX value for spans that are never executed is
        // not really safe. Maybe opt for an Option instead.
        init_expr: Span::new(usize::MAX, 0),
        ty: *ty,
    });
    let globals = ExtendedIdxVec::new(imported_globals.collect(), local_globals)
        .map_err(|IdxVecOverflowError| ValidationError::TooManyGlobals)?;

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let exports = handle_section(&mut wasm, &mut header, SectionTy::Export, |wasm, _| {
        wasm.read_vec(|wasm| {
            Export::read_and_validate(
                wasm,
                functions.inner(),
                tables.inner(),
                memories.inner(),
                globals.inner(),
            )
        })
    })?
    .unwrap_or_default();
    validation_context_refs.extend(exports.iter().filter_map(
        |Export { name: _, desc }| match *desc {
            ExportDesc::Func(func_idx) => Some(func_idx),
            _ => None,
        },
    ));

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let start = handle_section(&mut wasm, &mut header, SectionTy::Start, |wasm, _| {
        let func_idx = FuncIdx::read_and_validate(wasm, functions.inner())?;

        // start function signature must be [] -> []
        // https://webassembly.github.io/spec/core/valid/modules.html#start-function
        // SAFETY: We just validated this function index using the same
        // `IdxVec`.
        let type_idx = unsafe { functions.inner().get(func_idx) };

        // SAFETY: There exists only one `IdxVec<TypeIdx, FuncType>` in the
        // current function. Therefore, this has to be the same one used to
        // create and validate this `TypeIdx`.
        let func_type = unsafe { types.get(*type_idx) };
        if func_type
            != &(FuncType {
                params: ResultType {
                    valtypes: Vec::new(),
                },
                returns: ResultType {
                    valtypes: Vec::new(),
                },
            })
        {
            Err(ValidationError::InvalidStartFunctionSignature)
        } else {
            Ok(func_idx)
        }
    })?;

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let elements = handle_section(&mut wasm, &mut header, SectionTy::Element, |wasm, _| {
        ElemType::read_and_validate(
            wasm,
            functions.inner(),
            &mut validation_context_refs,
            tables.inner(),
            &imported_global_types,
        )
        .map(|elements| IdxVec::new(elements).expect("that index space creation never fails because the length of the elements vector is encoded as a 32-bit integer in the bytecode"))
    })?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    // As per the official documentation:
    //
    // The data count section is used to simplify single-pass validation. Since the data section occurs after the code section, the `memory.init` and `data.drop` and instructions would not be able to check whether the data segment index is valid until the data section is read. The data count section occurs before the code section, so a single-pass validator can use this count instead of deferring validation.
    let data_count: Option<u32> =
        handle_section(&mut wasm, &mut header, SectionTy::DataCount, |wasm, _| {
            wasm.read_var_u32()
        })?;
    if let Some(dc) = data_count {
        trace!("data count: {dc}");
    }

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let mut sidetable = Sidetable::new();
    let func_blocks_stps = handle_section(&mut wasm, &mut header, SectionTy::Code, |wasm, h| {
        // SAFETY: It is required that all passed index values are valid in all
        // passed `IdxVec`s. The current function does not take any index types
        // as arguments and every `IdxVec<..., ...>` is unique because they use
        // different generics. Therefore, all index types must be valid in their
        // relevant `IdxVec`s.
        unsafe {
            code::validate_code_section(
                wasm,
                h,
                &types,
                &functions,
                globals.inner(),
                memories.inner(),
                data_count,
                tables.inner(),
                &elements,
                &validation_context_refs,
                &mut sidetable,
            )
        }
    })?
    .unwrap_or_default();

    if func_blocks_stps.len() != functions.len_local_definitions().into_usize() {
        return Err(ValidationError::FunctionAndCodeSectionsHaveDifferentLengths);
    }

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    let data_section = handle_section(&mut wasm, &mut header, SectionTy::Data, |wasm, h| {
        // wasm.read_vec(DataSegment::read)
        data::validate_data_section(wasm, h, &imported_global_types, functions.inner(), memories.inner())
            .map(|data_segments| IdxVec::new(data_segments).expect("that index space creation never fails because the length of the data segments vector is encoded as a 32-bit integer in the bytecode"))
    })?
    .unwrap_or_default();

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    if let Some(data_count) = data_count {
        if data_count != data_section.len() {
            return Err(ValidationError::DataCountAndDataSectionsLengthAreDifferent);
        }
    }

    read_all_custom_sections(&mut wasm, &mut header, &mut custom_sections)?;

    // All sections should have been handled
    if let Some(header) = header {
        return Err(ValidationError::SectionOutOfOrder(header.ty));
    }

    debug!("Validation was successful");
    let validation_info = ValidationInfo {
        wasm: wasm.into_inner(),
        types,
        imports,
        functions,
        tables,
        memories,
        globals,
        exports,
        func_blocks_stps,
        sidetable,
        data: data_section,
        start,
        elements,
        custom_sections,
    };
    validate_no_duplicate_exports(&validation_info)?;

    Ok(validation_info)
}

fn read_next_header(
    wasm: &mut WasmReader,
    header: &mut Option<SectionHeader>,
) -> Result<(), ValidationError> {
    if header.is_none() && !wasm.remaining_bytes().is_empty() {
        *header = Some(SectionHeader::read(wasm)?);
    }
    Ok(())
}

#[inline(always)]
fn handle_section<'wasm, T, F>(
    wasm: &mut WasmReader<'wasm>,
    header: &mut Option<SectionHeader>,
    section_ty: SectionTy,
    handler: F,
) -> Result<Option<T>, ValidationError>
where
    T: 'wasm,
    F: FnOnce(&mut WasmReader<'wasm>, SectionHeader) -> Result<T, ValidationError>,
{
    match &header {
        Some(SectionHeader { ty, .. }) if *ty == section_ty => {
            let h = header.take().unwrap();
            trace!("Handling section {:?}", h.ty);
            let ret = handler(wasm, h)?;
            read_next_header(wasm, header)?;
            Ok(Some(ret))
        }
        _ => Ok(None),
    }
}

/// Reads the next sections as long as they are custom sections and pushes them
/// into the `custom_sections` vector.
fn read_all_custom_sections<'wasm>(
    wasm: &mut WasmReader<'wasm>,
    section_header: &mut Option<SectionHeader>,
    custom_sections: &mut Vec<CustomSection<'wasm>>,
) -> Result<(), ValidationError> {
    let mut read_custom_section = || {
        handle_section(
            wasm,
            section_header,
            SectionTy::Custom,
            CustomSection::read_and_validate,
        )
    };

    while let Some(custom_section) = read_custom_section()? {
        custom_sections.push(custom_section);
    }

    Ok(())
}

impl<'wasm> ValidationInfo<'wasm> {
    /// Returns the imports of this module as an iterator. Each import consist
    /// of a module name, a name and an extern type.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.5 - module_imports
    pub fn imports<'a>(
        &'a self,
    ) -> Map<
        core::slice::Iter<'a, Import<'wasm>>,
        impl FnMut(&'a Import<'wasm>) -> (&'a str, &'a str, ExternType),
    > {
        self.imports.iter().map(|import| {
            // SAFETY: This is sound because the argument is `self` and the
            // import desc also comes from `self`.
            let extern_type = unsafe { import.desc.extern_type(self) };
            (import.module_name, import.name, extern_type)
        })
    }

    /// Returns the exports of this module as an iterator. Each export consist
    /// of a name, and an extern type.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.5 - module_exports
    pub fn exports<'a>(
        &'a self,
    ) -> Map<
        core::slice::Iter<'a, Export<'wasm>>,
        impl FnMut(&'a Export<'wasm>) -> (&'a str, ExternType),
    > {
        self.exports.iter().map(|export| {
            // SAFETY: This is sound because the argument is `self` and the
            // export desc also comes from `self`.
            let extern_type = unsafe { export.desc.extern_type(self) };
            (export.name, extern_type)
        })
    }

    /// Returns a list of all custom sections in the bytecode. Every custom
    /// section consists of its name and the custom section's bytecode
    /// (excluding the name itself).
    pub fn custom_sections(&self) -> &[CustomSection<'wasm>] {
        &self.custom_sections
    }
}
