// /// This macro defines index types. Currently (2024-06-10) all indices are [`u32`].
// /// See <https://webassembly.github.io/spec/core/binary/modules.html#indices> for more information.
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

use alloc::{boxed::Box, vec::Vec};

use crate::ValidationError;

use super::reader::{types::FuncType, WasmReader};

#[derive(Copy, Clone, Debug)]
pub struct TypeIdx(u32);

impl TypeIdx {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_types: &CTypes,
    ) -> Result<Self, ValidationError> {
        let index = wasm.read_var_u32()?;

        let _type = c_types
            .0
            .get(index as usize)
            .ok_or(ValidationError::InvalidTypeIdx(index))?;

        Ok(Self(index))
    }
}

impl core::fmt::Display for TypeIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "type index {}", self.0)
    }
}

// We need a custom wrapper type per index space to guarantee full soundness of the Rust code.
#[derive(Clone, Debug)]
pub struct CTypes(Box<[FuncType]>);
impl CTypes {
    pub fn new(types: Vec<FuncType>) -> Self {
        Self(types.into_boxed_slice())
    }

    /// Gets a [`FuncType`] by its [`TypeIdx`]
    ///
    /// # Safety
    /// The caller must make sure that the given [`TypeIdx`] was created and validated through the current
    /// [`CTypes`] object through its [`TypeIdx::read_and_validate`] method.
    pub unsafe fn get(&self, idx: TypeIdx) -> &FuncType {
        // # Safety
        // `idx` was created and validated using the same instance of `self`. During this validation,
        // the bounds check was already proven to be `Ok` and because `self` stores a boxed slice,
        // its length could not have changed since. Therefore, the index stored inside the `TypeIdx`
        // must still be valid.
        unsafe { self.0.get(idx.0 as usize).unwrap_unchecked() }
    }
}

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
