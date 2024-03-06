use alloc::vec::Vec;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    Uninitialized,
    I32(u32),
    I64(u64),
    F32(f32),
    Vec(u128),
}
