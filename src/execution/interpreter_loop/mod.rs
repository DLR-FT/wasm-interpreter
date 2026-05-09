//! This module solely contains the actual interpretation loop that matches instructions, interpreting the WASM bytecode
//!
//!
//! # Note to Developer:
//!
//! 1. There must be only imports and one `impl` with one function (`run`) in it.
//! 2. This module must only use [`RuntimeError`] and never [`Error`](crate::core::error::ValidationError).

use alloc::vec::Vec;
use core::{array, hint::unreachable_unchecked, num::NonZeroU64};

use crate::{
    addrs::{AddrVec, DataAddr, ElemAddr, FuncAddr, MemAddr, ModuleAddr, TableAddr},
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{DataIdx, ElemIdx, MemIdx, TableIdx},
        reader::{
            types::{memarg::MemArg, opcode},
            WasmReader,
        },
        sidetable::{Sidetable, SidetableRef},
        utils::ToUsizeExt,
    },
    execution::{
        config::Config,
        interpreter_loop::dispatch_tables::{
            HasBaseDispatchTable, HasFcDispatchTable, HasFdDispatchTable,
        },
        store::Hostcode,
    },
    instances::{DataInst, ElemInst, FuncInst, MemInst, ModuleInst, TableInst},
    opcodes::opcode_byte_to_str,
    resumable::WasmResumable,
    unreachable_validated,
    value_stack::Stack,
    RuntimeError, TrapError, Value,
};

use super::{little_endian::LittleEndianBytes, store::Store, store::StoreInner};

mod control_instructions;
mod memory_instructions;
mod numeric_instructions;
mod parametric_instructions;
mod reference_instructions;
mod table_instructions;
mod variable_instructions;
mod vector_instructions;

mod dispatch_tables;

/// A non-error outcome of execution of the interpreter loop
pub enum InterpreterLoopOutcome {
    /// Execution has returned normally, i.e. the end of the bottom-most
    /// function on the stack was reached.
    ExecutionReturned,
    /// Execution was preempted because there was not enough fuel in the
    /// [`WasmResumable`] object.
    ///
    OutOfFuel {
        /// The amount of fuel required to continue execution at least the next
        /// instruction.
        required_fuel: NonZeroU64,
    },
    HostCalled {
        func_addr: FuncAddr,
        // TODO this allocation might be preventable. mutably borrow the stack
        // instead
        params: Vec<Value>,
        hostcode: Hostcode,
    },
}

type InstructionHandlerFn<T> =
    unsafe fn(Args<T>) -> (Result<InterpreterLoopOutcome, RuntimeError>, WasmResumable);

// A placeholder instruction for unassigned instruction bytes. This function is by definition dead
// code!
define_instruction!(unset, opcode::NOP, |Args { .. }: &mut Args<T>| {
    unreachable_validated!()
});

/// Interprets wasm native functions. Wasm parameters and Wasm return values are passed on the stack.
/// Returns `Ok(None)` in case execution successfully terminates, `Ok(Some(required_fuel))` if execution
/// terminates due to insufficient fuel, indicating how much fuel is required to resume with `required_fuel`,
/// and `[Error::RuntimeError]` otherwise.
///
/// # Safety
///
/// The given resumable must be valid in the given [`Store`].
#[inline(never)]
pub(super) unsafe fn run<T: Config>(
    resumable: WasmResumable,
    store: &mut Store<T>,
) -> Result<(InterpreterLoopOutcome, WasmResumable), RuntimeError> {
    let current_func_addr = resumable.current_func_addr;
    let pc = resumable.pc;
    // SAFETY: The caller ensures that the resumable and thus also its function
    // address is valid in the current store.
    let func_inst = unsafe { store.inner.functions.get(current_func_addr) };
    let FuncInst::WasmFunc(wasm_func_inst) = &func_inst else {
        unsafe { unreachable_unchecked() }
        // unreachable!(
        //     "the interpreter loop shall only be executed with native wasm functions as root call"
        // );
    };
    let current_module = wasm_func_inst.module_addr;

    // Start reading the function's instructions
    // SAFETY: This module address was just read from the current store. Every
    // store guarantees all addresses contained in it to be valid within itself.
    let module = unsafe { store.modules.get(current_module) };
    let wasm_bytecode = module.wasm_bytecode;
    let mut wasm = WasmReader::new(wasm_bytecode);

    let mut current_sidetable: SidetableRef = &module.sidetable;

    // let current_function_end_marker =
    // wasm_func_inst.code_expr.from() + wasm_func_inst.code_expr.len();

    let store_inner = &mut store.inner;
    let user_data = &mut store.user_data;

    // local variable for holding where the function code ends (last END instr address + 1) to avoid lookup at every END instr

    wasm.pc = pc;

    let args = Args {
        store_inner,
        modules: &store.modules,
        wasm,
        current_module,
        // current_function_end_marker,
        current_sidetable: &mut current_sidetable,
        resumable,
        user_data,
        prev_pc: 0, // this is set in dispatch function
    };

    // Throw away the resumable in case of an error. This is not done inside the instruction handlers because of Drop overhead.
    let (result, resumable) = dispatch_wrapper(args);
    result.map(|outcome| (outcome, resumable))
}

