use alloc::vec::Vec;

use value_stack::Stack;

use crate::core::indices::LocalIdx;
use crate::core::reader::types::{NumType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::locals::Locals;
use crate::execution::value::Value;
use crate::validation::code::read_declared_locals;
use crate::value::{InteropValue, InteropValueList};
use crate::{Result, ValidationInfo};

// TODO
pub(crate) mod assert_validated;
pub(crate) mod label;
pub(crate) mod locals;
pub(crate) mod value;
pub mod value_stack;

pub struct RuntimeInstance<'bytecode, 'validation> {
    validated: &'validation ValidationInfo<'bytecode>,
    // TODO store: Store,
    // TODO module: ModuleInstance,
    // TODO stack: ValueStack,
}

impl<'b, 'v> RuntimeInstance<'b, 'v> {
    pub fn new(validation_info: &'v ValidationInfo<'b>) -> Result<Self> {
        trace!("Starting instantiation of bytecode");

        // TODO execute start function

        Ok(RuntimeInstance {
            validated: &validation_info,
        })
    }
    /// Can only invoke functions with signature `[t1] -> [t2]` as of now.
    pub fn invoke_func<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        fn_idx: usize,
        mut param: Param,
    ) -> Returns {
        let fn_code_span = *self
            .validated
            .code_blocks
            .get(fn_idx)
            .expect("valid fn_idx");

        let func_ty = self
            .validated
            .types
            .get(*self.validated.functions.get(fn_idx).expect("valid fn_idx"))
            .unwrap();

        // TODO check if parameters and return types match the ones in `func_ty`

        let mut wasm = WasmReader::new(self.validated.wasm);
        wasm.move_to(fn_code_span);

        let mut locals = {
            let param_values = param.into_values();
            let local_tys = read_declared_locals(&mut wasm).unwrap_validated();
            Locals::new(param_values.into_iter(), local_tys.into_iter())
        };
        let mut stack = Stack::new();

        loop {
            match wasm.read_u8().unwrap_validated() {
                // end
                0x0B => {
                    break;
                }
                // local.get: [] -> [t]
                0x20 => {
                    let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                    let local = locals.get(local_idx);
                    trace!("Instruction: local.get [] -> [{local:?}]");
                    stack.push_value(local.clone());
                }
                // local.set [t] -> []
                0x21 => {
                    let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                    let local = locals.get_mut(local_idx);
                    let value = stack.pop_value(local.to_ty());
                    trace!("Instruction: local.set [{local:?}] -> []");
                    *local = value;
                }
                // i32.add: [i32 i32] -> [i32]
                0x6A => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1.wrapping_add(v2);

                    trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
                    stack.push_value(res.into());
                }
                // i32.const: [] -> [i32]
                0x41 => {
                    let constant = wasm.read_var_i32().unwrap_validated();
                    trace!("Instruction: i32.const [] -> [{constant}]");
                    stack.push_value(constant.into());
                }
                _ => {}
            }
        }

        let mut values = Returns::TYS
            .iter()
            .map(|ty| stack.pop_value(ty.clone()))
            .collect::<Vec<Value>>();
        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = values.into_iter().rev();
        let ret = Returns::from_values(reversed_values);
        debug!("Successfully invoked function");
        ret
    }
}
