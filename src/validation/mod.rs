use alloc::vec::Vec;

use crate::core::indices::{FuncIdx, TypeIdx};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::export::Export;
use crate::core::reader::types::import::Import;
use crate::core::reader::types::{FuncType, GlobalType, MemType, TableType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::{Error, Result};

pub(crate) mod code;

pub struct ValidationInfo<'bytecode> {
    pub(crate) wasm: &'bytecode [u8],
    pub(crate) types: Vec<FuncType>,
    pub(crate) imports: Vec<Import>,
    pub(crate) functions: Vec<TypeIdx>,
    pub(crate) tables: Vec<TableType>,
    pub(crate) memories: Vec<MemType>,
    pub(crate) globals: Vec<GlobalType>,
    pub(crate) exports: Vec<Export>,
    pub(crate) code_blocks: Vec<Span>,
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

    macro_rules! handle_section {
        ($section_ty:pat, $then:expr) => {
            #[allow(unreachable_code)]
            match &header {
                Some(SectionHeader {
                    ty: $section_ty, ..
                }) => {
                    let h = header.take().unwrap();
                    trace!("Handling section {:?}", h.ty);
                    let ret = $then(h);
                    read_next_header(&mut wasm, &mut header)?;
                    Some(ret)
                }
                _ => None,
            }
        };
    }
    macro_rules! skip_custom_sections {
        () => {
            let mut skip_section = || {
                handle_section!(SectionTy::Custom, |h: SectionHeader| {
                    wasm.skip(h.contents.len())
                })
                .transpose()
            };

            while let Some(_) = skip_section()? {}
        };
    }

    skip_custom_sections!();

    let types = handle_section!(SectionTy::Type, |h| {
        wasm.read_vec(|wasm| FuncType::read(wasm))
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    let imports = handle_section!(SectionTy::Import, |h| {
        wasm.read_vec(|wasm| Import::read(wasm))
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    let functions = handle_section!(SectionTy::Function, |h| {
        wasm.read_vec(|wasm| wasm.read_var_u32().map(|u| u as usize))
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    let tables = handle_section!(SectionTy::Table, |h| {
        wasm.read_vec(|wasm| TableType::read(wasm))
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    let memories = handle_section!(SectionTy::Memory, |h| {
        wasm.read_vec(|wasm| MemType::read(wasm))
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    let globals = handle_section!(SectionTy::Global, |h| {
        wasm.read_vec(|wasm| GlobalType::read(wasm))
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    let exports = handle_section!(SectionTy::Export, |h| {
        wasm.read_vec(|wasm| Export::read(wasm))
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    let start = handle_section!(SectionTy::Start, |h| {
        wasm.read_var_u32().map(|idx| idx as FuncIdx)
    })
    .transpose()?;

    skip_custom_sections!();

    handle_section!(SectionTy::Element, |_| {
        todo!("element section not yet supported")
    });

    skip_custom_sections!();

    handle_section!(SectionTy::DataCount, |_| {
        todo!("data count section not yet supported")
    });

    skip_custom_sections!();

    let code_blocks = handle_section!(SectionTy::Code, |h| {
        code::validate_code_section(&mut wasm, h, &types)
    })
    .transpose()?
    .unwrap_or_default();

    assert_eq!(code_blocks.len(), functions.len(), "these should be equal"); // TODO check if this is in the spec

    skip_custom_sections!();

    handle_section!(SectionTy::Data, |_| {
        todo!("data section not yet supported")
    });

    skip_custom_sections!();

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
        code_blocks,
        start,
    })
}

fn read_next_header(wasm: &mut WasmReader, header: &mut Option<SectionHeader>) -> Result<()> {
    if header.is_none() && wasm.remaining_bytes().len() > 0 {
        *header = Some(SectionHeader::read(wasm)?);
    }
    Ok(())
}
