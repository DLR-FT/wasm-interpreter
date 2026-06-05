use core::fmt;

use alloc::vec::Vec;

use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        reader::WasmReader,
        structure::modules::indices::{IdxVec, TypeIdx},
    },
    ValidationError,
};

/// <https://webassembly.github.io/spec/core/binary/types.html#number-types>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// <https://webassembly.github.io/spec/core/binary/types.html#vector-types>
pub struct VecType;

/// <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

/// <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
/// TODO flatten [NumType] and [RefType] enums, as they are not used individually and `wasmparser` also does it.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValType {
    NumType(NumType),
    VecType,
    RefType(RefType),
}

impl ValType {
    pub const fn size(&self) -> usize {
        match self {
            Self::NumType(NumType::I32 | NumType::F32) => 4,
            Self::NumType(NumType::I64 | NumType::F64) => 8,
            Self::VecType => 16,
            Self::RefType(_) => todo!(),
        }
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#value-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    pub valtypes: Vec<ValType>,
}

/// <https://webassembly.github.io/spec/core/binary/types.html#function-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: ResultType,
    pub returns: ResultType,
}

/// <https://webassembly.github.io/spec/core/binary/instructions.html#binary-blocktype>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Returns(ValType),
    Type(TypeIdx),
}

impl BlockType {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        if wasm.peek_u8()? == 0x40 {
            // Empty block type
            let _ = wasm.read_u8().unwrap_validated();
            Ok(BlockType::Empty)
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::read(wasm)) {
            // No parameters and given valtype as the result
            Ok(BlockType::Returns(val_ty))
        } else {
            // An index to a function type
            let index = wasm.read_var_i33_as_u32()?;
            TypeIdx::validate(index, c_types).map(BlockType::Type)
        }
    }

    /// Converts this block type to a specific [`FuncType`].
    ///
    /// A vector of function types is required, in case the current block type
    /// stores a type index.
    ///
    /// # Safety
    ///
    /// The given [`IdxVec<TypeIdx, FuncType>`] must be the same on that was
    /// used to validate `self` through [`BlockType::read_and_validate`].
    // TODO maybe make this function return a `Cow<'a, FuncType>`. This could
    // prevent one allocation per call.
    pub unsafe fn as_func_type(
        &self,
        func_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<FuncType, ValidationError> {
        match self {
            BlockType::Empty => Ok(FuncType {
                params: ResultType {
                    valtypes: Vec::new(),
                },
                returns: ResultType {
                    valtypes: Vec::new(),
                },
            }),
            BlockType::Returns(val_type) => Ok(FuncType {
                params: ResultType {
                    valtypes: Vec::new(),
                },
                returns: ResultType {
                    valtypes: [*val_type].into(),
                },
            }),
            BlockType::Type(type_idx) => {
                // SAFETY: The caller ensures that this `IdxVec` is the same one
                // used to validate the `TypeIdx` in `self`.
                let func_type = unsafe { func_types.get(*type_idx) };
                Ok(func_type.clone())
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Limits {
    // since the maximum amount of bytes is u32::MAX, the page size is 1 << 16
    // the max no. of pages = max bytes / page size = u32::MAX / (1 << 16) = 1 << 16
    pub const MAX_MEM_PAGES: u32 = 1 << 16;
    // https://webassembly.github.io/reference-types/core/syntax/types.html#limits
    // memtype is defined in terms of limits, which go from 0 to u32::MAX
    pub const MAX_MEM_BYTES: u32 = u32::MAX;
    // https://webassembly.github.io/reference-types/core/exec/runtime.html#memory-instances
    // memory size is 65536 (1 << 16)
    pub const MEM_PAGE_SIZE: u32 = 1 << 16;
}

impl fmt::Debug for Limits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        match self.max {
            Some(max) => f.write_fmt(format_args!("{}..{}", self.min, max)),
            None => f.write_fmt(format_args!("{}..", self.min)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    pub et: RefType,
    pub lim: Limits,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub ty: ValType,
    pub is_mut: bool,
}

// <https://webassembly.github.io/spec/core/valid/types.html#import-subtyping>
///<https://webassembly.github.io/spec/core/valid/types.html#external-types>
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExternType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}
