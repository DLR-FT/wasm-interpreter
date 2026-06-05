use crate::{
    core::{
        decoding::reader::WasmReader,
        structure::modules::indices::{
            DataIdx, ElemIdx, FuncIdx, GlobalIdx, Idx, LocalIdx, MemIdx, TableIdx, TypeIdx,
        },
    },
    DecodingError,
};

impl TypeIdx {
    /// Reads a type index from Wasm code without validating it. Using the
    /// returned type requires some other form of validation to be done.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid type index in the
    /// [`WasmReader`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl FuncIdx {
    /// Reads a function index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid function index in the
    /// [`WasmReader`] and that this index is valid for a specific [`IdxVec`]
    /// through [`Self::read_and_validate`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl TableIdx {
    /// Reads a table index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid table index in the
    /// [`WasmReader`] and that this index is valid for a specific [`ExtendedIdxVec`]
    /// through [`Self::read_and_validate`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl MemIdx {
    /// Reads a memory index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid memory index in the
    /// [`WasmReader`] and that this index is valid for a specific [`ExtendedIdxVec`]
    /// through [`Self::read_and_validate`].
    #[allow(unused)] // reason = "unused until multiple memories proposal is implemented"
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        Self::new(index)
    }
}

impl GlobalIdx {
    /// Reads a global index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid global index in the
    /// [`WasmReader`] and that this index is valid for a specific [`IdxVec`]
    /// through [`Self::read_and_validate`] or [`Self::validate`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl ElemIdx {
    /// Reads an element index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid element index in the
    /// [`WasmReader`] and that this index is valid for a specific [`IdxVec`]
    /// through [`Self::read_and_validate`] or [`Self::validate`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl DataIdx {
    /// Reads a data index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid data index in the
    /// [`WasmReader`] and that this index is valid for a specific [`IdxVec`]
    /// through [`Self::read_and_validate`] or [`Self::validate`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl LocalIdx {
    /// Reads a local index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid local index in the
    /// [`WasmReader`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        Self(index)
    }
}

/// Reads a label index from Wasm code without validating it.
pub fn read_label_idx(wasm: &mut WasmReader) -> Result<u32, DecodingError> {
    wasm.read_var_u32()
}

/// Reads a label index from Wasm code without validating it.
///
/// # Safety
///
/// The caller must ensure that there is a valid label index in the
/// [`WasmReader`].
pub unsafe fn read_label_idx_unchecked(wasm: &mut WasmReader) -> u32 {
    // TODO use `unwrap_unchecked` instead
    wasm.read_var_u32().unwrap()
}
