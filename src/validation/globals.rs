use alloc::vec::Vec;

use crate::{
    core::reader::{span::Span, types::global::GlobalType, WasmReader},
    Error, NumType, Result, ValType,
};

/// Read and validate constant expressions.
///
/// This function, alongside [`validate_value_stack()`](crate::validation::validate_value_stack) can be used to validate
/// that a constant expression produces the expected result. The main use case for this is to validate that an
/// initialization expression for a global returns the correct value.
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
pub(crate) fn read_constant_instructions(
    wasm: &mut WasmReader,
    value_stack: &mut Vec<ValType>,
    _globals_ty: &[GlobalType],
) -> Result<Span> {
    let start_pc = wasm.pc;

    let assert_pop_value_stack = |value_stack: &mut Vec<ValType>, expected_ty: ValType| {
        value_stack
            .pop()
            .ok_or(Error::InvalidValueStackType(None))
            .and_then(|ty| {
                (ty == expected_ty)
                    .then_some(())
                    .ok_or(Error::InvalidValueStackType(Some(ty)))
            })
    };

    loop {
        let Ok(first_instr_byte) = wasm.read_u8() else {
            return Err(Error::ExprMissingEnd);
        };
        trace!("Read cosntant instruction byte {first_instr_byte:#X?} ({first_instr_byte})");

        use crate::core::reader::types::opcode::*;
        match first_instr_byte {
            // Missing: ref.null, ref.func, global.get
            END => {
                return Ok(Span::new(start_pc, wasm.pc - start_pc + 1));
            }
            I32_CONST => {
                let _num = wasm.read_var_i32()?;
                value_stack.push(ValType::NumType(NumType::I32));
            }
            I64_CONST => {
                let _num = wasm.read_var_i64()?;
                value_stack.push(ValType::NumType(NumType::I64));
            }
            I32_ADD | I32_SUB | I32_MUL => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I32))?;

                value_stack.push(ValType::NumType(NumType::I32));
            }
            I64_ADD | I64_SUB | I64_MUL => {
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I64))?;
                assert_pop_value_stack(value_stack, ValType::NumType(NumType::I64))?;

                value_stack.push(ValType::NumType(NumType::I64));
            }
            _ => return Err(Error::InvalidInstr(first_instr_byte)),
        }
    }
}
