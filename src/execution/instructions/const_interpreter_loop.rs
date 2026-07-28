use alloc::vec::Vec;

use crate::{
    core::{
        decoding::decoder::{span::Span, WasmDecoder},
        structure::{
            modules::indices::{FuncIdx, GlobalIdx},
            types::{FuncType, ResultType},
        },
    },
    execution::{assert_validated::UnwrapValidatedExt, runtime_structure::value_stack::Stack},
    trace, unreachable_validated, Config, ModuleAddr, Ref, RefType, RuntimeError, Store, Value,
    F32, F64,
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
/// 1. the constant expression in the decoder must be valid
/// 2. the module address must be valid in the given store
///
// TODO this signature might change to support hooks or match the spec better
pub(crate) unsafe fn run_const<'wasm, T: Config>(
    wasm: &mut WasmDecoder<'wasm>,
    stack: &mut Stack,
    module: ModuleAddr,
    store: &Store<'wasm, T>,
) -> Result<(), RuntimeError> {
    use crate::core::structure::instructions::*;
    loop {
        let first_instr_byte = wasm.decode_u8().unwrap_validated();

        trace!(
            "Executing const instruction {} at pc={}",
            instruction_byte_to_str(first_instr_byte),
            wasm.pc
        );

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

        let state = State {
            wasm,
            stack,
            module,
            store,
        };

        // SAFETY: All possible instruction handler functions use the same safety requirements, as
        // they are defined through the same macro. These are the same requirements defined by the
        // current function, which must be fulfilled.
        let should_break = unsafe { instruction_fn(state) }?;
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
    let mut wasm = WasmDecoder::new(wasm);

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

struct State<'decoder, 'resumable, 'store, 'wasm, T: Config> {
    wasm: &'decoder mut WasmDecoder<'wasm>,
    stack: &'resumable mut Stack,
    module: ModuleAddr,
    store: &'store Store<'wasm, T>,
}

macro_rules! define_instruction {
    ($name:ident, $instruction:expr, $contents:expr) => {
        /// # Safety
        ///
        /// 1. the constant expression in the decoder must be valid
        /// 2. the module address must be valid in the given store
        // Disable inlining to inspect the emitted code of individual instruction handlers
        // #[inline(never)]
        unsafe fn $name<T: Config>(state: State<T>) -> Result<bool, RuntimeError> {
            $contents(state)
        }
    };
}

define_instruction!(end, instructions::END, |State { .. }| {
    trace!("Constant instruction: END");
    Ok(true)
});

define_instruction!(
    global_get,
    instructions::GLOBAL_GET,
    |State {
         wasm,
         module,
         store,
         stack,
     }| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::decode_unchecked(wasm) };

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
        stack.push_value(global.value)?;
        Ok(false)
    }
);

define_instruction!(
    i32_const,
    instructions::I32_CONST,
    |State { wasm, stack, .. }| {
        let constant = wasm.decode_var_i32().unwrap_validated();
        trace!("Constant instruction: i32.const [] -> [{constant}]");
        stack.push_value(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    f32_const,
    instructions::F32_CONST,
    |State { wasm, stack, .. }| {
        let constant = F32::from_bits(wasm.decode_f32().unwrap_validated());
        trace!("Constanting instruction: f32.const [] -> [{constant}]");
        stack.push_value(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    f64_const,
    instructions::F64_CONST,
    |State { wasm, stack, .. }| {
        let constant = F64::from_bits(wasm.decode_f64().unwrap_validated());
        trace!("Constanting instruction: f64.const [] -> [{constant}]");
        stack.push_value(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    i64_const,
    instructions::I64_CONST,
    |State { wasm, stack, .. }| {
        let constant = wasm.decode_var_i64().unwrap_validated();
        trace!("Constant instruction: i64.const [] -> [{constant}]");
        stack.push_value(constant.into())?;
        Ok(false)
    }
);

define_instruction!(
    ref_null,
    instructions::REF_NULL,
    |State { wasm, stack, .. }| {
        let reftype = RefType::decode(wasm).unwrap_validated();

        stack.push_value(Value::Ref(Ref::Null(reftype)))?;
        trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
        Ok(false)
    }
);

define_instruction!(
    ref_func,
    instructions::REF_FUNC,
    |State {
         wasm,
         module,
         store,
         stack,
     }| {
        // SAFETY: Validation guarantees there to be a valid function
        // index next.
        let func_idx = unsafe { FuncIdx::decode_unchecked(wasm) };
        // SAFETY: Validation guarantees the function index to be valid
        // in the current module.
        let func_addr = unsafe { store.modules.get(module).func_addrs.get(func_idx) };
        stack.push_value(Value::Ref(Ref::Func(*func_addr)))?;
        Ok(false)
    }
);

define_instruction!(
    fd_extensions,
    instructions::FD_EXTENSIONS,
    |State { wasm, stack, .. }| {
        use crate::core::structure::instructions::fd_extensions::*;
        let second_instruction_part = wasm.decode_var_u32().unwrap_validated();

        trace!(
            "Executing const FD instruction {} at pc={}",
            crate::core::structure::instructions::fd_extension_instruction_to_str(
                second_instruction_part
            ),
            wasm.pc
        );

        match second_instruction_part {
            V128_CONST => {
                let mut data = [0; 16];
                for byte_ref in &mut data {
                    *byte_ref = wasm.decode_u8().unwrap_validated();
                }

                stack.push_value(Value::V128(data))?;
            }
            0x00..=0x0B | 0x0D.. => unreachable_validated!(),
        }

        Ok(false)
    }
);
