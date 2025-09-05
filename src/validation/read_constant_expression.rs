use alloc::vec::Vec;

use crate::core::indices::{FuncIdx, GlobalIdx};
use crate::core::reader::span::Span;
use crate::core::reader::types::global::GlobalType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::{NumType, RefType, ValType, ValidationError};

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
    // The globals slice should contain ONLY imported globals IF AND ONLY IF we are calling `read_constant_expression` for local globals instantiation
    // As per https://webassembly.github.io/spec/core/valid/modules.html (bottom of the page):
    //
    //  Globals, however, are not recursive and not accessible within constant expressions when they are defined locally. The effect of defining the limited context C'
    //   for validating certain definitions is that they can only access functions and imported globals and nothing else.
    globals_ty: &[GlobalType],
    num_funcs: usize,
) -> Result<(Span, Vec<FuncIdx>), ValidationError> {
    let start_pc = wasm.pc;
    let mut seen_func_idxs: Vec<FuncIdx> = Vec::new();

    loop {
        let Ok(first_instr_byte) = wasm.read_u8() else {
            return Err(ValidationError::ExprMissingEnd);
        };

        #[cfg(not(debug_assertions))]
        trace!("Read constant instruction byte {first_instr_byte:#X?} ({first_instr_byte})");

        #[cfg(debug_assertions)]
        trace!(
            "Validation - Executing instruction {}",
            opcode_byte_to_str(first_instr_byte)
        );

        use crate::core::reader::types::opcode::*;
        match first_instr_byte {
            END => {
                // The code here for checking the global type was moved to where the global is actually validated
                return Ok((Span::new(start_pc, wasm.pc - start_pc), seen_func_idxs));
            }
            GLOBAL_GET => {
                let global_idx = wasm.read_var_u32()? as GlobalIdx;
                trace!("{:?}", globals_ty);
                let global = globals_ty
                    .get(global_idx)
                    .ok_or(ValidationError::InvalidGlobalIdx(global_idx))?;

                trace!("{:?}", global.ty);
                stack.push_valtype(global.ty);
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
            REF_NULL => {
                stack.push_valtype(ValType::RefType(RefType::read(wasm)?));
            }
            REF_FUNC => {
                let func_idx = wasm.read_var_u32()? as usize;

                // checking for existence suffices for checking whether this function has a valid type.
                if num_funcs <= func_idx {
                    return Err(ValidationError::FunctionIsNotDefined(func_idx));
                }

                // This func_idx is automatically in C.refs. No need to check.
                // as we are single pass validating, add it to C.refs set.
                seen_func_idxs.push(func_idx);

                stack.push_valtype(ValType::RefType(crate::RefType::FuncRef));
            }
            FD_EXTENSIONS => {
                use crate::core::reader::types::opcode::fd_extensions::*;

                let Ok(second_instr) = wasm.read_var_u32() else {
                    return Err(ValidationError::ExprMissingEnd);
                };
                match second_instr {
                    V128_CONST => {
                        for _ in 0..16 {
                            let _data = wasm.read_u8()?;
                        }
                        stack.push_valtype(ValType::VecType);
                    }
                    0x00..=0x0B | 0x0D.. => {
                        trace!("Encountered unknown multi-byte instruction in validation - constant expression - {first_instr_byte:x?} {second_instr}");
                        return Err(ValidationError::InvalidInstr(first_instr_byte));
                    }
                }
            }

            0x00..=0x0A
            | 0x0C..=0x22
            | 0x24..=0x40
            | 0x45..=0xBF
            | 0xC0..=0xCF
            | 0xD1
            | 0xD3..=0xFC
            | 0xFE..=0xFF => {
                trace!("Encountered unknown instruction in validation - constant expression - {first_instr_byte:x?}");
                return Err(ValidationError::InvalidInstr(first_instr_byte));
            }
        }
    }
}
