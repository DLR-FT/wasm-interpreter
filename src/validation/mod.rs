use alloc::collections::btree_set;
use alloc::vec::Vec;

use crate::core::indices::{FuncIdx, TypeIdx};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::data::DataSegment;
use crate::core::reader::types::element::ElemType;
use crate::core::reader::types::export::Export;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::{Import, ImportDesc};
use crate::core::reader::types::{FuncType, MemType, TableType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::sidetable::Sidetable;
use crate::{Error, Result};

pub(crate) mod code;
pub(crate) mod globals;
pub(crate) mod read_constant_expression;
pub(crate) mod validation_stack;

/// Information collected from validating a module.
/// This can be used to create a [crate::RuntimeInstance].
pub struct ValidationInfo<'bytecode> {
    pub(crate) wasm: &'bytecode [u8],
    pub(crate) types: Vec<FuncType>,
    pub(crate) imports: Vec<Import>,
    pub(crate) functions: Vec<TypeIdx>,
    pub(crate) tables: Vec<TableType>,
    pub(crate) memories: Vec<MemType>,
    pub(crate) globals: Vec<Global>,
    #[allow(dead_code)]
    pub(crate) exports: Vec<Export>,
    /// Each block contains the validated code section and the generated sidetable
    pub(crate) func_blocks_stps: Vec<(Span, usize)>,
    pub(crate) data: Vec<DataSegment>,
    /// The start function which is automatically executed during instantiation
    pub(crate) start: Option<FuncIdx>,
    pub(crate) elements: Vec<ElemType>,
    pub(crate) sidetable: Sidetable,
}

