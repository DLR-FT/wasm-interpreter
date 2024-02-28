//! Methods to read WASM Values from a [Wasm] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/values.html>
//!
//! Note: If any of these methods return `Err`, they may have consumed some bytes from the [Wasm] object and thus consequent calls may result in unexpected behaviour.
//! This is due to the fact that these methods read elemental types which cannot be split.

use crate::wasm::Wasm;
use crate::{Error, Result};
use alloc::vec::Vec;

impl Wasm<'_> {
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
        let mut result = 0_u32;
        for byte_idx in 0..5_u32 {
            // u32 max length is 5 bytes
            let read_byte = self.read_u8()? as u32;
            let (has_next_flag, byte_data) = (read_byte & 0b10000000, read_byte & 0b01111111);
            result |= byte_data << (byte_idx * 7);
            if has_next_flag >> 7 != 1 {
                break;
            }
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
        F: FnMut(&mut Wasm) -> Result<T>,
    {
        let len = self.read_var_u32()?;
        (0..len).map(|_| read_element(self)).collect()
    }
}
