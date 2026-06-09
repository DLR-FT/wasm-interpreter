use crate::{
    core::{
        decoding::reader::WasmDecoder,
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
    /// [`WasmDecoder`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl FuncIdx {
    /// Reads a function index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid function index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl TableIdx {
    /// Reads a table index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid table index in the [`WasmDecoder`].
    /// [`Self::decode_and_validate`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl MemIdx {
    /// Reads a memory index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid memory index in the [`WasmDecoder`].
    #[allow(unused)] // reason = "unused until multiple memories proposal is implemented"
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        Self::new(index)
    }
}

impl GlobalIdx {
    /// Reads a global index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid global index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl ElemIdx {
    /// Reads an element index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid element index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl DataIdx {
    /// Reads a data index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid data index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        <Self as Idx>::new(index)
    }
}

impl LocalIdx {
    /// Reads a local index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid local index in the [`WasmDecoder`].
    pub unsafe fn decode_unchecked(wasm: &mut WasmDecoder) -> Self {
        let index = wasm.decode_var_u32().unwrap();
        Self(index)
    }
}

/// Reads a label index from Wasm code without validating it.
pub fn decode_label_idx(wasm: &mut WasmDecoder) -> Result<u32, DecodingError> {
    wasm.decode_var_u32()
}

/// Reads a label index from Wasm code without validating it.
///
/// # Safety
///
/// The caller must ensure that there is a valid label index in the [`WasmDecoder`].
pub unsafe fn decode_label_idx_unchecked(wasm: &mut WasmDecoder) -> u32 {
    // TODO use `unwrap_unchecked` instead
    wasm.decode_var_u32().unwrap()
}
