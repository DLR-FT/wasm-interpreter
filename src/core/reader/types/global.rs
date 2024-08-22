use alloc::vec;

use crate::code::{read_constant_instructions, validate_value_stack};
use crate::core::reader::span::Span;
use crate::core::reader::types::{ResultType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::{unreachable_validated, Error, Result};

#[derive(Debug, Copy, Clone)]
pub struct Global {
    pub ty: GlobalType,
    // TODO validate init_expr during validation and execute during instantiation
    pub init_expr: Span,
}

impl WasmReadable for Global {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let ty = GlobalType::read(wasm)?;
        let mut init_expr = None;

        let expected_type = ResultType {
            valtypes: vec![ty.ty],
        };
        validate_value_stack(expected_type, |value_stack| {
            init_expr = Some(read_constant_instructions(
                wasm,
                value_stack,
                &[/* todo!(imported globals tpyes) */],
            )?);

            Ok(())
        })?;

        // At this point, we can assume that `init_expr` is `Some(_)`. `read_constant_instructions` returns a Span or an
        // Error. If an Error is returned it is pushed up the call stack.
        Ok(Self {
            ty,
            init_expr: init_expr.unwrap_validated(),
        })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        Self::read(wasm).unwrap_validated()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub ty: ValType,
    pub is_mut: bool,
}

impl WasmReadable for GlobalType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let ty = ValType::read(wasm)?;
        let is_mut = match wasm.read_u8()? {
            0x00 => false,
            0x01 => true,
            other => return Err(Error::InvalidMutType(other)),
        };
        Ok(Self { ty, is_mut })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let ty = ValType::read_unvalidated(wasm);
        let is_mut = match wasm.read_u8().unwrap_validated() {
            0x00 => false,
            0x01 => true,
            _ => unreachable_validated!(),
        };

        Self { ty, is_mut }
    }
}
