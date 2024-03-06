use crate::core::indices::TypeIdx;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use crate::{unreachable_validated, Error, Result};
use alloc::borrow::ToOwned;
use alloc::string::String;

pub struct Import {
    pub module_name: String,
    pub name: String,
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

pub enum ImportDesc {
    Func(TypeIdx),
    Table(()),  // TODO TableType
    Mem(()),    // TODO MemType
    Global(()), // TODO GlobalType
}

impl WasmReadable for ImportDesc {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let desc = match wasm.read_u8()? {
            0x00 => Self::Func(wasm.read_var_u32()? as TypeIdx),
            0x01 => todo!("read TableType"),
            0x02 => todo!("read MemType"),
            0x03 => todo!("read GlobalType"),
            other => return Err(Error::InvalidImportDesc(other)),
        };

        Ok(desc)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        match wasm.read_u8().unwrap_validated() {
            0x00 => Self::Func(wasm.read_var_u32().unwrap_validated() as TypeIdx),
            0x01 => todo!("read TableType"),
            0x02 => todo!("read MemType"),
            0x03 => todo!("read GlobalType"),
            _ => unreachable_validated!(),
        }
    }
}
