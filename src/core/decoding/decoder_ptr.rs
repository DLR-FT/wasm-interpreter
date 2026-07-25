use core::{hint::unreachable_unchecked, ptr::slice_from_raw_parts};

use crate::{
    core::{
        structure::types::{BlockType, MemArg, VecType},
        utils::ToUsizeExt,
    },
    trace, NumType, RefType, ValType,
};
use alloc::vec::Vec;

#[derive(Clone)]
pub struct WasmDecoderPtr(pub *const u8);

impl WasmDecoderPtr {
    pub const fn new(p: *const u8) -> Self {
        Self(p)
    }

    #[inline(always)]
    pub fn peek_u8(&self) -> u8 {
        unsafe { *self.0 }
    }

    pub fn strip_bytes<const N: usize>(&mut self) -> [u8; N] {
        let bytes = unsafe { &*slice_from_raw_parts(self.0, N) };
        self.0 = unsafe { self.0.wrapping_add(N) };
        unsafe { bytes.try_into().unwrap_unchecked() }
    }
}

/// Wasm encodes integers according to the LEB128 format, which specifies that
/// only 7 bits of every byte are used to store the integer's bits. The 8th bit
/// is always used as a bitflag for whether the next byte shall also be read as
/// part of the current integer. Therefore, it can be called a continuation bit,
/// which is stored here as a global constant to improve code readability.
const CONTINUATION_BIT: u8 = 0b10000000;

const INTEGER_BIT_FLAG: u8 = !CONTINUATION_BIT;

impl WasmDecoderPtr {
    #[inline(always)]
    pub fn decode_u8(&mut self) -> u8 {
        let byte = self.peek_u8();
        self.0 = unsafe { self.0.add(1) };
        byte
    }

    pub unsafe fn decode_u8_unchecked(&mut self) -> u8 {
        self.decode_u8()
    }

    /// Parses a variable-length `u32` as specified by [LEB128](https://en.wikipedia.org/wiki/LEB128#Unsigned_LEB128).
    /// Note: If `Err`, the [WasmDecoderPtr] object is no longer guaranteed to be in a valid state
    #[inline(always)]
    pub fn decode_var_u32(&mut self) -> u32 {
        /// Because up to 5 bytes (each storing 7 bits) may be used to store 32 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BIT_MASK: u8 = 0b01110000;

        let mut result: u32 = 0;

        let byte = self.decode_u8();
        result |= u32::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            return result;
        }

