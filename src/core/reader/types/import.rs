use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::core::indices::TypeIdx;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::{unreachable_validated, Error, Result};

use super::global::GlobalType;
use super::{MemType, TableType};

#[derive(Debug, Clone)]
pub struct Import {
    #[allow(warnings)]
    pub module_name: String,
    #[allow(warnings)]
    pub name: String,
    #[allow(warnings)]
    pub desc: ImportDesc,
}

impl WasmReadable for Import {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let module_name = wasm.read_name()?.to_owned();
        let name = wasm.read_name()?.to_owned();
        let desc = ImportDesc::read(wasm)?;

        Ok(Self {
            module_name,
            name,
            desc,
        })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let module_name = wasm.read_name().unwrap_validated().to_owned();
        let name = wasm.read_name().unwrap_validated().to_owned();
        let desc = ImportDesc::read_unvalidated(wasm);

        Self {
            module_name,
            name,
            desc,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl WasmReadable for ImportDesc {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let desc = match wasm.read_u8()? {
            0x00 => Self::Func(wasm.read_var_u32()? as TypeIdx),
            // https://webassembly.github.io/spec/core/binary/types.html#table-types
            0x01 => Self::Table(TableType::read(wasm)?),
            0x02 => Self::Mem(MemType::read(wasm)?),
            0x03 => Self::Global(GlobalType::read(wasm)?),
            other => return Err(Error::InvalidImportDesc(other)),
        };

        Ok(desc)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        match wasm.read_u8().unwrap_validated() {
            0x00 => Self::Func(wasm.read_var_u32().unwrap_validated() as TypeIdx),
            0x01 => Self::Table(TableType::read_unvalidated(wasm)),
            0x02 => todo!("read MemType"),
            0x03 => todo!("read GlobalType"),
            _ => unreachable_validated!(),
        }
    }
}
