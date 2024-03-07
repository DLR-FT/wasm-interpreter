use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::validation::sections::validate_code_section;
use crate::validation::sections::{
    read_export_section, read_function_section, validate_type_section,
};
use crate::{Error, Result};

pub mod sections;

pub struct ValidationInfo {}

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

    let fn_types = handle_section!(SectionTy::Type, |h| { validate_type_section(&mut wasm, h) })
        .transpose()?
        .unwrap_or_default();

    skip_custom_sections!();

    handle_section!(SectionTy::Import, |_| { todo!("import") });

    skip_custom_sections!();

    let _typeidxs = handle_section!(SectionTy::Function, |h| {
        read_function_section(&mut wasm, h)
    })
    .transpose()?;

    skip_custom_sections!();

    handle_section!(SectionTy::Table, |_| { todo!("table") });

    skip_custom_sections!();

    handle_section!(SectionTy::Memory, |_| { todo!("memory") });

    skip_custom_sections!();

    handle_section!(SectionTy::Global, |_| { todo!("global") });

    skip_custom_sections!();

    let _exports = handle_section!(SectionTy::Export, |h| { read_export_section(&mut wasm, h) })
        .transpose()?
        .unwrap_or_default();

    skip_custom_sections!();

    handle_section!(SectionTy::Start, |_| { todo!("start") });

    skip_custom_sections!();

    handle_section!(SectionTy::Element, |_| { todo!("element") });

    skip_custom_sections!();

    handle_section!(SectionTy::DataCount, |_| { todo!("data count") });

    skip_custom_sections!();

    handle_section!(SectionTy::Code, |h| {
        validate_code_section(&mut wasm, h, &fn_types)
    })
    .transpose()?;

    skip_custom_sections!();

    handle_section!(SectionTy::Data, |_| { todo!("data") });

    skip_custom_sections!();

    // All sections should have been handled
    if let Some(header) = header {
        return Err(Error::SectionOutOfOrder(header.ty));
    }

    debug!("Validation was successful");
    Ok(ValidationInfo {})
}

fn read_next_header(wasm: &mut WasmReader, header: &mut Option<SectionHeader>) -> Result<()> {
    if header.is_none() && wasm.remaining_bytes().len() > 0 {
        *header = Some(SectionHeader::read(wasm)?);
    }
    Ok(())
}