        let byte = self.decode_u8();
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            return result;
        }

        let byte = self.decode_u8();
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            return result;
        }

        let byte = self.decode_u8();
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            return result;
        }

        let byte = self.decode_u8();
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 28;

        result
    }

    #[inline(always)]
    pub unsafe fn decode_var_u32_branchless(&mut self, end: *const u8) -> u32 {
        if end as usize <= self.0 as usize {
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }

        let ptrs = [
            self.0,
            (self.0.wrapping_add(1) as usize).min(end as usize) as *const u8,
            (self.0.wrapping_add(2) as usize).min(end as usize) as *const u8,
            (self.0.wrapping_add(3) as usize).min(end as usize) as *const u8,
            (self.0.wrapping_add(4) as usize).min(end as usize) as *const u8,
        ];

        let byte1 = unsafe { *ptrs[0] };
        let byte2 = unsafe { *ptrs[1] };
        let byte3 = unsafe { *ptrs[2] };
        let byte4 = unsafe { *ptrs[3] };
        let byte5 = unsafe { *ptrs[4] };

        let bits1 = byte1 & 0x7F;
        let bits2 = byte2 & 0x7F;
        let bits3 = byte3 & 0x7F;
        let bits4 = byte4 & 0x7F;
        let bits5 = byte5 & 0x7F;

        let next_bit1_mask = (byte1.cast_signed() >> 7).cast_unsigned();
        let next_bit2_mask = (byte2.cast_signed() >> 7).cast_unsigned() & next_bit1_mask;
        let next_bit3_mask = (byte3.cast_signed() >> 7).cast_unsigned() & next_bit2_mask;
        let next_bit4_mask = (byte4.cast_signed() >> 7).cast_unsigned() & next_bit3_mask;

        let mut result = bits1 as u32;
        result |= ((bits2 & next_bit1_mask) as u32) << 7;
        result |= ((bits3 & next_bit2_mask) as u32) << 14;
        result |= ((bits4 & next_bit3_mask) as u32) << 21;
        result |= ((bits5 & next_bit4_mask) as u32) << 28;

        self.0 = self.0.wrapping_add(
            1 + (((next_bit1_mask > 0) as usize
                + (next_bit2_mask > 0) as usize
                + (next_bit3_mask > 0) as usize
                + (next_bit4_mask > 0) as usize)
                >> 7),
        );

        result
    }

    pub fn decode_f64_unchecked(&mut self) -> u64 {
        let bytes = self.strip_bytes::<8>();
        u64::from_le_bytes(bytes)
    }

    #[inline(always)]
    pub fn decode_var_i32(&mut self) -> i32 {
        /// Because up to 5 bytes (each storing 7 bits) may be used to store 32 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BITMASK: u8 = 0b01110000;

        /// This bitflag defines the position of the sign bit in the last byte.
        const SIGN_IN_LAST_BYTE_BITFLAG: u8 = 0b00001000;

        /// Number of bits in this number type
        const NUM_BITS: u32 = 32;

        let mut result: i32 = 0;

        let byte = self.decode_u8();
        result |= i32::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            /// before returning the result, we need to sign extend the unspecified bits
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 7;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 14;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 21;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 28;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 28;

        result
    }

    pub fn decode_var_i33_as_u32(&mut self) -> u32 {
        /// Because up to 5 bytes (each storing 7 bits) may be used to store 32 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BITMASK: u8 = 0b01100000;

        /// This bitflag defines the position of the sign bit in the last byte.
        const SIGN_IN_LAST_BYTE_BITFLAG: u8 = 0b00010000;

        /// Number of bits in this number type
        const NUM_BITS: u32 = 33;

        let mut result: i64 = 0;

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            /// before returning the result, we need to sign extend the unspecified bits
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 7;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return unsafe { u32::try_from(sign_extended_result).unwrap_unchecked() };
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 14;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return unsafe { u32::try_from(sign_extended_result).unwrap_unchecked() };
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 21;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return unsafe { u32::try_from(sign_extended_result).unwrap_unchecked() };
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 28;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return unsafe { u32::try_from(sign_extended_result).unwrap_unchecked() };
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 28;

        unsafe { u32::try_from(result).unwrap_unchecked() }
    }

    pub fn decode_f32(&mut self) -> u32 {
        let bytes = self.strip_bytes::<4>();
        u32::from_le_bytes(bytes)
    }

    pub fn decode_var_i64(&mut self) -> i64 {
        /// Because up to 10 bytes (each storing 7 bits) may be used to store 64 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BITMASK: u8 = 0b01111110;

        /// This bitflag defines the position of the sign bit in the last byte.
        const SIGN_IN_LAST_BYTE_BITFLAG: u8 = 0b00000001;

        /// Number of bits in this number type
        const NUM_BITS: u32 = 64;

        let mut result: i64 = 0;

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            /// before returning the result, we need to sign extend the unspecified bits
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 7;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 14;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 21;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 28;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 28;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 35;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 35;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 42;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 42;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 49;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 49;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 56;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 56;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 63;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return sign_extended_result;
        }

        let byte = self.decode_u8();
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 63;

        result
    }

    // TODO remove, see note on read_vec for more info
    pub fn decode_vec_enumerated<T, F>(&mut self, mut read_element: F) -> Vec<T>
    where
        F: FnMut(&mut WasmDecoderPtr, u32) -> T,
    {
        let mut idx = 0;
        self.decode_vec(|wasm| {
            let ret = read_element(wasm, idx);
            idx = idx
                .checked_add(1)
                .expect("the length of vectors to be encoded as a u32");
            ret
        })
    }

    /// Note: If `Err`, the [WasmDecoderPtr] object is no longer guaranteed to be in a valid state
    // TODO make this return `impl ExactSizeIterator<Item = T>` to prevent allocation. This will be
    // usedful if we want to decode some information on-demand in the future.
    pub fn decode_vec<T, F>(&mut self, mut read_element: F) -> Vec<T>
    where
        F: FnMut(&mut WasmDecoderPtr) -> T,
    {
        let len = self.decode_var_u32();
        core::iter::repeat_with(|| read_element(self))
            .take(len.into_usize())
            .collect()
    }
}

