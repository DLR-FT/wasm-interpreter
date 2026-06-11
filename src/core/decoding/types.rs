//! # Decoding for Universally-Valid Types
//!
//! This module contains two types of decoding logic as defined in [^binary-format]. One is safe
//! decoding logic for those types that are universally-valid[^validation] and the other is unsafe
//! decoding logic for types that have already been validated.
//!
//! [^validation]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
//! [^binary-format]: [WebAssembly Specification 2.0 - 5.3. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A6).

use alloc::vec::Vec;

use crate::{
    core::{
        decoding::decoder::WasmDecoder,
        structure::{
            modules::indices::TypeIdx,
            types::{
                BlockType, FuncType, GlobalType, MemArg, NumType, RefType, ResultType, ValType,
                VecType,
            },
        },
    },
    DecodingError,
};

impl NumType {
    /// Decodes a number type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.1. Number Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-numtype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        use NumType::*;

        let ty = match wasm.peek_u8()? {
            0x7F => I32,
            0x7E => I64,
            0x7D => F32,
            0x7C => F64,
            other => return Err(DecodingError::MalformedNumTypeDiscriminator(other)),
        };
        let _ = wasm.decode_u8();

        Ok(ty)
    }
}

impl VecType {
    /// Decodes a vector type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.2. Vector Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-vectype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        match wasm.peek_u8()? {
            0x7b => {
                let _ = wasm.decode_u8();
                Ok(VecType)
            }
            other => Err(DecodingError::MalformedVecTypeDiscriminator(other)),
        }
    }
}

impl RefType {
    /// Decodes a reference type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.3. Reference Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-reftype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode(wasm: &mut WasmDecoder) -> Result<RefType, DecodingError> {
        let ty = match wasm.peek_u8()? {
            0x70 => RefType::FuncRef,
            0x6F => RefType::ExternRef,
            other => return Err(DecodingError::MalformedRefTypeDiscriminator(other)),
        };
        let _ = wasm.decode_u8();

        Ok(ty)
    }
}

impl ValType {
    /// Decodes a value type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.4. Value Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-valtype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        if let Ok(numtype) = NumType::decode(wasm).map(ValType::NumType) {
            return Ok(numtype);
        };
        if let Ok(vectype) = VecType::decode(wasm).map(|_ty| ValType::VecType) {
            return Ok(vectype);
        };
        if let Ok(reftype) = RefType::decode(wasm).map(ValType::RefType) {
            return Ok(reftype);
        }

        Err(DecodingError::MalformedValType)
    }
}

impl ResultType {
    /// Decodes a result type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.5. Result Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-resulttype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        let valtypes = wasm
            .decode_vec_map(ValType::decode)?
            .collect::<Result<Vec<_>, DecodingError>>()?;

        Ok(ResultType { valtypes })
    }
}

impl FuncType {
    /// Decodes a function type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.6. Function Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-functype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode(wasm: &mut WasmDecoder) -> Result<FuncType, DecodingError> {
        match wasm.decode_u8()? {
            0x60 => {}
            other => return Err(DecodingError::MalformedFuncTypeDiscriminator(other)),
        };

        let params = ResultType::decode(wasm)?;
        let returns = ResultType::decode(wasm)?;

        Ok(FuncType { params, returns })
    }
}

impl BlockType {
    /// Decodes a block type that is assumed to be valid.
    ///
    /// See: [WebAssembly Specification 2.0 - 5.4.1. Control Instructions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#control-instructions%E2%91%A6).
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid block type to be read in
    /// the given [`WasmDecoder`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        if wasm.peek_u8().unwrap() as i8 == 0x40 {
            // Empty block type
            let _ = wasm.decode_u8().unwrap();
            BlockType::Empty
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::decode(wasm)) {
            // No parameters and given valtype as the result
            BlockType::Returns(val_ty)
        } else {
            // An index to a function type
            let index = wasm.decode_var_i33_as_u32().unwrap();
            BlockType::Type(TypeIdx::new(index))
        }
    }
}

impl GlobalType {
    /// Decodes a global type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.10. Global Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-globaltype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        let ty = ValType::decode(wasm)?;
        let is_mut = match wasm.decode_u8()? {
            0x00 => false,
            0x01 => true,
            other => return Err(DecodingError::MalformedMutDiscriminator(other)),
        };
        Ok(Self { ty, is_mut })
    }
}

impl MemArg {
    /// Decodes a memarg
    ///
    /// See: WebAssembly Specification 2.0 - 5.4.6 - Memory Instructions
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        let align = wasm.decode_var_u32()?;
        let offset = wasm.decode_var_u32()?;
        Ok(Self { offset, align })
    }
}
