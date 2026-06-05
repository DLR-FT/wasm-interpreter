use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        decoding::reader::WasmReader,
        structure::{
            modules::indices::{IdxVec, TypeIdx},
            types::BlockType,
        },
    },
    DecodingError, FuncType, Limits, MemType, RefType, TableType, ValType, ValidationError,
};

impl BlockType {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        if wasm.peek_u8()? == 0x40 {
            // Empty block type
            let _ = wasm.read_u8().unwrap_validated();
            Ok(BlockType::Empty)
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::read(wasm)) {
            // No parameters and given valtype as the result
            Ok(BlockType::Returns(val_ty))
        } else {
            // An index to a function type
            let index = wasm.read_var_i33_as_u32()?;
            TypeIdx::validate(index, c_types).map(BlockType::Type)
        }
    }
}

impl Limits {
    pub fn read_and_validate(wasm: &mut WasmReader, range: u32) -> Result<Self, ValidationError> {
        let limits = match wasm.read_u8()? {
            0x00 => {
                let min = wasm.read_var_u32()?;
                Self { min, max: None }
            }
            0x01 => {
                let min = wasm.read_var_u32()?;
                let max = wasm.read_var_u32()?;
                Self {
                    min,
                    max: Some(max),
                }
            }
            other => return Err(DecodingError::MalformedLimitsDiscriminator(other).into()),
        };

        if limits.min > range {
            return Err(ValidationError::LimitsNotWithinRange(range));
        }

        if let Some(max) = limits.max {
            if max > range {
                return Err(ValidationError::LimitsNotWithinRange(range));
            }

            if limits.min > max {
                return Err(ValidationError::LimitsMinLargerThanMax {
                    min: limits.min,
                    max,
                });
            }
        }

        Ok(limits)
    }
}

impl TableType {
    pub fn read_and_validate(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        const LIMITS_RANGE: u32 = u32::MAX; // = 2^32 - 1

        let et = RefType::read(wasm)?;
        let lim = Limits::read_and_validate(wasm, LIMITS_RANGE)?;

        Ok(Self { et, lim })
    }
}

impl MemType {
    pub fn read_and_validate(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        const LIMITS_RANGE: u32 = 2_u32.pow(16);

        let limits = Limits::read_and_validate(wasm, LIMITS_RANGE)?;

        Ok(Self { limits })
    }
}
