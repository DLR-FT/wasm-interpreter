use crate::core::reader::types::{NumType, ValType};
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::unreachable_validated;
use crate::values::{WasmValue, WasmValueList};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ValueStack {
    inner: Vec<u8>,
}

impl ValueStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pop<T: WasmValue>(&mut self) -> T {
        T::from_bytes(&*self.pop_bytes(T::SIZE))
    }

    pub fn push<T: WasmValue>(&mut self, t: T) {
        self.push_bytes(&*t.into_bytes());
    }

    pub fn pop_bytes(&mut self, n: usize) -> Box<[u8]> {
        let len = self.inner.len();
        if len < n {
            unreachable_validated!()
        }
        self.inner
            .drain((len - n)..)
            .collect::<Vec<u8>>()
            .into_boxed_slice()
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.inner.extend(bytes.iter());
    }

    pub fn pop_all<T: WasmValueList>(&mut self) -> T {
        let bytes = self.pop_bytes(T::SIZE);
        T::from_bytes_list(&*bytes)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
