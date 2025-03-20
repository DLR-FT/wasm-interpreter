use crate::core::reader::span::Span;
use crate::core::reader::types::global::GlobalType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::{Error, NumType, RefType, Result, ValType};

use super::validation_stack::ValidationStack;

/// Read and validate constant expressions.
///
/// This function is used to validate that a constant expression produces the expected result. The main use case for
/// this is to validate that an initialization expression for a global returns the correct value.
///
/// Note: to be valid, constant expressions may not leave garbage data on the stack. It may leave only what is expected
/// and nothing more.
///
/// Valid constant instructions are:
/// - Core: <https://webassembly.github.io/spec/core/valid/instructions.html#valid-constant>
/// - Extended Proposal: <https://webassembly.github.io/extended-const/core/valid/instructions.html#valid-constant>
///
/// # The Wonders of `global.get`
/// The `global.get` instruction is quite picky by nature. To make a long story short, there are two rules to follow to
/// be able to use this expression.
///
/// ## 1. The referenced global must be imported
/// Take the example code:
/// ```wat
/// (module
///     (global (export "g") (mut i32) (
///         i32.add (i32.const 1) (i32.const 2)
///     ))
///
///     (global (export "h1") i32 (
///         i32.const 1
///     ))
///
///     (global (export "h2") i32 (
///         global.get 1
///     ))
///
///     (func (export "f")
///         i32.const 100
///         global.set 0))
/// ```
///
/// When compiling with wat2wasm, the following error is thrown:
/// ```wat
/// Error: validate failed:
/// test.wast:11:24: error: initializer expression can only reference an imported global
///             global.get 1
///                        ^
/// ```
///
/// When compiling the code with the latest dev build of wasmtime, the following error is thrown:
/// ```wat
/// failed to parse WebAssembly module
///
/// Caused by:
///     constant expression required: global.get of locally defined global (at offset 0x24)
/// ```
///
/// ## 2. The referenced global must be immutable
///
///```wat
/// (module
///     (import "env" "g" (global (mut i32)))
///     (global (export "h") (mut i32) (
///         i32.add (i32.const 1) (global.get 0)
///     ))
///   )
/// ```
///
/// When compiling with wat2wasm, the following error is thrown:
/// ```wat
/// Error: validate failed:
/// test.wast:4:27: error: initializer expression cannot reference a mutable global
///     i32.add (i32.const 1) (global.get 0)
/// ```
///
/// # Note
/// The following instructions are not yet supported:
/// - `ref.null`
/// - `ref.func`
/// - `global.get`
pub fn read_constant_expression(
    wasm: &mut WasmReader,
    stack: &mut ValidationStack,
    this_global_valtype: Option<ValType>,
    _globals_ty: Option<&[GlobalType]>,
    funcs: Option<&[usize]>,
) -> Result<Span> {
    let start_pc = wasm.pc;

    loop {
        let Ok(first_instr_byte) = wasm.read_u8() else {
            return Err(Error::ExprMissingEnd);
        };
        trace!("Read constant instruction byte {first_instr_byte:#X?} ({first_instr_byte})");

        use crate::core::reader::types::opcode::*;
        match first_instr_byte {
            // Missing: ref.null, ref.func, global.get
            END => {
                // The stack must only contain the global's valtype
                if this_global_valtype.is_some() {
                    stack.assert_val_types(&[this_global_valtype.unwrap()])?;
                }
                return Ok(Span::new(start_pc, wasm.pc - start_pc));
            }
            I32_CONST => {
                let _num = wasm.read_var_i32()?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            F32_CONST => {
                let _num = wasm.read_var_f32();
                stack.push_valtype(ValType::NumType(NumType::F32));
            }
            F64_CONST => {
                let _num = wasm.read_var_f64();
                stack.push_valtype(ValType::NumType(NumType::F64));
            }
            I64_CONST => {
                let _num = wasm.read_var_i64()?;
                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            I32_ADD | I32_SUB | I32_MUL => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I32))?;

                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            I64_ADD | I64_SUB | I64_MUL => {
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;
                stack.assert_pop_val_type(ValType::NumType(NumType::I64))?;

                stack.push_valtype(ValType::NumType(NumType::I64));
            }
            REF_NULL => {
                stack.push_valtype(ValType::RefType(RefType::read(wasm)?));
            }
            REF_FUNC => {
                let func_idx = wasm.read_var_u32()? as usize;
                match funcs {
                    Some(funcs) => {
                        if func_idx >= funcs.len() {
                            return Err(Error::FunctionIsNotDefined(func_idx));
                        }
                    }
                    None => {
                        return Err(Error::FunctionIsNotDefined(u32::MAX as usize));
                    }
                }

                stack.push_valtype(ValType::RefType(crate::RefType::FuncRef));
            }
            _ => return Err(Error::InvalidInstr(first_instr_byte)),
        }
    }
}