use crate::core::structure::modules::indices::{
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, Idx, LocalIdx, MemIdx, TableIdx, TypeIdx,
};

impl TypeIdx {
    /// Reads a type index from Wasm code without validating it. Using the
    /// returned type requires some other form of validation to be done.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid type index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        <Self as Idx>::new(index)
    }
}

impl FuncIdx {
    /// Reads a function index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid function index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        <Self as Idx>::new(index)
    }
}

impl TableIdx {
    /// Reads a table index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid table index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        <Self as Idx>::new(index)
    }
}

impl MemIdx {
    /// Reads a memory index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid memory index in the [`WasmDecoder`].
    #[allow(unused)] // reason = "unused until multiple memories proposal is implemented"
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        Self::new(index)
    }
}

impl GlobalIdx {
    /// Reads a global index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid global index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        <Self as Idx>::new(index)
    }
}

impl ElemIdx {
    /// Reads an element index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid element index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        <Self as Idx>::new(index)
    }
}

impl DataIdx {
    /// Reads a data index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid data index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        <Self as Idx>::new(index)
    }
}

impl LocalIdx {
    /// Reads a local index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid local index in the [`WasmDecoder`].
    #[inline(always)]
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let index = wasm.decode_var_u32();
        Self(index)
    }
}

/// Reads a label index from Wasm code without validating it.
///
/// # Safety
///
/// The caller must ensure that there is a valid label index in the [`WasmDecoder`].
pub unsafe fn decode_label_idx_unchecked(wasm: &mut WasmDecoderPtr) -> u32 {
    // TODO use `unwrap_unchecked` instead
    wasm.decode_var_u32()
}
impl NumType {
    /// Decodes a number type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.1. Number Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-numtype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let ty = match wasm.peek_u8() {
            0x7F => Self::I32,
            0x7E => Self::I64,
            0x7D => Self::F32,
            0x7C => Self::F64,
            _ => unsafe { unreachable_unchecked() },
        };
        let _ = wasm.decode_u8();

        ty
    }
}

