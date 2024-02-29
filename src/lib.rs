#![no_std]

extern crate alloc;
#[macro_use]
extern crate log;

use alloc::borrow::ToOwned;

pub use error::{Error, Result};

use crate::section::{SectionHeader, SectionTy};
use crate::wasm::Wasm;

mod error;
pub(crate) mod section;
pub(crate) mod wasm;

pub struct ValidationInfo {}

pub fn validate(wasm: &[u8]) -> Result<ValidationInfo> {
    let mut wasm = Wasm::new(wasm);
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

    // let mut codes = None;

    // returns true if end was reached

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

    let types = handle_section!(SectionTy::Type, |h| { wasm.read_type_section(h) })
        .transpose()?
        .unwrap_or_default();

    skip_custom_sections!();

    handle_section!(SectionTy::Import, |_| { todo!("import") });

    skip_custom_sections!();

    let _typeidxs =
        handle_section!(SectionTy::Function, |h| { wasm.read_function_section(h) }).transpose()?;

    skip_custom_sections!();

    handle_section!(SectionTy::Table, |_| { todo!("table") });

    skip_custom_sections!();

    handle_section!(SectionTy::Memory, |_| { todo!("memory") });

    skip_custom_sections!();

    handle_section!(SectionTy::Global, |_| { todo!("global") });

    skip_custom_sections!();

    let _exports = handle_section!(SectionTy::Export, |h| { wasm.read_export_section(h) })
        .transpose()?
        .unwrap_or_default();

    skip_custom_sections!();

    handle_section!(SectionTy::Start, |_| { todo!("start") });

    skip_custom_sections!();

    handle_section!(SectionTy::Element, |_| { todo!("element") });

    skip_custom_sections!();

    handle_section!(SectionTy::DataCount, |_| { todo!("data count") });

    skip_custom_sections!();

    handle_section!(SectionTy::Code, |h| { wasm.read_code_section(&types) }).transpose()?;

    skip_custom_sections!();

    handle_section!(SectionTy::Data, |_| { todo!("data") });

    skip_custom_sections!();

    // All sections should have been handled
    if let Some(header) = header {
        return Err(Error::SectionOutOfOrder(header.ty));
    }

    info!("Validation was successful");
    Ok(ValidationInfo {})
}

fn read_next_header(wasm: &mut Wasm, header: &mut Option<SectionHeader>) -> Result<()> {
    if header.is_none() && wasm.remaining_bytes().len() > 0 {
        *header = Some(wasm.read_section_header()?);
    }
    Ok(())
}

fn handle_custom_sections(wasm: &mut Wasm, header: &mut Option<SectionHeader>) -> Result<()> {
    if let Some(SectionHeader {
        ty: SectionTy::Custom,
        ..
    }) = header
    {
        let h = header.take().unwrap();

        // skip custom sections for now
        wasm.skip(h.contents.len())?;
        read_next_header(wasm, header)?;
    }
    Ok(())
}
