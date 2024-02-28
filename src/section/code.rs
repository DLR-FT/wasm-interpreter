use alloc::vec::Vec;

use crate::section::{SectionHeader, SectionTy};
use crate::wasm::span::Span;
use crate::wasm::types::ValType;
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
    pub fn read_code_section(&mut self, section_header: SectionHeader) -> Result<Vec<Code>> {
        assert_eq!(section_header.ty, SectionTy::Code);

        let codes = self.read_vec(|wasm| wasm.read_code())?;
        debug!("Code section read: {:?}", &codes);
        Ok(codes)
    }

    fn read_code(&mut self) -> Result<Code> {
        let func_size = self.read_var_u32()?;
        let func_span = self.make_span(func_size as usize);

        let locals = self.read_vec(|wasm| wasm.read_local())?;

        let expr = self.read_expr()?;

        Ok(Code {
            func_span,
            locals,
            expr,
        })
    }

    fn read_local(&mut self) -> Result<Local> {
        let n = self.read_var_u32()?;
        let valtype = self.read_valtype()?;

        Ok(Local { n, valtype })
    }

    fn read_expr(&mut self) -> Result<Expr> {
        let mut instructions = Vec::new();
        loop {
            match self.peek_byte() {
                // reached end of expr
                Ok(0x0B) => {
                    let _ = self.read_u8();
                    break;
                }
                Ok(_other) => {
                    instructions.push(self.read_instr()?);
                }
                // reached eof
                Err(_) => return Err(Error::ExprMissingEnd),
            }
        }

        Ok(Expr { instructions })
    }

    fn read_instr(&mut self) -> Result<Instr> {
        // TODO read instr

        while self.peek_byte()? != 0x0B {
            let _ = self.read_u8();
        }

        Ok(Instr::Placeholder)
    }
}
