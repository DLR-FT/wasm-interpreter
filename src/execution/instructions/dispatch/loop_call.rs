use core::ops::ControlFlow;

use crate::{
    core::{
        decoding::decoder::WasmDecoder,
        sidetable::Sidetable,
        utils::{BytecodeProvider, ToUsizeExt},
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::InterpreterLoopOutcome,
        runtime_structure::{
            function_instances::FuncInst, module_instances::ModuleInst, store::StoreInner,
        },
    },
    AddrVec, Config, ModuleAddr, RuntimeError, Store, WasmResumable,
};

//TODO: Add trait bound T2: BytecodeProvider in the future when rust type checker can check for it
type InstructionHandlerFn<T2> =
    for<'wasm, 'modules> unsafe fn(
        wasm: &mut WasmDecoder<'wasm>,
        resumable: &mut WasmResumable,
        current_sidetable: &mut &'modules Sidetable,
        store_inner: &mut StoreInner,
        modules: &'modules AddrVec<ModuleAddr, ModuleInst>,
        current_module: &mut ModuleAddr,
        current_function_end_marker: &mut usize,
        bytecode_provider: &'wasm T2,
    )
        -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError>;

/// Interprets Wasm bytecode using a loop-call construct.
///
/// The given [`WasmResumable`] contains the state for execution, like the program counter, the
/// stack, etc. The [`Store`] contains the global execution context.
///
/// Returns either an [`InterpreterLoopOutcome`] or a [`RuntimeError`]. Depending on how execution
/// ended, the outcome enum may contain more information about the reason and if execution may be
/// resumed.
///
/// # Safety
///
/// The given resumable must be valid in the given store and the store itself must be valid.
#[inline(never)]
pub unsafe fn run<T: Config, T2: BytecodeProvider>(
    resumable: &mut WasmResumable,
    store: &mut Store<T>,
    bytecode_provider: &T2,
) -> Result<InterpreterLoopOutcome, RuntimeError> {
    let current_func_addr = resumable.current_func_addr;
    let pc = resumable.pc;
    // SAFETY: The caller ensures that the resumable and thus also its function
    // address is valid in the current store.
    let func_inst = unsafe { store.inner.functions.get(current_func_addr) };
    let FuncInst::WasmFunc(wasm_func_inst) = &func_inst else {
        unreachable!(
            "the interpreter loop shall only be executed with native wasm functions as root call"
        );
    };
    let mut current_module = wasm_func_inst.module_addr;

    // Start reading the function's instructions
    // SAFETY: This module address was just read from the current store. Every
    // store guarantees all addresses contained in it to be valid within itself.
    let module = unsafe { store.modules.get(current_module) };
    let wasm_bytecode = bytecode_provider.get_bytecode(module.bytecode_id);
    let mut wasm = WasmDecoder::new(wasm_bytecode);

    let mut current_sidetable: &Sidetable = &module.sidetable;

    let mut current_function_end_marker =
        wasm_func_inst.code_expr.from() + wasm_func_inst.code_expr.len();

    let store_inner = &mut store.inner;

    wasm.pc = pc;

    loop {
        // call the instruction hook
        store.user_data.instruction_hook(wasm_bytecode, wasm.pc);

        let prev_pc = wasm.pc;

        let first_instr_byte = wasm.decode_u8().unwrap_validated();

        let instruction_fn = T::DISPATCH_TABLE
            .get(usize::from(first_instr_byte))
            .and_then(Option::as_ref)
            .expect("the instruction to be valid because the code is validated");

        // SAFETY: All safety requirements of `State` are fulfilled:
        // - The wasm decoder was created initialized with the Wasm code for the current module.
        //   Also it points into the current function, as guarantees by the fact that the resumable
        //   is valid.
        // - The `StoreInner` is valid because the `Store` was valid.
        // - The caller ensures that the resumable is valid in the `Store`, therefore also in the
        //   `StoreInner`.
        // - The current sidetable was determined through the current module.
        // - The end marker for the current function was computed using the current function
        //   instance.
        let instruction_result = unsafe {
            instruction_fn(
                &mut wasm,
                resumable,
                &mut current_sidetable,
                store_inner,
                &store.modules,
                &mut current_module,
                &mut current_function_end_marker,
                bytecode_provider,
            )
        };

        if let ControlFlow::Break(interpreter_loop_outcome) = instruction_result? {
            if let InterpreterLoopOutcome::OutOfFuel { .. } = interpreter_loop_outcome {
                wasm.pc = prev_pc;
            }

            resumable.pc = wasm.pc;
            return Ok(interpreter_loop_outcome);
        }
    }
}

/// # Safety
///
/// All arguments must be valid according to the same rules that exist for
/// [`State`](crate::execution::instructions::State).
#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn fc_extensions<'wasm, 'modules, T: Config, T2: BytecodeProvider>(
    wasm: &mut WasmDecoder<'wasm>,
    resumable: &mut WasmResumable,
    current_sidetable: &mut &'modules Sidetable,
    store_inner: &mut StoreInner,
    modules: &'modules AddrVec<ModuleAddr, ModuleInst>,
    current_module: &mut ModuleAddr,
    current_function_end_marker: &mut usize,
    bytecode_provider: &'wasm T2,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // should we call instruction hook here as well? multibyte instruction
    let second_instr = wasm.decode_var_u32().unwrap_validated();

    let instruction_fn = T::FC_DISPATCH_TABLE
        .get(second_instr.into_usize())
        .and_then(Option::as_ref)
        .expect("the instruction to be valid because the code is validated");

    // SAFETY: The caller ensures that all safety requirements of `State` are fulfilled.
    unsafe {
        instruction_fn(
            wasm,
            resumable,
            current_sidetable,
            store_inner,
            modules,
            current_module,
            current_function_end_marker,
            bytecode_provider,
        )
    }
}