macro_rules! dispatch_macro {
    ($args:expr) => {{
        let mut args: Args<T> = $args;

        // call the instruction hook
        args.user_data
            .instruction_hook(args.wasm.full_wasm_binary, args.wasm.pc);

        args.prev_pc = args.wasm.pc;

        let first_instr_byte = args.wasm.read_u8().unwrap_validated();

        #[cfg(debug_assertions)]
        trace!(
            "Executing instruction {}",
            crate::execution::interpreter_loop::opcode_byte_to_str(first_instr_byte)
        );

        use crate::execution::interpreter_loop::dispatch_tables::HasBaseDispatchTable;

        let instruction_handler: crate::execution::interpreter_loop::InstructionHandlerFn<T> =
            unsafe { *T::DISPATCH_TABLE.get_unchecked(usize::from(first_instr_byte)) };

        // SAFETY: All possible instruction handler functions use the same safety requirements, as
        // they are defined through the same macro: The caller ensures that the resumable is valid
        // in the current store. Also all other address types passed via the `Args` must come from
        // the current store itself. Therefore, they are automatically valid in this store.
        unsafe { become instruction_handler(args) }
    }};
}

pub(crate) use dispatch_macro;

// #[inline(always)]
fn dispatch_wrapper<T: Config>(
    args: Args<T>,
) -> (Result<InterpreterLoopOutcome, RuntimeError>, WasmResumable) {
    let _: InstructionHandlerFn<T> = dispatch_wrapper::<T>;
    dispatch_macro!(args)
}

//helper function for avoiding code duplication at intraprocedural jumps
fn do_sidetable_control_transfer(
    wasm: &mut WasmReader,
    stack: &mut Stack,
    current_stp: &mut usize,
    current_sidetable: SidetableRef,
) -> Result<(), RuntimeError> {
    let sidetable_entry = &current_sidetable[*current_stp];

    stack.remove_in_between(sidetable_entry.popcnt, sidetable_entry.valcnt);

    *current_stp = current_stp.checked_add_signed(sidetable_entry.delta_stp)
        .expect("that adding the delta stp never causes the stp to go out of bounds unless there is a bug in the sidetable generation");
    wasm.pc = wasm.pc.checked_add_signed(sidetable_entry.delta_pc)
    .expect("that adding the delta pc never causes the pc to go out of bounds unless there is a bug in the sidetable generation");

    Ok(())
}

#[inline(always)]
fn calculate_mem_address(memarg: &MemArg, relative_address: u32) -> Result<usize, RuntimeError> {
    memarg
        .offset
        // The spec states that this should be a 33 bit integer, e.g. it is not legal to wrap if the
        // sum of offset and relative_address exceeds u32::MAX. To emulate this behavior, we use a
        // checked addition.
        // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
        .checked_add(relative_address)
        .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?
        .try_into()
        .map_err(|_| TrapError::MemoryOrDataAccessOutOfBounds.into())
}

