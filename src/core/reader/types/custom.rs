use crate::{
    core::reader::{
        section_header::{SectionHeader, SectionTy},
        WasmReadable, WasmReader,
    },
    Error, Result,
};
use alloc::{string::*, vec::Vec};

pub fn handle_custom_section(wasm: &mut WasmReader, h: SectionHeader) -> Result<()> {
    // customsec ::= section_0(custom)
    // custom ::= name byte*
    // name ::= b*:vec(byte) => name (if utf8(name) = b*)
    // vec(B) ::= n:u32 (x:B)^n => x^n

    if h.contents.len() == 0 {
        return Ok(());
    }

    let name = wasm.read_name()?.to_string();
    trace!("Custom section. Name: '{}'", name);

    let remaining_bytes = match h
        .contents
        .from()
        .checked_add(h.contents.len())
        .and_then(|res| res.checked_sub(wasm.pc.clone()))
    {
        None => Err(Error::InvalidSection(
            SectionTy::Custom,
            "Remaining bytes less than 0 after reading name!".to_string(),
        )),
        Some(remaining_bytes) => Ok(remaining_bytes),
    }?;

    match name.as_str() {
        "name" => {
            // Name Section - https://webassembly.github.io/spec/core/appendix/custom.html#name-section
            if remaining_bytes == 0 {
                return Ok(());
            }
            let mut remaining_bytes = remaining_bytes;
            let end = wasm.pc + remaining_bytes;

            let mut i: u32 = 0;
            let mut previous_subsection_type: u32 = 0;

            while wasm.pc != end {
                let name_type = wasm.read_var_u32()?;
                // we need to check for duplicate or out-of-order sub-section
                if i != 0 {
                    if name_type == previous_subsection_type {
                        return Err(Error::InvalidSection(
                            SectionTy::Custom,
                            "Duplicate Subsection".to_string(),
                        ));
                    }
                    if name_type < previous_subsection_type {
                        return Err(Error::InvalidSection(
                            SectionTy::Custom,
                            "Out-of-order Subsection".to_string(),
                        ));
                    }
                }
                previous_subsection_type = name_type;

                let subsection_size = wasm.read_var_u32()? as usize;
                let subsection_end = wasm.pc + subsection_size;

                if subsection_end > end {
                    return Err(Error::InvalidSection(
                        SectionTy::Custom,
                        "Subsection size is past the end of the section".to_string(),
                    ));
                }

                let ty = NameSectionSubsection::from(name_type as usize);

                match ty {
                    _ => todo!(),
                }
            }

            Ok(())
        }
        _ => {
            // Unknown Section
            // TODO: maybe do something with these remaining bytes instead of skipping them?
            wasm.skip(remaining_bytes)
        }
    }
}

pub enum SubsectionTy {
    ModuleName = 0,
    FunctionNames = 1,
    LocalNames = 2,
}

pub struct NameAssoc {
    idx: usize,
    name: String,
}
pub struct NameMap(Vec<NameAssoc>);

pub struct IndirectNameAssoc<'a> {
    idx: usize,
    // might be worth making this a usize, as well, and having an index space for them?
    namemap: &'a NameMap,
}

pub struct IndirectNameMap<'a>(Vec<IndirectNameAssoc<'a>>);
pub struct ModuleNameSubsec(String);
pub struct FuncNameSubsec(NameMap);
pub struct LocalNameSubsec<'a>(IndirectNameMap<'a>);

pub enum NameSectionSubsection {
    Module = 0,
    Function = 1,
    Local = 2,
    Label = 3,
    Type = 4,
    Table = 5,
    Memory = 6,
    Global = 7,
    ElemSegment = 8,
    DataSegment = 9,
    Tag = 10,

    Unknown,
}

impl From<usize> for NameSectionSubsection {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Module,
            1 => Self::Function,
            2 => Self::Local,
            3 => Self::Label,
            4 => Self::Type,
            5 => Self::Table,
            6 => Self::Memory,
            7 => Self::Global,
            8 => Self::ElemSegment,
            9 => Self::DataSegment,
            10 => Self::Tag,
            _ => Self::Unknown,
        }
    }
}
