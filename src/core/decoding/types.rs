use crate::{
    core::{
        decoding::reader::WasmReader,
        structure::{
            modules::indices::TypeIdx,
            types::{
                BlockType, FuncType, GlobalType, MemArg, NumType, RefType, ResultType, ValType,
                VecType,
            },
        },
    },
    DecodingError,
};

impl NumType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, DecodingError> {
        use NumType::*;

        let ty = match wasm.peek_u8()? {
            0x7F => I32,
            0x7E => I64,
            0x7D => F32,
            0x7C => F64,
            other => return Err(DecodingError::MalformedNumTypeDiscriminator(other)),
        };
        let _ = wasm.read_u8();

        Ok(ty)
    }
}

impl VecType {
    fn read(wasm: &mut WasmReader) -> Result<Self, DecodingError> {
        match wasm.peek_u8()? {
            0x7b => {
                let _ = wasm.read_u8();
                Ok(VecType)
            }
            other => Err(DecodingError::MalformedVecTypeDiscriminator(other)),
        }
    }
}

impl RefType {
    pub fn read(wasm: &mut WasmReader) -> Result<RefType, DecodingError> {
        let ty = match wasm.peek_u8()? {
            0x70 => RefType::FuncRef,
            0x6F => RefType::ExternRef,
            other => return Err(DecodingError::MalformedRefTypeDiscriminator(other)),
        };
        let _ = wasm.read_u8();

        Ok(ty)
    }
}

impl ValType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, DecodingError> {
        if let Ok(numtype) = NumType::read(wasm).map(ValType::NumType) {
            return Ok(numtype);
        };
        if let Ok(vectype) = VecType::read(wasm).map(|_ty| ValType::VecType) {
            return Ok(vectype);
        };
        if let Ok(reftype) = RefType::read(wasm).map(ValType::RefType) {
            return Ok(reftype);
        }

        Err(DecodingError::MalformedValType)
    }
}

impl ResultType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, DecodingError> {
        let valtypes = wasm.read_vec(ValType::read)?;

        Ok(ResultType { valtypes })
    }
}

impl FuncType {
    pub fn read(wasm: &mut WasmReader) -> Result<FuncType, DecodingError> {
        match wasm.read_u8()? {
            0x60 => {}
            other => return Err(DecodingError::MalformedFuncTypeDiscriminator(other)),
        };

        let params = ResultType::read(wasm)?;
        let returns = ResultType::read(wasm)?;

        Ok(FuncType { params, returns })
    }
}

impl BlockType {
    /// # Safety
    ///
    /// The caller must ensure that there is a valid block type to be read in
    /// the given [`WasmReader`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        if wasm.peek_u8().unwrap() as i8 == 0x40 {
            // Empty block type
            let _ = wasm.read_u8().unwrap();
            BlockType::Empty
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::read(wasm)) {
            // No parameters and given valtype as the result
            BlockType::Returns(val_ty)
        } else {
            // An index to a function type
            let index = wasm.read_var_i33_as_u32().unwrap();
            BlockType::Type(TypeIdx::new(index))
        }
    }
}

impl GlobalType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, DecodingError> {
        let ty = ValType::read(wasm)?;
        let is_mut = match wasm.read_u8()? {
            0x00 => false,
            0x01 => true,
            other => return Err(DecodingError::MalformedMutDiscriminator(other)),
        };
        Ok(Self { ty, is_mut })
    }
}

impl MemArg {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, DecodingError> {
        let align = wasm.read_var_u32()?;
        let offset = wasm.read_var_u32()?;
        Ok(Self { offset, align })
    }
}
