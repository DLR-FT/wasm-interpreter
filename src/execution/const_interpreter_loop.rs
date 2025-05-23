use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::GlobalIdx,
        reader::{span::Span, WasmReadable, WasmReader},
    },
    value::{self, FuncAddr, Ref},
    value_stack::Stack,
    ModuleInst, NumType, RefType, Store, ValType, Value,
};

// TODO update this documentation
/// Execute a previosly-validated constant expression. These type of expressions are used for initializing global
/// variables, data and element segments.
///
/// # Arguments
/// TODO
///
/// # Safety
/// This function assumes that the expression has been validated. Passing unvalidated code will likely result in a
/// panic, or undefined behaviour.
// TODO this signature might change to support hooks or match the spec better
pub(crate) fn run_const(
    wasm: &mut WasmReader,
    stack: &mut Stack,
    module: &ModuleInst,
    store: &Store,
) {
    use crate::core::reader::types::opcode::*;
    loop {
        let first_instr_byte = wasm.read_u8().unwrap_validated();

        #[cfg(debug_assertions)]
        crate::core::utils::print_beautiful_instruction_name_1_byte(first_instr_byte, wasm.pc);

        #[cfg(not(debug_assertions))]
        trace!("Read instruction byte {first_instr_byte:#04X?} ({first_instr_byte}) at wasm_binary[{}]", wasm.pc);

        match first_instr_byte {
            END => {
                trace!("Constant instruction: END");
                break;
            }
            GLOBAL_GET => {
                let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;

                //TODO replace double indirection
                let global = &store.globals[module.globals[global_idx]];

                trace!(
                    "Constant instruction: global.get [{global_idx}] -> [{:?}]",
                    global
                );
                stack.push_value(global.value);
            }
            I32_CONST => {
                let constant = wasm.read_var_i32().unwrap_validated();
                trace!("Constant instruction: i32.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            F32_CONST => {
                let constant = value::F32::from_bits(wasm.read_var_f32().unwrap_validated());
                trace!("Constanting instruction: f32.const [] -> [{constant}]");
                stack.push_value(constant.into());
            }
            F64_CONST => {
                let constant = value::F64::from_bits(wasm.read_var_f64().unwrap_validated());
                trace!("Constanting instruction: f64.const [] -> [{constant}]");
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
            F32_ADD => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 + v2;

                trace!("Instruction: f32.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_SUB => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 - v2;

                trace!("Instruction: f32.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_MUL => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 * v2;

                trace!("Instruction: f32.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F32_DIV => {
                let v2: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let v1: value::F32 = stack.pop_value(ValType::NumType(NumType::F32)).into();
                let res: value::F32 = v1 / v2;

                trace!("Instruction: f32.div [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_ADD => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 + v2;

                trace!("Instruction: f64.add [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_SUB => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 - v2;

                trace!("Instruction: f64.sub [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_MUL => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 * v2;

                trace!("Instruction: f64.mul [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            F64_DIV => {
                let v2: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let v1: value::F64 = stack.pop_value(ValType::NumType(NumType::F64)).into();
                let res: value::F64 = v1 / v2;

                trace!("Instruction: f64.div [{v1} {v2}] -> [{res}]");
                stack.push_value(res.into());
            }
            REF_NULL => {
                let reftype = RefType::read_unvalidated(wasm);

                stack.push_value(Value::Ref(reftype.to_null_ref()));
                trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
            }
            REF_FUNC => {
                // we already checked for the func_idx to be in bounds during validation
                let func_idx = wasm.read_var_u32().unwrap_validated() as usize;
                // TODO replace double indirection
                stack.push_value(Value::Ref(Ref::Func(FuncAddr::new(Some(
                    module.functions[func_idx],
                )))));
            }
            other => {
                unreachable!("Unknown constant instruction {other:#x}, validation allowed an unimplemented instruction.");
            }
        }
    }
}

pub(crate) fn run_const_span(
    wasm: &[u8],
    span: &Span,
    module: &ModuleInst,
    store: &Store,
) -> Option<Value> {
    let mut wasm = WasmReader::new(wasm);

    wasm.move_start_to(*span).unwrap_validated();

    let mut stack = Stack::new();
    run_const(&mut wasm, &mut stack, module, store);

    stack.peek_unknown_value()
}
