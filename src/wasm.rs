use crate::wasm::span::Span;
use crate::{Error, Result};

pub(crate) mod indices;
pub(crate) mod types;
pub(crate) mod values;

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
    pub fn peek_byte(&self) -> Result<u8> {
        self.current.get(0).copied().ok_or(Error::MissingValue)
    }

    pub fn measure_num_read_bytes<T>(
        &mut self,
        f: impl FnOnce(&mut Wasm) -> Result<T>,
    ) -> Result<(T, usize)> {
        let before = self.current_idx();
        let ret = f(self)?;
        let num_read_bytes = self.current_idx() - before;

        Ok((ret, num_read_bytes))
    }

    pub fn skip(&mut self, num_bytes: usize) -> Result<()> {
        if self.current.len() < num_bytes {
            return Err(Error::MissingValue);
        }
        self.current = &self.current[num_bytes..];
        Ok(())
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
