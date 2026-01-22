use alloc::collections::btree_set::{self, BTreeSet};
use alloc::vec::Vec;

use crate::core::indices::{
    ExtendedIdxVec, FuncIdx, IdxVec, IdxVecOverflowError, MemIdx, TableIdx, TypeIdx,
};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::data::DataSegment;
use crate::core::reader::types::element::ElemType;
use crate::core::reader::types::export::{Export, ExportDesc};
use crate::core::reader::types::global::{Global, GlobalType};
use crate::core::reader::types::import::{Import, ImportDesc};
use crate::core::reader::types::{FuncType, MemType, ResultType, TableType};
use crate::core::reader::WasmReader;
use crate::core::sidetable::Sidetable;
use crate::ValidationError;

pub(crate) mod code;
pub(crate) mod data;
pub(crate) mod globals;
pub(crate) mod read_constant_expression;
pub(crate) mod validation_stack;

#[derive(Clone, Debug)]
pub(crate) struct ImportsLength {
    pub imported_functions: usize,
    pub imported_globals: usize,
    pub imported_memories: usize,
    pub imported_tables: usize,
}

/// Information collected from validating a module.
///
/// This can be used to instantiate a new module instance in some
/// [`Store`](crate::Store) either through
/// [`Store::module_instantiate_unchecked`](crate::Store::module_instantiate_unchecked)
/// or
/// [`Linker::module_instantiate_unchecked`](crate::execution::linker::Linker::module_instantiate_unchecked).
#[derive(Clone, Debug)]
pub struct ValidationInfo<'bytecode> {
    pub(crate) wasm: &'bytecode [u8],
    pub(crate) types: IdxVec<TypeIdx, FuncType>,
    pub(crate) imports: Vec<Import>,
    pub(crate) functions: ExtendedIdxVec<FuncIdx, TypeIdx>,
    pub(crate) tables: ExtendedIdxVec<TableIdx, TableType>,
    pub(crate) memories: ExtendedIdxVec<MemIdx, MemType>,
    pub(crate) globals: Vec<Global>,
    pub(crate) exports: Vec<Export>,
    /// Each block contains the validated code section and the stp corresponding to
    /// the beginning of that code section
    pub(crate) func_blocks_stps: Vec<(Span, usize)>,
    pub(crate) sidetable: Sidetable,
    pub(crate) data: Vec<DataSegment>,
    /// The start function which is automatically executed during instantiation
    pub(crate) start: Option<FuncIdx>,
    pub(crate) elements: Vec<ElemType>,
    pub(crate) imports_length: ImportsLength,
    // pub(crate) exports_length: Exported,
}

fn validate_exports(validation_info: &ValidationInfo) -> Result<(), ValidationError> {
    let mut found_export_names: btree_set::BTreeSet<&str> = btree_set::BTreeSet::new();
    use crate::core::reader::types::export::ExportDesc::*;
    for export in &validation_info.exports {
        if found_export_names.contains(export.name.as_str()) {
            return Err(ValidationError::DuplicateExportName);
        }
        found_export_names.insert(export.name.as_str());
        match export.desc {
            Func(_) => {
                // Function indices are already validated upon creation
            }
            Table(_) => {
                // Table indices are already validated upon creation
            }
            Mem(_) => {
                // Memory indices are already validated upon creation
            }
            Global(global_idx) => {
                if validation_info.globals.len() + validation_info.imports_length.imported_globals
                    <= global_idx
                {
                    return Err(ValidationError::InvalidGlobalIdx(global_idx));
                }
            }
        }
    }
    Ok(())
}

