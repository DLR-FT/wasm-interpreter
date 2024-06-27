use alloc::vec::Vec;

use value_stack::Stack;

use crate::core::indices::{FuncIdx, GlobalIdx, LocalIdx};
use crate::core::reader::types::memarg::MemArg;
use crate::core::reader::types::{FuncType, NumType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::{EmptyHookSet, HookSet};
use crate::execution::locals::Locals;
use crate::execution::store::{FuncInst, GlobalInst, MemInst, Store};
use crate::execution::value::Value;
use crate::validation::code::read_declared_locals;
use crate::value::InteropValueList;
use crate::Error::RuntimeError;
use crate::RuntimeError::{DivideBy0, UnrepresentableResult};
use crate::{Result, ValidationInfo};

// TODO
pub(crate) mod assert_validated;
pub mod hooks;
pub(crate) mod label;
pub(crate) mod locals;
pub(crate) mod store;
pub(crate) mod value;
pub mod value_stack;

pub struct RuntimeInstance<'b, H = EmptyHookSet>
where
    H: HookSet,
{
    pub wasm_bytecode: &'b [u8],
    types: Vec<FuncType>,
    store: Store,
    pub hook_set: H,
}

impl<'b> RuntimeInstance<'b, EmptyHookSet> {
    pub fn new(validation_info: &'_ ValidationInfo<'b>) -> Result<Self> {
        Self::new_with_hooks(validation_info, EmptyHookSet)
    }
}

impl<'b, H> RuntimeInstance<'b, H>
where
    H: HookSet,
{
    pub fn new_with_hooks(validation_info: &'_ ValidationInfo<'b>, hook_set: H) -> Result<Self> {
        trace!("Starting instantiation of bytecode");

        let store = Self::init_store(validation_info);

        let mut instance = RuntimeInstance {
            wasm_bytecode: validation_info.wasm,
            types: validation_info.types.clone(),
            store,
            hook_set,
        };

        if let Some(start) = validation_info.start {
            let result = instance.invoke_func::<(), ()>(start, ());
            result?;
        }

        Ok(instance)
    }

    /// Can only invoke functions with signature `[t1] -> [t2]` as of now.
    pub fn invoke_func<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        func_idx: FuncIdx,
        param: Param,
    ) -> Result<Returns> {
        let func_inst = self.store.funcs.get(func_idx).expect("valid FuncIdx");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

        // Check correct function parameters and return types
        if func_ty.params.valtypes != Param::TYS {
            panic!("Invalid `Param` generics");
        }
        if func_ty.returns.valtypes != Returns::TYS {
            panic!("Invalid `Returns` generics");
        }

        let mut stack = Stack::new();

        // Push parameters on stack
        for parameter in param.into_values() {
            stack.push_value(parameter);
        }

        let error = self.function(func_idx, &mut stack);
        error?;

        // Pop return values from stack
        let return_values = Returns::TYS
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret = Returns::from_values(reversed_values);
        debug!("Successfully invoked function");
        Ok(ret)
    }

    /// Interprets a functions. Parameters and return values are passed on the stack.
    fn function(&mut self, idx: FuncIdx, stack: &mut Stack) -> Result<()> {
        let inst = self.store.funcs.get(idx).unwrap_validated();

        // Pop parameters from stack
        let func_type = self.types.get(inst.ty).unwrap_validated();
        let mut params: Vec<Value> = func_type
            .params
            .valtypes
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect();
        params.reverse();

        // Create locals from parameters and declared locals
        let mut locals = Locals::new(params.into_iter(), inst.locals.iter().cloned());

        // Start reading the function's instructions
        let mut wasm = WasmReader::new(self.wasm_bytecode);
        wasm.move_start_to(inst.code_expr);

        use crate::core::reader::types::opcode::*;
        loop {
            // call the instruction hook
            #[cfg(feature = "hooks")]
            H::instruction_hook(self);

            let first_instr_byte = wasm.read_u8().unwrap_validated();

            match first_instr_byte {
                END => {
                    break;
                }
                LOCAL_GET => {
                    let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                    let local = locals.get(local_idx);
                    trace!("Instruction: local.get [] -> [{local:?}]");
                    stack.push_value(local.clone());
                }
                LOCAL_SET => {
                    let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                    let local = locals.get_mut(local_idx);
                    let value = stack.pop_value(local.to_ty());
                    trace!("Instruction: local.set [{local:?}] -> []");
                    *local = value;
                }
                GLOBAL_GET => {
                    let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                    let global = self.store.globals.get(global_idx).unwrap_validated();

                    stack.push_value(global.value.clone());
                }
                GLOBAL_SET => {
                    let global_idx = wasm.read_var_u32().unwrap_validated() as GlobalIdx;
                    let global = self.store.globals.get_mut(global_idx).unwrap_validated();

                    global.value = stack.pop_value(global.global.ty.ty)
                }
                I32_LOAD => {
                    let memarg = MemArg::read_unvalidated(&mut wasm);
                    let relative_address: u32 =
                        stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let mem = self.store.mems.first().unwrap_validated(); // there is only one memory allowed as of now

                    let data: u32 = {
                        // The spec states that this should be a 33 bit integer
                        // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                        let _address = memarg.offset.checked_add(relative_address);
                        let data = memarg
                            .offset
                            .checked_add(relative_address)
                            .and_then(|address| {
                                let address = address as usize;
                                mem.data.get(address..(address + 4))
                            })
                            .expect("TODO trap here");

                        let data: [u8; 4] = data.try_into().expect("this to be exactly 4 bytes");
                        u32::from_le_bytes(data)
                    };

                    stack.push_value(Value::I32(data));
                    trace!("Instruction: i32.load [{relative_address}] -> [{data}]");
                }
                I32_STORE => {
                    let memarg = MemArg::read_unvalidated(&mut wasm);

                    let data_to_store: u32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let relative_address: u32 =
                        stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let mem = self.store.mems.get_mut(0).unwrap_validated(); // there is only one memory allowed as of now

                    // The spec states that this should be a 33 bit integer
                    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
                    let address = memarg.offset.checked_add(relative_address);
                    let memory_location = address
                        .and_then(|address| {
                            let address = address as usize;
                            mem.data.get_mut(address..(address + 4))
                        })
                        .expect("TODO trap here");

                    memory_location.copy_from_slice(&data_to_store.to_le_bytes());
                    trace!("Instruction: i32.store [{relative_address} {data_to_store}] -> []");
                }
                I32_CONST => {
                    let constant = wasm.read_var_i32().unwrap_validated();
                    trace!("Instruction: i32.const [] -> [{constant}]");
                    stack.push_value(constant.into());
                }
                I32_CLZ => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1.leading_zeros() as i32;

                    trace!("Instruction: i32.clz [{v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_CTZ => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1.trailing_zeros() as i32;

                    trace!("Instruction: i32.ctz [{v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_POPCNT => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1.count_ones() as i32;

                    trace!("Instruction: i32.popcnt [{v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_ADD => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1.wrapping_add(v2);

                    trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_MUL => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1.wrapping_mul(v2);

                    trace!("Instruction: i32.mul [{v1} {v2}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_DIV_S => {
                    let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    if dividend == 0 {
                        return Err(RuntimeError(DivideBy0));
                    }
                    if divisor == i32::MIN && dividend == -1 {
                        return Err(RuntimeError(UnrepresentableResult));
                    }

                    let res = divisor / dividend;

                    trace!("Instruction: i32.div_s [{divisor} {dividend}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_DIV_U => {
                    let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let dividend = dividend as u32;
                    let divisor = divisor as u32;

                    if dividend == 0 {
                        return Err(RuntimeError(DivideBy0));
                    }

                    let res = (divisor / dividend) as i32;

                    trace!("Instruction: i32.div_u [{divisor} {dividend}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_REM_S => {
                    let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    if dividend == 0 {
                        return Err(RuntimeError(DivideBy0));
                    }

                    let res = divisor.checked_rem(dividend);
                    let res = res.unwrap_or_default();

                    trace!("Instruction: i32.rem_s [{divisor} {dividend}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_REM_U => {
                    let dividend: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let divisor: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let dividend = dividend as u32;
                    let divisor = divisor as u32;

                    if dividend == 0 {
                        return Err(RuntimeError(DivideBy0));
                    }

                    let res = divisor.checked_rem(dividend);
                    let res = res.unwrap_or_default() as i32;

                    trace!("Instruction: i32.rem_u [{divisor} {dividend}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_AND => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1 & v2;

                    trace!("Instruction: i32.and [{v1} {v2}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_OR => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1 | v2;

                    trace!("Instruction: i32.or [{v1} {v2}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_XOR => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v1 ^ v2;

                    trace!("Instruction: i32.xor [{v1} {v2}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_SHL => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let res = v2.wrapping_shl(v1 as u32);

                    trace!("Instruction: i32.shl [{v2} {v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_SHR_S => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let res = v2.wrapping_shr(v1 as u32);

                    trace!("Instruction: i32.shr_s [{v2} {v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_SHR_U => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let res = (v2 as u32).wrapping_shr(v1 as u32) as i32;

                    trace!("Instruction: i32.shr_u [{v2} {v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_ROTL => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let res = v2.rotate_left(v1 as u32);

                    trace!("Instruction: i32.rotl [{v2} {v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                I32_ROTR => {
                    let v1: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();
                    let v2: i32 = stack.pop_value(ValType::NumType(NumType::I32)).into();

                    let res = v2.rotate_right(v1 as u32);

                    trace!("Instruction: i32.rotr [{v2} {v1}] -> [{res}]");
                    stack.push_value(res.into());
                }
                other => {
                    trace!("Unknown instruction {other:#x}, skipping..");
                }
            }
        }
        Ok(())
    }

    fn init_store(validation_info: &ValidationInfo) -> Store {
        let function_instances: Vec<FuncInst> = {
            let mut wasm_reader = WasmReader::new(validation_info.wasm);

            let functions = validation_info.functions.iter();
            let func_blocks = validation_info.func_blocks.iter();

            functions
                .zip(func_blocks)
                .map(|(ty, func)| {
                    wasm_reader.move_start_to(*func);

                    let (locals, bytes_read) = wasm_reader
                        .measure_num_read_bytes(read_declared_locals)
                        .unwrap_validated();

                    let code_expr = wasm_reader.make_span(func.len() - bytes_read);

                    FuncInst {
                        ty: *ty,
                        locals,
                        code_expr,
                    }
                })
                .collect()
        };

        let memory_instances: Vec<MemInst> = validation_info
            .memories
            .iter()
            .map(|ty| MemInst::new(*ty))
            .collect();

        let global_instances: Vec<GlobalInst> = validation_info
            .globals
            .iter()
            .map(|global| {
                // TODO execute `global.init_expr` to get initial value. For now just use a default value.
                let value = Value::default_from_ty(global.ty.ty);

                GlobalInst {
                    global: *global,
                    value,
                }
            })
            .collect();

        Store {
            funcs: function_instances,
            mems: memory_instances,
            globals: global_instances,
        }
    }
}
