use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::iter;

use crate::section::r#type::TypeStorage;
use crate::wasm::indices::LocalIdx;
use crate::wasm::span::Span;
use crate::wasm::types::{FuncType, NumType, ValType};
use crate::wasm::Wasm;
use crate::{Error, Result};

#[derive(Debug)]
pub struct Code {
    func_span: Span,
    locals: Vec<Local>,
    expr: Expr,
}

/// One or more locals of the same type in a function
#[derive(Debug)]
pub struct Local {
    n: u32,
    valtype: ValType,
}

#[derive(Debug)]
struct Expr {
    instructions: Vec<Instr>,
}

#[derive(Debug)]
enum Instr {
    Placeholder,
}

impl<'a> Wasm<'a> {
    pub fn read_code_section(&mut self, types: &TypeStorage) -> Result<()> {
        self.read_vec(|wasm| {
            // TODO hardcoded funcidx=0 for now, because only one function is supported
            let func_idx = 0;
            let func_ty = types[0].clone();
            trace!("Validating function with index {func_idx}");

            let _func_size = wasm.read_var_u32()?;

            let locals = {
                let params = func_ty.params.valtypes.iter().cloned();
                let declared_locals = wasm.read_declared_locals()?;
                params.chain(declared_locals).collect::<Vec<ValType>>()
            };

            validate_value_stack(func_ty, |value_stack| {
                wasm.read_instructions(value_stack, &locals)
            })
        })
        .map(|_| ())
    }

    fn read_declared_locals(&mut self) -> Result<Vec<(ValType)>> {
        let locals = self.read_vec(|wasm| {
            let n = wasm.read_var_u32()? as usize;
            let valtype = wasm.read_valtype()?;

            Ok((n, valtype))
        })?;

        // Flatten local types where n > 1
        let locals = locals
            .into_iter()
            .flat_map(|entry| iter::repeat(entry.1).take(entry.0))
            .collect::<Vec<ValType>>();

        Ok(locals)
    }

    fn read_instructions(
        &mut self,
        value_stack: &mut VecDeque<ValType>,
        locals: &[ValType],
    ) -> Result<()> {
        loop {
            let Ok(instr) = self.read_u8() else {
                return Err(Error::ExprMissingEnd);
            };
            trace!("Read instruction byte {instr:#x?} ({instr})");
            match instr {
                // nop
                0x01 => {}
                // end
                0x0B => {
                    return Ok(());
                }
                // local.get: [] -> [t]
                0x20 => {
                    let local_idx = self.read_var_u32()? as LocalIdx;
                    let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                    value_stack.push_back(local_ty.clone());
                }
                // local.set [t] -> [0]
                0x21 => {
                    let local_idx = self.read_var_u32()? as LocalIdx;
                    let local_ty = locals.get(local_idx).ok_or(Error::InvalidLocalIdx)?;
                    let popped = value_stack.pop_back();
                    if popped.as_ref() != Some(local_ty) {
                        return Err(Error::InvalidValueStackType(popped));
                    }
                }
                // i32.add: [i32 i32] -> [i32]
                0x6A => {
                    let ty1 = value_stack
                        .pop_back()
                        .ok_or(Error::InvalidValueStackType(None))?;
                    let ty2 = value_stack
                        .pop_back()
                        .ok_or(Error::InvalidValueStackType(None))?;
                    let ValType::NumType(NumType::I32) = ty1 else {
                        return Err(Error::InvalidValueStackType(Some(ty1)));
                    };
                    let ValType::NumType(NumType::I32) = ty2 else {
                        return Err(Error::InvalidValueStackType(Some(ty2)));
                    };
                }
                // i32.const: [] -> [i32]
                0x41 => {
                    let _num = self.read_var_i32()?;
                    value_stack.push_back(ValType::NumType(NumType::I32));
                }
                other => {
                    return Err(Error::InvalidInstr(other));
                }
            }
        }
    }
}

fn validate_value_stack<F>(func_ty: FuncType, mut f: F) -> Result<()>
where
    F: FnOnce(&mut VecDeque<ValType>) -> Result<()>,
{
    let mut value_stack: VecDeque<ValType> = VecDeque::new();
    // TODO check for correct valtype order
    value_stack.extend(func_ty.params.valtypes);

    f(&mut value_stack)?;

    // TODO also check here for correct valtype order
    if value_stack != func_ty.returns.valtypes {
        return Err(Error::EndInvalidValueStack);
    }
    Ok(())
}
