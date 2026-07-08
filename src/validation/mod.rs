use alloc::{
    collections::btree_set::{self, BTreeSet},
    vec::Vec,
};
use core::iter::Map;

use crate::{
    core::{
        decoding::{
            modules::sections::{decode_section_if_ty_matches, SectionTy},
            reader::{span::Span, WasmDecoder},
        },
        sidetable::Sidetable,
        structure::{
            modules::{
                data_segments::DataSegment,
                element_segments::ElemType,
                exports::{Export, ExportDesc},
                globals::Global,
                imports::{Import, ImportDesc},
                indices::{
                    DataIdx, ElemIdx, ExtendedIdxVec, FuncIdx, GlobalIdx, IdxVec,
                    IdxVecOverflowError, MemIdx, TableIdx, TypeIdx,
                },
            },
            types::{ExternType, FuncType, GlobalType, MemType, ResultType, TableType},
        },
        utils::ToUsizeExt,
    },
    validation::{config::ValidationConfig, modules::functions::decode_and_validate_code_section},
    CustomSection, DecodingError, ValidationError,
};

pub mod error;
pub mod instructions;
pub mod modules;
pub mod types;
pub mod validation_stack;

pub mod config;

/// Information collected from validating a module.
///
/// This can be used to instantiate a new module instance in some
/// [`Store`](crate::Store) thorugh
/// [`Store::module_instantiate`](crate::Store::module_instantiate)
#[derive(Clone, Debug)]
pub struct Module<'bytecode> {
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

fn validate_no_duplicate_exports(validation_info: &Module) -> Result<(), ValidationError> {
    let mut found_export_names: btree_set::BTreeSet<&str> = btree_set::BTreeSet::new();
    for export in &validation_info.exports {
        if found_export_names.contains(export.name) {
            return Err(ValidationError::DuplicateExportName);
        }
        found_export_names.insert(export.name);
    }
    Ok(())
}

