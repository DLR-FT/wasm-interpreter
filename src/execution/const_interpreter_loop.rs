use alloc::vec::Vec;

use crate::{
    addrs::ModuleAddr,
    assert_validated::UnwrapValidatedExt,
    config::Config,
    core::{
        indices::{FuncIdx, GlobalIdx},
        reader::{
            span::Span,
            types::{FuncType, ResultType},
            WasmReader,
        },
    },
    unreachable_validated,
    value::{self, Ref},
    value_stack::Stack,
    RefType, RuntimeError, Store, Value,
};

// TODO update this documentation
/// Execute a validated constant expression. These type of expressions are used
/// for initializing global variables, data and element segments.
///
/// # Arguments
/// TODO
///
/// # Safety
///
/// 1. the constant expression in the reader must be valid
/// 2. the module address must be valid in the given store
///
// TODO this signature might change to support hooks or match the spec better
pub(crate) unsafe fn run_const<T: Config>(
    wasm: &mut WasmReader,
    stack: &mut Stack,
    module: ModuleAddr,
    store: &Store<T>,
) -> Result<(), RuntimeError> {
    use crate::core::reader::types::opcode::*;
    loop {
        let first_instr_byte = wasm.read_u8().unwrap_validated();

        #[cfg(feature = "log")]
        crate::core::utils::print_beautiful_instruction_name_1_byte(first_instr_byte, wasm.pc);

        match first_instr_byte {
            END => {
                trace!("Constant instruction: END");
                break;
            }
            GLOBAL_GET => {
                // SAFETY: Validation guarantees there to be a valid global
                // index next.
                let global_idx = unsafe { GlobalIdx::read_unchecked(wasm) };

                // SAFETY: The caller ensures that the given module address is
                // valid in the given store.
                let module_instance = unsafe { store.modules.get(module) };

                // SAFETY: Validation guarantees the global index to be valid in
                // the current module.
                let global_addr = *unsafe { module_instance.global_addrs.get(global_idx) };

                // SAFETY: The global address just came from the same store.
                // Therefore, it must be valid in this store.
                let global = unsafe { store.globals.get(global_addr) };

                trace!(
                    "Constant instruction: global.get [{global_idx}] -> [{:?}]",
                    global
                );
                stack.push_value(global.value)?;
            }
            I32_CONST => {
                let constant = wasm.read_var_i32().unwrap_validated();
                trace!("Constant instruction: i32.const [] -> [{constant}]");
                stack.push_value(constant.into())?;
            }
            F32_CONST => {
                let constant = value::F32::from_bits(wasm.read_f32().unwrap_validated());
                trace!("Constanting instruction: f32.const [] -> [{constant}]");
                stack.push_value(constant.into())?;
            }
            F64_CONST => {
                let constant = value::F64::from_bits(wasm.read_f64().unwrap_validated());
                trace!("Constanting instruction: f64.const [] -> [{constant}]");
                stack.push_value(constant.into())?;
            }
            I64_CONST => {
                let constant = wasm.read_var_i64().unwrap_validated();
                trace!("Constant instruction: i64.const [] -> [{constant}]");
                stack.push_value(constant.into())?;
            }
            REF_NULL => {
                let reftype = RefType::read(wasm).unwrap_validated();

                stack.push_value(Value::Ref(Ref::Null(reftype)))?;
                trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
            }
            REF_FUNC => {
                // SAFETY: Validation guarantees there to be a valid function
                // index next.
                let func_idx = unsafe { FuncIdx::read_unchecked(wasm) };
                // SAFETY: Validation guarantees the function index to be valid
                // in the current module.
                let func_addr = unsafe { store.modules.get(module).func_addrs.get(func_idx) };
                stack.push_value(Value::Ref(Ref::Func(*func_addr)))?;
            }

            FD_EXTENSIONS => {
                use crate::core::reader::types::opcode::fd_extensions::*;

                match wasm.read_var_u32().unwrap_validated() {
                    V128_CONST => {
                        let mut data = [0; 16];
                        for byte_ref in &mut data {
                            *byte_ref = wasm.read_u8().unwrap_validated();
                        }

                        stack.push_value(Value::V128(data))?;
                    }
                    0x00..=0x0B | 0x0D.. => unreachable_validated!(),
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
                unreachable_validated!();
            }
        }
    }
    Ok(())
}

/// # Safety
///
/// 1. the constant expression in bytecode in the given span must be valid
/// 2. the module address must be valid in the given store
pub(crate) unsafe fn run_const_span<T: Config>(
    wasm: &[u8],
    span: &Span,
    module: ModuleAddr,
    store: &Store<T>,
) -> Result<Option<Value>, RuntimeError> {
    let mut wasm = WasmReader::new(wasm);

    wasm.move_start_to(*span).unwrap_validated();

    let mut stack = Stack::new::<T>(
        Vec::new(),
        &FuncType {
            params: ResultType {
                valtypes: Vec::new(),
            },
            returns: ResultType {
                valtypes: Vec::new(),
            },
        },
        &[],
    )?;
    // SAFETY: The current caller makes the same safety guarantees.
    unsafe { run_const(&mut wasm, &mut stack, module, store)? };

    Ok(stack.peek_value())
}
