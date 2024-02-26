use core::ops::Index;

use crate::wasm::span::Span;
use crate::{Error, Result};

/// A struct for managing the WASM bytecode.
/// Its purpose is mostly to abstract parsing basic WASM values from the bytecode.
pub struct Wasm<'a> {
    full_contents: &'a [u8],
    current: &'a [u8],
}

impl<'a> Wasm<'a> {
    pub fn new(wasm: &'a [u8]) -> Self {
        Self {
            full_contents: wasm,
            current: wasm,
        }
    }

    pub fn remaining_bytes(&self) -> &[u8] {
        &self.current
    }

    pub fn current_idx(&self) -> usize {
        self.full_contents.len() - self.current.len()
    }
    pub fn make_span(&self, len: usize) -> Span {
        Span::new(self.current_idx(), len)
    }

    pub fn strip_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        if N > self.current.len() {
            return Err(Error::MissingValue);
        }

        let (bytes, rest) = self.current.split_at(N);
        self.current = rest;

        Ok(bytes.try_into().expect("the slice length to be exactly N"))
    }
}

// Methods to read WASM Values as specified in the spec.
// See https://webassembly.github.io/spec/core/binary/values.html
impl Wasm<'_> {
    pub fn strip_u8(&mut self) -> Result<u8> {
        let value = *self.current.get(0).ok_or(Error::MissingValue)?;

        self.current = &self
            .current
            .get(1..)
            .expect("slice to contain at least 1 element");

        Ok(value)
    }

    /// Parses a variable-length `u32` as specified by [LEB128](https://en.wikipedia.org/wiki/LEB128#Unsigned_LEB128).
    pub fn strip_var_u32(&mut self) -> Result<u32> {
        let mut result = 0_u32;
        for byte_idx in 0..5_u32 {
            // u32 max length is 5 bytes
            let read_byte = self.strip_u8()? as u32;
            let (has_next_flag, byte_data) = (read_byte & 0b10000000, read_byte & 0b01111111);
            result |= byte_data << (byte_idx * 7);
            if has_next_flag >> 7 != 1 {
                break;
            }
        }

        Ok(result)
    }

    pub fn strip_name(&mut self) -> Result<&str> {
        let len = self.strip_var_u32()? as usize;

        if len > self.current.len() {
            return Err(Error::MissingValue);
        }
        let (utf8_str, rest) = self.current.split_at(len); // Cannot panic because check is done above
        self.current = rest;

        core::str::from_utf8(utf8_str).map_err(|err| Error::MalformedUtf8String(err))
    }
}

pub mod span {
    use core::ops::Index;

    use crate::wasm::Wasm;

    #[derive(Copy, Clone, Debug, Hash)]
    pub struct Span {
        from: usize,
        len: usize,
    }

    impl Span {
        pub fn new(from: usize, len: usize) -> Self {
            Self { from, len }
        }
        pub fn len(&self) -> usize {
            self.len
        }
    }

    impl<'a> Index<Span> for Wasm<'a> {
        type Output = [u8];

        fn index(&self, index: Span) -> &'a Self::Output {
            &self.full_contents[index.from..(index.from + index.len)]
        }
    }
}
