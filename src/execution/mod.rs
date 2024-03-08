use crate::core::indices::LocalIdx;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::locals::Locals;
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use crate::validation::code::read_declared_locals;
use crate::values::stack::ValueStack;
use crate::values::{WasmValue, WasmValueList};
use crate::{Result, ValidationInfo};

// TODO
pub(crate) mod locals;
pub(crate) mod unwrap_validated;
pub mod values;

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
    pub fn invoke_func<Param: WasmValueList, Returns: WasmValueList>(
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

        // VALIDATION_ASSERT: All valtypes are correct, thus we only care about their sizes
        let locals_sizes = read_declared_locals(&mut wasm)
            .unwrap_validated()
            .into_iter()
            .map(|ty| ty.size());

        let param_bytes = param.into_bytes_list();
        let mut locals = Locals::new(param_bytes.iter().map(|p| &**p), locals_sizes);
        let mut value_stack = ValueStack::new();

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
                    value_stack.push_bytes(local);
                }
                // local.set [t] -> []
                0x21 => {
                    let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                    let local = locals.get_mut(local_idx);
                    let stack_bytes = value_stack.pop_bytes(local.len());
                    local.copy_from_slice(&*stack_bytes);
                }
                // i32.add: [i32 i32] -> [i32]
                0x6A => {
                    let v1 = value_stack.pop::<i32>();
                    let v2 = value_stack.pop::<i32>();
                    value_stack.push(v1.wrapping_add(v2));
                }
                // i32.const: [] -> [i32]
                0x41 => {
                    let constant = wasm.read_var_i32().unwrap_validated();
                    value_stack.push(constant);
                }
                _ => {}
            }
        }

        let ret = value_stack.pop_all::<Returns>();
        debug!("Successfully invoked function");
        ret
    }
}
