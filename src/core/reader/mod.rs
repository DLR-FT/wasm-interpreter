use crate::core::reader::span::Span;
use crate::{Error, Result};

pub mod section_header;
pub mod types;

/// A struct for managing and reading WASM bytecode
///
/// Its purpose is to abstract parsing basic WASM values from the bytecode.
pub struct WasmReader<'a> {
    /// Entire WASM binary as slice
    pub full_wasm_binary: &'a [u8],

    /// Current program counter, i. e. index of the next byte to be consumed from the WASM binary
    ///
    /// # Correctness Note
    ///
    /// The `pc` points to the next byte to be consumed from the WASM binary. Therefore, after
    /// consuming last byte, this cursor will advance past the last byte; for a WASM binary that is
    /// 100 bytes long (valid indexes start with 0 and end with 99), the `pc` therefore can become
    /// 100. However, it can not advance further.
    ///
    /// The table below illustrates this with an example for a WASM binary that is 5 bytes long:
    ///
    /// |                     Index |   0  |   1  |   2  |   3  |   4  | 5 | 6 |
    /// |--------------------------:|:----:|:----:|:----:|:----:|:----:|:-:|:-:|
    /// | `full_wasm_binary[index]` | 0xaa | 0xbb | 0xcc | 0xee | 0xff | - | - |
    /// |      Valid `pc` position? |   ✅  |   ✅  |   ✅  |   ✅  |   ✅  | ✅ | ❌ |
    pub pc: usize,
}

impl<'a> WasmReader<'a> {
    /// Initialize a new [WasmReader] from a WASM byte slice
    pub const fn new(wasm: &'a [u8]) -> Self {
        Self {
            full_wasm_binary: wasm,
            pc: 0,
        }
    }

    /// Advance the cursor to the first byte of the provided [Span] and validates that entire [Span] fits the WASM binary
    ///
    /// # Note
    ///
    /// This allows setting the [`pc`](WasmReader::pc) to one byte *past* the end of
    /// [full_wasm_binary](WasmReader::full_wasm_binary), **if** the [Span]'s length is 0. For
    /// further information, refer to the [field documentation of `pc`](WasmReader::pc).
    pub fn move_start_to(&mut self, span: Span) -> Result<()> {
        if span.from + span.len > self.full_wasm_binary.len() {
            return Err(Error::Eof);
        }

        self.pc = span.from;

        Ok(())
    }

    /// Byte slice to the remainder of the WASM binary, beginning from the current [`pc`](Self::pc)
    pub fn remaining_bytes(&self) -> &[u8] {
        &self.full_wasm_binary[self.pc..]
    }

    /// Create a [Span] starting from [`pc`](Self::pc) for the next `len` bytes
    ///
    /// Does **not** verify the span to fit the WASM binary, i.e. the span could exceed the WASM
    /// binary's bounds.
    pub fn make_span_unchecked(&self, len: usize) -> Span {
        Span::new(self.pc, len)
    }

    /// Take `N` bytes starting from [`pc`](Self::pc), then advance the [`pc`](Self::pc) by `N`
    ///
    /// This yields back an array of the correct length
    ///
    /// # Note
    ///
    /// This allows setting the [`pc`](WasmReader::pc) to one byte *past* the end of
    /// [full_wasm_binary](WasmReader::full_wasm_binary), **if** `N` equals the remaining bytes
    /// slice's length. For further information, refer to the [field documentation of `pc`]
    /// (WasmReader::pc).
    pub fn strip_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        if N > self.full_wasm_binary.len() - self.pc {
            return Err(Error::Eof);
        }

        let bytes = &self.full_wasm_binary[self.pc..(self.pc + N)];
        self.pc += N;

        Ok(bytes.try_into().expect("the slice length to be exactly N"))
    }

    /// Read the current byte without advancing the [`pc`](Self::pc)
    ///
    /// May yield an error if the [`pc`](Self::pc) advanced past the end of the WASM binary slice
    pub fn peek_u8(&self) -> Result<u8> {
        self.full_wasm_binary
            .get(self.pc)
            .copied()
            .ok_or(Error::Eof)
    }

    /// Call a closure that may mutate the [WasmReader]
    ///
    /// Returns a tuple of the closure's return value and the number of bytes that the [`WasmReader`]
    /// was advanced by.
    ///
    /// # Panics
    ///
    /// May panic if the closure moved the [`pc`](Self::pc) backwards, e.g. when
    /// [move_start_to](Self::move_start_to) is called.
    pub fn measure_num_read_bytes<T>(
        &mut self,
        f: impl FnOnce(&mut WasmReader) -> Result<T>,
    ) -> Result<(T, usize)> {
        let before = self.pc;
        let ret = f(self)?;

        // TODO maybe use checked sub, that is slower but guarantees no surprises
        debug_assert!(
            self.pc >= before,
            "pc was advanced backwards towards the start"
        );

        let num_read_bytes = self.pc - before;
        Ok((ret, num_read_bytes))
    }

    /// Skip `num_bytes`, advancing the [`pc`](Self::pc) accordingly
    ///
    /// # Note
    ///
    /// This can move the [`pc`](Self::pc) past the last byte of the WASM binary, so that reading
    /// more than 0 further bytes would panick. However, it can not move the [`pc`](Self::pc) any
    /// further than that, instead an error is returned. For further information, refer to the
    /// [field documentation of `pc`] (WasmReader::pc).
    pub fn skip(&mut self, num_bytes: usize) -> Result<()> {
        if num_bytes > self.full_wasm_binary.len() - self.pc {
            return Err(Error::Eof);
        }
        self.pc += num_bytes;
        Ok(())
    }

    /// Consumes [Self], yielding back the internal reference to the WASM binary
    pub fn into_inner(self) -> &'a [u8] {
        self.full_wasm_binary
    }
}

