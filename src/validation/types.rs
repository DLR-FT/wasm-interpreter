//! # Decoding and Validation Logic for Types
//!
//! TODO write description for this module

use crate::{
    core::{
        decoding::decoder::WasmDecoder,
        structure::{
            modules::indices::{IdxVec, TypeIdx},
            types::BlockType,
        },
    },
    execution::assert_validated::UnwrapValidatedExt,
    DecodingError, FuncType, Limits, MemType, RefType, TableType, ValType, ValidationError,
};

impl BlockType {
    /// Decodes a block type[^binary-format] and validates it[^validation].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.4.1. Control Instructions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-blocktype).
    /// [^validation]: [WebAssembly Specification 2.0 - 3.2.2. Block Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#valid-blocktype).
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        if wasm.peek_u8()? == 0x40 {
            // Empty block type
            let _ = wasm.decode_u8().unwrap_validated();
            Ok(BlockType::Empty)
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::decode(wasm)) {
            // No parameters and given valtype as the result
            Ok(BlockType::Returns(val_ty))
        } else {
            // An index to a function type
            let index = wasm.decode_var_i33_as_u32()?;
            TypeIdx::validate(index, c_types).map(BlockType::Type)
        }
    }

    /// Converts this block type to a specific [`FuncType`].
    ///
    /// A vector of function types is required, in case the current block type
    /// stores a type index.
    ///
    /// # Safety
    ///
    /// The given [`IdxVec<TypeIdx, FuncType>`] must be the same on that was
    /// used to validate `self` through [`BlockType::decode_and_validate`].
    // TODO maybe make this function return a `Cow<'a, FuncType>`. This could
    // prevent one allocation per call.
    pub unsafe fn as_func_type(&self, func_types: &IdxVec<TypeIdx, FuncType>) -> FuncType {
        match self {
            BlockType::Empty => FuncType::new_empty(),
            BlockType::Returns(val_type) => FuncType::new_returning(*val_type),
            BlockType::Type(type_idx) => {
                // SAFETY: The caller ensures that this `IdxVec` is the same one
                // used to validate the `TypeIdx` in `self`.
                unsafe { func_types.get(*type_idx) }.clone()
            }
        }
    }
}

impl Limits {
    /// Decodes a limits object[^binary-format] and validates it[^validation].
    ///
    /// Note: This includes the decoding logic for shared memory types from the threads proposal.
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.7. Limits](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-limits).
    /// [^validation]: [WebAssembly Specification 2.0 - 3.2.1. Limits](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#valid-limits).
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        range: u32,
    ) -> Result<Self, ValidationError> {
        let limits = match wasm.decode_u8()? {
            0x00 => {
                let min = wasm.decode_var_u32()?;
                Self {
                    min,
                    max: None,
                    shared: false,
                }
            }
            0x01 => {
                let min = wasm.decode_var_u32()?;
                let max = wasm.decode_var_u32()?;
                Self {
                    min,
                    max: Some(max),
                    shared: false,
                }
            }
            0x03 => {
                let min = wasm.decode_var_u32()?;
                let max = wasm.decode_var_u32()?;
                Self {
                    min,
                    max: Some(max),
                    shared: true,
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
    /// Decodes a table type[^binary-format] and validates it[^validation].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.9. Table Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-tabletype).
    /// [^validation]: [WebAssembly Specification 2.0 - 3.2.4. Table Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#valid-tabletype).
    pub fn decode_and_validate(wasm: &mut WasmDecoder) -> Result<Self, ValidationError> {
        const LIMITS_RANGE: u32 = u32::MAX; // = 2^32 - 1

        let et = RefType::decode(wasm)?;
        let lim = Limits::decode_and_validate(wasm, LIMITS_RANGE)?;

        if lim.shared {
            return Err(DecodingError::SharedTablesNotYetImplemented.into());
        }

        Ok(Self { et, lim })
    }
}

impl MemType {
    /// Decodes a memory type[^binary-format] and validates it[^validation].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.8. Memory Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-memtype).
    /// [^validation]: [WebAssembly Specification 2.0 - 3.2.5. Memory Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#valid-memtype).
    pub fn decode_and_validate(wasm: &mut WasmDecoder) -> Result<Self, ValidationError> {
        const LIMITS_RANGE: u32 = 2_u32.pow(16);

        let limits = Limits::decode_and_validate(wasm, LIMITS_RANGE)?;

        Ok(Self { limits })
    }
}
