#![no_std]

extern crate alloc;
#[macro_use]
extern crate log;

pub use error::{Error, Result};

use crate::section::{SectionTy, SectionTypeOrderValidator};
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

    let mut section_order = SectionTypeOrderValidator::new();

    let mut types = None;
    let mut typeidxs = None;
    let mut exports = None;
    let mut codes = None;

    while wasm.remaining_bytes().len() > 0 {
        let section = wasm.read_section_header()?;
        section_order.validate(section.ty)?;
        trace!(
            "Validating section {:?}({}B)",
            section.ty,
            section.contents.len()
        );

        match section.ty {
            SectionTy::Type => {
                types = Some(wasm.read_type_section(section)?);
            }
            SectionTy::Function => {
                typeidxs = Some(wasm.read_function_section(section)?);
            }
            SectionTy::Export => {
                exports = Some(wasm.read_export_section(section)?);
            }
            SectionTy::Code => {
                codes = Some(wasm.read_code_section(section)?);
            }
            SectionTy::Custom => {
                wasm.read_custom_section(section)?;
            }
            _ => {
                todo!("validate sections for remaining section types")
            }
        }
    }

    info!("Validation was successful");
    Ok(ValidationInfo {})
}