//helpers for avoiding code duplication during module instantiation
/// # Safety
///
/// 1. The module address `current_module` must be valid in `store_modules` for a module instance `module_inst`.
/// 2. The table index `table_idx` must be valid in `module_inst` for a table address `table_addr`.
/// 3. `table_addr` must be valid in `store_tables`.
/// 4. The element index `elem_idx` must be valid in `module_inst` for an element address `elem_addr`.
/// 5. `elem_addr` must be valid in `store_elements`.
// TODO instead of passing all module instances and the current module addr
// separately, directly pass a `&ModuleInst`.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn table_init(
    store_modules: &AddrVec<ModuleAddr, ModuleInst>,
    store_tables: &mut AddrVec<TableAddr, TableInst>,
    store_elements: &AddrVec<ElemAddr, ElemInst>,
    current_module: ModuleAddr,
    elem_idx: ElemIdx,
    table_idx: TableIdx,
    n: u32,
    s: i32,
    d: i32,
) -> Result<(), RuntimeError> {
    let n = n.into_usize();
    let s = s.cast_unsigned().into_usize();
    let d = d.cast_unsigned().into_usize();

    // SAFETY: The caller ensures that this module address is valid in this
    // address vector (1).
    let module_inst = unsafe { store_modules.get(current_module) };
    // SAFETY: The caller ensures that `table_idx` is valid for this specific
    // `IdxVec` (2).
    let table_addr = *unsafe { module_inst.table_addrs.get(table_idx) };
    // SAFETY: The caller ensures that `elem_idx` is valid for this specific
    // `IdxVec` (4).
    let elem_addr = *unsafe { module_inst.elem_addrs.get(elem_idx) };
    // SAFETY: The caller ensures that this table address is valid in this
    // address vector (3).
    let tab = unsafe { store_tables.get_mut(table_addr) };
    // SAFETY: The caller ensures that this element address is valid in this
    // address vector (5).
    let elem = unsafe { store_elements.get(elem_addr) };

    trace!(
        "Instruction: table.init '{}' '{}' [{} {} {}] -> []",
        elem_idx,
        table_idx,
        d,
        s,
        n
    );

    let final_src_offset = s
        .checked_add(n)
        .filter(|&res| res <= elem.len())
        .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

    if d.checked_add(n).filter(|&res| res <= tab.len()).is_none() {
        return Err(TrapError::TableOrElementAccessOutOfBounds.into());
    }

    let dest = &mut tab.elem[d..];
    let src = &elem.references[s..final_src_offset];
    dest[..src.len()].copy_from_slice(src);
    Ok(())
}

/// # Safety
///
/// 1. The module address `current_module` must be valid in `store_modules` for some module instance `module_inst`.
/// 2. The element index `elem_idx` must be valid in `module_inst` for some element address `elem_addr`.
/// 3. `elem_addr` must be valid in `store_elements`.
#[inline(always)]
pub(super) unsafe fn elem_drop(
    store_modules: &AddrVec<ModuleAddr, ModuleInst>,
    store_elements: &mut AddrVec<ElemAddr, ElemInst>,
    current_module: ModuleAddr,
    elem_idx: ElemIdx,
) {
    // WARN: i'm not sure if this is okay or not

    // SAFETY: The caller ensures that this module address is valid in this
    // address vector (1).
    let module_inst = unsafe { store_modules.get(current_module) };
    // SAFETY: The caller ensures that `elem_idx` is valid for this specific
    // `IdxVec` (2).
    let elem_addr = *unsafe { module_inst.elem_addrs.get(elem_idx) };

    // SAFETY: The caller ensures that this element address is valid in this
    // address vector (3).
    let elem = unsafe { store_elements.get_mut(elem_addr) };

    elem.references.clear();
}

/// # Safety
///
/// 1. The module address `current_module` must be valid in `store_modules` for some module instance `module_inst`.
/// 2. The memory index `mem_idx` must be valid in `module_inst` for some memory address `mem_addr`.
/// 3. `mem_addr` must be valid in `store_memories` for some memory instance `mem.
/// 4. The data index `data_idx` must be valid in `module_inst` for some data address `data_addr`.
/// 5. `data_addr` must be valid in `store_data`.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn memory_init(
    store_modules: &AddrVec<ModuleAddr, ModuleInst>,
    store_memories: &mut AddrVec<MemAddr, MemInst>,
    store_data: &AddrVec<DataAddr, DataInst>,
    current_module: ModuleAddr,
    data_idx: DataIdx,
    mem_idx: MemIdx,
    n: u32,
    s: u32,
    d: u32,
) -> Result<(), RuntimeError> {
    let n = n.into_usize();
    let s = s.into_usize();
    let d = d.into_usize();

    // SAFETY: The caller ensures that this is module address is valid in this
    // address vector (1).
    let module_inst = unsafe { store_modules.get(current_module) };
    // SAFETY: The caller ensures that `mem_idx` is valid for this specific
    // `IdxVec` (2).
    let mem_addr = *unsafe { module_inst.mem_addrs.get(mem_idx) };
    // SAFETY: The caller ensures that this memory address is valid in this
    // address vector (3).
    let mem = unsafe { store_memories.get(mem_addr) };
    // SAFETY: The caller ensures that `data_idx` is valid for this specific
    // `IdxVec` (4).
    let data_addr = *unsafe { module_inst.data_addrs.get(data_idx) };
    // SAFETY: The caller ensures that this data address is valid in this
    // address vector (5).
    let data = unsafe { store_data.get(data_addr) };

    mem.mem.init(d, &data.data, s, n)?;

    trace!("Instruction: memory.init");
    Ok(())
}