pub fn validate(wasm: &[u8]) -> Result<ValidationInfo> {
    let mut wasm = WasmReader::new(wasm);
    trace!("Starting validation of bytecode");

    trace!("Validating magic value");
    let [0x00, 0x61, 0x73, 0x6d] = wasm.strip_bytes::<4>()? else {
        return Err(Error::InvalidMagic);
    };

    trace!("Validating version number");
    let [0x01, 0x00, 0x00, 0x00] = wasm.strip_bytes::<4>()? else {
        return Err(Error::InvalidVersion);
    };
    debug!("Header ok");

    let mut header = None;
    read_next_header(&mut wasm, &mut header)?;

    let skip_section = |wasm: &mut WasmReader, section_header: &mut Option<SectionHeader>| {
        handle_section(wasm, section_header, SectionTy::Custom, |wasm, h| {
            use alloc::string::*;
            // customsec ::= section_0(custom)
            // custom ::= name byte*
            // name ::= b*:vec(byte) => name (if utf8(name) = b*)
            // vec(B) ::= n:u32 (x:B)^n => x^n

            if h.contents.len() == 0 {
                return Ok(());
            }
            let _name = wasm.read_name()?;

            let remaining_bytes = match h
                .contents
                .from()
                .checked_add(h.contents.len())
                .and_then(|res| res.checked_sub(wasm.pc))
            {
                None => Err(Error::InvalidSection(
                    SectionTy::Custom,
                    "Remaining bytes less than 0 after reading name!".to_string(),
                )),
                Some(remaining_bytes) => Ok(remaining_bytes),
            }?;

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
        wasm.read_vec(FuncType::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let imports = handle_section(&mut wasm, &mut header, SectionTy::Import, |wasm, _| {
        wasm.read_vec(Import::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    // The `Function` section only covers module-level (or "local") functions.
    // Imported functions have their types known in the `import` section. Both
    // local and imported functions share the same index space.
    //
    // Imported functions are given priority and have the first indicies, and
    // only after that do the local functions get assigned their indices.
    let local_functions =
        handle_section(&mut wasm, &mut header, SectionTy::Function, |wasm, _| {
            wasm.read_vec(|wasm| wasm.read_var_u32().map(|u| u as usize))
        })?
        .unwrap_or_default();

    let imported_functions = imports.iter().filter_map(|import| match &import.desc {
        ImportDesc::Func(type_idx) => Some(*type_idx),
        _ => None,
    });

    let all_functions = imported_functions
        .clone()
        .chain(local_functions.iter().cloned())
        .collect::<Vec<TypeIdx>>();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let tables = handle_section(&mut wasm, &mut header, SectionTy::Table, |wasm, _| {
        wasm.read_vec(TableType::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let local_memories = handle_section(&mut wasm, &mut header, SectionTy::Memory, |wasm, _| {
        wasm.read_vec(MemType::read)
    })?
    .unwrap_or_default();
    if local_memories.len() > 1 {
        return Err(Error::MoreThanOneMemory);
    }

    let imported_memories = imports
        .iter()
        .filter_map(|import| match &import.desc {
            ImportDesc::Mem(mem_type) => Some(*mem_type),
            _ => None,
        })
        .collect::<Vec<MemType>>();

    if imported_memories.len() > 1 {
        return Err(Error::MoreThanOneMemory);
    }

    let all_memories = imported_memories
        .iter()
        .chain(local_memories.iter())
        .cloned()
        .collect::<Vec<MemType>>();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let globals = handle_section(&mut wasm, &mut header, SectionTy::Global, |wasm, h| {
        globals::validate_global_section(wasm, h)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let exports = handle_section(&mut wasm, &mut header, SectionTy::Export, |wasm, _| {
        wasm.read_vec(Export::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let start = handle_section(&mut wasm, &mut header, SectionTy::Start, |wasm, _| {
        wasm.read_var_u32().map(|idx| idx as FuncIdx)
    })?;

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let mut referenced_functions = btree_set::BTreeSet::new();
    let elements: Vec<ElemType> =
        handle_section(&mut wasm, &mut header, SectionTy::Element, |wasm, _| {
            ElemType::read_from_wasm(
                wasm,
                &all_functions,
                &mut referenced_functions,
                tables.len(),
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
    if data_count.is_some() {
        trace!("data count: {}", data_count.unwrap());
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let mut sidetable = Sidetable::new();
    let func_blocks_stps = handle_section(&mut wasm, &mut header, SectionTy::Code, |wasm, h| {
        code::validate_code_section(
            wasm,
            h,
            &types,
            &all_functions,
            imported_functions.count(),
            &globals,
            &all_memories,
            imported_memories.len(),
            &data_count,
            &tables,
            &elements,
            &referenced_functions,
            &mut sidetable,
        )
    })?
    .unwrap_or_default();

    assert_eq!(
        func_blocks_stps.len(),
        local_functions.len(),
        "these should be equal"
    ); // TODO check if this is in the spec

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let data_section = handle_section(&mut wasm, &mut header, SectionTy::Data, |wasm, _| {
        wasm.read_vec(DataSegment::read)
    })?
    .unwrap_or_default();

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    if data_count.is_some() {
        assert_eq!(data_count.unwrap() as usize, data_section.len());
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    // All sections should have been handled
    if let Some(header) = header {
        return Err(Error::SectionOutOfOrder(header.ty));
    }

    debug!("Validation was successful");
    Ok(ValidationInfo {
        wasm: wasm.into_inner(),
        types,
        imports,
        functions: local_functions,
        tables,
        memories: local_memories,
        globals,
        exports,
        func_blocks_stps,
        data: data_section,
        start,
        elements,
        sidetable,
    })
}

fn read_next_header(wasm: &mut WasmReader, header: &mut Option<SectionHeader>) -> Result<()> {
    if header.is_none() && !wasm.remaining_bytes().is_empty() {
        *header = Some(SectionHeader::read(wasm)?);
    }
    Ok(())
}

#[inline(always)]
fn handle_section<T, F: FnOnce(&mut WasmReader, SectionHeader) -> Result<T>>(
    wasm: &mut WasmReader,
    header: &mut Option<SectionHeader>,
    section_ty: SectionTy,
    handler: F,
) -> Result<Option<T>> {
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
