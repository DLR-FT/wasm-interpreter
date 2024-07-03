use alloc::vec::Vec;

use crate::core::indices::{FuncIdx, TypeIdx};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::export::Export;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::Import;
use crate::core::reader::types::{FuncType, MemType, TableType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::{Error, Result};

pub(crate) mod code;

/// Information collected from validating a module.
/// This can be used to create a [crate::RuntimeInstance].
pub struct ValidationInfo<'bytecode> {
    pub(crate) wasm: &'bytecode [u8],
    pub(crate) types: Vec<FuncType>,
    #[allow(dead_code)]
    pub(crate) imports: Vec<Import>,
    pub(crate) functions: Vec<TypeIdx>,
    #[allow(dead_code)]
    pub(crate) tables: Vec<TableType>,
    pub(crate) memories: Vec<MemType>,
    pub(crate) globals: Vec<Global>,
    #[allow(dead_code)]
    pub(crate) exports: Vec<Export>,
    pub(crate) func_blocks: Vec<Span>,
    /// The start function which is automatically executed during instantiation
    pub(crate) start: Option<FuncIdx>,
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
            wasm.skip(h.contents.len())
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

    let functions = handle_section(&mut wasm, &mut header, SectionTy::Function, |wasm, _| {
        wasm.read_vec(|wasm| wasm.read_var_u32().map(|u| u as usize))
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let tables = handle_section(&mut wasm, &mut header, SectionTy::Table, |wasm, _| {
        wasm.read_vec(TableType::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let memories = handle_section(&mut wasm, &mut header, SectionTy::Memory, |wasm, _| {
        wasm.read_vec(MemType::read)
    })?
    .unwrap_or_default();
    if memories.len() > 1 {
        return Err(Error::MoreThanOneMemory);
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let globals = handle_section(&mut wasm, &mut header, SectionTy::Global, |wasm, _| {
        wasm.read_vec(|wasm| {
            // TODO validate instructions in `global.init_expr`. Furthermore all of these instructions need to be constant.
            //  See https://webassembly.github.io/spec/core/valid/instructions.html#valid-constant
            //  Maybe we can also execute constant expressions right here so they do not even exist in the runtime environment. <-- Needs further research to check if this is even possible
            Global::read(wasm)
        })
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

    let _: Option<()> = handle_section(&mut wasm, &mut header, SectionTy::Element, |_, _| {
        todo!("element section not yet supported")
    })?;

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let _: Option<()> = handle_section(&mut wasm, &mut header, SectionTy::DataCount, |_, _| {
        todo!("data count section not yet supported")
    })?;

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let func_blocks = handle_section(&mut wasm, &mut header, SectionTy::Code, |wasm, h| {
        code::validate_code_section(wasm, h, &types, &globals)
    })?
    .unwrap_or_default();

    assert_eq!(func_blocks.len(), functions.len(), "these should be equal"); // TODO check if this is in the spec

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let _: Option<()> = handle_section(&mut wasm, &mut header, SectionTy::Data, |_, _| {
        todo!("data section not yet supported")
    })?;

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
        functions,
        tables,
        memories,
        globals,
        exports,
        func_blocks,
        start,
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
