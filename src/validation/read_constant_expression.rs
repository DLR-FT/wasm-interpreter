use alloc::vec::Vec;

use crate::core::indices::{FuncIdx, IdxVec, TypeIdx};
use crate::core::reader::span::Span;
use crate::core::reader::types::global::GlobalType;
use crate::core::reader::WasmReader;
use crate::core::utils::ToUsizeExt;
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
    imported_globals: &[GlobalType],
    c_funcs: &IdxVec<FuncIdx, TypeIdx>,
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
                // Unfortunately, we cannot use the GlobalIdx type yet, because
                // constant expressions may only access imported globals.  Wasm
                // specifies that only imported globals may be accessed (see
                // comment on `imported_globals` parameter).
                let imported_global_idx = wasm.read_var_u32()?;

                let global = imported_globals
                    .get(imported_global_idx.into_usize())
                    .ok_or(ValidationError::InvalidGlobalIdx(imported_global_idx))?;

                stack.push_valtype(global.ty);
            }
            I32_CONST => {
                let _num = wasm.read_var_i32()?;
                stack.push_valtype(ValType::NumType(NumType::I32));
            }
            F32_CONST => {
                let _num = wasm.read_f32();
                stack.push_valtype(ValType::NumType(NumType::F32));
            }
            F64_CONST => {
                let _num = wasm.read_f64();
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
                let func_idx = FuncIdx::read_and_validate(wasm, c_funcs)?;

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

                    V128_LOAD
                    | V128_LOAD8X8_S
                    | V128_LOAD8X8_U
                    | V128_LOAD16X4_S
                    | V128_LOAD16X4_U
                    | V128_LOAD32X2_S
                    | V128_LOAD32X2_U
                    | V128_LOAD8_SPLAT
                    | V128_LOAD16_SPLAT
                    | V128_LOAD32_SPLAT
                    | V128_LOAD64_SPLAT
                    | V128_STORE
                    | I8X16_SHUFFLE
                    | I8X16_SWIZZLE
                    | I8X16_SPLAT
                    | I16X8_SPLAT
                    | I32X4_SPLAT
                    | I64X2_SPLAT
                    | F32X4_SPLAT
                    | F64X2_SPLAT
                    | I8X16_EXTRACT_LANE_S
                    | I8X16_EXTRACT_LANE_U
                    | I8X16_REPLACE_LANE
                    | I16X8_EXTRACT_LANE_S
                    | I16X8_EXTRACT_LANE_U
                    | I16X8_REPLACE_LANE
                    | I32X4_EXTRACT_LANE
                    | I32X4_REPLACE_LANE
                    | I64X2_EXTRACT_LANE
                    | I64X2_REPLACE_LANE
                    | F32X4_EXTRACT_LANE
                    | F32X4_REPLACE_LANE
                    | F64X2_EXTRACT_LANE
                    | F64X2_REPLACE_LANE
                    | I8X16_EQ
                    | I8X16_NE
                    | I8X16_LT_S
                    | I8X16_LT_U
                    | I8X16_GT_S
                    | I8X16_GT_U
                    | I8X16_LE_S
                    | I8X16_LE_U
                    | I8X16_GE_S
                    | I8X16_GE_U
                    | I16X8_EQ
                    | I16X8_NE
                    | I16X8_LT_S
                    | I16X8_LT_U
                    | I16X8_GT_S
                    | I16X8_GT_U
                    | I16X8_LE_S
                    | I16X8_LE_U
                    | I16X8_GE_S
                    | I16X8_GE_U
                    | I32X4_EQ
                    | I32X4_NE
                    | I32X4_LT_S
                    | I32X4_LT_U
                    | I32X4_GT_S
                    | I32X4_GT_U
                    | I32X4_LE_S
                    | I32X4_LE_U
                    | I32X4_GE_S
                    | I32X4_GE_U
                    | F32X4_EQ
                    | F32X4_NE
                    | F32X4_LT
                    | F32X4_GT
                    | F32X4_LE
                    | F32X4_GE
                    | F64X2_EQ
                    | F64X2_NE
                    | F64X2_LT
                    | F64X2_GT
                    | F64X2_LE
                    | F64X2_GE
                    | V128_NOT
                    | V128_AND
                    | V128_ANDNOT
                    | V128_OR
                    | V128_XOR
                    | V128_BITSELECT
                    | V128_ANY_TRUE
                    | V128_LOAD8_LANE
                    | V128_LOAD16_LANE
                    | V128_LOAD32_LANE
                    | V128_LOAD64_LANE
                    | V128_STORE8_LANE
                    | V128_STORE16_LANE
                    | V128_STORE32_LANE
                    | V128_STORE64_LANE
                    | V128_LOAD32_ZERO
                    | V128_LOAD64_ZERO
                    | F32X4_DEMOTE_F64X2_ZERO
                    | F64X2_PROMOTE_LOW_F32X4
                    | I8X16_ABS
                    | I8X16_NEG
                    | I8X16_POPCNT
                    | I8X16_ALL_TRUE
                    | I8X16_BITMASK
                    | I8X16_NARROW_I16X8_S
                    | I8X16_NARROW_I16X8_U
                    | F32X4_CEIL
                    | F32X4_FLOOR
                    | F32X4_TRUNC
                    | F32X4_NEAREST
                    | I8X16_SHL
                    | I8X16_SHR_S
                    | I8X16_SHR_U
                    | I8X16_ADD
                    | I8X16_ADD_SAT_S
                    | I8X16_ADD_SAT_U
                    | I8X16_SUB
                    | I8X16_SUB_SAT_S
                    | I8X16_SUB_SAT_U
                    | F64X2_CEIL
                    | F64X2_FLOOR
                    | I8X16_MIN_S
                    | I8X16_MIN_U
                    | I8X16_MAX_S
                    | I8X16_MAX_U
                    | F64X2_TRUNC
                    | I8X16_AVGR_U
                    | I16X8_EXTADD_PAIRWISE_I8X16_S
                    | I16X8_EXTADD_PAIRWISE_I8X16_U
                    | I32X4_EXTADD_PAIRWISE_I16X8_S
                    | I32X4_EXTADD_PAIRWISE_I16X8_U
                    | I16X8_ABS
                    | I16X8_NEG
                    | I16X8_Q15MULRSAT_S
                    | I16X8_ALL_TRUE
                    | I16X8_BITMASK
                    | I16X8_NARROW_I32X4_S
                    | I16X8_NARROW_I32X4_U
                    | I16X8_EXTEND_LOW_I8X16_S
                    | I16X8_EXTEND_HIGH_I8X16_S
                    | I16X8_EXTEND_LOW_I8X16_U
                    | I16X8_EXTEND_HIGH_I8X16_U
                    | I16X8_SHL
                    | I16X8_SHR_S
                    | I16X8_SHR_U
                    | I16X8_ADD
                    | I16X8_ADD_SAT_S
                    | I16X8_ADD_SAT_U
                    | I16X8_SUB
                    | I16X8_SUB_SAT_S
                    | I16X8_SUB_SAT_U
                    | F64X2_NEAREST
                    | I16X8_MUL
                    | I16X8_MIN_S
                    | I16X8_MIN_U
                    | I16X8_MAX_S
                    | I16X8_MAX_U
                    | I16X8_AVGR_U
                    | I16X8_EXTMUL_LOW_I8X16_S
                    | I16X8_EXTMUL_HIGH_I8X16_S
                    | I16X8_EXTMUL_LOW_I8X16_U
                    | I16X8_EXTMUL_HIGH_I8X16_U
                    | I32X4_ABS
                    | I32X4_NEG
                    | I32X4_ALL_TRUE
                    | I32X4_BITMASK
                    | I32X4_EXTEND_LOW_I16X8_S
                    | I32X4_EXTEND_HIGH_I16X8_S
                    | I32X4_EXTEND_LOW_I16X8_U
                    | I32X4_EXTEND_HIGH_I16X8_U
                    | I32X4_SHL
                    | I32X4_SHR_S
                    | I32X4_SHR_U
                    | I32X4_ADD
                    | I32X4_SUB
                    | I32X4_MUL
                    | I32X4_MIN_S
                    | I32X4_MIN_U
                    | I32X4_MAX_S
                    | I32X4_MAX_U
                    | I32X4_DOT_I16X8_S
                    | I32X4_EXTMUL_LOW_I16X8_S
                    | I32X4_EXTMUL_HIGH_I16X8_S
                    | I32X4_EXTMUL_LOW_I16X8_U
                    | I32X4_EXTMUL_HIGH_I16X8_U
                    | I64X2_ABS
                    | I64X2_NEG
                    | I64X2_ALL_TRUE
                    | I64X2_BITMASK
                    | I64X2_EXTEND_LOW_I32X4_S
                    | I64X2_EXTEND_HIGH_I32X4_S
                    | I64X2_EXTEND_LOW_I32X4_U
                    | I64X2_EXTEND_HIGH_I32X4_U
                    | I64X2_SHL
                    | I64X2_SHR_S
                    | I64X2_SHR_U
                    | I64X2_ADD
                    | I64X2_SUB
                    | I64X2_MUL
                    | I64X2_EQ
                    | I64X2_NE
                    | I64X2_LT_S
                    | I64X2_GT_S
                    | I64X2_LE_S
                    | I64X2_GE_S
                    | I64X2_EXTMUL_LOW_I32X4_S
                    | I64X2_EXTMUL_HIGH_I32X4_S
                    | I64X2_EXTMUL_LOW_I32X4_U
                    | I64X2_EXTMUL_HIGH_I32X4_U
                    | F32X4_ABS
                    | F32X4_NEG
                    | F32X4_SQRT
                    | F32X4_ADD
                    | F32X4_SUB
                    | F32X4_MUL
                    | F32X4_DIV
                    | F32X4_MIN
                    | F32X4_MAX
                    | F32X4_PMIN
                    | F32X4_PMAX
                    | F64X2_ABS
                    | F64X2_NEG
                    | F64X2_SQRT
                    | F64X2_ADD
                    | F64X2_SUB
                    | F64X2_MUL
                    | F64X2_DIV
                    | F64X2_MIN
                    | F64X2_MAX
                    | F64X2_PMIN
                    | F64X2_PMAX
                    | I32X4_TRUNC_SAT_F32X4_S
                    | I32X4_TRUNC_SAT_F32X4_U
                    | F32X4_CONVERT_I32X4_S
                    | F32X4_CONVERT_I32X4_U
                    | I32X4_TRUNC_SAT_F64X2_S_ZERO
                    | I32X4_TRUNC_SAT_F64X2_U_ZERO
                    | F64X2_CONVERT_LOW_I32X4_S
                    | F64X2_CONVERT_LOW_I32X4_U => {
                        return Err(ValidationError::InvalidConstMultiByteInstr(
                            first_instr_byte,
                            second_instr,
                        ))
                    }

                    F32X4_RELAXED_MADD
                    | F32X4_RELAXED_MAX
                    | F32X4_RELAXED_MIN
                    | F32X4_RELAXED_NMADD
                    | F64X2_RELAXED_MADD
                    | F64X2_RELAXED_MAX
                    | F64X2_RELAXED_MIN
                    | F64X2_RELAXED_NMADD
                    | I8X16_RELAXED_LANESELECT
                    | I16X8_RELAXED_LANESELECT
                    | I32X4_RELAXED_LANESELECT
                    | I64X2_RELAXED_LANESELECT
                    | I32X4_RELAXED_TRUNC_F32X4_S
                    | I32X4_RELAXED_TRUNC_F32X4_U
                    | I32X4_RELAXED_TRUNC_F64X2_S_ZERO
                    | I32X4_RELAXED_TRUNC_F64X2_U_ZERO
                    | I8X16_RELAXED_SWIZZLE
                    | 154
                    | 187
                    | 194
                    | 256.. => {
                        trace!("Encountered unknown multi-byte instruction in validation - constant expression - {first_instr_byte:x?} {second_instr}");
                        return Err(ValidationError::InvalidMultiByteInstr(
                            first_instr_byte,
                            second_instr,
                        ));
                    }
                }
            }

            UNREACHABLE | NOP | BLOCK | LOOP | IF | ELSE | BR | BR_IF | BR_TABLE | RETURN
            | CALL | DROP | SELECT | SELECT_T | CALL_INDIRECT | LOCAL_GET | LOCAL_SET
            | LOCAL_TEE | GLOBAL_SET | TABLE_GET | TABLE_SET | I32_LOAD | I64_LOAD | F32_LOAD
            | F64_LOAD | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U | I64_LOAD8_S
            | I64_LOAD8_U | I64_LOAD16_S | I64_LOAD16_U | I64_LOAD32_S | I64_LOAD32_U
            | I32_STORE | I64_STORE | F32_STORE | F64_STORE | I32_STORE8 | I32_STORE16
            | I64_STORE8 | I64_STORE16 | I64_STORE32 | MEMORY_SIZE | MEMORY_GROW | I32_EQZ
            | I32_EQ | I32_NE | I32_LT_S | I32_LT_U | I32_GT_S | I32_GT_U | I32_LE_S | I32_LE_U
            | I32_GE_S | I32_GE_U | I64_EQZ | I64_EQ | I64_NE | I64_LT_S | I64_LT_U | I64_GT_S
            | I64_GT_U | I64_LE_S | I64_LE_U | I64_GE_S | I64_GE_U | F32_EQ | F32_NE | F32_LT
            | F32_GT | F32_LE | F32_GE | F64_EQ | F64_NE | F64_LT | F64_GT | F64_LE | F64_GE
            | I32_ADD | I32_SUB | I32_MUL | I32_DIV_S | I32_DIV_U | I32_REM_S | I32_CLZ
            | I32_CTZ | I32_POPCNT | I32_REM_U | I32_AND | I32_OR | I32_XOR | I32_SHL
            | I32_SHR_S | I32_SHR_U | I32_ROTL | I32_ROTR | I64_CLZ | I64_CTZ | I64_POPCNT
            | I64_ADD | I64_SUB | I64_MUL | I64_DIV_S | I64_DIV_U | I64_REM_S | I64_REM_U
            | I64_AND | I64_OR | I64_XOR | I64_SHL | I64_SHR_S | I64_SHR_U | I64_ROTL
            | I64_ROTR | F32_ABS | F32_NEG | F32_CEIL | F32_FLOOR | F32_TRUNC | F32_NEAREST
            | F32_SQRT | F32_ADD | F32_SUB | F32_MUL | F32_DIV | F32_MIN | F32_MAX
            | F32_COPYSIGN | F64_ABS | F64_NEG | F64_CEIL | F64_FLOOR | F64_TRUNC | F64_NEAREST
            | F64_SQRT | F64_ADD | F64_SUB | F64_MUL | F64_DIV | F64_MIN | F64_MAX
            | F64_COPYSIGN | I32_WRAP_I64 | I32_TRUNC_F32_S | I32_TRUNC_F32_U | I32_TRUNC_F64_S
            | I32_TRUNC_F64_U | I64_EXTEND_I32_S | I64_EXTEND_I32_U | I64_TRUNC_F32_S
            | I64_TRUNC_F32_U | I64_TRUNC_F64_S | I64_TRUNC_F64_U | F32_CONVERT_I32_S
            | F32_CONVERT_I32_U | F32_CONVERT_I64_S | F32_CONVERT_I64_U | F32_DEMOTE_F64
            | F64_CONVERT_I32_S | F64_CONVERT_I32_U | F64_CONVERT_I64_S | F64_CONVERT_I64_U
            | F64_PROMOTE_F32 | I32_REINTERPRET_F32 | I64_REINTERPRET_F64 | F32_REINTERPRET_I32
            | F64_REINTERPRET_I64 | REF_IS_NULL | FC_EXTENSIONS | I32_EXTEND8_S
            | I32_EXTEND16_S | I64_EXTEND8_S | I64_EXTEND16_S | I64_EXTEND32_S => {
                return Err(ValidationError::InvalidConstInstr(first_instr_byte))
            }

            0x06..=0x0A
            | 0x12..=0x19
            | 0x1C..=0x1F
            | 0x25..=0x27
            | 0xC0..=0xFA
            | 0xFB
            | 0xFE
            | 0xFF => {
                trace!("Encountered unknown instruction in validation - constant expression - {first_instr_byte:x?}");
                return Err(ValidationError::InvalidInstr(first_instr_byte));
            }
        }
    }
}
