//! Methods to read basic WASM Values from a [Wasm] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/values.html>
//!
//! Note: If any of these methods return `Err`, they may have consumed some bytes from the [Wasm] object and thus consequent calls may result in unexpected behaviour.
//! This is due to the fact that these methods read elemental types which cannot be split.

use alloc::vec::Vec;
use core::mem;

use crate::{Error, Result};
use crate::core::reader::WasmReader;

impl WasmReader<'_> {
    /// Note: If `Err`, the [Wasm] object is no longer guaranteed to be in a valid state
    pub fn read_u8(&mut self) -> Result<u8> {
        let value = *self.current.get(0).ok_or(Error::MissingValue)?;

        self.current = &self
            .current
            .get(1..)
            .expect("slice to contain at least 1 element");

        Ok(value)
    }

    /// Parses a variable-length `u32` as specified by [LEB128](https://en.wikipedia.org/wiki/LEB128#Unsigned_LEB128).
    /// Note: If `Err`, the [Wasm] object is no longer guaranteed to be in a valid state
    pub fn read_var_u32(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.read_u8()? as u32;
            result |= (byte & 0b01111111) << shift;
            if (byte & 0b10000000) == 0 {
                break;
            }
            shift += 7;
        }

        Ok(result)
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

    /// Note: If `Err`, the [Wasm] object is no longer guaranteed to be in a valid state
    pub fn read_name(&mut self) -> Result<&str> {
        let len = self.read_var_u32()? as usize;

        if len > self.current.len() {
            return Err(Error::MissingValue);
        }
        let (utf8_str, rest) = self.current.split_at(len); // Cannot panic because check is done above
        self.current = rest;

        core::str::from_utf8(utf8_str).map_err(|err| Error::MalformedUtf8String(err))
    }

    /// Note: If `Err`, the [Wasm] object is no longer guaranteed to be in a valid state
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
