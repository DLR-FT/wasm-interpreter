/*
# This file incorporates code from Wasmtime, originally
# available at https://github.com/bytecodealliance/wasm-tools.
#
# The original code is licensed under the Apache License, Version 2.0
# (the "License"); you may not use this file except in compliance
# with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
*/
//! Methods to read basic WASM Values from a [WasmReader] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/values.html>
//!
//! Note: If any of these methods return `Err`, they may have consumed some bytes from the [WasmReader] object and thus consequent calls may result in unexpected behaviour.
//! This is due to the fact that these methods read elemental types which cannot be split.

use alloc::vec::Vec;
use core::mem;

use crate::core::reader::WasmReader;
use crate::ValidationError;

impl WasmReader<'_> {
    /// Tries to read one byte and fails if the end of file is reached.
    pub fn read_u8(&mut self) -> Result<u8, ValidationError> {
        let byte = self.peek_u8()?;
        self.pc += 1;
        Ok(byte)
    }

    const CONTINUATION_BIT: u8 = 1 << 7;

    /// Parses a variable-length `u64` (can be casted to a smaller uint if the result fits)
    /// Taken from <https://github.com/bytecodealliance/wasm-tools>
    #[allow(unused)]
    pub fn read_var_u64(&mut self) -> Result<u64, ValidationError> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let mut byte = self.read_u8()?;
            // maximum allowed num of leb bytes for u32 is ceil(32.0/7.0) == 10
            // shift >= 63 checks we're at the 10th bit or larger
            // byte != 1 checks whether (this byte lost bits when shifted) or (the continuation bit is set)
            if shift == 63 && byte != 0 && byte != 1 {
                while byte & Self::CONTINUATION_BIT != 0 {
                    byte = self.read_u8()?;
                }
                return Err(ValidationError::MalformedVariableLengthInteger);
            }

            let low_bits = (byte & !Self::CONTINUATION_BIT) as u64;
            result |= low_bits << shift;

            if byte & Self::CONTINUATION_BIT == 0 {
                return Ok(result);
            }

            shift += 7;
        }
    }

    /// Parses a variable-length `u32` as specified by [LEB128](https://en.wikipedia.org/wiki/LEB128#Unsigned_LEB128).
    /// Note: If `Err`, the [WasmReader] object is no longer guaranteed to be in a valid state
    pub fn read_var_u32(&mut self) -> Result<u32, ValidationError> {
        let mut result: u32 = 0;
        let mut shift = 0;

        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u32) << shift;
            // maximum allowed num of leb bytes for u32 is ceil(32.0/7.0) == 5
            // shift >= 28 checks we're at the 5th bit or larger
            // byte >> 32-28 checks whether (this byte lost bits when shifted) or (the continuation bit is set)
            if shift >= 28 && byte >> (32 - shift) != 0 {
                return Err(ValidationError::MalformedVariableLengthInteger);
            }

            if byte & Self::CONTINUATION_BIT == 0 {
                return Ok(result);
            }

            shift += 7;
        }
    }

    pub fn read_f64(&mut self) -> Result<u64, ValidationError> {
        let bytes = self.strip_bytes::<8>().map_err(|_| ValidationError::Eof)?;
        let word = u64::from_le_bytes(bytes);
        Ok(word)
    }

    /// Adapted from <https://github.com/bytecodealliance/wasm-tools>
    pub fn read_var_i32(&mut self) -> Result<i32, ValidationError> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i32) << shift;

            if shift >= 28 {
                // maximum allowed num of leb bytes for u32 is ceil(32.0/7.0) == 5
                // shift >= 28 checks we're at the 5th bit or larger
                let there_are_more_bytes = (byte & Self::CONTINUATION_BIT) != 0;
                let ashifted_unused_bits = (byte << 1) as i8 >> (32 - shift);
                // the top unused bits of 35 bytes should be either 111 for negative numbers or 000 for positive numbers
                // therefore ashifted_unused_bits should be -1 or 0
                if there_are_more_bytes || (ashifted_unused_bits != 0 && ashifted_unused_bits != -1)
                {
                    return Err(ValidationError::MalformedVariableLengthInteger);
                } else {
                    // no need to ashift unfilled bits, all 32 bits are filled
                    return Ok(result);
                }
            }

            shift += 7;

            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        // fill in unfilled bits with sign bit
        let ashift = mem::size_of::<i32>() * 8 - shift as usize;
        Ok((result << ashift) >> ashift)
    }

    pub fn read_var_i33(&mut self) -> Result<i64, ValidationError> {
        let mut result: i64 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i64) << shift;

            if shift >= 28 {
                // maximum allowed num of leb bytes for i33 is ceil(33.0/7.0) == 5
                // shift >= 28 checks we're at the 5th bit or larger
                let there_are_more_bytes = (byte & Self::CONTINUATION_BIT) != 0;
                let ashifted_unused_bits = (byte << 1) as i8 >> (33 - shift);
                // the top unused bits of 35 bytes should be either 11 for negative numbers or 00 for positive numbers
                // therefore ashifted_unused_bits should be -1 or 0
                if there_are_more_bytes || (ashifted_unused_bits != 0 && ashifted_unused_bits != -1)
                {
                    return Err(ValidationError::MalformedVariableLengthInteger);
                }
            }

            shift += 7;

            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        // fill in unfilled bits with sign bit
        let ashift = mem::size_of::<i64>() * 8 - shift as usize;
        Ok((result << ashift) >> ashift)
    }

    pub fn read_f32(&mut self) -> Result<u32, ValidationError> {
        if self.full_wasm_binary.len() - self.pc < 4 {
            return Err(ValidationError::Eof);
        }

        let word = u32::from_le_bytes(
            self.full_wasm_binary[self.pc..self.pc + 4]
                .try_into()
                .unwrap(),
        );

        self.strip_bytes::<4>()?;

        Ok(word)
    }

    pub fn read_var_i64(&mut self) -> Result<i64, ValidationError> {
        let mut result: i64 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i64) << shift;

            if shift >= 63 {
                // maximum allowed num of leb bytes for i33 is ceil(64.0/7.0) == 10
                // shift >= 63 checks we're at the 10th bit or larger
                let there_are_more_bytes = (byte & Self::CONTINUATION_BIT) != 0;
                let ashifted_unused_bits = (byte << 1) as i8 >> (64 - shift);
                // the top unused bits of 70 bytes should be either 111111 for negative numbers or 000000 for positive numbers
                // therefore ashifted_unused_bits should be -1 or 0
                if there_are_more_bytes || (ashifted_unused_bits != 0 && ashifted_unused_bits != -1)
                {
                    return Err(ValidationError::MalformedVariableLengthInteger);
                } else {
                    // no need to ashift unfilled bits, all 64 bits are filled
                    return Ok(result);
                }
            }

            shift += 7;

            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        // fill in unfilled bits with sign bit
        let ashift = mem::size_of::<i64>() * 8 - shift as usize;
        Ok((result << ashift) >> ashift)
    }

    /// Note: If `Err`, the [WasmReader] object is no longer guaranteed to be in a valid state
    pub fn read_name(&mut self) -> Result<&str, ValidationError> {
        let len = self.read_var_u32()? as usize;

        if len > self.full_wasm_binary.len() - self.pc {
            return Err(ValidationError::Eof);
        }

        let utf8_str = &self.full_wasm_binary[self.pc..(self.pc + len)]; // Cannot panic because check is done above
        self.pc += len;

        core::str::from_utf8(utf8_str).map_err(ValidationError::MalformedUtf8)
    }

    pub fn read_vec_enumerated<T, F>(
        &mut self,
        mut read_element: F,
    ) -> Result<Vec<T>, ValidationError>
    where
        F: FnMut(&mut WasmReader, usize) -> Result<T, ValidationError>,
    {
        let mut idx = 0;
        self.read_vec(|wasm| {
            let ret = read_element(wasm, idx);
            idx += 1;
            ret
        })
    }

    /// Note: If `Err`, the [WasmReader] object is no longer guaranteed to be in a valid state
    pub fn read_vec<T, F>(&mut self, mut read_element: F) -> Result<Vec<T>, ValidationError>
    where
        F: FnMut(&mut WasmReader) -> Result<T, ValidationError>,
    {
        let len = self.read_var_u32()?;
        (0..len).map(|_| read_element(self)).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::reader::WasmReader;

    #[test]
    fn test_var_i32() {
        let bytes = [0xC0, 0xBB, 0x78];
        let mut wasm = WasmReader::new(&bytes);

        assert_eq!(wasm.read_var_i32(), Ok(-123456));
    }
}
