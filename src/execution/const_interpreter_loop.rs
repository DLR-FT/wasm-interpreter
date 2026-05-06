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
pub(crate) unsafe fn run_const<'wasm, T: Config>(
    wasm: &mut WasmReader<'wasm>,
    stack: &mut Stack,
    module: ModuleAddr,
    store: &Store<'wasm, T>,
) -> Result<(), RuntimeError> {
    use crate::core::reader::types::opcode::*;
    loop {
        let first_instr_byte = wasm.read_u8().unwrap_validated();

        #[cfg(feature = "log")]
        crate::core::utils::print_beautiful_instruction_name_1_byte(first_instr_byte, wasm.pc);

        let instruction_fn = match first_instr_byte {
            END => end::<T>,
            GLOBAL_GET => global_get::<T>,
            I32_CONST => i32_const::<T>,
            F32_CONST => f32_const::<T>,
            F64_CONST => f64_const::<T>,
            I64_CONST => i64_const::<T>,
            REF_NULL => ref_null::<T>,
            REF_FUNC => ref_func::<T>,
            FD_EXTENSIONS => fd_extensions::<T>,

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
        };

        let args = Args {
            wasm,
            stack,
            module,
            store,
        };

        // SAFETY: All possible instruction handler functions use the same safety requirements, as
        // they are defined through the same macro. These are the same requirements defined by the
        // current function, which must be fulfilled.
        let should_break = unsafe { instruction_fn(args) }?;
        if should_break {
            break;
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

struct Args<'reader, 'resumable, 'store, 'wasm, T: Config> {
    wasm: &'reader mut WasmReader<'wasm>,
    stack: &'resumable mut Stack,
    module: ModuleAddr,
    store: &'store Store<'wasm, T>,
}

macro_rules! define_instruction {
    ($name:ident, $opcode:expr, $contents:expr) => {
        /// # Safety
        ///
        /// 1. the constant expression in the reader must be valid
        /// 2. the module address must be valid in the given store
        // Disable inlining to inspect the emitted code of individual instruction handlers
        // #[inline(never)]
        unsafe fn $name<T: Config>(args: Args<T>) -> Result<bool, RuntimeError> {
            $contents(args)
        }
    };
}

define_instruction!(end, opcode::END, |Args { .. }| {
    trace!("Constant instruction: END");
    Ok(true)
});

define_instruction!(
    global_get,
    opcode::GLOBAL_GET,
    |Args {
         wasm,
         module,
         store,
         stack,
     }| {
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
        let global = unsafe { store.inner.globals.get(global_addr) };

        trace!(
            "Constant instruction: global.get [{global_idx}] -> [{:?}]",
            global
        );
        stack.push_value::<T>(global.value)?;
        Ok(false)
    }
);

define_instruction!(
    i32_const,
    opcode::I32_CONST,
    |Args { wasm, stack, .. }| {
        let constant = wasm.read_var_i32().unwrap_validated();
        trace!("Constant instruction: i32.const [] -> [{constant}]");
        stack.push_value::<T>(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    f32_const,
    opcode::F32_CONST,
    |Args { wasm, stack, .. }| {
        let constant = value::F32::from_bits(wasm.read_f32().unwrap_validated());
        trace!("Constanting instruction: f32.const [] -> [{constant}]");
        stack.push_value::<T>(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    f64_const,
    opcode::F64_CONST,
    |Args { wasm, stack, .. }| {
        let constant = value::F64::from_bits(wasm.read_f64().unwrap_validated());
        trace!("Constanting instruction: f64.const [] -> [{constant}]");
        stack.push_value::<T>(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    i64_const,
    opcode::I64_CONST,
    |Args { wasm, stack, .. }| {
        let constant = wasm.read_var_i64().unwrap_validated();
        trace!("Constant instruction: i64.const [] -> [{constant}]");
        stack.push_value::<T>(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    ref_null,
    opcode::REF_NULL,
    |Args { wasm, stack, .. }| {
        let reftype = RefType::read(wasm).unwrap_validated();

        stack.push_value::<T>(Value::Ref(Ref::Null(reftype)))?;
        trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
        Ok(false)
    }
);

define_instruction!(
    ref_func,
    opcode::REF_FUNC,
    |Args {
         wasm,
         module,
         store,
         stack,
     }| {
        // SAFETY: Validation guarantees there to be a valid function
        // index next.
        let func_idx = unsafe { FuncIdx::read_unchecked(wasm) };
        // SAFETY: Validation guarantees the function index to be valid
        // in the current module.
        let func_addr = unsafe { store.modules.get(module).func_addrs.get(func_idx) };
        stack.push_value::<T>(Value::Ref(Ref::Func(*func_addr)))?;
        Ok(false)
    }
);

define_instruction!(
    fd_extensions,
    opcode::FD_EXTENSIONS,
    |Args { wasm, stack, .. }| {
        use crate::core::reader::types::opcode::fd_extensions::*;

        match wasm.read_var_u32().unwrap_validated() {
            V128_CONST => {
                let mut data = [0; 16];
                for byte_ref in &mut data {
                    *byte_ref = wasm.read_u8().unwrap_validated();
                }

                stack.push_value::<T>(Value::V128(data))?;
            }
            0x00..=0x0B | 0x0D.. => unreachable_validated!(),
        }

        Ok(false)
    }
);