pub trait WasmReadable: Sized {
    fn read(wasm: &mut WasmReader) -> Result<Self>;
    fn read_unvalidated(wasm: &mut WasmReader) -> Self;
}

pub mod span {
    use core::ops::Index;

    use crate::core::reader::WasmReader;

    /// An index and offset to describe a (sub-) slice into WASM bytecode
    ///
    /// Can be used to index into a [WasmReader], yielding a byte slice. As it does not
    /// actually own the indexed data, this struct is free of lifetimes. Caution is advised when
    /// indexing unknown slices, as a [Span] does not validate the length of the indexed slice.
    #[derive(Copy, Clone, Debug, Hash)]
    pub struct Span {
        pub(super) from: usize,
        pub(super) len: usize,
    }

    impl Span {
        /// Create a new [Span], starting from `from` and ranging `len` elements
        pub const fn new(from: usize, len: usize) -> Self {
            Self { from, len }
        }

        /// Returns the length of this [Span]
        pub const fn len(&self) -> usize {
            self.len
        }
    }

    impl<'a> Index<Span> for WasmReader<'a> {
        type Output = [u8]; // TODO make this Result

        fn index(&self, index: Span) -> &'a Self::Output {
            &self.full_wasm_binary[index.from..(index.from + index.len)]
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::vec;

    #[test]
    fn move_start_to() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        let span = Span::new(0, 0);
        wasm_reader.move_start_to(span).unwrap();
        // this actually dangerous, we did not validate there to be more than 0 bytes using the Span
        wasm_reader.peek_u8().unwrap();

        let span = Span::new(0, my_bytes.len());
        wasm_reader.move_start_to(span).unwrap();
        wasm_reader.peek_u8().unwrap();
        assert_eq!(wasm_reader[span], my_bytes);

        let span = Span::new(my_bytes.len(), 0);
        wasm_reader.move_start_to(span).unwrap();
        // span had zero length, hence wasm_reader.peek_u8() would be allowed to fail

        let span = Span::new(my_bytes.len() - 1, 1);
        wasm_reader.move_start_to(span).unwrap();

        assert_eq!(wasm_reader.peek_u8().unwrap(), *my_bytes.last().unwrap());
    }

    #[test]
    fn move_start_to_out_of_bounds_1() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        let span = Span::new(my_bytes.len(), 1);
        assert_eq!(wasm_reader.move_start_to(span), Err(Error::Eof));
    }

    #[test]
    fn move_start_to_out_of_bounds_2() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        let span = Span::new(0, my_bytes.len() + 1);
        assert_eq!(wasm_reader.move_start_to(span), Err(Error::Eof));
    }

    #[test]
    fn remaining_bytes_1() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        assert_eq!(wasm_reader.remaining_bytes(), my_bytes);
        wasm_reader.skip(4).unwrap();
        assert_eq!(wasm_reader.peek_u8().unwrap(), 0x15);

        assert_eq!(wasm_reader.remaining_bytes(), &my_bytes[4..]);
    }

    #[test]
    fn remaining_bytes_2() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        assert_eq!(wasm_reader.remaining_bytes(), my_bytes);
        wasm_reader.skip(5).unwrap();
        assert_eq!(wasm_reader.remaining_bytes(), &my_bytes[5..]);
        assert_eq!(wasm_reader.remaining_bytes(), &[]);
    }

    #[test]
    fn strip_bytes_1() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        assert_eq!(wasm_reader.remaining_bytes(), my_bytes);
        let stripped_bytes = wasm_reader.strip_bytes::<4>().unwrap();
        assert_eq!(&stripped_bytes, &my_bytes[..4]);
        assert_eq!(wasm_reader.remaining_bytes(), &[0x15]);
    }

    #[test]
    fn strip_bytes_2() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        assert_eq!(wasm_reader.remaining_bytes(), my_bytes);
        wasm_reader.skip(1).unwrap();
        let stripped_bytes = wasm_reader.strip_bytes::<4>().unwrap();
        assert_eq!(&stripped_bytes, &my_bytes[1..5]);
        assert_eq!(wasm_reader.remaining_bytes(), &[]);
    }

    #[test]
    fn strip_bytes_3() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        assert_eq!(wasm_reader.remaining_bytes(), my_bytes);
        wasm_reader.skip(2).unwrap();
        let stripped_bytes = wasm_reader.strip_bytes::<4>();
        assert_eq!(stripped_bytes, Err(Error::Eof));
    }

    #[test]
    fn strip_bytes_4() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);

        assert_eq!(wasm_reader.remaining_bytes(), my_bytes);
        wasm_reader.skip(5).unwrap();
        let stripped_bytes = wasm_reader.strip_bytes::<0>().unwrap();
        assert_eq!(stripped_bytes, [0u8; 0]);
    }

    #[test]
    fn skip_1() {
        let my_bytes = vec![0x11, 0x12, 0x13, 0x14, 0x15];
        let mut wasm_reader = WasmReader::new(&my_bytes);
        assert_eq!(wasm_reader.remaining_bytes(), my_bytes);
        assert_eq!(wasm_reader.skip(6), Err(Error::Eof));
    }
}
