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
use crate::{Error, Result};

impl WasmReader<'_> {
    /// Note: If `Err`, the [WasmReader] object is no longer guaranteed to be in a valid state
    pub fn read_u8(&mut self) -> Result<u8> {
        match self.peek_u8() {
            Err(e) => Err(e),
            Ok(byte) => {
                self.pc += 1;
                Ok(byte)
            }
        }
    }

    const CONTINUATION_BIT: u8 = 1 << 7;

    /// Parses a variable-length `u64` (can be casted to a smaller uint if the result fits)
    /// Taken from <https://github.com/bytecodealliance/wasm-tools>
    pub fn read_var_u64(&mut self) -> Result<u64> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let mut byte = self.read_u8()?;
            if shift == 63 && byte != 0 && byte != 1 {
                while byte & Self::CONTINUATION_BIT != 0 {
                    byte = self.read_u8()?;
                }
                return Err(Error::Overflow);
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
    pub fn read_var_u32(&mut self) -> Result<u32> {
        Self::read_var_u64(self)?
            .try_into()
            .map_err(|_| Error::Overflow)
    }

    pub fn read_var_f64(&mut self) -> Result<u64> {
        let bytes = self.strip_bytes::<8>().map_err(|_| Error::Eof)?;
        let word = u64::from_le_bytes(bytes);
        Ok(word)
    }

    pub fn read_var_i32(&mut self) -> Result<i32> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;

        let mut byte: i32;
        loop {
            byte = self.read_u8()? as i32;
            result |= (byte & 0b01111111) << shift;
            shift += 7;
            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        if (shift < mem::size_of::<i32>() as u32 * 8) && (byte & 0x40 != 0) {
            result |= !0 << shift;
        }

        Ok(result)
    }

    pub fn read_var_i33(&mut self) -> Result<i64> {
        let mut result: i64 = 0;
        let mut shift: u64 = 0;

        let mut byte: i64;
        loop {
            byte = self.read_u8()? as i64;
            result |= (byte & 0b0111_1111) << shift;
            shift += 7;
            if (byte & 0b1000_0000) == 0 {
                break;
            }
        }

        if shift < 33 && (byte & 0x40 != 0) {
            result |= !0 << shift;
        }

        Ok(result)
    }

    pub fn read_var_f32(&mut self) -> Result<u32> {
        if self.full_wasm_binary.len() - self.pc < 4 {
            return Err(Error::Eof);
        }

        let word = u32::from_le_bytes(
            self.full_wasm_binary[self.pc..self.pc + 4]
                .try_into()
                .unwrap(),
        );

        self.strip_bytes::<4>()?;

        Ok(word)
    }

    pub fn read_var_i64(&mut self) -> Result<i64> {
        let mut result: i64 = 0;
        let mut shift: u64 = 0;

        let mut byte: i64;
        loop {
            byte = self.read_u8()? as i64;
            result |= (byte & 0b0111_1111) << shift;
            shift += 7;
            if (byte & 0b1000_0000) == 0 {
                break;
            }
        }

        if shift < 64 && (byte & 0x40 != 0) {
            result |= !0 << shift;
        }

        Ok(result)
    }

    /// Note: If `Err`, the [WasmReader] object is no longer guaranteed to be in a valid state
    pub fn read_name(&mut self) -> Result<&str> {
        let len = self.read_var_u32()? as usize;

        if len > self.full_wasm_binary.len() - self.pc {
            return Err(Error::Eof);
        }

        let utf8_str = &self.full_wasm_binary[self.pc..(self.pc + len)]; // Cannot panic because check is done above
        self.pc += len;

        core::str::from_utf8(utf8_str).map_err(Error::MalformedUtf8String)
    }

    pub fn read_vec_enumerated<T, F>(&mut self, mut read_element: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut WasmReader, usize) -> Result<T>,
    {
        let mut idx = 0;
        self.read_vec(|wasm| {
            let ret = read_element(wasm, idx);
            idx += 1;
            ret
        })
    }

    /// Note: If `Err`, the [WasmReader] object is no longer guaranteed to be in a valid state
    pub fn read_vec<T, F>(&mut self, mut read_element: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut WasmReader) -> Result<T>,
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
