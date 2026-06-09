use crate::{
    core::{
        decoding::reader::WasmDecoder,
        structure::modules::indices::{
            DataIdx, ElemIdx, FuncIdx, GlobalIdx, Idx, IdxVec, LocalIdx, MemIdx, TableIdx, TypeIdx,
        },
    },
    FuncType, ValType, ValidationError,
};

impl TypeIdx {
    /// Validates that a given index is a valid type index.
    ///
    /// On success a new [`TypeIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate(
        index: u32,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        c_types
            .validate_index(index)
            .ok_or(ValidationError::InvalidTypeIdx(index))
    }

    /// Reads a type index from Wasm code and validates that it is a valid index
    /// for a given types vector.
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        Self::validate(index, c_types)
    }
}

impl FuncIdx {
    /// Validates that a given index is a valid function index.
    ///
    /// On success a new [`FuncIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate<T>(index: u32, c_funcs: &IdxVec<FuncIdx, T>) -> Result<Self, ValidationError> {
        c_funcs
            .validate_index(index)
            .ok_or(ValidationError::InvalidFuncIdx(index))
    }

    /// Reads a function index from Wasm code and validates that it is a valid
    /// index for a given functions vector.
    pub fn decode_and_validate<T>(
        wasm: &mut WasmDecoder,
        c_funcs: &IdxVec<FuncIdx, T>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        Self::validate(index, c_funcs)
    }
}

impl TableIdx {
    /// Validates that a given index is a valid table index.
    ///
    /// On success a new [`TableIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate<T>(
        index: u32,
        c_tables: &IdxVec<TableIdx, T>,
    ) -> Result<Self, ValidationError> {
        c_tables
            .validate_index(index)
            .ok_or(ValidationError::InvalidTableIdx(index))
    }

    /// Reads a table index from Wasm code and validates that it is a valid
    /// index for a given tables vector.
    pub fn decode_and_validate<T>(
        wasm: &mut WasmDecoder,
        c_tables: &IdxVec<TableIdx, T>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        Self::validate(index, c_tables)
    }
}

impl MemIdx {
    /// Validates that a given index is a valid memory index.
    ///
    /// On success a new [`MemIdx`] is returned, otherwise a [`ValidationError`]
    /// is returned.
    pub fn validate<T>(index: u32, c_mems: &IdxVec<MemIdx, T>) -> Result<Self, ValidationError> {
        c_mems
            .validate_index(index)
            .ok_or(ValidationError::InvalidMemIdx(index))
    }

    /// Reads a memory index from Wasm code and validates that it is a valid
    /// index for a given memories vector.
    pub fn decode_and_validate<T>(
        wasm: &mut WasmDecoder,
        c_mems: &IdxVec<MemIdx, T>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        Self::validate(index, c_mems)
    }
}

impl GlobalIdx {
    /// Validates that a given index is a valid global index.
    ///
    /// On success a new [`GlobalIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate<T>(
        index: u32,
        c_globals: &IdxVec<GlobalIdx, T>,
    ) -> Result<Self, ValidationError> {
        c_globals
            .validate_index(index)
            .ok_or(ValidationError::InvalidGlobalIdx(index))
    }

    /// Reads a global index from Wasm code and validates that it is a valid
    /// index for a given globals vector.
    pub fn decode_and_validate<T>(
        wasm: &mut WasmDecoder,
        c_globals: &IdxVec<GlobalIdx, T>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        Self::validate(index, c_globals)
    }
}

impl ElemIdx {
    /// Validates that a given index is a valid element index.
    ///
    /// On success a new [`ElemIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate<T>(index: u32, c_elems: &IdxVec<ElemIdx, T>) -> Result<Self, ValidationError> {
        c_elems
            .validate_index(index)
            .ok_or(ValidationError::InvalidElemIdx(index))
    }

    /// Reads an element index from Wasm code and validates that it is a valid
    /// index for a given elements vector.
    pub fn decode_and_validate<T>(
        wasm: &mut WasmDecoder,
        c_elems: &IdxVec<ElemIdx, T>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        Self::validate(index, c_elems)
    }
}

impl DataIdx {
    /// Validates that a given index is a valid data index.
    ///
    /// On success a new [`DataIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate(index: u32, data_count: u32) -> Result<Self, ValidationError> {
        (index < data_count)
            .then_some(<Self as Idx>::new(index))
            .ok_or(ValidationError::InvalidDataIdx(index))
    }

    /// Reads a data index from Wasm code and validates that it is a valid
    /// by comparing it to the total number of data segments.
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        data_count: u32,
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        Self::validate(index, data_count)
    }
}

impl LocalIdx {
    /// Reads a local index from Wasm code and validates that it is valid for a
    /// given slice of locals.
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        locals_of_current_function: &[ValType],
    ) -> Result<Self, ValidationError> {
        let index = wasm.decode_var_u32()?;
        let index_as_usize = usize::try_from(index).expect("architecture to be at least 32 bits");

        match locals_of_current_function.get(index_as_usize) {
            Some(_local) => Ok(Self(index)),
            None => Err(ValidationError::InvalidLocalIdx(index)),
        }
    }
}
