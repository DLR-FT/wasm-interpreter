// macro_rules! def_idx_types {
//     ($($name:ident),*) => {
//         $(
//             /// <https://webassembly.github.io/spec/core/binary/modules.html#indices>
//             pub type $name = usize;
//         )*
//     };
// }

// // #[allow(dead_code)]
// def_idx_types!(TypeIdx, FuncIdx, TableIdx, MemIdx, GlobalIdx, /* ElemIdx, DataIdx, */ LocalIdx/* , LabelIdx */);

pub type TypeIdx = usize;
pub type FuncIdx = usize;
pub type TableIdx = usize;
pub type MemIdx = usize;
pub type GlobalIdx = usize;
#[allow(dead_code)]
pub type ElemIdx = usize;
#[allow(dead_code)]
pub type DataIdx = usize;
pub type LocalIdx = usize;
#[allow(dead_code)]
pub type LabelIdx = usize;
