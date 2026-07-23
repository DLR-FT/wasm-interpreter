//! Methods to read basic WASM Values from a [WasmDecoder] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/values.html>
//!
//! Note: If any of these methods return `Err`, they may have consumed some bytes from the [WasmDecoder] object and thus consequent calls may result in unexpected behaviour.
//! This is due to the fact that these methods read elemental types which cannot be split.

use alloc::vec::Vec;

use crate::{
    core::{decoding::decoder::WasmDecoder, utils::ToUsizeExt},
    DecodingError,
};

/// Wasm encodes integers according to the LEB128 format, which specifies that
/// only 7 bits of every byte are used to store the integer's bits. The 8th bit
/// is always used as a bitflag for whether the next byte shall also be read as
/// part of the current integer. Therefore, it can be called a continuation bit,
/// which is stored here as a global constant to improve code readability.
const CONTINUATION_BIT: u8 = 0b10000000;

const INTEGER_BIT_FLAG: u8 = !CONTINUATION_BIT;

impl<'wasm> WasmDecoder<'wasm> {
    /// Tries to read one byte and fails if the end of file is reached.
    #[inline(always)]
    pub fn decode_u8(&mut self) -> Result<u8, DecodingError> {
        let byte = self.peek_u8()?;
        self.pc += 1;
        Ok(byte)
    }

    #[inline(always)]
    pub unsafe fn decode_u8_unchecked(&mut self) -> u8 {
        let byte = unsafe { self.peek_u8_unchecked() };
        self.pc += 1;
        byte
    }

    /// Parses a variable-length `u32` as specified by [LEB128](https://en.wikipedia.org/wiki/LEB128#Unsigned_LEB128).
    /// Note: If `Err`, the [WasmDecoder] object is no longer guaranteed to be in a valid state
    #[inline(always)]
    pub fn decode_var_u32(&mut self) -> Result<u32, DecodingError> {
        /// Because up to 5 bytes (each storing 7 bits) may be used to store 32 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BIT_MASK: u8 = 0b01110000;

        let mut result: u32 = 0;

        let byte = self.decode_u8()?;
        result |= u32::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            return Ok(result);
        }