/// # Safety
///
/// 1. The module address `current_module` must be valid in `store_modules` for some module instance `module_inst`.
/// 2. The data index `data_idx` must be valid in `module_inst` for some data address `data_addr`.
/// 3. `data_addr` must be valid in `store_data`.
#[inline(always)]
pub(super) unsafe fn data_drop(
    store_modules: &AddrVec<ModuleAddr, ModuleInst>,
    store_data: &mut AddrVec<DataAddr, DataInst>,
    current_module: ModuleAddr,
    data_idx: DataIdx,
) {
    // Here is debatable
    // If we were to be on par with the spec we'd have to use a DataInst struct
    // But since memory.init is specifically made for Passive data segments
    // I thought that using DataMode would be better because we can see if the
    // data segment is passive or active

    // Also, we should set data to null here (empty), which we do by clearing it
    // SAFETY: The caller guarantees this module to be valid in this address
    // vector (1).
    let module_inst = unsafe { store_modules.get(current_module) };
    // SAFETY: The caller ensures that `data_idx` is valid for this specific
    // `IdxVec` (2).
    let data_addr = *unsafe { module_inst.data_addrs.get(data_idx) };
    // SAFETY: The caller ensures that this data address is valid in this
    // address vector (3).
    let data = unsafe { store_data.get_mut(data_addr) };

    data.data.clear();
}

#[inline(always)]
pub(crate) fn to_lanes<const M: usize, const N: usize, T: LittleEndianBytes<M>>(
    data: [u8; 16],
) -> [T; N] {
    assert_eq!(M * N, 16);

    let mut lanes = data
        .chunks(M)
        .map(|chunk| T::from_le_bytes(chunk.try_into().unwrap()));
    array::from_fn(|_| lanes.next().unwrap())
}

#[inline(always)]
pub(crate) fn from_lanes<const M: usize, const N: usize, T: LittleEndianBytes<M>>(
    lanes: [T; N],
) -> [u8; 16] {
    assert_eq!(M * N, 16);

    let mut bytes = lanes.into_iter().flat_map(T::to_le_bytes);
    array::from_fn(|_| bytes.next().unwrap())
}

pub(crate) struct Args<'sidetable, 'wasm, 'other, 'user_data, T> {
    wasm: WasmReader<'wasm>,
    resumable: WasmResumable,
    current_sidetable: SidetableRef<'sidetable>,
    store_inner: &'other mut StoreInner,
    modules: &'sidetable AddrVec<ModuleAddr, ModuleInst<'wasm>>,
    current_module: ModuleAddr,
    // current_function_end_marker: usize,
    user_data: &'user_data mut T,
    prev_pc: usize,
}