pub fn decode_and_validate<'wasm, T: ValidationConfig>(
    wasm: &'wasm [u8],
    user_data: &mut T,
) -> Result<Module<'wasm>, ValidationError> {
    let mut wasm = WasmDecoder::new(wasm);

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
        return Err(DecodingError::InvalidMagic.into());
    };

    trace!("Validating version number");
    let [0x01, 0x00, 0x00, 0x00] = wasm.strip_bytes::<4>()? else {
        return Err(DecodingError::InvalidBinaryFormatVersion.into());
    };
    debug!("Header ok");

    let mut custom_sections = Vec::new();
    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let types = decode_section_if_ty_matches(&mut wasm, SectionTy::Type, |wasm, _| {
        wasm.decode_vec(FuncType::decode).map(|types| IdxVec::new(types).expect("that index space creation never fails because the length of the types vector is encoded as a 32-bit integer in the bytecode"))
    }) ?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let imports = decode_section_if_ty_matches(&mut wasm, SectionTy::Import, |wasm, _| {
        wasm.decode_vec(|wasm| Import::decode_and_validate(wasm, &types))
    })?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    // The `Function` section only covers module-level (or "local") functions.
    // Imported functions have their types known in the `import` section. Both
    // local and imported functions share the same index space.
    //
    // Imported functions are given priority and have the first indicies, and
    // only after that do the local functions get assigned their indices.
    let local_functions =
        decode_section_if_ty_matches(&mut wasm, SectionTy::Function, |wasm, _| {
            wasm.decode_vec(|wasm| TypeIdx::decode_and_validate(wasm, &types))
        })?
        .unwrap_or_default();

    let imported_functions = imports.iter().filter_map(|import| match &import.desc {
        ImportDesc::Func(type_idx) => Some(*type_idx),
        _ => None,
    });

    let functions = ExtendedIdxVec::new(imported_functions.collect(), local_functions)
        .map_err(|IdxVecOverflowError| ValidationError::TooManyFunctions)?;

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let imported_tables = imports.iter().filter_map(|m| match m.desc {
        ImportDesc::Table(table) => Some(table),
        _ => None,
    });
    let local_tables = decode_section_if_ty_matches(&mut wasm, SectionTy::Table, |wasm, _| {
        wasm.decode_vec(TableType::decode_and_validate)
    })?
    .unwrap_or_default();

    let tables = ExtendedIdxVec::new(imported_tables.collect(), local_tables)
        .map_err(|IdxVecOverflowError| ValidationError::TooManyTables)?;

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let imported_memories = imports.iter().filter_map(|m| match m.desc {
        ImportDesc::Mem(mem) => Some(mem),
        _ => None,
    });
    // let imported_memories_length = imported_memories.len();
    let local_memories = decode_section_if_ty_matches(&mut wasm, SectionTy::Memory, |wasm, _| {
        wasm.decode_vec(MemType::decode_and_validate)
    })?
    .unwrap_or_default();

    let memories = ExtendedIdxVec::new(imported_memories.collect(), local_memories)
        .map_err(|IdxVecOverflowError| ValidationError::TooManyMemories)?;

    if memories.inner().len() > 1 {
        return Err(ValidationError::UnsupportedMultipleMemoriesProposal);
    }

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let imported_global_types: Vec<GlobalType> = imports
        .iter()
        .filter_map(|m| match m.desc {
            ImportDesc::Global(global) => Some(global),
            _ => None,
        })
        .collect();
    let local_globals = decode_section_if_ty_matches(&mut wasm, SectionTy::Global, |wasm, _| {
        wasm.decode_vec(|wasm| {
            Global::decode_and_validate(
                wasm,
                &imported_global_types,
                &mut validation_context_refs,
                functions.inner(),
            )
        })
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

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let exports = decode_section_if_ty_matches(&mut wasm, SectionTy::Export, |wasm, _| {
        wasm.decode_vec(|wasm| {
            Export::decode_and_validate(
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

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let start = decode_section_if_ty_matches(&mut wasm, SectionTy::Start, |wasm, _| {
        let func_idx = FuncIdx::decode_and_validate(wasm, functions.inner())?;

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

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let elements = decode_section_if_ty_matches(&mut wasm, SectionTy::Element, |wasm, _| {
        ElemType::decode_and_validate(
            wasm,
            functions.inner(),
            &mut validation_context_refs,
            tables.inner(),
            &imported_global_types,
        )
        .map(|elements| IdxVec::new(elements).expect("that index space creation never fails because the length of the elements vector is encoded as a 32-bit integer in the bytecode"))
    })?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    // As per the official documentation:
    //
    // The data count section is used to simplify single-pass validation. Since the data section occurs after the code section, the `memory.init` and `data.drop` and instructions would not be able to check whether the data segment index is valid until the data section is read. The data count section occurs before the code section, so a single-pass validator can use this count instead of deferring validation.
    let data_count: Option<u32> =
        decode_section_if_ty_matches(&mut wasm, SectionTy::DataCount, |wasm, _| {
            wasm.decode_var_u32()
        })?;

    trace!("data count: {data_count:?}");

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let mut sidetable = Sidetable::new();
    let func_blocks_stps = decode_section_if_ty_matches(&mut wasm, SectionTy::Code, |wasm, _| {
        // SAFETY: It is required that all passed index values are valid in all
        // passed `IdxVec`s. The current function does not take any index types
        // as arguments and every `IdxVec<..., ...>` is unique because they use
        // different generics. Therefore, all index types must be valid in their
        // relevant `IdxVec`s.
        unsafe {
            decode_and_validate_code_section(
                wasm,
                &types,
                &functions,
                globals.inner(),
                memories.inner(),
                data_count,
                tables.inner(),
                &elements,
                &validation_context_refs,
                &mut sidetable,
                user_data,
            )
        }
    })?
    .unwrap_or_default();

    if func_blocks_stps.len() != functions.len_local_definitions().into_usize() {
        return Err(ValidationError::FunctionAndCodeSectionsHaveDifferentLengths);
    }

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let data_section = decode_section_if_ty_matches(&mut wasm, SectionTy::Data, |wasm, _| {
    wasm.decode_vec(|wasm| {
        DataSegment::decode_and_validate(wasm, &imported_global_types, functions.inner(), memories.inner())
    })
            .map(|data_segments| IdxVec::new(data_segments).expect("that index space creation never fails because the length of the data segments vector is encoded as a 32-bit integer in the bytecode"))
    })?
    .unwrap_or_default();

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    if let Some(data_count) = data_count {
        if data_count != data_section.len() {
            return Err(ValidationError::DataCountAndDataSectionsLengthAreDifferent);
        }
    }

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    // All sections should have been handled
    if !wasm.remaining_bytes().is_empty() {
        let remaining_section_ty = SectionTy::decode(&mut wasm).expect(
            "that the section type is not malformed, because it must have been peeked before",
        );
        return Err(DecodingError::SectionOutOfOrder(remaining_section_ty).into());
    }

    debug!("Validation was successful");
    let validation_info = Module {
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

/// Reads the next sections as long as they are custom sections and pushes them
/// into the `custom_sections` vector.
fn read_all_custom_sections<'wasm>(
    wasm: &mut WasmDecoder<'wasm>,
    custom_sections: &mut Vec<CustomSection<'wasm>>,
) -> Result<(), ValidationError> {
    while let Some(custom_section) =
        decode_section_if_ty_matches(wasm, SectionTy::Custom, CustomSection::decode)?
    {
        custom_sections.push(custom_section);
    }

    Ok(())
}

impl<'wasm> Module<'wasm> {
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
