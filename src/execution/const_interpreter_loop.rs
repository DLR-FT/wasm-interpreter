use crate::{
    assert_validated::UnwrapValidatedExt, core::reader::WasmReader, value_stack::Stack, NumType,
    ValType,
};

/// Execute a previosly-validated constant expression. These type of expressions are used for initializing global
/// variables.
///
/// # Arguments
/// - `wasm` - a [WasmReader] whose [program counter](WasmReader::pc) is set at the beginning of the constant
///   expression. Reader will be consumed.
/// - `stack` - a [Stack]. It is preferrable for it to be clean, but that is not required. As long as the executed code
///   is validated, the values on this stack will remain the same except for the addition of the return value of this
///   code sequence. A global's final value can be popped off the top of the stack.
/// - `imported_globals` (TODO) - instances of all imported globals. They are required as local globals can reference
///   imported globals in their initialization.
///
/// # Safety
/// This function assumes that the expression has been validated. Passing unvalidated code will likely result in a
/// panic, or undefined behaviour.
///
/// # Note
/// The following instructions are not yet supported:
/// - `ref.null`
/// - `ref.func`
/// - `global.get`
pub(crate) fn run_const(
    mut wasm: WasmReader,
    stack: &mut Stack,
    _imported_globals: (), /*todo!*/
) {
    use crate::core::reader::types::opcode::*;
    loop {
        let first_instr_byte = wasm.read_u8().unwrap_validated();

        match first_instr_byte {
            END => {
                break;
            }
            I32_CONST => {
                let constant = wasm.read_var_i32().unwrap_validated();
                trace!("Constant instruction: i32.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            I32_ADD => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.wrapping_add(v2);

                trace!("Constant instruction: i32.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_SUB => {
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.wrapping_sub(v2);

                trace!("Constant instruction: i32.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I32_MUL => {
                let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                let res = v1.wrapping_mul(v2);

                trace!("Constant instruction: i32.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_CONST => {
                let constant = wasm.read_var_i64().unwrap_validated();
                trace!("Constant instruction: i64.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            I64_ADD => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.wrapping_add(v2);

                trace!("Constant instruction: i64.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_SUB => {
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.wrapping_sub(v2);

                trace!("Constant instruction: i64.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            I64_MUL => {
                let v1: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let v2: i64 = stack.pop_value(ValType::NumType(NumType::I64)).into();
                let res = v1.wrapping_mul(v2);

                trace!("Constant instruction: i64.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            other => {
                panic!("Unknown constant instruction {other:#x}, validation allowed an unimplemented instruction.");
            }
        }
    }
}