/// # Safety
///
/// All arguments must be valid according to the same rules that exist for
/// [`State`](crate::execution::instructions::State).
#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn fd_extensions<'wasm, 'modules, T: Config, T2: BytecodeProvider>(
    wasm: &mut WasmDecoder<'wasm>,
    resumable: &mut WasmResumable,
    current_sidetable: &mut &'modules Sidetable,
    store_inner: &mut StoreInner,
    modules: &'modules AddrVec<ModuleAddr, ModuleInst>,
    current_module: &mut ModuleAddr,
    current_function_end_marker: &mut usize,
    bytecode_provider: &'wasm T2,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // Should we call instruction hook here as well? Multibyte instruction
    let second_instr = wasm.decode_var_u32().unwrap_validated();

    let instruction_fn = T::FD_DISPATCH_TABLE
        .get(second_instr.into_usize())
        .and_then(Option::as_ref)
        .expect("the instruction to be valid because the code is validated");

    // SAFETY: The caller ensures that all safety requirements of `State` are fulfilled.
    unsafe {
        instruction_fn(
            wasm,
            resumable,
            current_sidetable,
            store_inner,
            modules,
            current_module,
            current_function_end_marker,
            bytecode_provider,
        )
    }
}

mod wrappers {
    use core::ops::ControlFlow;

    use crate::{
        core::{decoding::decoder::WasmDecoder, sidetable::Sidetable, utils::BytecodeProvider},
        execution::{
            instructions::{
                decrement_fuel,
                dispatch::{
                    for_all_instructions, for_all_instructions_fc, for_all_instructions_fd,
                },
                InterpreterLoopOutcome, State,
            },
            resumable::WasmResumable,
            runtime_structure::{
                addresses::{AddrVec, ModuleAddr},
                module_instances::ModuleInst,
                store::StoreInner,
            },
        },
        Config, RuntimeError,
    };

    macro_rules! define_wrappers {
        ($(($name:ident, $handler_fn:path, $opcode:path, $fuel_check:expr)),*) => {

            $(
                /// # Safety
                ///
                /// All arguments must be valid according to the same rules that exist for
                /// [`State`].
                #[allow(
                    clippy::extra_unused_type_parameters,
                    reason = "T is only used by some instructions"
                )]
                #[allow(clippy::too_many_arguments)]
                pub(crate) unsafe fn $name<'wasm, 'modules, T: Config, T2: BytecodeProvider>(
                wasm: &mut WasmDecoder<'wasm>,
                resumable: &mut WasmResumable,
                current_sidetable: &mut &'modules Sidetable,
                store_inner: &mut StoreInner,
                modules: &'modules AddrVec<ModuleAddr, ModuleInst>,
                current_module: &mut ModuleAddr,
                current_function_end_marker: &mut usize,
                bytecode_provider: &'wasm T2,
            ) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
                    if $fuel_check {
                        let opcode: u8 = $opcode;
                        if let core::ops::ControlFlow::Break(outcome) = decrement_fuel(
                            T::get_flat_cost(opcode),
                            &mut resumable.maybe_fuel,
                        ) {
                            return Ok(core::ops::ControlFlow::Break(outcome));
                        }
                    }

                    let state = State {
                        store_inner,
                        modules,
                        wasm,
                        current_module,
                        current_function_end_marker,
                        current_sidetable,
                        resumable,
                        bytecode_provider
                    };

                    // SAFETY: All instruction handlers require that the passed `State` is valid
                    // according to its safety documentation. The caller of the current function
                    // guarantees the same for all fields that were used to construct it.
                    unsafe { $handler_fn(state) }
                }
            )*
        };
    }

    macro_rules! define_wrappers_fc {
        ($(($name:ident, $handler_fn:path, $opcode:path, $fuel_check:expr)),*) => {
            $(
                /// # Safety
                ///
                /// All arguments must be valid according to the same rules that exist for
                /// [`State`].
                #[allow(
                    clippy::extra_unused_type_parameters,
                    reason = "T is only used by some instructions"
                )]
                #[allow(clippy::too_many_arguments)]
                pub(crate) unsafe fn $name<
                'wasm,
                'modules,
                T: Config,
                T2: BytecodeProvider,
            >(
                wasm: & mut WasmDecoder<'wasm>,
                resumable: & mut WasmResumable,
                current_sidetable: & mut &'modules Sidetable,
                store_inner: & mut StoreInner,
                modules: &'modules AddrVec<ModuleAddr, ModuleInst>,
                current_module: & mut ModuleAddr,
                current_function_end_marker: & mut usize,
                bytecode_provider: &'wasm T2,
            ) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
                    if $fuel_check {
                        let opcode: u32 = $opcode;
                        if let ControlFlow::Break(outcome) = decrement_fuel(
                            T::get_fc_extension_flat_cost(opcode),
                            &mut resumable.maybe_fuel,
                        ) {
                            return Ok(ControlFlow::Break(outcome));
                        }
                    }

                    let state = State {
                        store_inner,
                        modules,
                        wasm,
                        current_module,
                        current_function_end_marker,
                        current_sidetable,
                        resumable,
                        bytecode_provider
                    };

                    // SAFETY: All instruction handlers require that the passed `State` is valid
                    // according to its safety documentation. The caller of the current function
                    // guarantees the same for all fields that were used to construct it.
                    unsafe { $handler_fn(state) }
                }
            )*
        };
    }

    macro_rules! define_wrappers_fd {
        ($(($name:ident, $handler_fn:path, $opcode:path, $fuel_check:expr)),*) => {
            $(
                /// # Safety
                ///
                /// All arguments must be valid according to the same rules that exist for
                /// [`State`].
                #[allow(
                    clippy::extra_unused_type_parameters,
                    reason = "T is only used by some instructions"
                )]
                #[allow(clippy::too_many_arguments)]
                pub(crate) unsafe fn $name<
                'wasm,
                'modules,
                T: Config,
                T2: BytecodeProvider,
            >(
                wasm: & mut WasmDecoder<'wasm>,
                resumable: & mut WasmResumable,
                current_sidetable: & mut &'modules Sidetable,
                store_inner: & mut StoreInner,
                modules: &'modules AddrVec<ModuleAddr, ModuleInst>,
                current_module: & mut ModuleAddr,
                current_function_end_marker: & mut usize,
                bytecode_provider: &'wasm T2,
            ) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError>  {
                    if $fuel_check {
                        let opcode: u32 = $opcode;
                        if let ControlFlow::Break(outcome) = decrement_fuel(
                            T::get_fd_extension_flat_cost(opcode),
                            &mut resumable.maybe_fuel,
                        ) {
                            return Ok(ControlFlow::Break(outcome));
                        }
                    }

                    let state = State {
                        store_inner,
                        modules,
                        wasm,
                        current_module,
                        current_function_end_marker,
                        current_sidetable,
                        resumable,
                        bytecode_provider
                    };

                    // SAFETY: All instruction handlers require that the passed `State` is valid
                    // according to its safety documentation. The caller of the current function
                    // guarantees the same for all fields that were used to construct it.
                    unsafe { $handler_fn(state) }
                }
            )*
        };
    }

    for_all_instructions!(define_wrappers);
    for_all_instructions_fc!(define_wrappers_fc);
    for_all_instructions_fd!(define_wrappers_fd);
}