macro_rules! define_instruction {
    (no_fuel_check, $name:ident, $opcode:expr, $contents:expr) => {
        /// # Safety
        ///
        /// The given [`WasmResumable`](crate::execution::resumable::WasmResumable) and all address
        /// types contained in the [`Args`](crate::execution::interpreter_loop::Args) must be valid
        /// in the [`StoreInner`](crate::execution::store::StoreInner) that is also contained in the
        /// [`Args`](crate::execution::interpreter_loop::Args).
        // Disable inlining to inspect the emitted code of individual instruction handlers:
        // #[inline(never)]
        pub(crate) unsafe fn $name<T: crate::config::Config>(
            mut args: Args<T>,
        ) -> (
            Result<crate::execution::interpreter_loop::InterpreterLoopOutcome, crate::RuntimeError>,
            crate::execution::resumable::WasmResumable,
        ) {
            let instruction_handler = $contents;

            let maybe_interpreter_loop_outcome: Result<
                Option<crate::execution::interpreter_loop::InterpreterLoopOutcome>,
                crate::RuntimeError,
            > = instruction_handler(&mut args);

            let maybe_outcome = match maybe_interpreter_loop_outcome {
                Ok(maybe_outcome) => maybe_outcome,
                Err(err) => return (Err(err), args.resumable),
            };

            if let Some(interpreter_loop_outcome) = maybe_outcome {
                if let crate::execution::interpreter_loop::InterpreterLoopOutcome::OutOfFuel {
                    ..
                } = interpreter_loop_outcome
                {
                    args.wasm.pc = args.prev_pc;
                }

                args.resumable.pc = args.wasm.pc;
                return (Ok(interpreter_loop_outcome), args.resumable);
            }

            crate::execution::interpreter_loop::dispatch_macro!(args)
        }
    };

    ($name:ident, $opcode:expr, $contents:expr) => {
        define_instruction!(no_fuel_check, $name, $opcode, |args: &mut Args<T>| {
            if let Some(outcome) = crate::execution::interpreter_loop::decrement_fuel(
                T::get_flat_cost($opcode),
                &mut args.resumable.maybe_fuel,
            ) {
                return Ok(Some(outcome));
            }

            $contents(args)
        });
    };

    (fc_fuel_check, $name: ident, $opcode: expr, $contents:expr) => {
        define_instruction!(no_fuel_check, $name, $opcode, |args: &mut Args<T>| {
            if let Some(outcome) = crate::execution::interpreter_loop::decrement_fuel(
                T::get_fc_extension_flat_cost($opcode),
                &mut args.resumable.maybe_fuel,
            ) {
                return Ok(Some(outcome));
            }

            $contents(args)
        });
    };

    (fd_fuel_check, $name: ident, $opcode: expr, $contents:expr) => {
        define_instruction!(no_fuel_check, $name, $opcode, |args: &mut Args<T>| {
            if let Some(outcome) = crate::execution::interpreter_loop::decrement_fuel(
                T::get_fd_extension_flat_cost($opcode),
                &mut args.resumable.maybe_fuel,
            ) {
                return Ok(Some(outcome));
            }

            $contents(args)
        });
    };
}

pub(crate) use define_instruction;

#[inline(always)]
fn decrement_fuel(cost: u64, maybe_fuel: &mut Option<u64>) -> Option<InterpreterLoopOutcome> {
    if let Some(fuel) = maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            return Some(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            });
        }
    }

    None
}

/// # Safety
///
/// The given [`WasmResumable`](crate::execution::resumable::WasmResumable) and all address
/// types contained in the [`Args`](crate::execution::interpreter_loop::Args) must be valid
/// in the [`StoreInner`](crate::execution::store::StoreInner) that is also contained in the
/// [`Args`](crate::execution::interpreter_loop::Args).
pub(crate) unsafe fn fc_extensions<T: crate::config::Config>(
    mut args: Args<T>,
) -> (
    Result<crate::execution::interpreter_loop::InterpreterLoopOutcome, crate::RuntimeError>,
    WasmResumable,
) {
    // should we call instruction hook here as well? multibyte instruction
    let second_instr = args.wasm.read_var_u32().unwrap_validated();

    let instruction_fn: InstructionHandlerFn<T> = *T::FC_DISPATCH_TABLE
        .get(second_instr.into_usize())
        .expect("the instruction to be valid because the code is validated");

    // SAFETY: All possible instruction handler functions use the same safety requirements, as
    // they are defined through the same macro: The caller ensures that the resumable is valid
    // in the current store. Also all other address types passed via the `Args` must come from
    // the current store itself. Therefore, they are automatically valid in this store.
    unsafe { become instruction_fn(args) }
}

/// # Safety
///
/// The given [`WasmResumable`](crate::execution::resumable::WasmResumable) and all address
/// types contained in the [`Args`](crate::execution::interpreter_loop::Args) must be valid
/// in the [`StoreInner`](crate::execution::store::StoreInner) that is also contained in the
/// [`Args`](crate::execution::interpreter_loop::Args).
#[inline(never)]
pub(crate) unsafe fn fd_extensions<T: crate::config::Config>(
    mut args: Args<T>,
) -> (
    Result<crate::execution::interpreter_loop::InterpreterLoopOutcome, crate::RuntimeError>,
    WasmResumable,
) {
    // Should we call instruction hook here as well? Multibyte instruction
    let second_instr = args.wasm.read_var_u32().unwrap_validated();

    let instruction_fn: InstructionHandlerFn<T> = *T::FD_DISPATCH_TABLE
        .get(second_instr.into_usize())
        .expect("the instruction to be valid because the code is validated");

    // SAFETY: All possible instruction handler functions use the same safety requirements, as
    // they are defined through the same macro: The caller ensures that the resumable is valid
    // in the current store. Also all other address types passed via the `Args` must come from
    // the current store itself. Therefore, they are automatically valid in this store.
    unsafe { become instruction_fn(args) }
}
