use core::ops::ControlFlow;

use crate::{
    core::{decoding::decoder::WasmDecoder, sidetable::Sidetable, structure::instructions},
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{
            decrement_fuel,
            dispatch::{for_all_instructions, for_all_instructions_fc, for_all_instructions_fd},
            InterpreterLoopOutcome, State,
        },
        runtime_structure::function_instances::FuncInst,
    },
    Config, RuntimeError, Store, WasmResumable,
};

/// Interprets Wasm bytecode using a loop-match construct.
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
/// The given resumable must be valid in the given store.
#[inline(never)]
pub unsafe fn run<T: Config>(
    resumable: &mut WasmResumable,
    store: &mut Store<T>,
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
    let wasm_bytecode = module.wasm_bytecode;
    let wasm = &mut WasmDecoder::new(wasm_bytecode);

    let mut current_sidetable: &Sidetable = &module.sidetable;

    let mut current_function_end_marker =
        wasm_func_inst.code_expr.from() + wasm_func_inst.code_expr.len();

    let store_inner = &mut store.inner;

    wasm.pc = pc;

    let mut prev_pc;

    let outcome = loop {
        // call the instruction hook
        store.user_data.instruction_hook(wasm_bytecode, wasm.pc);

        prev_pc = wasm.pc;

        let first_instr_byte = wasm.decode_u8().unwrap_validated();

        let state = State {
            store_inner,
            modules: &store.modules,
            wasm,
            current_module: &mut current_module,
            current_function_end_marker: &mut current_function_end_marker,
            current_sidetable: &mut current_sidetable,
            resumable,
        };

        match first_instr_byte {
            instructions::FC_EXTENSIONS => {
                let second_instr_byte = state.wasm.decode_var_u32().unwrap_validated();
                macro_rules! make_match_fc {
                    ($(($name:ident, $handler_fn:path, $opcode:path, $fuel_check:expr)),*) => {
                        match second_instr_byte {
                            $(
                                $opcode => {
                                    if $fuel_check {
                                        let opcode: u32 = $opcode;
                                        if let ControlFlow::Break(outcome) = decrement_fuel(
                                            T::get_fc_extension_flat_cost(opcode),
                                            &mut state.resumable.maybe_fuel,
                                        ) {
                                            break outcome;
                                        }
                                    }
                                    if let ControlFlow::Break(outcome) = unsafe { $handler_fn(state) }? {
                                        break outcome;
                                    }
                                }
                            ),*,
                            _ => {
                                unreachable!("invalid instruction is impossible if expression is validated")
                            }
                        }
                    }
                }
                for_all_instructions_fc!(make_match_fc);
            }
            instructions::FD_EXTENSIONS => {
                let second_instr_byte = state.wasm.decode_var_u32().unwrap_validated();
                macro_rules! make_match_fd {
                    ($(($name:ident, $handler_fn:path, $opcode:path, $fuel_check:expr)),*) => {
                        match second_instr_byte {
                            $(
                                $opcode => {
                                    if $fuel_check {
                                        let opcode: u32 = $opcode;
                                        if let ControlFlow::Break(outcome) = decrement_fuel(
                                            T::get_fd_extension_flat_cost(opcode),
                                            &mut state.resumable.maybe_fuel,
                                        ) {
                                            break outcome;
                                        }
                                    }
                                    if let ControlFlow::Break(outcome) = unsafe { $handler_fn(state) }? {
                                        break outcome;
                                    }
                                }
                            ),*,
                            _ => {
                                unreachable!("invalid instruction is impossible if expression is validated")
                            }
                        }
                    }
                }
                for_all_instructions_fd!(make_match_fd);
            }
            _ => {
                macro_rules! make_match {
                    ($(($name:ident, $handler_fn:path, $opcode:path, $fuel_check:expr)),*) => {
                        match first_instr_byte {
                            $(
                                $opcode => {
                                    if $fuel_check {
                                        let opcode: u8 = $opcode;
                                        if let ControlFlow::Break(outcome) = decrement_fuel(
                                            T::get_flat_cost(opcode),
                                            &mut state.resumable.maybe_fuel,
                                        ) {
                                            break outcome;
                                        }
                                    }

                                    // SAFETY: All possible instruction handler functions use the
                                    // same safety requirements documented on `State`.  The caller
                                    // ensures that the resumable is valid in the current store.
                                    // Also all other address types passed via the `State` must come
                                    // from the current store itself.  Therefore, they are
                                    // automatically valid in this store.
                                    if let ControlFlow::Break(outcome) = unsafe { $handler_fn(state) }? {
                                        break outcome;
                                    }
                                }
                            ),*,
                            _ => {
                                unreachable!("invalid instruction is impossible if expression is validated")
                            }
                        }
                    }
                }
                for_all_instructions!(make_match);
            }
        }
    };

    if let InterpreterLoopOutcome::OutOfFuel { .. } = outcome {
        wasm.pc = prev_pc;
    }

    resumable.pc = wasm.pc;

    Ok(outcome)
}
