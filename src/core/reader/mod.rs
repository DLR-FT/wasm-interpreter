use crate::core::reader::span::Span;
use crate::{Error, Result};

pub mod section_header;
pub mod types;

/// A struct for managing and reading WASM bytecode.
/// Its purpose is mostly to abstract parsing basic WASM values from the bytecode.
pub struct WasmReader<'a> {
    pub(crate) full_contents: &'a [u8],
    pub(crate) current: &'a [u8],
}

impl<'a> WasmReader<'a> {
    pub fn new(wasm: &'a [u8]) -> Self {
        Self {
            full_contents: wasm,
            current: wasm,
        }
    }
    // TODO this is not very intuitive but we cannot shorten `self.current`'s end
    //  because some methods rely on the property that `self.current`'s and
    //  `self.full_contents`'s last element are equal.
    pub fn move_start_to(&mut self, span: Span) {
        self.current =
            &self.full_contents[span.from../* normally we would have the end of the span here*/];
    }

    pub fn remaining_bytes(&self) -> &[u8] {
        self.current
    }

    pub fn current_idx(&self) -> usize {
        self.full_contents.len() - self.current.len()
    }
    pub fn make_span(&self, len: usize) -> Span {
        Span::new(self.current_idx(), len)
    }

    pub fn strip_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        if N > self.current.len() {
            return Err(Error::Eof);
        }

        let (bytes, rest) = self.current.split_at(N);
        self.current = rest;

        Ok(bytes.try_into().expect("the slice length to be exactly N"))
    }
    pub fn peek_u8(&self) -> Result<u8> {
        self.current.first().copied().ok_or(Error::Eof)
    }

    pub fn measure_num_read_bytes<T>(
        &mut self,
        f: impl FnOnce(&mut WasmReader) -> Result<T>,
    ) -> Result<(T, usize)> {
        let before = self.current_idx();
        let ret = f(self)?;
        let num_read_bytes = self.current_idx() - before;

        Ok((ret, num_read_bytes))
    }

    pub fn skip(&mut self, num_bytes: usize) -> Result<()> {
        if self.current.len() < num_bytes {
            return Err(Error::Eof);
        }
        self.current = &self.current[num_bytes..];
        Ok(())
    }
    pub fn into_inner(self) -> &'a [u8] {
        self.full_contents
    }
}

pub trait WasmReadable: Sized {
    fn read(wasm: &mut WasmReader) -> Result<Self>;
    fn read_unvalidated(wasm: &mut WasmReader) -> Self;
}

pub mod span {
    use core::ops::Index;

    use crate::core::reader::WasmReader;

    #[derive(Copy, Clone, Debug, Hash)]
    pub struct Span {
        pub(super) from: usize,
        pub(super) len: usize,
    }

    impl Span {
        pub fn new(from: usize, len: usize) -> Self {
            Self { from, len }
        }
        pub fn len(&self) -> usize {
            self.len
        }
    }

    impl<'a> Index<Span> for WasmReader<'a> {
        type Output = [u8];

        fn index(&self, index: Span) -> &'a Self::Output {
            &self.full_contents[index.from..(index.from + index.len)]
        }
    }
}
