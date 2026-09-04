//! This module contains common definitions required by the instruction handlers, which exist in
//! submodules of this module.
//!
//! The logic for dispatching the execution of instruction handlers resides in the [`dispatch`]
//! submodule, which itself provides multiple dispatch mechanisms. There execution is started via
//! [`dispatch::run`].
//!
//! Additionally, the [`const_interpreter_loop`] submodule contains the execution logic for const
//! expressions.

use alloc::{boxed::Box, vec::Vec};
use core::{array, num::NonZeroU64, ops::ControlFlow};

use crate::{
    core::{
        decoding::decoder::WasmDecoder,
        sidetable::Sidetable,
        structure::{
            modules::indices::{DataIdx, ElemIdx, MemIdx, TableIdx},
            types::MemArg,
        },
        utils::ToUsizeExt,
    },
    execution::{
        numerics::representations::LittleEndianBytes,
        runtime_structure::{
            data_instances::DataInst,
            element_instances::ElemInst,
            memory_instances::MemInst,
            module_instances::ModuleInst,
            store::{Hostcode, StoreInner},
            table_instances::TableInst,
            value_stack::Stack,
        },
    },
    AddrVec, DataAddr, ElemAddr, FuncAddr, MemAddr, ModuleAddr, RuntimeError, TableAddr, TrapError,
    Value, WasmResumable,
};

mod control;
mod memory;
mod numeric;
mod parametric;
mod reference;
mod table;
mod variable;
mod vector;

pub mod const_interpreter_loop;
pub(crate) mod dispatch;

/// A non-error outcome of interpretation
pub enum InterpreterLoopOutcome {
    /// Execution has returned normally, i.e. the end of the bottom-most function on the stack was
    /// reached. The return values for the initially invoked function are on the stack.
    ExecutionReturned,
    /// Execution was preempted because there was not enough fuel in the [`WasmResumable`] object.
    OutOfFuel {
        /// The amount of fuel required to continue execution at least the next instruction.
        required_fuel: NonZeroU64,
    },
    /// A host function instance was called. The arguments for the host function call have been
    /// collected into `params` already.
    HostCalled {
        func_addr: FuncAddr,
        // TODO this allocation might be preventable. mutably borrow the stack instead
        params: Vec<Value>,
        hostcode: Hostcode,
    },
}

/// The execution state interacted with by all instructions.
///
/// # Safety
///
/// - The [`WasmDecoder`] must point to the Wasm code for the module of the current module instance.
/// - The [`WasmDecoder`] must point into Wasm code of the current function as set in
///   `resumable.current_func_addr`.
/// - The [`StoreInner`] must be valid.
/// - The [`WasmResumable`] must be valid in [`StoreInner`].
/// - All address types contained in this struct must be valid in the [`StoreInner`].
/// - The current sidetable must be correct for the module of the current module instance.
/// - The end marker for the current function must point to the end index of the current function in
///   the current module's bytecode.
// TODO possibly improve safety requirements
pub(crate) struct State<'a, 'sidetable, 'wasm> {
    wasm: &'a mut WasmDecoder<'wasm>,
    resumable: &'a mut WasmResumable,
    current_sidetable: &'a mut &'sidetable Sidetable,
    store_inner: &'a mut StoreInner,
    modules: &'sidetable AddrVec<ModuleAddr, ModuleInst<'wasm>>,
    current_module: &'a mut ModuleAddr,
    current_function_end_marker: &'a mut usize,
}

//helper function for avoiding code duplication at intraprocedural jumps
fn do_sidetable_control_transfer(
    wasm: &mut WasmDecoder,
    stack: &mut Stack,
    current_stp: &mut usize,
    current_sidetable: &Sidetable,
) -> Result<(), RuntimeError> {
    let sidetable_entry = &current_sidetable[*current_stp];

    stack.remove_in_between(sidetable_entry.popcnt, sidetable_entry.valcnt);

    *current_stp = sidetable_entry.stp;
    wasm.pc = sidetable_entry.pc;

    Ok(())
}

#[inline(always)]
fn calculate_mem_address(memarg: &MemArg, relative_address: u32) -> Result<usize, RuntimeError> {
    // The spec states that this should be a 33 bit integer, e.g. it is not legal to wrap if the
    // sum of offset and relative_address exceeds u32::MAX. To emulate this behavior, we use a
    // checked addition.
    // See: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
    let effective_address = memarg
        .offset
        .checked_add(relative_address)
        .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

    Ok(effective_address.into_usize())
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

    let final_src_offset = s
        .checked_add(n)
        .filter(|&res| res <= elem.len())
        .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

    if d.checked_add(n)
        .filter(|&res| res <= tab.len().into_usize())
        .is_none()
    {
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

    // Free the existing memory allocation and replace it with a dangling pointer.
    elem.references = Box::from([]);
}

/// # Safety
///
/// 1. The module address `current_module` must be valid in `store_modules` for some module instance `module_inst`.
/// 2. The memory index `mem_idx` must be valid in `module_inst` for some memory address `mem_addr`.
/// 3. `mem_addr` must be valid in `store_memories` for some memory instance `mem`.
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
    let mem = unsafe { store_memories.get_mut(mem_addr) };
    // SAFETY: The caller ensures that `data_idx` is valid for this specific
    // `IdxVec` (4).
    let data_addr = *unsafe { module_inst.data_addrs.get(data_idx) };
    // SAFETY: The caller ensures that this data address is valid in this
    // address vector (5).
    let data = unsafe { store_data.get(data_addr) };

    match mem {
        MemInst::Unshared(unshared_mem) => {
            unshared_mem.mem.init(d, &data.data, s, n)?;
        }
        MemInst::Shared(shared_mem) => {
            shared_mem.mem.init(d, &data.data, s, n)?;
        }
    }

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

    // Free the existing memory allocation and replace it with a dangling pointer.
    data.data = Box::from([]);
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

#[inline(always)]
fn decrement_fuel(cost: u64, maybe_fuel: &mut Option<u64>) -> ControlFlow<InterpreterLoopOutcome> {
    if let Some(fuel) = maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            return ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            });
        }
    }

    ControlFlow::Continue(())
}