pub(crate) trait HasBaseDispatchTable<T2: BytecodeProvider> {
    const DISPATCH_TABLE: [Option<InstructionHandlerFn<T2>>; 256];
}

pub(crate) trait HasFcDispatchTable<T2: BytecodeProvider> {
    const FC_DISPATCH_TABLE: [Option<InstructionHandlerFn<T2>>; 18];
}

pub(crate) trait HasFdDispatchTable<T2: BytecodeProvider> {
    const FD_DISPATCH_TABLE: [Option<InstructionHandlerFn<T2>>; 256];
}

impl<T: Config, T2: BytecodeProvider> HasBaseDispatchTable<T2> for T {
    const DISPATCH_TABLE: [Option<InstructionHandlerFn<T2>>; 256] = [
        Some(wrappers::unreachable::<T, T2>),
        Some(wrappers::nop::<T, T2>),
        Some(wrappers::block::<T, T2>),
        Some(wrappers::r#loop::<T, T2>),
        Some(wrappers::r#if::<T, T2>),
        Some(wrappers::r#else::<T, T2>),
        None,
        None,
        None,
        None,
        None,
        Some(wrappers::end::<T, T2>),
        Some(wrappers::br::<T, T2>),
        Some(wrappers::br_if::<T, T2>),
        Some(wrappers::br_table::<T, T2>),
        Some(wrappers::r#return::<T, T2>),
        Some(wrappers::call::<T, T2>),
        Some(wrappers::call_indirect::<T, T2>),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(wrappers::drop::<T, T2>),
        Some(wrappers::select::<T, T2>),
        Some(wrappers::select_t::<T, T2>),
        None,
        None,
        None,
        Some(wrappers::local_get::<T, T2>),
        Some(wrappers::local_set::<T, T2>),
        Some(wrappers::local_tee::<T, T2>),
        Some(wrappers::global_get::<T, T2>),
        Some(wrappers::global_set::<T, T2>),
        Some(wrappers::table_get::<T, T2>),
        Some(wrappers::table_set::<T, T2>),
        None,
        Some(wrappers::i32_load::<T, T2>),
        Some(wrappers::i64_load::<T, T2>),
        Some(wrappers::f32_load::<T, T2>),
        Some(wrappers::f64_load::<T, T2>),
        Some(wrappers::i32_load8_s::<T, T2>),
        Some(wrappers::i32_load8_u::<T, T2>),
        Some(wrappers::i32_load16_s::<T, T2>),
        Some(wrappers::i32_load16_u::<T, T2>),
        Some(wrappers::i64_load8_s::<T, T2>),
        Some(wrappers::i64_load8_u::<T, T2>),
        Some(wrappers::i64_load16_s::<T, T2>),
        Some(wrappers::i64_load16_u::<T, T2>),
        Some(wrappers::i64_load32_s::<T, T2>),
        Some(wrappers::i64_load32_u::<T, T2>),
        Some(wrappers::i32_store::<T, T2>),
        Some(wrappers::i64_store::<T, T2>),
        Some(wrappers::f32_store::<T, T2>),
        Some(wrappers::f64_store::<T, T2>),
        Some(wrappers::i32_store8::<T, T2>),
        Some(wrappers::i32_store16::<T, T2>),
        Some(wrappers::i64_store8::<T, T2>),
        Some(wrappers::i64_store16::<T, T2>),
        Some(wrappers::i64_store32::<T, T2>),
        Some(wrappers::memory_size::<T, T2>),
        Some(wrappers::memory_grow::<T, T2>),
        Some(wrappers::i32_const::<T, T2>),
        Some(wrappers::i64_const::<T, T2>),
        Some(wrappers::f32_const::<T, T2>),
        Some(wrappers::f64_const::<T, T2>),
        Some(wrappers::i32_eqz::<T, T2>),
        Some(wrappers::i32_eq::<T, T2>),
        Some(wrappers::i32_ne::<T, T2>),
        Some(wrappers::i32_lt_s::<T, T2>),
        Some(wrappers::i32_lt_u::<T, T2>),
        Some(wrappers::i32_gt_s::<T, T2>),
        Some(wrappers::i32_gt_u::<T, T2>),
        Some(wrappers::i32_le_s::<T, T2>),
        Some(wrappers::i32_le_u::<T, T2>),
        Some(wrappers::i32_ge_s::<T, T2>),
        Some(wrappers::i32_ge_u::<T, T2>),
        Some(wrappers::i64_eqz::<T, T2>),
        Some(wrappers::i64_eq::<T, T2>),
        Some(wrappers::i64_ne::<T, T2>),
        Some(wrappers::i64_lt_s::<T, T2>),
        Some(wrappers::i64_lt_u::<T, T2>),
        Some(wrappers::i64_gt_s::<T, T2>),
        Some(wrappers::i64_gt_u::<T, T2>),
        Some(wrappers::i64_le_s::<T, T2>),
        Some(wrappers::i64_le_u::<T, T2>),
        Some(wrappers::i64_ge_s::<T, T2>),
        Some(wrappers::i64_ge_u::<T, T2>),
        Some(wrappers::f32_eq::<T, T2>),
        Some(wrappers::f32_ne::<T, T2>),
        Some(wrappers::f32_lt::<T, T2>),
        Some(wrappers::f32_gt::<T, T2>),
        Some(wrappers::f32_le::<T, T2>),
        Some(wrappers::f32_ge::<T, T2>),
        Some(wrappers::f64_eq::<T, T2>),
        Some(wrappers::f64_ne::<T, T2>),
        Some(wrappers::f64_lt::<T, T2>),
        Some(wrappers::f64_gt::<T, T2>),
        Some(wrappers::f64_le::<T, T2>),
        Some(wrappers::f64_ge::<T, T2>),
        Some(wrappers::i32_clz::<T, T2>),
        Some(wrappers::i32_ctz::<T, T2>),
        Some(wrappers::i32_popcnt::<T, T2>),
        Some(wrappers::i32_add::<T, T2>),
        Some(wrappers::i32_sub::<T, T2>),
        Some(wrappers::i32_mul::<T, T2>),
        Some(wrappers::i32_div_s::<T, T2>),
        Some(wrappers::i32_div_u::<T, T2>),
        Some(wrappers::i32_rem_s::<T, T2>),
        Some(wrappers::i32_rem_u::<T, T2>),
        Some(wrappers::i32_and::<T, T2>),
        Some(wrappers::i32_or::<T, T2>),
        Some(wrappers::i32_xor::<T, T2>),
        Some(wrappers::i32_shl::<T, T2>),
        Some(wrappers::i32_shr_s::<T, T2>),
        Some(wrappers::i32_shr_u::<T, T2>),
        Some(wrappers::i32_rotl::<T, T2>),
        Some(wrappers::i32_rotr::<T, T2>),
        Some(wrappers::i64_clz::<T, T2>),
        Some(wrappers::i64_ctz::<T, T2>),
        Some(wrappers::i64_popcnt::<T, T2>),
        Some(wrappers::i64_add::<T, T2>),
        Some(wrappers::i64_sub::<T, T2>),
        Some(wrappers::i64_mul::<T, T2>),
        Some(wrappers::i64_div_s::<T, T2>),
        Some(wrappers::i64_div_u::<T, T2>),
        Some(wrappers::i64_rem_s::<T, T2>),
        Some(wrappers::i64_rem_u::<T, T2>),
        Some(wrappers::i64_and::<T, T2>),
        Some(wrappers::i64_or::<T, T2>),
        Some(wrappers::i64_xor::<T, T2>),
        Some(wrappers::i64_shl::<T, T2>),
        Some(wrappers::i64_shr_s::<T, T2>),
        Some(wrappers::i64_shr_u::<T, T2>),
        Some(wrappers::i64_rotl::<T, T2>),
        Some(wrappers::i64_rotr::<T, T2>),
        Some(wrappers::f32_abs::<T, T2>),
        Some(wrappers::f32_neg::<T, T2>),
        Some(wrappers::f32_ceil::<T, T2>),
        Some(wrappers::f32_floor::<T, T2>),
        Some(wrappers::f32_trunc::<T, T2>),
        Some(wrappers::f32_nearest::<T, T2>),
        Some(wrappers::f32_sqrt::<T, T2>),
        Some(wrappers::f32_add::<T, T2>),
        Some(wrappers::f32_sub::<T, T2>),
        Some(wrappers::f32_mul::<T, T2>),
        Some(wrappers::f32_div::<T, T2>),
        Some(wrappers::f32_min::<T, T2>),
        Some(wrappers::f32_max::<T, T2>),
        Some(wrappers::f32_copysign::<T, T2>),
        Some(wrappers::f64_abs::<T, T2>),
        Some(wrappers::f64_neg::<T, T2>),
        Some(wrappers::f64_ceil::<T, T2>),
        Some(wrappers::f64_floor::<T, T2>),
        Some(wrappers::f64_trunc::<T, T2>),
        Some(wrappers::f64_nearest::<T, T2>),
        Some(wrappers::f64_sqrt::<T, T2>),
        Some(wrappers::f64_add::<T, T2>),
        Some(wrappers::f64_sub::<T, T2>),
        Some(wrappers::f64_mul::<T, T2>),
        Some(wrappers::f64_div::<T, T2>),
        Some(wrappers::f64_min::<T, T2>),
        Some(wrappers::f64_max::<T, T2>),
        Some(wrappers::f64_copysign::<T, T2>),
        Some(wrappers::i32_wrap_i64::<T, T2>),
        Some(wrappers::i32_trunc_f32_s::<T, T2>),
        Some(wrappers::i32_trunc_f32_u::<T, T2>),
        Some(wrappers::i32_trunc_f64_s::<T, T2>),
        Some(wrappers::i32_trunc_f64_u::<T, T2>),
        Some(wrappers::i64_extend_i32_s::<T, T2>),
        Some(wrappers::i64_extend_i32_u::<T, T2>),
        Some(wrappers::i64_trunc_f32_s::<T, T2>),
        Some(wrappers::i64_trunc_f32_u::<T, T2>),
        Some(wrappers::i64_trunc_f64_s::<T, T2>),
        Some(wrappers::i64_trunc_f64_u::<T, T2>),
        Some(wrappers::f32_convert_i32_s::<T, T2>),
        Some(wrappers::f32_convert_i32_u::<T, T2>),
        Some(wrappers::f32_convert_i64_s::<T, T2>),
        Some(wrappers::f32_convert_i64_u::<T, T2>),
        Some(wrappers::f32_demote_f64::<T, T2>),
        Some(wrappers::f64_convert_i32_s::<T, T2>),
        Some(wrappers::f64_convert_i32_u::<T, T2>),
        Some(wrappers::f64_convert_i64_s::<T, T2>),
        Some(wrappers::f64_convert_i64_u::<T, T2>),
        Some(wrappers::f64_promote_f32::<T, T2>),
        Some(wrappers::i32_reinterpret_f32::<T, T2>),
        Some(wrappers::i64_reinterpret_f64::<T, T2>),
        Some(wrappers::f32_reinterpret_i32::<T, T2>),
        Some(wrappers::f64_reinterpret_i64::<T, T2>),
        Some(wrappers::i32_extend8_s::<T, T2>),
        Some(wrappers::i32_extend16_s::<T, T2>),
        Some(wrappers::i64_extend8_s::<T, T2>),
        Some(wrappers::i64_extend16_s::<T, T2>),
        Some(wrappers::i64_extend32_s::<T, T2>),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(wrappers::ref_null::<T, T2>),
        Some(wrappers::ref_is_null::<T, T2>),
        Some(wrappers::ref_func::<T, T2>),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(fc_extensions::<T, T2>),
        Some(fd_extensions::<T, T2>),
        None,
        None,
    ];
}

impl<T: Config, T2: BytecodeProvider> HasFcDispatchTable<T2> for T {
    const FC_DISPATCH_TABLE: [Option<InstructionHandlerFn<T2>>; 18] = [
        Some(wrappers::i32_trunc_sat_f32_s::<T, T2>),
        Some(wrappers::i32_trunc_sat_f32_u::<T, T2>),
        Some(wrappers::i32_trunc_sat_f64_s::<T, T2>),
        Some(wrappers::i32_trunc_sat_f64_u::<T, T2>),
        Some(wrappers::i64_trunc_sat_f32_s::<T, T2>),
        Some(wrappers::i64_trunc_sat_f32_u::<T, T2>),
        Some(wrappers::i64_trunc_sat_f64_s::<T, T2>),
        Some(wrappers::i64_trunc_sat_f64_u::<T, T2>),
        Some(wrappers::memory_init_fn::<T, T2>),
        Some(wrappers::data_drop_fn::<T, T2>),
        Some(wrappers::memory_copy::<T, T2>),
        Some(wrappers::memory_fill::<T, T2>),
        Some(wrappers::table_init_fn::<T, T2>),
        Some(wrappers::elem_drop_fn::<T, T2>),
        Some(wrappers::table_copy::<T, T2>),
        Some(wrappers::table_grow::<T, T2>),
        Some(wrappers::table_size::<T, T2>),
        Some(wrappers::table_fill::<T, T2>),
    ];
}

impl<T: Config, T2: BytecodeProvider> HasFdDispatchTable<T2> for T {
    const FD_DISPATCH_TABLE: [Option<InstructionHandlerFn<T2>>; 256] = [
        Some(wrappers::v128_load::<T, T2>),
        Some(wrappers::v128_load8x8_s::<T, T2>),
        Some(wrappers::v128_load8x8_u::<T, T2>),
        Some(wrappers::v128_load16x4_s::<T, T2>),
        Some(wrappers::v128_load16x4_u::<T, T2>),
        Some(wrappers::v128_load32x2_s::<T, T2>),
        Some(wrappers::v128_load32x2_u::<T, T2>),
        Some(wrappers::v128_load8_splat::<T, T2>),
        Some(wrappers::v128_load16_splat::<T, T2>),
        Some(wrappers::v128_load32_splat::<T, T2>),
        Some(wrappers::v128_load64_splat::<T, T2>),
        Some(wrappers::v128_store::<T, T2>),
        Some(wrappers::v128_const::<T, T2>),
        Some(wrappers::i8x16_shuffle::<T, T2>),
        Some(wrappers::i8x16_swizzle::<T, T2>),
        Some(wrappers::i8x16_splat::<T, T2>),
        Some(wrappers::i16x8_splat::<T, T2>),
        Some(wrappers::i32x4_splat::<T, T2>),
        Some(wrappers::i64x2_splat::<T, T2>),
        Some(wrappers::f32x4_splat::<T, T2>),
        Some(wrappers::f64x2_splat::<T, T2>),
        Some(wrappers::i8x16_extract_lane_s::<T, T2>),
        Some(wrappers::i8x16_extract_lane_u::<T, T2>),
        Some(wrappers::i8x16_replace_lane::<T, T2>),
        Some(wrappers::i16x8_extract_lane_s::<T, T2>),
        Some(wrappers::i16x8_extract_lane_u::<T, T2>),
        Some(wrappers::i16x8_replace_lane::<T, T2>),
        Some(wrappers::i32x4_extract_lane::<T, T2>),
        Some(wrappers::i32x4_replace_lane::<T, T2>),
        Some(wrappers::i64x2_extract_lane::<T, T2>),
        Some(wrappers::i64x2_replace_lane::<T, T2>),
        Some(wrappers::f32x4_extract_lane::<T, T2>),
        Some(wrappers::f32x4_replace_lane::<T, T2>),
        Some(wrappers::f64x2_extract_lane::<T, T2>),
        Some(wrappers::f64x2_replace_lane::<T, T2>),
        // 35
        Some(wrappers::i8x16_eq::<T, T2>),
        Some(wrappers::i8x16_ne::<T, T2>),
        Some(wrappers::i8x16_lt_s::<T, T2>),
        Some(wrappers::i8x16_lt_u::<T, T2>),
        Some(wrappers::i8x16_gt_s::<T, T2>),
        Some(wrappers::i8x16_gt_u::<T, T2>),
        Some(wrappers::i8x16_le_s::<T, T2>),
        Some(wrappers::i8x16_le_u::<T, T2>),
        Some(wrappers::i8x16_ge_s::<T, T2>),
        Some(wrappers::i8x16_ge_u::<T, T2>),
        // 45
        Some(wrappers::i16x8_eq::<T, T2>),
        Some(wrappers::i16x8_ne::<T, T2>),
        Some(wrappers::i16x8_lt_s::<T, T2>),
        Some(wrappers::i16x8_lt_u::<T, T2>),
        Some(wrappers::i16x8_gt_s::<T, T2>),
        Some(wrappers::i16x8_gt_u::<T, T2>),
        Some(wrappers::i16x8_le_s::<T, T2>),
        Some(wrappers::i16x8_le_u::<T, T2>),
        Some(wrappers::i16x8_ge_s::<T, T2>),
        Some(wrappers::i16x8_ge_u::<T, T2>),
        // 55
        Some(wrappers::i32x4_eq::<T, T2>),
        Some(wrappers::i32x4_ne::<T, T2>),
        Some(wrappers::i32x4_lt_s::<T, T2>),
        Some(wrappers::i32x4_lt_u::<T, T2>),
        Some(wrappers::i32x4_gt_s::<T, T2>),
        Some(wrappers::i32x4_gt_u::<T, T2>),
        Some(wrappers::i32x4_le_s::<T, T2>),
        Some(wrappers::i32x4_le_u::<T, T2>),
        Some(wrappers::i32x4_ge_s::<T, T2>),
        Some(wrappers::i32x4_ge_u::<T, T2>),
        // 65
        Some(wrappers::f32x4_eq::<T, T2>),
        Some(wrappers::f32x4_ne::<T, T2>),
        Some(wrappers::f32x4_lt::<T, T2>),
        Some(wrappers::f32x4_gt::<T, T2>),
        Some(wrappers::f32x4_le::<T, T2>),
        Some(wrappers::f32x4_ge::<T, T2>),
        // 71
        Some(wrappers::f64x2_eq::<T, T2>),
        Some(wrappers::f64x2_ne::<T, T2>),
        Some(wrappers::f64x2_lt::<T, T2>),
        Some(wrappers::f64x2_gt::<T, T2>),
        Some(wrappers::f64x2_le::<T, T2>),
        Some(wrappers::f64x2_ge::<T, T2>),
        // 77
        Some(wrappers::v128_not::<T, T2>),
        Some(wrappers::v128_and::<T, T2>),
        Some(wrappers::v128_andnot::<T, T2>),
        Some(wrappers::v128_or::<T, T2>),
        Some(wrappers::v128_xor::<T, T2>),
        Some(wrappers::v128_bitselect::<T, T2>),
        Some(wrappers::v128_any_true::<T, T2>),
        // 84
        Some(wrappers::v128_load8_lane::<T, T2>),
        Some(wrappers::v128_load16_lane::<T, T2>),
        Some(wrappers::v128_load32_lane::<T, T2>),
        Some(wrappers::v128_load64_lane::<T, T2>),
        Some(wrappers::v128_store8_lane::<T, T2>),
        Some(wrappers::v128_store16_lane::<T, T2>),
        Some(wrappers::v128_store32_lane::<T, T2>),
        Some(wrappers::v128_store64_lane::<T, T2>),
        Some(wrappers::v128_load32_zero::<T, T2>),
        Some(wrappers::v128_load64_zero::<T, T2>),
        // 94
        Some(wrappers::f32x4_demote_f64x2_zero::<T, T2>),
        Some(wrappers::f64x2_promote_low_f32x4::<T, T2>),
        // 96
        Some(wrappers::i8x16_abs::<T, T2>),
        Some(wrappers::i8x16_neg::<T, T2>),
        Some(wrappers::i8x16_popcnt::<T, T2>),
        Some(wrappers::i8x16_all_true::<T, T2>),
        Some(wrappers::i8x16_bitmask::<T, T2>),
        Some(wrappers::i8x16_narrow_i16x8_s::<T, T2>),
        Some(wrappers::i8x16_narrow_i16x8_u::<T, T2>),
        // 103
        Some(wrappers::f32x4_ceil::<T, T2>),
        Some(wrappers::f32x4_floor::<T, T2>),
        Some(wrappers::f32x4_trunc::<T, T2>),
        Some(wrappers::f32x4_nearest::<T, T2>),
        // 107
        Some(wrappers::i8x16_shl::<T, T2>),
        Some(wrappers::i8x16_shr_s::<T, T2>),
        Some(wrappers::i8x16_shr_u::<T, T2>),
        Some(wrappers::i8x16_add::<T, T2>),
        Some(wrappers::i8x16_add_sat_s::<T, T2>),
        Some(wrappers::i8x16_add_sat_u::<T, T2>),
        Some(wrappers::i8x16_sub::<T, T2>),
        Some(wrappers::i8x16_sub_sat_s::<T, T2>),
        Some(wrappers::i8x16_sub_sat_u::<T, T2>),
        // 116
        Some(wrappers::f64x2_ceil::<T, T2>),
        Some(wrappers::f64x2_floor::<T, T2>),
        // 118
        Some(wrappers::i8x16_min_s::<T, T2>),
        Some(wrappers::i8x16_min_u::<T, T2>),
        Some(wrappers::i8x16_max_s::<T, T2>),
        Some(wrappers::i8x16_max_u::<T, T2>),
        // 122
        Some(wrappers::f64x2_trunc::<T, T2>),
        // 123
        Some(wrappers::i8x16_avgr_u::<T, T2>),
        // 124
        Some(wrappers::i16x8_extadd_pairwise_i8x16_s::<T, T2>),
        Some(wrappers::i16x8_extadd_pairwise_i8x16_u::<T, T2>),
        // 126
        Some(wrappers::i32x4_extadd_pairwise_i16x8_s::<T, T2>),
        Some(wrappers::i32x4_extadd_pairwise_i16x8_u::<T, T2>),
        // 128
        Some(wrappers::i16x8_abs::<T, T2>),
        Some(wrappers::i16x8_neg::<T, T2>),
        Some(wrappers::i16x8_q15mulrsat_s::<T, T2>),
        Some(wrappers::i16x8_all_true::<T, T2>),
        Some(wrappers::i16x8_bitmask::<T, T2>),
        Some(wrappers::i16x8_narrow_i32x4_s::<T, T2>),
        Some(wrappers::i16x8_narrow_i32x4_u::<T, T2>),
        Some(wrappers::i16x8_extend_low_i8x16_s::<T, T2>),
        Some(wrappers::i16x8_extend_high_i8x16_s::<T, T2>),
        Some(wrappers::i16x8_extend_low_i8x16_u::<T, T2>),
        Some(wrappers::i16x8_extend_high_i8x16_u::<T, T2>),
        Some(wrappers::i16x8_shl::<T, T2>),
        Some(wrappers::i16x8_shr_s::<T, T2>),
        Some(wrappers::i16x8_shr_u::<T, T2>),
        Some(wrappers::i16x8_add::<T, T2>),
        Some(wrappers::i16x8_add_sat_s::<T, T2>),
        Some(wrappers::i16x8_add_sat_u::<T, T2>),
        Some(wrappers::i16x8_sub::<T, T2>),
        Some(wrappers::i16x8_sub_sat_s::<T, T2>),
        Some(wrappers::i16x8_sub_sat_u::<T, T2>),
        // 148
        Some(wrappers::f64x2_nearest::<T, T2>),
        // 149
        Some(wrappers::i16x8_mul::<T, T2>),
        Some(wrappers::i16x8_min_s::<T, T2>),
        Some(wrappers::i16x8_min_u::<T, T2>),
        Some(wrappers::i16x8_max_s::<T, T2>),
        Some(wrappers::i16x8_max_u::<T, T2>),
        // 154
        None,
        // 155
        Some(wrappers::i16x8_avgr_u::<T, T2>),
        Some(wrappers::i16x8_extmul_low_i8x16_s::<T, T2>),
        Some(wrappers::i16x8_extmul_high_i8x16_s::<T, T2>),
        Some(wrappers::i16x8_extmul_low_i8x16_u::<T, T2>),
        Some(wrappers::i16x8_extmul_high_i8x16_u::<T, T2>),
        // 160
        Some(wrappers::i32x4_abs::<T, T2>),
        Some(wrappers::i32x4_neg::<T, T2>),
        // 162
        None,
        // 163
        Some(wrappers::i32x4_all_true::<T, T2>),
        Some(wrappers::i32x4_bitmask::<T, T2>),
        // 165,
        None,
        None,
        // 167
        Some(wrappers::i32x4_extend_low_i16x8_s::<T, T2>),
        Some(wrappers::i32x4_extend_high_i16x8_s::<T, T2>),
        Some(wrappers::i32x4_extend_low_i16x8_u::<T, T2>),
        Some(wrappers::i32x4_extend_high_i16x8_u::<T, T2>),
        Some(wrappers::i32x4_shl::<T, T2>),
        Some(wrappers::i32x4_shr_s::<T, T2>),
        Some(wrappers::i32x4_shr_u::<T, T2>),
        Some(wrappers::i32x4_add::<T, T2>),
        // 175
        None,
        None,
        // 177
        Some(wrappers::i32x4_sub::<T, T2>),
        // 178
        None,
        None,
        None,
        // 181
        Some(wrappers::i32x4_mul::<T, T2>),
        Some(wrappers::i32x4_min_s::<T, T2>),
        Some(wrappers::i32x4_min_u::<T, T2>),
        Some(wrappers::i32x4_max_s::<T, T2>),
        Some(wrappers::i32x4_max_u::<T, T2>),
        Some(wrappers::i32x4_dot_i16x8_s::<T, T2>),
        // 187
        None,
        // 188
        Some(wrappers::i32x4_extmul_low_i16x8_s::<T, T2>),
        Some(wrappers::i32x4_extmul_high_i16x8_s::<T, T2>),
        Some(wrappers::i32x4_extmul_low_i16x8_u::<T, T2>),
        Some(wrappers::i32x4_extmul_high_i16x8_u::<T, T2>),
        // 192
        Some(wrappers::i64x2_abs::<T, T2>),
        Some(wrappers::i64x2_neg::<T, T2>),
        // 194
        None,
        // 195
        Some(wrappers::i64x2_all_true::<T, T2>),
        Some(wrappers::i64x2_bitmask::<T, T2>),
        // 197
        None,
        None,
        // 199
        Some(wrappers::i64x2_extend_low_i32x4_s::<T, T2>),
        Some(wrappers::i64x2_extend_high_i32x4_s::<T, T2>),
        Some(wrappers::i64x2_extend_low_i32x4_u::<T, T2>),
        Some(wrappers::i64x2_extend_high_i32x4_u::<T, T2>),
        Some(wrappers::i64x2_shl::<T, T2>),
        Some(wrappers::i64x2_shr_s::<T, T2>),
        Some(wrappers::i64x2_shr_u::<T, T2>),
        Some(wrappers::i64x2_add::<T, T2>),
        // 207
        None,
        None,
        // 209
        Some(wrappers::i64x2_sub::<T, T2>),
        // 210
        None,
        None,
        None,
        // 213
        Some(wrappers::i64x2_mul::<T, T2>),
        // 214
        Some(wrappers::i64x2_eq::<T, T2>),
        Some(wrappers::i64x2_ne::<T, T2>),
        Some(wrappers::i64x2_lt_s::<T, T2>),
        Some(wrappers::i64x2_gt_s::<T, T2>),
        Some(wrappers::i64x2_le_s::<T, T2>),
        Some(wrappers::i64x2_ge_s::<T, T2>),
        // 220
        Some(wrappers::i64x2_extmul_low_i32x4_s::<T, T2>),
        Some(wrappers::i64x2_extmul_high_i32x4_s::<T, T2>),
        Some(wrappers::i64x2_extmul_low_i32x4_u::<T, T2>),
        Some(wrappers::i64x2_extmul_high_i32x4_u::<T, T2>),
        // 224
        Some(wrappers::f32x4_abs::<T, T2>),
        Some(wrappers::f32x4_neg::<T, T2>),
        // 226
        None,
        // 227
        Some(wrappers::f32x4_sqrt::<T, T2>),
        Some(wrappers::f32x4_add::<T, T2>),
        Some(wrappers::f32x4_sub::<T, T2>),
        Some(wrappers::f32x4_mul::<T, T2>),
        Some(wrappers::f32x4_div::<T, T2>),
        Some(wrappers::f32x4_min::<T, T2>),
        Some(wrappers::f32x4_max::<T, T2>),
        Some(wrappers::f32x4_pmin::<T, T2>),
        Some(wrappers::f32x4_pmax::<T, T2>),
        // 236
        Some(wrappers::f64x2_abs::<T, T2>),
        Some(wrappers::f64x2_neg::<T, T2>),
        // 238
        None,
        // 239
        Some(wrappers::f64x2_sqrt::<T, T2>),
        Some(wrappers::f64x2_add::<T, T2>),
        Some(wrappers::f64x2_sub::<T, T2>),
        Some(wrappers::f64x2_mul::<T, T2>),
        Some(wrappers::f64x2_div::<T, T2>),
        Some(wrappers::f64x2_min::<T, T2>),
        Some(wrappers::f64x2_max::<T, T2>),
        Some(wrappers::f64x2_pmin::<T, T2>),
        Some(wrappers::f64x2_pmax::<T, T2>),
        // 248
        Some(wrappers::i32x4_trunc_sat_f32x4_s::<T, T2>),
        Some(wrappers::i32x4_trunc_sat_f32x4_u::<T, T2>),
        // 250
        Some(wrappers::f32x4_convert_i32x4_s::<T, T2>),
        Some(wrappers::f32x4_convert_i32x4_u::<T, T2>),
        // 252
        Some(wrappers::i32x4_trunc_sat_f64x2_s_zero::<T, T2>),
        Some(wrappers::i32x4_trunc_sat_f64x2_u_zero::<T, T2>),
        // 254
        Some(wrappers::f64x2_convert_low_i32x4_s::<T, T2>),
        Some(wrappers::f64x2_convert_low_i32x4_u::<T, T2>),
    ];
}
