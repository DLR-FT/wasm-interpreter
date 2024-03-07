use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::mem;

pub mod stack;

pub trait WasmValue: Copy + Debug + PartialEq {
    // Sadly we cannot use `SIZE` to return fixed-sized arrays because this is still unstable.
    // See feature(generic_const_exprs)
    const SIZE: usize;
    fn into_bytes(self) -> Box<[u8]>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl WasmValue for u32 {
    const SIZE: usize = mem::size_of::<Self>();

    fn into_bytes(self) -> Box<[u8]> {
        self.to_le_bytes().to_vec().into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes: [u8; <Self as WasmValue>::SIZE] = bytes.try_into().unwrap();
        u32::from_le_bytes(bytes)
    }
}

impl WasmValue for i32 {
    const SIZE: usize = mem::size_of::<Self>();

    fn into_bytes(self) -> Box<[u8]> {
        self.to_le_bytes().to_vec().into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes: [u8; <Self as WasmValue>::SIZE] = bytes.try_into().unwrap();
        i32::from_le_bytes(bytes)
    }
}

impl WasmValue for f32 {
    const SIZE: usize = mem::size_of::<Self>();

    fn into_bytes(self) -> Box<[u8]> {
        self.to_le_bytes().to_vec().into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes: [u8; <Self as WasmValue>::SIZE] = bytes.try_into().unwrap();
        f32::from_le_bytes(bytes)
    }
}

pub trait WasmValueList {
    const SIZE: usize;
    fn into_bytes_list(self) -> Vec<Box<[u8]>>;
    fn from_bytes_list(bytes: &[u8]) -> Self;
}

impl<A: WasmValue> WasmValueList for A {
    const SIZE: usize = A::SIZE;

    fn into_bytes_list(self) -> Vec<Box<[u8]>> {
        vec![self.into_bytes()]
    }

    fn from_bytes_list(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl<A: WasmValue> WasmValueList for (A,) {
    const SIZE: usize = A::SIZE;
    fn into_bytes_list(self) -> Vec<Box<[u8]>> {
        vec![self.0.into_bytes()]
    }

    fn from_bytes_list(bytes: &[u8]) -> Self {
        let (a, bytes) = bytes.split_at(A::SIZE);
        (A::from_bytes(a),)
    }
}

impl<A: WasmValue, B: WasmValue> WasmValueList for (A, B) {
    const SIZE: usize = A::SIZE + B::SIZE;
    fn into_bytes_list(self) -> Vec<Box<[u8]>> {
        vec![self.0.into_bytes(), self.1.into_bytes()]
    }

    fn from_bytes_list(bytes: &[u8]) -> Self {
        let (a, bytes) = bytes.split_at(A::SIZE);
        let (b, bytes) = bytes.split_at(B::SIZE);
        (A::from_bytes(a), B::from_bytes(b))
    }
}

impl<A: WasmValue, B: WasmValue, C: WasmValue> WasmValueList for (A, B, C) {
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE;
    fn into_bytes_list(self) -> Vec<Box<[u8]>> {
        vec![
            self.0.into_bytes(),
            self.1.into_bytes(),
            self.2.into_bytes(),
        ]
    }

    fn from_bytes_list(bytes: &[u8]) -> Self {
        let (a, bytes) = bytes.split_at(A::SIZE);
        let (b, bytes) = bytes.split_at(B::SIZE);
        let (c, bytes) = bytes.split_at(C::SIZE);
        (A::from_bytes(a), B::from_bytes(b), C::from_bytes(c))
    }
}
