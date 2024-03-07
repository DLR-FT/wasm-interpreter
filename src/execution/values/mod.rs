use alloc::boxed::Box;
use core::fmt::Debug;
use core::mem;

pub mod stack;
pub mod value_vec;

pub trait WasmValue: Copy + Debug + PartialEq {
    // Sadly we cannot use `SIZE` to return fixed-sized arrays because this is still unstable.
    // See feature(generic_const_exprs)
    const SIZE: usize;
    fn to_bytes(self) -> Box<[u8]>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl WasmValue for u32 {
    const SIZE: usize = mem::size_of::<Self>();

    fn to_bytes(self) -> Box<[u8]> {
        self.to_le_bytes().to_vec().into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes: [u8; Self::SIZE] = bytes.try_into().unwrap();
        u32::from_le_bytes(bytes)
    }
}

impl WasmValue for i32 {
    const SIZE: usize = mem::size_of::<Self>();

    fn to_bytes(self) -> Box<[u8]> {
        self.to_le_bytes().to_vec().into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes: [u8; Self::SIZE] = bytes.try_into().unwrap();
        i32::from_le_bytes(bytes)
    }
}

impl WasmValue for f32 {
    const SIZE: usize = mem::size_of::<Self>();


    fn to_bytes(self) -> Box<[u8]> {
        self.to_le_bytes().to_vec().into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes: [u8; Self::SIZE] = bytes.try_into().unwrap();
        f32::from_le_bytes(bytes)
    }
}
