use core::fmt::Debug;

use crate::{core::reader::WasmReader, ValidationError};

#[derive(Debug)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}

impl MemArg {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let align = wasm.read_var_u32()?;
        let offset = match wasm.read_var_u32() {
            Ok(offset) => offset,
            Err(ValidationError::VariableLengthIntegerOverflowed) => {
                return Err(ValidationError::MemArgOffsetOverflowed)
            }
            Err(other) => return Err(other),
        };

        // The specification does not include this requirement and this check is
        // practically irrelevant, because all aligns >= 32 are caught during
        // validation when they are used together with some instruction.
        //
        // However, the reference interpreter contains this check and it is
        // tested for in the spectests.
        if align >= 32 {
            return Err(ValidationError::MalformedMemArgFlags);
        }

        Ok(Self { offset, align })
    }
}