fn get_imports_length(imports: &Vec<Import>) -> ImportsLength {
    let mut imports_length = ImportsLength {
        imported_functions: 0,
        imported_globals: 0,
        imported_memories: 0,
        imported_tables: 0,
    };

    for import in imports {
        match import.desc {
            ImportDesc::Func(_) => imports_length.imported_functions += 1,
            ImportDesc::Global(_) => imports_length.imported_globals += 1,
            ImportDesc::Mem(_) => imports_length.imported_memories += 1,
            ImportDesc::Table(_) => imports_length.imported_tables += 1,
        }
    }

    imports_length
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

    let skip_section = |wasm: &mut WasmReader, section_header: &mut Option<SectionHeader>| {
        handle_section(wasm, section_header, SectionTy::Custom, |wasm, h| {
            // customsec ::= section_0(custom)
            // custom ::= name byte*
            // name ::= b*:vec(byte) => name (if utf8(name) = b*)
            // vec(B) ::= n:u32 (x:B)^n => x^n
            let _name = wasm.read_name()?;

            let remaining_bytes = h
                .contents
                .from()
                .checked_add(h.contents.len())
                .and_then(|res| res.checked_sub(wasm.pc))
                .ok_or(ValidationError::InvalidCustomSectionLength)?;

            // TODO: maybe do something with these remaining bytes?
            let mut _bytes = Vec::new();
            for _ in 0..remaining_bytes {
                _bytes.push(wasm.read_u8()?)
            }
            Ok(())
        })
    };

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let types = handle_section(&mut wasm, &mut header, SectionTy::Type, |wasm, _| {
        wasm.read_vec(FuncType::read).map(|types| IdxVec::new(types).expect("that index space creation never fails because the length of the types vector is encoded as a 32-bit integer in the bytecode"))
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let imports = handle_section(&mut wasm, &mut header, SectionTy::Import, |wasm, _| {
        wasm.read_vec(|wasm| Import::read_and_validate(wasm, &types))
    })?
    .unwrap_or_default();
    let imports_length = get_imports_length(&imports);

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

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

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

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

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

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

    if memories.len() > 1 {
        return Err(ValidationError::UnsupportedMultipleMemoriesProposal);
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    // we start off with the imported globals
    let /* mut */ imported_global_types = imports
        .iter()
        .filter_map(|m| match m.desc {
            ImportDesc::Global(global) => Some(global),
            _ => None,
        })
        .collect::<Vec<GlobalType>>();
    let imported_global_types_len = imported_global_types.len();
    let globals = handle_section(&mut wasm, &mut header, SectionTy::Global, |wasm, h| {
        globals::validate_global_section(
            wasm,
            h,
            &imported_global_types,
            &mut validation_context_refs,
            &functions,
        )
    })?
    .unwrap_or_default();
    let mut all_globals = Vec::new();
    for item in imported_global_types.iter().take(imported_global_types_len) {
        all_globals.push(Global {
            init_expr: Span::new(usize::MAX, 0),
            ty: *item,
        })
    }
    for item in &globals {
        all_globals.push(*item)
    }

    // All globals need to be addressable by a u32
    if all_globals.len() > usize::try_from(u32::MAX).expect("pointer width to be at least 32 bits")
    {
        return Err(ValidationError::TooManyGlobals);
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let exports = handle_section(&mut wasm, &mut header, SectionTy::Export, |wasm, _| {
        wasm.read_vec(|wasm| Export::read_and_validate(wasm, &functions, &tables, &memories))
    })?
    .unwrap_or_default();
    validation_context_refs.extend(exports.iter().filter_map(
        |Export { name: _, desc }| match *desc {
            ExportDesc::Func(func_idx) => Some(func_idx),
            _ => None,
        },
    ));

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let start = handle_section(&mut wasm, &mut header, SectionTy::Start, |wasm, _| {
        let func_idx = FuncIdx::read_and_validate(wasm, &functions)?;

        // start function signature must be [] -> []
        // https://webassembly.github.io/spec/core/valid/modules.html#start-function
        // SAFETY: We just validated this function index using the same
        // `IdxVec`.
        let type_idx = unsafe { functions.get(func_idx) };

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

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let elements: Vec<ElemType> =
        handle_section(&mut wasm, &mut header, SectionTy::Element, |wasm, _| {
            ElemType::read_from_wasm(
                wasm,
                &functions,
                &mut validation_context_refs,
                &tables,
                &imported_global_types,
            )
        })?
        .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

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

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let mut sidetable = Sidetable::new();
    let func_blocks_stps = handle_section(&mut wasm, &mut header, SectionTy::Code, |wasm, h| {
        // SAFETY: It is required that all passed index values are valid in all
        // passed `IdxVec`s. The current function does not take any index types
        // as arguments and every `IdxVec<..., ...>` is unique, i.e. uses
        // different generics. Therefore, all index types must be valid in their
        // relevant `IdxVec`s.
        unsafe {
            code::validate_code_section(
                wasm,
                h,
                &types,
                &functions,
                &all_globals,
                &memories,
                &data_count,
                &tables,
                &elements,
                &validation_context_refs,
                &mut sidetable,
            )
        }
    })?
    .unwrap_or_default();

    if func_blocks_stps.len()
        != usize::try_from(functions.len_local_definitions())
            .expect("architecture to be at least 32 bits")
    {
        return Err(ValidationError::FunctionAndCodeSectionsHaveDifferentLengths);
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let data_section = handle_section(&mut wasm, &mut header, SectionTy::Data, |wasm, h| {
        // wasm.read_vec(DataSegment::read)
        data::validate_data_section(wasm, h, &imported_global_types, &memories, &functions)
    })?
    .unwrap_or_default();

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    if let (Some(data_count), data_len) = (data_count, data_section.len()) {
        if data_count as usize != data_len {
            return Err(ValidationError::DataCountAndDataSectionsLengthAreDifferent);
        }
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

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
        imports_length,
    };
    validate_exports(&validation_info)?;

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
fn handle_section<T, F: FnOnce(&mut WasmReader, SectionHeader) -> Result<T, ValidationError>>(
    wasm: &mut WasmReader,
    header: &mut Option<SectionHeader>,
    section_ty: SectionTy,
    handler: F,
) -> Result<Option<T>, ValidationError> {
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
