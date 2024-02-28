#![no_std]

extern crate alloc;
#[macro_use]
extern crate log;

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

    let mut types = None;
    let mut typeidxs = None;
    let mut exports = None;
    let mut codes = None;

    // returns true if end was reached

    let mut header = None;
    read_next_header(&mut wasm, &mut header)?;

    macro_rules! handle_section {
        ($section_ty:expr, $section_header_ident:ident, $then:stmt) => {
            match header.take() {
                Some($section_header_ident @ SectionHeader {ty, ..},) if ty == $section_ty => {
                    $then
                    read_next_header(&mut wasm, &mut header)?;
                },
                _ => {},
            }
        };
    }

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Type, h, {
        types = Some(wasm.read_type_section(h)?);
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Import, h, {
        todo!("import");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Function, h, {
        typeidxs = Some(wasm.read_function_section(h)?);
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Table, h, {
        todo!("table");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Memory, h, {
        todo!("memory");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Global, h, {
        todo!("global");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Export, h, {
        exports = Some(wasm.read_export_section(h)?);
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Start, h, {
        todo!("start");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Element, h, {
        todo!("element");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::DataCount, h, {
        todo!("data count");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Code, h, {
        codes = Some(wasm.read_code_section(h)?);
    });

    handle_custom_sections(&mut wasm, &mut header)?;

    handle_section!(SectionTy::Data, h, {
        todo!("data");
    });

    handle_custom_sections(&mut wasm, &mut header)?;

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
