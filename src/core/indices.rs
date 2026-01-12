// /// This macro defines index types. Currently (2024-06-10) all indices are [`u32`].
// /// See <https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#indices%E2%91%A2> for more information.
// macro_rules! def_idx_types {
//     ($($name:ident),*) => {
//         $(
//             /// <https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#indices%E2%91%A2>
//             pub type $name = usize;
//         )*
//     };
// }

// // #[allow(dead_code)]
// def_idx_types!(TypeIdx, FuncIdx, TableIdx, MemIdx, GlobalIdx, /* ElemIdx, DataIdx, */ LocalIdx/* , LabelIdx */);

// TODO check whether is is clever to internally use usize instead of u32; potential problems are:
// - unsound on architectures where `usize` < `u32`
// - wasteful in memory on architectures where `usize` > `u32`
pub type TypeIdx = usize;
pub type FuncIdx = usize;
pub type TableIdx = usize;
pub type MemIdx = usize;
pub type GlobalIdx = usize;
#[allow(dead_code)]
pub type ElemIdx = usize;
pub type DataIdx = usize;
pub type LocalIdx = usize;
#[allow(dead_code)]
pub type LabelIdx = usize;