        let byte = self.decode_u8()?;
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            return Ok(result);
        }

        let byte = self.decode_u8()?;
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            return Ok(result);
        }

        let byte = self.decode_u8()?;
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            return Ok(result);
        }

        let byte = self.decode_u8()?;
        result |= u32::from(byte & INTEGER_BIT_FLAG) << 28;

        // there can only be a maximum number of 5 bytes for a 32-bit integer
        let has_next_byte = byte & CONTINUATION_BIT > 0;
        let padding_bits_are_not_zero = byte & PADDING_IN_LAST_BYTE_BIT_MASK > 0;
        if has_next_byte || padding_bits_are_not_zero {
            // TODO distinguish between both error variants
            return Err(DecodingError::MalformedVariableLengthInteger);
        }

        Ok(result)
    }

    pub fn decode_f64(&mut self) -> Result<u64, DecodingError> {
        let bytes = self.strip_bytes::<8>()?;
        Ok(u64::from_le_bytes(bytes))
    }

    #[inline(always)]
    pub fn decode_var_i32(&mut self) -> Result<i32, DecodingError> {
        /// Because up to 5 bytes (each storing 7 bits) may be used to store 32 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BITMASK: u8 = 0b01110000;

        /// This bitflag defines the position of the sign bit in the last byte.
        const SIGN_IN_LAST_BYTE_BITFLAG: u8 = 0b00001000;

        /// Number of bits in this number type
        const NUM_BITS: u32 = 32;

        let mut result: i32 = 0;

        let byte = self.decode_u8()?;
        result |= i32::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            /// before returning the result, we need to sign extend the unspecified bits
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 7;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 14;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 21;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 28;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i32::from(byte & INTEGER_BIT_FLAG) << 28;

        // there can only be a maximum number of 5 bytes for a 32-bit integer
        let has_next_byte = byte & CONTINUATION_BIT > 0;
        if has_next_byte {
            // TODO distinguish between both error variants
            return Err(DecodingError::MalformedVariableLengthInteger);
        }

        // Verify that the padding and sign bits are either all ones or all
        // zeros. To do this we count the ones and check if that number is zero
        // or equal to the number of ones in both bitmasks combined.
        const PADDING_AND_SIGN_BITMASK: u8 =
            PADDING_IN_LAST_BYTE_BITMASK | SIGN_IN_LAST_BYTE_BITFLAG;
        let number_of_ones_in_padding_and_sign_bits =
            (byte & PADDING_AND_SIGN_BITMASK).count_ones();
        let padding_bits_match_sign_bit = number_of_ones_in_padding_and_sign_bits
            == PADDING_AND_SIGN_BITMASK.count_ones()
            || number_of_ones_in_padding_and_sign_bits == 0;
        if !padding_bits_match_sign_bit {
            // TODO distinguish between both error variants
            return Err(DecodingError::MalformedVariableLengthInteger);
        }

        Ok(result)
    }

    pub fn decode_var_i33_as_u32(&mut self) -> Result<u32, DecodingError> {
        /// Because up to 5 bytes (each storing 7 bits) may be used to store 32 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BITMASK: u8 = 0b01100000;

        /// This bitflag defines the position of the sign bit in the last byte.
        const SIGN_IN_LAST_BYTE_BITFLAG: u8 = 0b00010000;

        /// Number of bits in this number type
        const NUM_BITS: u32 = 33;

        let mut result: i64 = 0;

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            /// before returning the result, we need to sign extend the unspecified bits
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 7;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return u32::try_from(sign_extended_result).map_err(|_| DecodingError::I33IsNegative);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 14;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return u32::try_from(sign_extended_result).map_err(|_| DecodingError::I33IsNegative);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 21;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return u32::try_from(sign_extended_result).map_err(|_| DecodingError::I33IsNegative);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 28;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return u32::try_from(sign_extended_result).map_err(|_| DecodingError::I33IsNegative);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 28;

        // there can only be a maximum number of 5 bytes for a 33-bit integer
        let has_next_byte = byte & CONTINUATION_BIT > 0;
        if has_next_byte {
            // TODO distinguish between both error variants
            return Err(DecodingError::MalformedVariableLengthInteger);
        }

        // Verify that the padding and sign bits are either all ones or all
        // zeros. To do this we count the ones and check if that number is zero
        // or equal to the number of ones in both bitmasks combined.
        const PADDING_AND_SIGN_BITMASK: u8 =
            PADDING_IN_LAST_BYTE_BITMASK | SIGN_IN_LAST_BYTE_BITFLAG;
        let number_of_ones_in_padding_and_sign_bits =
            (byte & PADDING_AND_SIGN_BITMASK).count_ones();
        let padding_bits_match_sign_bit = number_of_ones_in_padding_and_sign_bits
            == PADDING_AND_SIGN_BITMASK.count_ones()
            || number_of_ones_in_padding_and_sign_bits == 0;
        if !padding_bits_match_sign_bit {
            // TODO distinguish between both error variants
            return Err(DecodingError::MalformedVariableLengthInteger);
        }

        u32::try_from(result).map_err(|_| DecodingError::I33IsNegative)
    }

    pub fn decode_f32(&mut self) -> Result<u32, DecodingError> {
        let bytes = self.strip_bytes::<4>()?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn decode_var_i64(&mut self) -> Result<i64, DecodingError> {
        /// Because up to 10 bytes (each storing 7 bits) may be used to store 64 bits,
        /// some bits in the last byte will be left unused. This is a bitmask for
        /// exactly these bits in the last byte.
        const PADDING_IN_LAST_BYTE_BITMASK: u8 = 0b01111110;

        /// This bitflag defines the position of the sign bit in the last byte.
        const SIGN_IN_LAST_BYTE_BITFLAG: u8 = 0b00000001;

        /// Number of bits in this number type
        const NUM_BITS: u32 = 64;

        let mut result: i64 = 0;

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG);
        if byte & CONTINUATION_BIT == 0 {
            /// before returning the result, we need to sign extend the unspecified bits
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 7;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 7;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 14;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 14;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 21;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 21;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 28;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 28;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 35;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 35;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 42;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 42;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 49;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 49;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 56;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 56;
        if byte & CONTINUATION_BIT == 0 {
            const NUM_UNSPECIFIED_BITS: u32 = NUM_BITS - 63;
            let sign_extended_result = (result << NUM_UNSPECIFIED_BITS) >> NUM_UNSPECIFIED_BITS;
            return Ok(sign_extended_result);
        }

        let byte = self.decode_u8()?;
        result |= i64::from(byte & INTEGER_BIT_FLAG) << 63;

        // there can only be a maximum number of 10 bytes for a 64-bit integer
        let has_next_byte = byte & CONTINUATION_BIT > 0;
        if has_next_byte {
            // TODO distinguish between both error variants
            return Err(DecodingError::MalformedVariableLengthInteger);
        }

        // Verify that the padding and sign bits are either all ones or all
        // zeros. To do this we count the ones and check if that number is zero
        // or equal to the number of ones in both bitmasks combined.
        const PADDING_AND_SIGN_BITMASK: u8 =
            PADDING_IN_LAST_BYTE_BITMASK | SIGN_IN_LAST_BYTE_BITFLAG;
        let number_of_ones_in_padding_and_sign_bits =
            (byte & PADDING_AND_SIGN_BITMASK).count_ones();
        let padding_bits_match_sign_bit = number_of_ones_in_padding_and_sign_bits
            == PADDING_AND_SIGN_BITMASK.count_ones()
            || number_of_ones_in_padding_and_sign_bits == 0;
        if !padding_bits_match_sign_bit {
            // TODO distinguish between both error variants
            return Err(DecodingError::MalformedVariableLengthInteger);
        }

        Ok(result)
    }

    /// Note: If `Err`, the [WasmDecoder] object is no longer guaranteed to be in a valid state
    pub fn decode_name(&mut self) -> Result<&'wasm str, DecodingError> {
        let len = self.decode_var_u32()?.into_usize();

        let utf8_str = &self
            .full_wasm_binary
            .get(self.pc..(self.pc + len))
            .ok_or(DecodingError::Eof)?;

        self.pc += len;

        core::str::from_utf8(utf8_str).map_err(DecodingError::MalformedUtf8)
    }

    // TODO remove, see note on read_vec for more info
    pub fn decode_vec_enumerated<T, F, E>(&mut self, mut read_element: F) -> Result<Vec<T>, E>
    where
        T: 'wasm,
        F: FnMut(&mut WasmDecoder<'wasm>, u32) -> Result<T, E>,
        E: From<DecodingError>,
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

    /// Note: If `Err`, the [WasmDecoder] object is no longer guaranteed to be in a valid state
    // TODO make this return `impl ExactSizeIterator<Item = T>` to prevent allocation. This will be
    // usedful if we want to decode some information on-demand in the future.
    pub fn decode_vec<T, F, E>(&mut self, mut read_element: F) -> Result<Vec<T>, E>
    where
        T: 'wasm,
        F: FnMut(&mut WasmDecoder<'wasm>) -> Result<T, E>,
        E: From<DecodingError>,
    {
        let len = self.decode_var_u32()?;
        core::iter::repeat_with(|| read_element(self))
            .take(len.into_usize())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::decoding::decoder::WasmDecoder;

    #[test]
    fn test_var_i32() {
        let bytes = [0xC0, 0xBB, 0x78];
        let mut wasm = WasmDecoder::new(&bytes);

        assert_eq!(wasm.decode_var_i32(), Ok(-123456));
    }
}