impl VecType {
    /// Decodes a vector type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.2. Vector Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-vectype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    fn decode_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        match wasm.peek_u8() {
            0x7b => {
                let _ = wasm.decode_u8();
                VecType
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl RefType {
    /// Decodes a reference type[^binary-format] which is always valid[^always-valid].
    ///
    /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.3. Reference Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-reftype).
    /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
    pub fn decode_ptr(wasm: &mut WasmDecoderPtr) -> RefType {
        let ty = match wasm.peek_u8() {
            0x70 => RefType::FuncRef,
            0x6F => RefType::ExternRef,
            _ => unsafe { unreachable_unchecked() },
        };
        let _ = wasm.decode_u8();

        ty
    }
}

// impl ValType {
//     /// Decodes a value type[^binary-format] which is always valid[^always-valid].
//     ///
//     /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.4. Value Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-valtype).
//     /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
//     pub fn decode(wasm: &mut WasmDecoderPtr) -> Result<Self, DecodingError> {

//         if let Ok(numtype) = NumType::decode(wasm).map(ValType::NumType) {
//             return Ok(numtype);
//         };
//         if let Ok(vectype) = VecType::decode(wasm).map(|_ty| ValType::VecType) {
//             return Ok(vectype);
//         };
//         if let Ok(reftype) = RefType::decode(wasm).map(ValType::RefType) {
//             return Ok(reftype);
//         }

//         Err(DecodingError::MalformedValType)
//     }
// }

// impl ResultType {
//     /// Decodes a result type[^binary-format] which is always valid[^always-valid].
//     ///
//     /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.5. Result Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-resulttype).
//     /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
//     pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
//         let valtypes = wasm.decode_vec(ValType::decode)?;

//         Ok(ResultType { valtypes })
//     }
// }

// impl FuncType {
//     /// Decodes a function type[^binary-format] which is always valid[^always-valid].
//     ///
//     /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.6. Function Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-functype).
//     /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
//     pub fn decode(wasm: &mut WasmDecoder) -> Result<FuncType, DecodingError> {
//         match wasm.decode_u8()? {
//             0x60 => {}
//             other => return Err(DecodingError::MalformedFuncTypeDiscriminator(other)),
//         };

//         let params = ResultType::decode(wasm)?;
//         let returns = ResultType::decode(wasm)?;

//         Ok(FuncType { params, returns })
//     }
// }

impl BlockType {
    /// Decodes a block type that is assumed to be valid.
    ///
    /// See: [WebAssembly Specification 2.0 - 5.4.1. Control Instructions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#control-instructions%E2%91%A6).
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid block type to be read in
    /// the given [`WasmDecoder`].
    pub unsafe fn decode_unchecked_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        match wasm.peek_u8() {
            0x40 => {
                let _ = wasm.decode_u8();
                BlockType::Empty
            }
            0x7B => {
                let _ = wasm.decode_u8();
                BlockType::Returns(ValType::VecType)
            }
            0x7F => {
                let _ = wasm.decode_u8();
                BlockType::Returns(ValType::NumType(NumType::I32))
            }

            0x7E => {
                let _ = wasm.decode_u8();
                BlockType::Returns(ValType::NumType(NumType::I64))
            }

            0x7D => {
                let _ = wasm.decode_u8();
                BlockType::Returns(ValType::NumType(NumType::F32))
            }
            0x7C => {
                let _ = wasm.decode_u8();
                BlockType::Returns(ValType::NumType(NumType::F64))
            }
            0x70 => {
                let _ = wasm.decode_u8();
                BlockType::Returns(ValType::RefType(RefType::FuncRef))
            }
            0x6B => {
                let _ = wasm.decode_u8();
                BlockType::Returns(ValType::RefType(RefType::ExternRef))
            }

            _ => BlockType::Type(TypeIdx::new(wasm.decode_var_i33_as_u32())),
        }
    }
}

// impl GlobalType {
//     /// Decodes a global type[^binary-format] which is always valid[^always-valid].
//     ///
//     /// [^binary-format]: [WebAssembly Specification 2.0 - 5.3.10. Global Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#binary-globaltype).
//     /// [^always-valid]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
//     pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
//         let ty = ValType::decode(wasm)?;
//         let is_mut = match wasm.decode_u8()? {
//             0x00 => false,
//             0x01 => true,
//             other => return Err(DecodingError::MalformedMutDiscriminator(other)),
//         };
//         Ok(Self { ty, is_mut })
//     }
// }

impl MemArg {
    /// Decodes a memarg
    ///
    /// See: WebAssembly Specification 2.0 - 5.4.6 - Memory Instructions
    #[inline(always)]
    pub fn decode_ptr(wasm: &mut WasmDecoderPtr) -> Self {
        let align = wasm.decode_var_u32();
        let offset = wasm.decode_var_u32();
        Self { offset, align }
    }
}
