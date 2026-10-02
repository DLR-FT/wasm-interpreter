//! Methods to read basic WASM Values from a [WasmDecoder] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/values.html>
//!
//! Note: If any of these methods return `Err`, they may have consumed some bytes from the [WasmDecoder] object and thus consequent calls may result in unexpected behaviour.
//! This is due to the fact that these methods read elemental types which cannot be split.

use core::{convert::Infallible, iter};

use alloc::boxed::Box;

use crate::{
    core::{
        decoding::decoder::WasmDecoder, fixed_capacity_vec::FixedCapacityVec, utils::ToUsizeExt,
    },
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
    pub fn decode_u8(&mut self) -> Result<u8, DecodingError> {
        let byte = self.peek_u8()?;
        self.pc += 1;
        Ok(byte)
    }

    /// Parses a variable-length `u32` as specified by [LEB128](https://en.wikipedia.org/wiki/LEB128#Unsigned_LEB128).
    /// Note: If `Err`, the [WasmDecoder] object is no longer guaranteed to be in a valid state
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

    /// Starts decoding a vector. The returned [`DecodeVec`] object can be used to further read and
    /// map vector elements.
    pub fn decode_vec<'a>(&'a mut self) -> Result<DecodeVec<'a, 'wasm>, DecodingError> {
        let len = self.decode_var_u32()?;
        Ok(DecodeVec { len, decoder: self })
    }
}

pub struct DecodeVec<'a, 'wasm> {
    len: u32,
    decoder: &'a mut WasmDecoder<'wasm>,
}

/// With the trusted_len feature, we could provide individual methods like `enumerate`, `map` and
/// `collect`. But instead we have to merge them so that we always know the exact length to be
/// relied on by unsafe code.
impl<'a, 'wasm> DecodeVec<'a, 'wasm> {
    pub fn enumerate_and_map_and_collect<F, T, E>(
        mut self,
        mut read_and_map_element: F,
    ) -> Result<Box<[T]>, E>
    where
        F: FnMut(&mut WasmDecoder<'wasm>, u32) -> Result<T, E>,
        T: 'wasm,
    {
        let len = self.len;
        let mut elements = FixedCapacityVec::with_capacity(len.into_usize());

        for i in 0..len {
            let next_element = read_and_map_element(&mut self.decoder, i)?;

            debug_assert!(!elements.is_full());
            // SAFETY: The vector has the capacity `len` and the for loop only iterates up to `len`
            // times, so there is always space.
            unsafe { elements.push_unchecked(next_element) };
        }

        // SAFETY: `len` elements were pushed, therefore the vector with capacity `len` must be
        // full.
        let elements = unsafe { elements.into_boxed_slice() };

        Ok(elements)
    }

    pub fn map_and_collect<F, T, E>(self, mut read_and_map_element: F) -> Result<Box<[T]>, E>
    where
        F: FnMut(&mut WasmDecoder<'wasm>) -> Result<T, E>,
        T: 'wasm,
    {
        self.enumerate_and_map_and_collect(|wasm, _index| read_and_map_element(wasm))
    }

    pub fn flat_map<F, U, T>(
        self,
        mut read_and_map_element: F,
    ) -> impl Iterator<Item = T> + use<'a, 'wasm, F, U, T>
    where
        F: FnMut(&mut WasmDecoder<'wasm>) -> U,
        U: IntoIterator<Item = T>,
        T: 'wasm,
    {
        let Self {
            len: mut num_element_left,
            decoder,
        } = self;

        let mut current_iterator: Option<U::IntoIter> = None;

        iter::from_fn(move || {
            loop {
                if let Some(current_iterator) = &mut current_iterator {
                    if let Some(next_element) = current_iterator.next() {
                        return Some(next_element);
                    }
                }

                if num_element_left == 0 {
                    return None;
                }

                // If we get to this point, there is no current iterator or it is empty and there
                // are still elements in this vector left to read.
                num_element_left -= 1;
                let next_iterator = read_and_map_element(decoder).into_iter();
                current_iterator = Some(next_iterator);
            }
        })
    }

    pub fn for_each<F>(self, mut runner: F)
    where
        F: FnMut(&mut WasmDecoder<'wasm>),
    {
        let result: Result<(), Infallible> = self.try_for_each(|wasm| Ok(runner(wasm)));

        debug_assert!(result.is_ok());
        // SAFETY: Error type is infallible
        unsafe { result.unwrap_unchecked() }
    }

    pub fn try_for_each<F, E>(self, mut runner: F) -> Result<(), E>
    where
        F: FnMut(&mut WasmDecoder<'wasm>) -> Result<(), E>,
    {
        for _ in 0..self.len {
            runner(self.decoder)?;
        }

        Ok(())
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
