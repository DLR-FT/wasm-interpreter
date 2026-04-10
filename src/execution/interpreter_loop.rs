//! This module solely contains the actual interpretation loop that matches instructions, interpreting the WASM bytecode
//!
//!
//! # Note to Developer:
//!
//! 1. There must be only imports and one `impl` with one function (`run`) in it.
//! 2. This module must only use [`RuntimeError`] and never [`Error`](crate::core::error::ValidationError).

use alloc::vec::Vec;
use core::{
    num::NonZeroU64,
    {
        array,
        ops::{Add, Div, Mul, Neg, Sub},
    },
};

use crate::{
    addrs::{AddrVec, DataAddr, ElemAddr, FuncAddr, MemAddr, ModuleAddr, TableAddr},
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{
            read_label_idx_unchecked, DataIdx, ElemIdx, FuncIdx, GlobalIdx, Idx, LocalIdx, MemIdx,
            TableIdx, TypeIdx,
        },
        reader::{
            types::{memarg::MemArg, opcode, BlockType},
            WasmReader,
        },
        sidetable::Sidetable,
        utils::ToUsizeExt,
    },
    execution::store::Hostcode,
    instances::{DataInst, ElemInst, FuncInst, MemInst, ModuleInst, TableInst},
    resumable::WasmResumable,
    unreachable_validated,
    value::{self, Ref, F32, F64},
    value_stack::Stack,
    RefType, RuntimeError, TrapError, ValType, Value,
};

use crate::execution::config::Config;

use super::{little_endian::LittleEndianBytes, store::Store, store::StoreInner};

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

/// Interprets wasm native functions. Wasm parameters and Wasm return values are passed on the stack.
/// Returns `Ok(None)` in case execution successfully terminates, `Ok(Some(required_fuel))` if execution
/// terminates due to insufficient fuel, indicating how much fuel is required to resume with `required_fuel`,
/// and `[Error::RuntimeError]` otherwise.
///
/// # Safety
///
/// The given resumable must be valid in the given [`Store`].
pub(super) unsafe fn run<T: Config>(
    resumable: &mut WasmResumable,
    store: &mut Store<T>,
) -> Result<InterpreterLoopOutcome, RuntimeError> {
    let stack = &mut resumable.stack;
    let mut current_func_addr = resumable.current_func_addr;
    let pc = resumable.pc;
    let mut stp = resumable.stp;
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
    let wasm = &mut WasmReader::new(wasm_bytecode);

    let mut current_sidetable: &Sidetable = &module.sidetable;

    let mut current_function_end_marker =
        wasm_func_inst.code_expr.from() + wasm_func_inst.code_expr.len();

    let store_inner = &mut store.inner;

    // local variable for holding where the function code ends (last END instr address + 1) to avoid lookup at every END instr

    wasm.pc = pc;

    use crate::core::reader::types::opcode::*;
    loop {
        // call the instruction hook
        store.user_data.instruction_hook(wasm_bytecode, wasm.pc);

        let prev_pc = wasm.pc;

        let first_instr_byte = wasm.read_u8().unwrap_validated();

        #[cfg(debug_assertions)]
        trace!(
            "Executing instruction {}",
            opcode_byte_to_str(first_instr_byte)
        );

        let instruction_fn = match first_instr_byte {
            NOP => nop::<T>,
            END => end::<T>,
            IF => r#if::<T>,
            ELSE => r#else::<T>,
            BR_IF => br_if::<T>,
            BR_TABLE => br_table::<T>,
            BR => br::<T>,
            BLOCK => block::<T>,
            LOOP => r#loop::<T>,
            RETURN => r#return::<T>,
            CALL => call::<T>,
            CALL_INDIRECT => call_indirect::<T>,
            DROP => drop::<T>,
            SELECT => select::<T>,
            SELECT_T => select_t::<T>,
            LOCAL_GET => local_get::<T>,
            LOCAL_SET => local_set::<T>,
            LOCAL_TEE => local_tee::<T>,
            GLOBAL_GET => global_get::<T>,
            GLOBAL_SET => global_set::<T>,
            TABLE_GET => table_get::<T>,
            TABLE_SET => table_set::<T>,
            UNREACHABLE => unreachable::<T>,
            I32_LOAD => i32_load::<T>,
            I64_LOAD => i64_load::<T>,
            F32_LOAD => f32_load::<T>,
            F64_LOAD => f64_load::<T>,
            I32_LOAD8_S => i32_load8_s::<T>,
            I32_LOAD8_U => i32_load8_u::<T>,
            I32_LOAD16_S => i32_load16_s::<T>,
            I32_LOAD16_U => i32_load16_u::<T>,
            I64_LOAD8_S => i64_load8_s::<T>,
            I64_LOAD8_U => i64_load8_u::<T>,
            I64_LOAD16_S => i64_load16_s::<T>,
            I64_LOAD16_U => i64_load16_u::<T>,
            I64_LOAD32_S => i64_load32_s::<T>,
            I64_LOAD32_U => i64_load32_u::<T>,
            I32_STORE => i32_store::<T>,
            I64_STORE => i64_store::<T>,
            F32_STORE => f32_store::<T>,
            F64_STORE => f64_store::<T>,
            I32_STORE8 => i32_store8::<T>,
            I32_STORE16 => i32_store16::<T>,
            I64_STORE8 => i64_store8::<T>,
            I64_STORE16 => i64_store16::<T>,
            I64_STORE32 => i64_store32::<T>,
            MEMORY_SIZE => memory_size::<T>,
            MEMORY_GROW => memory_grow::<T>,
            I32_CONST => i32_const::<T>,
            F32_CONST => f32_const::<T>,
            I32_EQZ => i32_eqz::<T>,
            I32_EQ => i32_eq::<T>,
            I32_NE => i32_ne::<T>,
            I32_LT_S => i32_lt_s::<T>,
            I32_LT_U => i32_lt_u::<T>,
            I32_GT_S => i32_gt_s::<T>,
            I32_GT_U => i32_gt_u::<T>,
            I32_LE_S => i32_le_s::<T>,
            I32_LE_U => i32_le_u::<T>,
            I32_GE_S => i32_ge_s::<T>,
            I32_GE_U => i32_ge_u::<T>,
            I64_EQZ => i64_eqz::<T>,
            I64_EQ => i64_eq::<T>,
            I64_NE => i64_ne::<T>,
            I64_LT_S => i64_lt_s::<T>,
            I64_LT_U => i64_lt_u::<T>,
            I64_GT_S => i64_gt_s::<T>,
            I64_GT_U => i64_gt_u::<T>,
            I64_LE_S => i64_le_s::<T>,
            I64_LE_U => i64_le_u::<T>,
            I64_GE_S => i64_ge_s::<T>,
            I64_GE_U => i64_ge_u::<T>,
            F32_EQ => f32_eq::<T>,
            F32_NE => f32_ne::<T>,
            F32_LT => f32_lt::<T>,
            F32_GT => f32_gt::<T>,
            F32_LE => f32_le::<T>,
            F32_GE => f32_ge::<T>,
            F64_EQ => f64_eq::<T>,
            F64_NE => f64_ne::<T>,
            F64_LT => f64_lt::<T>,
            F64_GT => f64_gt::<T>,
            F64_LE => f64_le::<T>,
            F64_GE => f64_ge::<T>,
            I32_CLZ => i32_clz::<T>,
            I32_CTZ => i32_ctz::<T>,
            I32_POPCNT => i32_popcnt::<T>,
            I64_CONST => i64_const::<T>,
            F64_CONST => f64_const::<T>,
            I32_ADD => i32_add::<T>,
            I32_SUB => i32_sub::<T>,
            I32_MUL => i32_mul::<T>,
            I32_DIV_S => i32_div_s::<T>,
            I32_DIV_U => i32_div_u::<T>,
            I32_REM_S => i32_rem_s::<T>,
            I64_CLZ => i64_clz::<T>,
            I64_CTZ => i64_ctz::<T>,
            I64_POPCNT => i64_popcnt::<T>,
            I64_ADD => i64_add::<T>,
            I64_SUB => i64_sub::<T>,
            I64_MUL => i64_mul::<T>,
            I64_DIV_S => i64_div_s::<T>,
            I64_DIV_U => i64_div_u::<T>,
            I64_REM_S => i64_rem_s::<T>,
            I64_REM_U => i64_rem_u::<T>,
            I64_AND => i64_and::<T>,
            I64_OR => i64_or::<T>,
            I64_XOR => i64_xor::<T>,
            I64_SHL => i64_shl::<T>,
            I64_SHR_S => i64_shr_s::<T>,
            I64_SHR_U => i64_shr_u::<T>,
            I64_ROTL => i64_rotl::<T>,
            I64_ROTR => i64_rotr::<T>,
            I32_REM_U => i32_rem_u::<T>,
            I32_AND => i32_and::<T>,
            I32_OR => i32_or::<T>,
            I32_XOR => i32_xor::<T>,
            I32_SHL => i32_shl::<T>,
            I32_SHR_S => i32_shr_s::<T>,
            I32_SHR_U => i32_shr_u::<T>,
            I32_ROTL => i32_rotl::<T>,
            I32_ROTR => i32_rotr::<T>,
            F32_ABS => f32_abs::<T>,
            F32_NEG => f32_neg::<T>,
            F32_CEIL => f32_ceil::<T>,
            F32_FLOOR => f32_floor::<T>,
            F32_TRUNC => f32_trunc::<T>,
            F32_NEAREST => f32_nearest::<T>,
            F32_SQRT => f32_sqrt::<T>,
            F32_ADD => f32_add::<T>,
            F32_SUB => f32_sub::<T>,
            F32_MUL => f32_mul::<T>,
            F32_DIV => f32_div::<T>,
            F32_MIN => f32_min::<T>,
            F32_MAX => f32_max::<T>,
            F32_COPYSIGN => f32_copysign::<T>,
            F64_ABS => f64_abs::<T>,
            F64_NEG => f64_neg::<T>,
            F64_CEIL => f64_ceil::<T>,
            F64_FLOOR => f64_floor::<T>,
            F64_TRUNC => f64_trunc::<T>,
            F64_NEAREST => f64_nearest::<T>,
            F64_SQRT => f64_sqrt::<T>,
            F64_ADD => f64_add::<T>,
            F64_SUB => f64_sub::<T>,
            F64_MUL => f64_mul::<T>,
            F64_DIV => f64_div::<T>,
            F64_MIN => f64_min::<T>,
            F64_MAX => f64_max::<T>,
            F64_COPYSIGN => f64_copysign::<T>,
            I32_WRAP_I64 => i32_wrap_i64::<T>,
            I32_TRUNC_F32_S => i32_trunc_f32_s::<T>,
            I32_TRUNC_F32_U => i32_trunc_f32_u::<T>,
            I32_TRUNC_F64_S => i32_trunc_f64_s::<T>,
            I32_TRUNC_F64_U => i32_trunc_f64_u::<T>,
            I64_EXTEND_I32_S => i64_extend_i32_s::<T>,
            I64_EXTEND_I32_U => i64_extend_i32_u::<T>,
            I64_TRUNC_F32_S => i64_trunc_f32_s::<T>,
            I64_TRUNC_F32_U => i64_trunc_f32_u::<T>,
            I64_TRUNC_F64_S => i64_trunc_f64_s::<T>,
            I64_TRUNC_F64_U => i64_trunc_f64_u::<T>,
            F32_CONVERT_I32_S => f32_convert_i32_s::<T>,
            F32_CONVERT_I32_U => f32_convert_i32_u::<T>,
            F32_CONVERT_I64_S => f32_convert_i64_s::<T>,
            F32_CONVERT_I64_U => f32_convert_i64_u::<T>,
            F32_DEMOTE_F64 => f32_demote_f64::<T>,
            F64_CONVERT_I32_S => f64_convert_i32_s::<T>,
            F64_CONVERT_I32_U => f64_convert_i32_u::<T>,
            F64_CONVERT_I64_S => f64_convert_i64_s::<T>,
            F64_CONVERT_I64_U => f64_convert_i64_u::<T>,
            F64_PROMOTE_F32 => f64_promote_f32::<T>,
            I32_REINTERPRET_F32 => i32_reinterpret_f32::<T>,
            I64_REINTERPRET_F64 => i64_reinterpret_f64::<T>,
            F32_REINTERPRET_I32 => f32_reinterpret_i32::<T>,
            F64_REINTERPRET_I64 => f64_reinterpret_i64::<T>,
            REF_NULL => ref_null::<T>,
            REF_IS_NULL => ref_is_null::<T>,
            REF_FUNC => ref_func::<T>,
            I32_EXTEND8_S => i32_extend8_s::<T>,
            I32_EXTEND16_S => i32_extend16_s::<T>,
            I64_EXTEND8_S => i64_extend8_s::<T>,
            I64_EXTEND16_S => i64_extend16_s::<T>,
            I64_EXTEND32_S => i64_extend32_s::<T>,
            FC_EXTENSIONS => fc_extensions::<T>,
            FD_EXTENSIONS => fd_extensions::<T>,

            // Unimplemented or invalid instructions
            0x06..=0x0A
            | 0x12..=0x19
            | 0x1C..=0x1F
            | 0x25..=0x27
            | 0xC0..=0xFA
            | 0xFB
            | 0xFE
            | 0xFF => {
                unreachable_validated!();
            }
        };

        let args = Args {
            store_inner,
            modules: &store.modules,
            prev_pc,
            stack,
            wasm,
            stp: &mut stp,
            current_func_addr: &mut current_func_addr,
            current_module: &mut current_module,
            maybe_fuel: &mut resumable.maybe_fuel,
            current_function_end_marker: &mut current_function_end_marker,
            current_sidetable: &mut current_sidetable,
        };

        // SAFETY: All possible instruction handler functions use the same safety requirements, as
        // they are defined through the same macro: The caller ensures that the resumable is valid
        // in the current store. Also all other address types passed via the `Args` must come from
        // the current store itself. Therefore, they are automatically valid in this store.
        let maybe_interpreter_loop_outcome = unsafe { instruction_fn(args) }?;

        if let Some(interpreter_loop_outcome) = maybe_interpreter_loop_outcome {
            if let InterpreterLoopOutcome::OutOfFuel { .. } = interpreter_loop_outcome {
                wasm.pc = prev_pc;
            }

            resumable.current_func_addr = current_func_addr;
            resumable.stp = stp;
            resumable.pc = wasm.pc;
            return Ok(interpreter_loop_outcome);
        }
    }
}

//helper function for avoiding code duplication at intraprocedural jumps
fn do_sidetable_control_transfer(
    wasm: &mut WasmReader,
    stack: &mut Stack,
    current_stp: &mut usize,
    current_sidetable: &Sidetable,
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
fn to_lanes<const M: usize, const N: usize, T: LittleEndianBytes<M>>(data: [u8; 16]) -> [T; N] {
    assert_eq!(M * N, 16);

    let mut lanes = data
        .chunks(M)
        .map(|chunk| T::from_le_bytes(chunk.try_into().unwrap()));
    array::from_fn(|_| lanes.next().unwrap())
}

#[inline(always)]
fn from_lanes<const M: usize, const N: usize, T: LittleEndianBytes<M>>(lanes: [T; N]) -> [u8; 16] {
    assert_eq!(M * N, 16);

    let mut bytes = lanes.into_iter().flat_map(T::to_le_bytes);
    array::from_fn(|_| bytes.next().unwrap())
}

struct Args<'a, 'sidetable, 'wasm, 'other> {
    store_inner: &'other mut StoreInner,
    modules: &'sidetable AddrVec<ModuleAddr, ModuleInst<'wasm>>,
    prev_pc: usize,
    stack: &'a mut Stack,
    wasm: &'a mut WasmReader<'wasm>,
    stp: &'a mut usize,
    current_func_addr: &'a mut FuncAddr,
    current_module: &'a mut ModuleAddr,
    maybe_fuel: &'a mut Option<u64>,
    current_function_end_marker: &'a mut usize,
    current_sidetable: &'a mut &'sidetable Sidetable,
}

macro_rules! define_instruction {
    (no_fuel_check, $name:ident, $opcode:expr, $contents:expr) => {
        /// # Safety
        ///
        /// The given [`WasmResumable`] and all address types contained in the [`Args`] must be
        /// valid in the [`StoreInner`] that is also contained in the [`Args`].
        // Disable inlining to inspect the emitted code of individual instruction handlers
        // #[inline(never)]
        unsafe fn $name<T: Config>(
            args: Args,
        ) -> Result<Option<InterpreterLoopOutcome>, RuntimeError> {
            $contents(args)
        }
    };

    ($name:ident, $opcode:expr, $contents:expr) => {
        define_instruction!(no_fuel_check, $name, $opcode, |args: Args| {
            if let Some(outcome) = decrement_fuel(
                T::get_flat_cost($opcode),
                args.maybe_fuel,
                args.wasm,
                args.prev_pc,
            ) {
                return Ok(Some(outcome));
            }

            $contents(args)
        });
    };

    (fc_fuel_check, $name: ident, $opcode: expr, $contents:expr) => {
        define_instruction!(no_fuel_check, $name, $opcode, |args: Args| {
            if let Some(outcome) = decrement_fuel(
                T::get_fc_extension_flat_cost($opcode),
                args.maybe_fuel,
                args.wasm,
                args.prev_pc,
            ) {
                return Ok(Some(outcome));
            }

            $contents(args)
        });
    };

    (fd_fuel_check, $name: ident, $opcode: expr, $contents:expr) => {
        define_instruction!(no_fuel_check, $name, $opcode, |args: Args| {
            if let Some(outcome) = decrement_fuel(
                T::get_fd_extension_flat_cost($opcode),
                args.maybe_fuel,
                args.wasm,
                args.prev_pc,
            ) {
                return Ok(Some(outcome));
            }

            $contents(args)
        });
    };
}

#[inline(always)]
fn decrement_fuel(
    cost: u64,
    maybe_fuel: &mut Option<u64>,
    wasm: &mut WasmReader,
    prev_pc: usize,
) -> Option<InterpreterLoopOutcome> {
    if let Some(fuel) = maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            wasm.pc = prev_pc; // the instruction was fetched already, we roll this back
            return Some(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            });
        }
    }

    None
}

define_instruction!(nop, opcode::NOP, |_args| Ok(None));

define_instruction!(end, opcode::END, |Args {
                                           store_inner,
                                           modules,
                                           stack,
                                           wasm,
                                           stp,
                                           current_func_addr,
                                           current_module,
                                           current_function_end_marker,
                                           current_sidetable,
                                           ..
                                       }| {
    // There might be multiple ENDs in a single function. We want to
    // exit only when the outermost block (aka function block) ends.
    if wasm.pc != *current_function_end_marker {
        return Ok(None);
    }

    let Some((maybe_return_func_addr, maybe_return_address, maybe_return_stp)) =
        stack.pop_call_frame()
    else {
        // We finished this entire invocation if this was the base call frame.
        return Ok(Some(InterpreterLoopOutcome::ExecutionReturned));
    };
    // If there are one or more call frames, we need to continue
    // from where the callee was called from.

    trace!("end of function reached, returning to previous call frame");
    *current_func_addr = maybe_return_func_addr;

    // SAFETY: The current function address must come from the given
    // resumable or the current store, because these are the only
    // parameters to this function. The resumable, including its
    // function address, is guaranteed to be valid in the current
    // store by the caller, and the store can only contain addresses
    // that are valid within itself.
    let current_function = unsafe { store_inner.functions.get(*current_func_addr) };
    let FuncInst::WasmFunc(current_wasm_func_inst) = current_function else {
        unreachable!("function addresses on the stack always correspond to native wasm functions")
    };
    *current_module = current_wasm_func_inst.module_addr;

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { modules.get(*current_module) };

    wasm.full_wasm_binary = module.wasm_bytecode;
    wasm.pc = maybe_return_address;
    *stp = maybe_return_stp;

    *current_sidetable = &module.sidetable;

    *current_function_end_marker =
        current_wasm_func_inst.code_expr.from() + current_wasm_func_inst.code_expr.len();

    trace!("Instruction: END");

    Ok(None)
});

define_instruction!(r#if, opcode::IF, |Args {
                                           stack,
                                           wasm,
                                           stp,
                                           current_sidetable,
                                           ..
                                       }| {
    // SAFETY: Validation guarantees there to be a valid block type
    // next.
    let _block_type = unsafe { BlockType::read_unchecked(wasm) };

    let test_val: i32 = stack.pop_value().try_into().unwrap_validated();

    if test_val != 0 {
        *stp += 1;
    } else {
        do_sidetable_control_transfer(wasm, stack, stp, current_sidetable)?;
    }
    trace!("Instruction: IF");

    Ok(None)
});

define_instruction!(
    r#else,
    opcode::ELSE,
    |Args {
         wasm,
         stack,
         stp,
         current_sidetable,
         ..
     }| {
        do_sidetable_control_transfer(wasm, stack, stp, current_sidetable)?;
        Ok(None)
    }
);

define_instruction!(
    br_if,
    opcode::BR_IF,
    |Args {
         stack,
         wasm,
         stp,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid label index
        // next.
        let _label_idx = unsafe { read_label_idx_unchecked(wasm) };

        let test_val: i32 = stack.pop_value().try_into().unwrap_validated();

        if test_val != 0 {
            do_sidetable_control_transfer(wasm, stack, stp, current_sidetable)?;
        } else {
            *stp += 1;
        }
        trace!("Instruction: BR_IF");
        Ok(None)
    }
);

define_instruction!(
    br_table,
    opcode::BR_TABLE,
    |Args {
         stack,
         wasm,
         current_sidetable,
         stp,
         ..
     }| {
        let label_vec = wasm
            .read_vec(|wasm| {
                // SAFETY: Validation guarantees that there is a
                // valid vec of label indices.
                Ok(unsafe { read_label_idx_unchecked(wasm) })
            })
            .unwrap_validated();

        // SAFETY: Validation guarantees there to be another label index
        // for the default case.
        let _default_label_idx = unsafe { read_label_idx_unchecked(wasm) };

        // TODO is this correct?
        let case_val_i32: i32 = stack.pop_value().try_into().unwrap_validated();
        let case_val = case_val_i32.cast_unsigned().into_usize();

        if case_val >= label_vec.len() {
            *stp += label_vec.len();
        } else {
            *stp += case_val;
        }

        do_sidetable_control_transfer(wasm, stack, stp, current_sidetable)?;
        Ok(None)
    }
);

define_instruction!(br, opcode::BR, |Args {
                                         stack,
                                         wasm,
                                         current_sidetable,
                                         stp,
                                         ..
                                     }| {
    // SAFETY: Validation guarantees there to be a valid label index
    // next.
    let _label_idx = unsafe { read_label_idx_unchecked(wasm) };
    do_sidetable_control_transfer(wasm, stack, stp, current_sidetable)?;
    Ok(None)
});

define_instruction!(block, opcode::BLOCK, |Args { wasm, .. }| {
    // SAFETY: Validation guarantess there to be a valid block type
    // next.
    let _ = unsafe { BlockType::read_unchecked(wasm) };
    Ok(None)
});

define_instruction!(r#loop, opcode::LOOP, |Args { wasm, .. }| {
    // SAFETY: Validation guarantees there to be a valid block type
    // next.
    let _ = unsafe { BlockType::read_unchecked(wasm) };
    Ok(None)
});

define_instruction!(
    r#return,
    opcode::RETURN,
    |Args {
         stack,
         wasm,
         current_sidetable,
         stp,
         ..
     }| {
        // same as BR
        do_sidetable_control_transfer(wasm, stack, stp, current_sidetable)?;
        Ok(None)
    }
);

define_instruction!(
    call,
    opcode::CALL,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         stp,
         current_func_addr,
         current_module,
         current_function_end_marker,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid function
        // index next.
        let func_idx = unsafe { FuncIdx::read_unchecked(wasm) };

        // SAFETY: The current function address must come from the given
        // resumable or the current store, because these are the only
        // parameters to this function. The resumable, including its
        // function address, is guaranteed to be valid in the current
        // store by the caller, and the store can only contain addresses
        // that are valid within itself.
        let FuncInst::WasmFunc(current_wasm_func_inst) =
            (unsafe { store_inner.functions.get(*current_func_addr) })
        else {
            unreachable!()
        };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let current_module_inst = unsafe { modules.get(current_wasm_func_inst.module_addr) };

        // SAFETY: Validation guarantees the function index to be
        // valid in the current module.
        let func_to_call_addr = unsafe { current_module_inst.func_addrs.get(func_idx) };

        // SAFETY: This function address just came from the current
        // store. Therefore, it must be valid in the current store.
        let func_to_call_inst = unsafe { store_inner.functions.get(*func_to_call_addr) };

        trace!("Instruction: call [{func_to_call_addr:?}]");

        match func_to_call_inst {
            FuncInst::HostFunc(host_func_to_call_inst) => {
                let params = stack
                    .pop_tail_iter(host_func_to_call_inst.function_type.params.valtypes.len())
                    .collect();

                return Ok(Some(InterpreterLoopOutcome::HostCalled {
                    params,
                    func_addr: *func_to_call_addr,
                    hostcode: host_func_to_call_inst.hostcode,
                }));
            }
            FuncInst::WasmFunc(wasm_func_to_call_inst) => {
                let remaining_locals = &wasm_func_to_call_inst.locals;

                stack.push_call_frame::<T>(
                    *current_func_addr,
                    &wasm_func_to_call_inst.function_type,
                    remaining_locals,
                    wasm.pc,
                    *stp,
                )?;

                *current_func_addr = *func_to_call_addr;
                *current_module = wasm_func_to_call_inst.module_addr;

                // SAFETY: The current module address was just set to an
                // address that came from the current store. Therefore,
                // this address must automatically be valid in the
                // current store.
                let module = unsafe { modules.get(*current_module) };

                wasm.full_wasm_binary = module.wasm_bytecode;
                wasm.move_start_to(wasm_func_to_call_inst.code_expr)
                    .expect("code expression spans to always be valid");

                *stp = wasm_func_to_call_inst.stp;
                *current_sidetable = &module.sidetable;
                *current_function_end_marker = wasm_func_to_call_inst.code_expr.from()
                    + wasm_func_to_call_inst.code_expr.len();
            }
        }
        trace!("Instruction: CALL");

        Ok(None)
    }
);

// TODO: fix push_call_frame, because the func idx that you get from the table is global func idx
define_instruction!(
    call_indirect,
    opcode::CALL_INDIRECT,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         stp,
         current_func_addr,
         current_module,
         current_function_end_marker,
         current_sidetable,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid type index
        // next.
        let given_type_idx = unsafe { TypeIdx::read_unchecked(wasm) };
        // SAFETY: Validation guarantees there to be a valid table index
        // next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be valid in
        // the current module.
        let table_addr = unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the current
        // store. Therefore, it is valid in the current store.
        let tab = unsafe { store_inner.tables.get(*table_addr) };
        // SAFETY: Validation guarantees the type index to be valid in
        // the current module.
        let func_ty = unsafe { module.types.get(given_type_idx) };

        let i: u32 = stack.pop_value().try_into().unwrap_validated();

        let r = tab
            .elem
            .get(i.into_usize())
            .ok_or(TrapError::TableAccessOutOfBounds)
            .and_then(|r| {
                if matches!(r, Ref::Null(_)) {
                    trace!("table_idx ({table_idx}) --- element index in table ({i})");
                    Err(TrapError::UninitializedElement)
                } else {
                    Ok(r)
                }
            })?;

        let func_to_call_addr = match *r {
            Ref::Func(func_addr) => func_addr,
            Ref::Null(_) => return Err(TrapError::IndirectCallNullFuncRef.into()),
            Ref::Extern(_) => unreachable_validated!(),
        };

        // SAFETY: This function address just came from a table of the
        // current store. Therefore, it must be valid in the current
        // store.
        let func_to_call_inst = unsafe { store_inner.functions.get(func_to_call_addr) };

        if func_ty != func_to_call_inst.ty() {
            return Err(TrapError::SignatureMismatch.into());
        }

        trace!("Instruction: call [{func_to_call_addr:?}]");

        match func_to_call_inst {
            FuncInst::HostFunc(host_func_to_call_inst) => {
                let params = stack
                    .pop_tail_iter(host_func_to_call_inst.function_type.params.valtypes.len())
                    .collect();

                return Ok(Some(InterpreterLoopOutcome::HostCalled {
                    params,
                    func_addr: func_to_call_addr,
                    hostcode: host_func_to_call_inst.hostcode,
                }));
            }
            FuncInst::WasmFunc(wasm_func_to_call_inst) => {
                let remaining_locals = &wasm_func_to_call_inst.locals;

                stack.push_call_frame::<T>(
                    *current_func_addr,
                    &wasm_func_to_call_inst.function_type,
                    remaining_locals,
                    wasm.pc,
                    *stp,
                )?;

                *current_func_addr = func_to_call_addr;
                *current_module = wasm_func_to_call_inst.module_addr;

                // SAFETY: The current module address was just set to an
                // address that came from the current store. Therefore,
                // this address must automatically be valid in the
                // current store.
                let module = unsafe { modules.get(*current_module) };
                wasm.full_wasm_binary = module.wasm_bytecode;
                wasm.move_start_to(wasm_func_to_call_inst.code_expr)
                    .expect("code expression spans to always be valid");

                *stp = wasm_func_to_call_inst.stp;
                *current_sidetable = &module.sidetable;
                *current_function_end_marker = wasm_func_to_call_inst.code_expr.from()
                    + wasm_func_to_call_inst.code_expr.len();
            }
        }
        trace!("Instruction: CALL_INDIRECT");
        Ok(None)
    }
);

define_instruction!(drop, opcode::DROP, |Args { stack, .. }| {
    stack.pop_value();
    trace!("Instruction: DROP");
    Ok(None)
});

define_instruction!(select, opcode::SELECT, |Args { stack, .. }| {
    let test_val: i32 = stack.pop_value().try_into().unwrap_validated();
    let val2 = stack.pop_value();
    let val1 = stack.pop_value();
    if test_val != 0 {
        stack.push_value::<T>(val1)?;
    } else {
        stack.push_value::<T>(val2)?;
    }
    trace!("Instruction: SELECT");
    Ok(None)
});

define_instruction!(
    select_t,
    opcode::SELECT_T,
    |Args { stack, wasm, .. }| {
        let _type_vec = wasm.read_vec(ValType::read).unwrap_validated();
        let test_val: i32 = stack.pop_value().try_into().unwrap_validated();
        let val2 = stack.pop_value();
        let val1 = stack.pop_value();
        if test_val != 0 {
            stack.push_value::<T>(val1)?;
        } else {
            stack.push_value::<T>(val2)?;
        }
        trace!("Instruction: SELECT_T");
        Ok(None)
    }
);

define_instruction!(
    local_get,
    opcode::LOCAL_GET,
    |Args { stack, wasm, .. }| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::read_unchecked(wasm) };
        let value = *stack.get_local(local_idx);
        stack.push_value::<T>(value)?;
        trace!("Instruction: local.get {} [] -> [t]", local_idx);
        Ok(None)
    }
);

define_instruction!(
    local_set,
    opcode::LOCAL_SET,
    |Args { stack, wasm, .. }| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::read_unchecked(wasm) };
        let value = stack.pop_value();
        *stack.get_local_mut(local_idx) = value;
        trace!("Instruction: local.set {} [t] -> []", local_idx);
        Ok(None)
    }
);

define_instruction!(
    local_tee,
    opcode::LOCAL_TEE,
    |Args { stack, wasm, .. }| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::read_unchecked(wasm) };
        let value = stack.peek_value().unwrap_validated();
        *stack.get_local_mut(local_idx) = value;
        trace!("Instruction: local.tee {} [t] -> [t]", local_idx);
        Ok(None)
    }
);

define_instruction!(
    global_get,
    opcode::GLOBAL_GET,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::read_unchecked(wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the global index to be valid in
        // the current module.
        let global_addr = *unsafe { module.global_addrs.get(global_idx) };
        // SAFETY: This global address was just read from the current
        // store. Therefore, it is valid in the current store.
        let global = unsafe { store_inner.globals.get(global_addr) };

        stack.push_value::<T>(global.value)?;

        trace!(
            "Instruction: global.get '{}' [<GLOBAL>] -> [{:?}]",
            global_idx,
            global.value
        );
        Ok(None)
    }
);

define_instruction!(
    global_set,
    opcode::GLOBAL_SET,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::read_unchecked(wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };
        // SAFETY: Validation guarantees the global index to be valid in
        // the current module.
        let global_addr = *unsafe { module.global_addrs.get(global_idx) };
        // SAFETY: This global address was just read from the current
        // store. Therefore, it is valid in the current store.
        let global = unsafe { store_inner.globals.get_mut(global_addr) };

        global.value = stack.pop_value();
        trace!("Instruction: GLOBAL_SET");
        Ok(None)
    }
);

define_instruction!(
    table_get,
    opcode::TABLE_GET,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid table index
        // next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be valid in
        // the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the current
        // store. Therefore, it is valid in the current store.
        let tab = unsafe { store_inner.tables.get(table_addr) };

        let i: i32 = stack.pop_value().try_into().unwrap_validated();

        let val = tab
            .elem
            .get(i.cast_unsigned().into_usize())
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

        stack.push_value::<T>((*val).into())?;
        trace!(
            "Instruction: table.get '{}' [{}] -> [{}]",
            table_idx,
            i,
            val
        );
        Ok(None)
    }
);

define_instruction!(
    table_set,
    opcode::TABLE_SET,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be valid table index
        // next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be valid in
        // the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the current
        // store. Therefore, it is valid in the current store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let val: Ref = stack.pop_value().try_into().unwrap_validated();
        let i: i32 = stack.pop_value().try_into().unwrap_validated();

        tab.elem
            .get_mut(i.cast_unsigned().into_usize())
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)
            .map(|r| *r = val)?;
        trace!(
            "Instruction: table.set '{}' [{} {}] -> []",
            table_idx,
            i,
            val
        );
        Ok(None)
    }
);

define_instruction!(unreachable, opcode::UNREACHABLE, |Args { .. }| {
    Err(TrapError::ReachedUnreachable.into())
});

define_instruction!(
    i32_load,
    opcode::I32_LOAD,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem_inst = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data = mem_inst.mem.load(idx)?;

        stack.push_value::<T>(Value::I32(data))?;
        trace!("Instruction: i32.load [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i64_load,
    opcode::I64_LOAD,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I64(data))?;
        trace!("Instruction: i64.load [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    f32_load,
    opcode::F32_LOAD,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::F32(data))?;
        trace!("Instruction: f32.load [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    f64_load,
    opcode::F64_LOAD,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::F64(data))?;
        trace!("Instruction: f64.load [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i32_load8_s,
    opcode::I32_LOAD8_S,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: i8 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I32(data as u32))?;
        trace!("Instruction: i32.load8_s [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i32_load8_u,
    opcode::I32_LOAD8_U,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: u8 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I32(data as u32))?;
        trace!("Instruction: i32.load8_u [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i32_load16_s,
    opcode::I32_LOAD16_S,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: i16 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I32(data as u32))?;
        trace!("Instruction: i32.load16_s [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i32_load16_u,
    opcode::I32_LOAD16_U,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: u16 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I32(data as u32))?;
        trace!("Instruction: i32.load16_u [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i64_load8_s,
    opcode::I64_LOAD8_S,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: i8 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I64(data as u64))?;
        trace!("Instruction: i64.load8_s [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i64_load8_u,
    opcode::I64_LOAD8_U,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: u8 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I64(data as u64))?;
        trace!("Instruction: i64.load8_u [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i64_load16_s,
    opcode::I64_LOAD16_S,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: i16 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I64(data as u64))?;
        trace!("Instruction: i64.load16_s [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i64_load16_u,
    opcode::I64_LOAD16_U,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: u16 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I64(data as u64))?;
        trace!("Instruction: i64.load16_u [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i64_load32_s,
    opcode::I64_LOAD32_S,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: i32 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I64(data as u64))?;
        trace!("Instruction: i64.load32_s [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i64_load32_u,
    opcode::I64_LOAD32_U,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: u32 = mem.mem.load(idx)?;

        stack.push_value::<T>(Value::I64(data as u64))?;
        trace!("Instruction: i64.load32_u [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    i32_store,
    opcode::I32_STORE,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: u32 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: i32.store [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    i64_store,
    opcode::I64_STORE,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: u64 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: i64.store [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    f32_store,
    opcode::F32_STORE,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: F32 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: f32.store [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    f64_store,
    opcode::F64_STORE,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: F64 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: f64.store [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    i32_store8,
    opcode::I32_STORE8,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i32 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as i8;

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i32.store8 [{relative_address} {wrapped_data}] -> []");
        Ok(None)
    }
);

define_instruction!(
    i32_store16,
    opcode::I32_STORE16,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i32 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as i16;

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i32.store16 [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    i64_store8,
    opcode::I64_STORE8,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i64 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as i8;

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i64.store8 [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    i64_store16,
    opcode::I64_STORE16,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i64 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as i16;

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i64.store16 [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    i64_store32,
    opcode::I64_STORE32,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i64 = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as i32;

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i64.store32 [{relative_address} {data_to_store}] -> []");
        Ok(None)
    }
);

define_instruction!(
    memory_size,
    opcode::MEMORY_SIZE,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         ..
     }| {
        // Note: This zero byte is reserved for the multiple memories
        // proposal.
        let _zero = wasm.read_u8().unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };
        let size = mem.size() as u32;
        stack.push_value::<T>(Value::I32(size))?;
        trace!("Instruction: memory.size [] -> [{}]", size);
        Ok(None)
    }
);

define_instruction!(
    no_fuel_check,
    memory_grow,
    opcode::MEMORY_GROW,
    |Args {
         store_inner,
         modules,
         stack,
         wasm,
         current_module,
         maybe_fuel,
         ..
     }| {
        // Note: This zero byte is reserved for the multiple memories
        // proposal.
        let _zero = wasm.read_u8().unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the current
        // store. Therefore, it is valid in the current store.
        let mem = unsafe { store_inner.memories.get_mut(mem_addr) };

        let sz: u32 = mem.size() as u32;

        let n: u32 = stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_flat_cost(opcode::MEMORY_GROW)
            + u64::from(n) * T::get_cost_per_element(opcode::MEMORY_GROW);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.

                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
        // for now we execute it always according to the following match expr.
        // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the stack per spec
        let pushed_value = match mem.grow(n) {
            Ok(_) => sz,
            Err(_) => u32::MAX,
        };
        stack.push_value::<T>(Value::I32(pushed_value))?;
        trace!("Instruction: memory.grow [{}] -> [{}]", n, pushed_value);
        Ok(None)
    }
);

define_instruction!(
    i32_const,
    opcode::I32_CONST,
    |Args { stack, wasm, .. }| {
        let constant = wasm.read_var_i32().unwrap_validated();
        trace!("Instruction: i32.const [] -> [{constant}]");
        stack.push_value::<T>(constant.into())?;
        Ok(None)
    }
);

define_instruction!(
    f32_const,
    opcode::F32_CONST,
    |Args { stack, wasm, .. }| {
        let constant = F32::from_bits(wasm.read_f32().unwrap_validated());
        trace!("Instruction: f32.const [] -> [{constant:.7}]");
        stack.push_value::<T>(constant.into())?;
        Ok(None)
    }
);

define_instruction!(i32_eqz, opcode::I32_EQZ, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 == 0 { 1 } else { 0 };

    trace!("Instruction: i32.eqz [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_eq, opcode::I32_EQ, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: i32.eq [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_ne, opcode::I32_NE, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: i32.ne [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_lt_s, opcode::I32_LT_S, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: i32.lt_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_lt_u, opcode::I32_LT_U, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u32) < (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.lt_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_gt_s, opcode::I32_GT_S, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: i32.gt_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_gt_u, opcode::I32_GT_U, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u32) > (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.gt_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_le_s, opcode::I32_LE_S, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: i32.le_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_le_u, opcode::I32_LE_U, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u32) <= (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.le_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_ge_s, opcode::I32_GE_S, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: i32.ge_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_ge_u, opcode::I32_GE_U, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u32) >= (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.ge_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_eqz, opcode::I64_EQZ, |Args { stack, .. }| {
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 == 0 { 1 } else { 0 };

    trace!("Instruction: i64.eqz [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_eq, opcode::I64_EQ, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: i64.eq [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_ne, opcode::I64_NE, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: i64.ne [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_lt_s, opcode::I64_LT_S, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: i64.lt_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_lt_u, opcode::I64_LT_U, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u64) < (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.lt_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_gt_s, opcode::I64_GT_S, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: i64.gt_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_gt_u, opcode::I64_GT_U, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u64) > (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.gt_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_le_s, opcode::I64_LE_S, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: i64.le_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_le_u, opcode::I64_LE_U, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u64) <= (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.le_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_ge_s, opcode::I64_GE_S, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: i64.ge_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_ge_u, opcode::I64_GE_U, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = if (v1 as u64) >= (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.ge_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_eq, opcode::F32_EQ, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: f32.eq [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_ne, opcode::F32_NE, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: f32.ne [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_lt, opcode::F32_LT, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: f32.lt [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_gt, opcode::F32_GT, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: f32.gt [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_le, opcode::F32_LE, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: f32.le [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_ge, opcode::F32_GE, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: f32.ge [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_eq, opcode::F64_EQ, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: f64.eq [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_ne, opcode::F64_NE, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: f64.ne [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_lt, opcode::F64_LT, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: f64.lt [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_gt, opcode::F64_GT, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: f64.gt [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_le, opcode::F64_LE, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: f64.le [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_ge, opcode::F64_GE, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: f64.ge [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_clz, opcode::I32_CLZ, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.leading_zeros() as i32;

    trace!("Instruction: i32.clz [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_ctz, opcode::I32_CTZ, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.trailing_zeros() as i32;

    trace!("Instruction: i32.ctz [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_popcnt, opcode::I32_POPCNT, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.count_ones() as i32;

    trace!("Instruction: i32.popcnt [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(
    i64_const,
    opcode::I64_CONST,
    |Args { wasm, stack, .. }| {
        let constant = wasm.read_var_i64().unwrap_validated();
        trace!("Instruction: i64.const [] -> [{constant}]");
        stack.push_value::<T>(constant.into())?;
        Ok(None)
    }
);

define_instruction!(
    f64_const,
    opcode::F64_CONST,
    |Args { wasm, stack, .. }| {
        let constant = F64::from_bits(wasm.read_f64().unwrap_validated());
        trace!("Instruction: f64.const [] -> [{constant}]");
        stack.push_value::<T>(constant.into())?;
        Ok(None)
    }
);

define_instruction!(i32_add, opcode::I32_ADD, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.wrapping_add(v2);

    trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_sub, opcode::I32_SUB, |Args { stack, .. }| {
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.wrapping_sub(v2);

    trace!("Instruction: i32.sub [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_mul, opcode::I32_MUL, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.wrapping_mul(v2);

    trace!("Instruction: i32.mul [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_div_s, opcode::I32_DIV_S, |Args { stack, .. }| {
    let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }
    if divisor == i32::MIN && dividend == -1 {
        return Err(TrapError::UnrepresentableResult.into());
    }

    let res = divisor / dividend;

    trace!("Instruction: i32.div_s [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_div_u, opcode::I32_DIV_U, |Args { stack, .. }| {
    let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

    let dividend = dividend as u32;
    let divisor = divisor as u32;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = (divisor / dividend) as i32;

    trace!("Instruction: i32.div_u [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_rem_s, opcode::I32_REM_S, |Args { stack, .. }| {
    let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = divisor.checked_rem(dividend);
    let res = res.unwrap_or_default();

    trace!("Instruction: i32.rem_s [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_clz, opcode::I64_CLZ, |Args { stack, .. }| {
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.leading_zeros() as i64;

    trace!("Instruction: i64.clz [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_ctz, opcode::I64_CTZ, |Args { stack, .. }| {
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.trailing_zeros() as i64;

    trace!("Instruction: i64.ctz [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_popcnt, opcode::I64_POPCNT, |Args { stack, .. }| {
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.count_ones() as i64;

    trace!("Instruction: i64.popcnt [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_add, opcode::I64_ADD, |Args { stack, .. }| {
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.wrapping_add(v2);

    trace!("Instruction: i64.add [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_sub, opcode::I64_SUB, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.wrapping_sub(v2);

    trace!("Instruction: i64.sub [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_mul, opcode::I64_MUL, |Args { stack, .. }| {
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let res = v1.wrapping_mul(v2);

    trace!("Instruction: i64.mul [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_div_s, opcode::I64_DIV_S, |Args { stack, .. }| {
    let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }
    if divisor == i64::MIN && dividend == -1 {
        return Err(TrapError::UnrepresentableResult.into());
    }

    let res = divisor / dividend;

    trace!("Instruction: i64.div_s [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_div_u, opcode::I64_DIV_U, |Args { stack, .. }| {
    let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

    let dividend = dividend as u64;
    let divisor = divisor as u64;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = (divisor / dividend) as i64;

    trace!("Instruction: i64.div_u [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_rem_s, opcode::I64_REM_S, |Args { stack, .. }| {
    let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = divisor.checked_rem(dividend);
    let res = res.unwrap_or_default();

    trace!("Instruction: i64.rem_s [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_rem_u, opcode::I64_REM_U, |Args { stack, .. }| {
    let dividend: i64 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i64 = stack.pop_value().try_into().unwrap_validated();

    let dividend = dividend as u64;
    let divisor = divisor as u64;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = (divisor % dividend) as i64;

    trace!("Instruction: i64.rem_u [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_and, opcode::I64_AND, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = v1 & v2;

    trace!("Instruction: i64.and [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_or, opcode::I64_OR, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = v1 | v2;

    trace!("Instruction: i64.or [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_xor, opcode::I64_XOR, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = v1 ^ v2;

    trace!("Instruction: i64.xor [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_shl, opcode::I64_SHL, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = v1.wrapping_shl((v2 & 63) as u32);

    trace!("Instruction: i64.shl [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_shr_s, opcode::I64_SHR_S, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = v1.wrapping_shr((v2 & 63) as u32);

    trace!("Instruction: i64.shr_s [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_shr_u, opcode::I64_SHR_U, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = (v1 as u64).wrapping_shr((v2 & 63) as u32);

    trace!("Instruction: i64.shr_u [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_rotl, opcode::I64_ROTL, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = v1.rotate_left((v2 & 63) as u32);

    trace!("Instruction: i64.rotl [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i64_rotr, opcode::I64_ROTR, |Args { stack, .. }| {
    let v2: i64 = stack.pop_value().try_into().unwrap_validated();
    let v1: i64 = stack.pop_value().try_into().unwrap_validated();

    let res = v1.rotate_right((v2 & 63) as u32);

    trace!("Instruction: i64.rotr [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_rem_u, opcode::I32_REM_U, |Args { stack, .. }| {
    let dividend: i32 = stack.pop_value().try_into().unwrap_validated();
    let divisor: i32 = stack.pop_value().try_into().unwrap_validated();

    let dividend = dividend as u32;
    let divisor = divisor as u32;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = divisor.checked_rem(dividend);
    let res = res.unwrap_or_default() as i32;

    trace!("Instruction: i32.rem_u [{divisor} {dividend}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_and, opcode::I32_AND, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1 & v2;

    trace!("Instruction: i32.and [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_or, opcode::I32_OR, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1 | v2;

    trace!("Instruction: i32.or [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_xor, opcode::I32_XOR, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v1 ^ v2;

    trace!("Instruction: i32.xor [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_shl, opcode::I32_SHL, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();
    let res = v2.wrapping_shl(v1 as u32);

    trace!("Instruction: i32.shl [{v2} {v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_shr_s, opcode::I32_SHR_S, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = v2.wrapping_shr(v1 as u32);

    trace!("Instruction: i32.shr_s [{v2} {v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_shr_u, opcode::I32_SHR_U, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = (v2 as u32).wrapping_shr(v1 as u32) as i32;

    trace!("Instruction: i32.shr_u [{v2} {v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_rotl, opcode::I32_ROTL, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = v2.rotate_left(v1 as u32);

    trace!("Instruction: i32.rotl [{v2} {v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(i32_rotr, opcode::I32_ROTR, |Args { stack, .. }| {
    let v1: i32 = stack.pop_value().try_into().unwrap_validated();
    let v2: i32 = stack.pop_value().try_into().unwrap_validated();

    let res = v2.rotate_right(v1 as u32);

    trace!("Instruction: i32.rotr [{v2} {v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_abs, opcode::F32_ABS, |Args { stack, .. }| {
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.abs();

    trace!("Instruction: f32.abs [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_neg, opcode::F32_NEG, |Args { stack, .. }| {
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.neg();

    trace!("Instruction: f32.neg [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_ceil, opcode::F32_CEIL, |Args { stack, .. }| {
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.ceil();

    trace!("Instruction: f32.ceil [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_floor, opcode::F32_FLOOR, |Args { stack, .. }| {
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.floor();

    trace!("Instruction: f32.floor [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_trunc, opcode::F32_TRUNC, |Args { stack, .. }| {
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.trunc();

    trace!("Instruction: f32.trunc [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(
    f32_nearest,
    opcode::F32_NEAREST,
    |Args { stack, .. }| {
        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = v1.nearest();

        trace!("Instruction: f32.nearest [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(f32_sqrt, opcode::F32_SQRT, |Args { stack, .. }| {
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.sqrt();

    trace!("Instruction: f32.sqrt [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_add, opcode::F32_ADD, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1 + v2;

    trace!("Instruction: f32.add [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_sub, opcode::F32_SUB, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1 - v2;

    trace!("Instruction: f32.sub [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_mul, opcode::F32_MUL, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1 * v2;

    trace!("Instruction: f32.mul [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_div, opcode::F32_DIV, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1 / v2;

    trace!("Instruction: f32.div [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_min, opcode::F32_MIN, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.min(v2);

    trace!("Instruction: f32.min [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f32_max, opcode::F32_MAX, |Args { stack, .. }| {
    let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F32 = v1.max(v2);

    trace!("Instruction: f32.max [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(
    f32_copysign,
    opcode::F32_COPYSIGN,
    |Args { stack, .. }| {
        let v2: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = v1.copysign(v2);

        trace!("Instruction: f32.copysign [{v1} {v2}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(f64_abs, opcode::F64_ABS, |Args { stack, .. }| {
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.abs();

    trace!("Instruction: f64.abs [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_neg, opcode::F64_NEG, |Args { stack, .. }| {
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.neg();

    trace!("Instruction: f64.neg [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_ceil, opcode::F64_CEIL, |Args { stack, .. }| {
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.ceil();

    trace!("Instruction: f64.ceil [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_floor, opcode::F64_FLOOR, |Args { stack, .. }| {
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.floor();

    trace!("Instruction: f64.floor [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_trunc, opcode::F64_TRUNC, |Args { stack, .. }| {
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.trunc();

    trace!("Instruction: f64.trunc [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(
    f64_nearest,
    opcode::F64_NEAREST,
    |Args { stack, .. }| {
        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = v1.nearest();

        trace!("Instruction: f64.nearest [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(f64_sqrt, opcode::F64_SQRT, |Args { stack, .. }| {
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.sqrt();

    trace!("Instruction: f64.sqrt [{v1}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_add, opcode::F64_ADD, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1 + v2;

    trace!("Instruction: f64.add [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_sub, opcode::F64_SUB, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1 - v2;

    trace!("Instruction: f64.sub [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_mul, opcode::F64_MUL, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1 * v2;

    trace!("Instruction: f64.mul [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_div, opcode::F64_DIV, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1 / v2;

    trace!("Instruction: f64.div [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_min, opcode::F64_MIN, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.min(v2);

    trace!("Instruction: f64.min [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(f64_max, opcode::F64_MAX, |Args { stack, .. }| {
    let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
    let res: value::F64 = v1.max(v2);

    trace!("Instruction: f64.max [{v1} {v2}] -> [{res}]");
    stack.push_value::<T>(res.into())?;
    Ok(None)
});

define_instruction!(
    f64_copysign,
    opcode::F64_COPYSIGN,
    |Args { stack, .. }| {
        let v2: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = v1.copysign(v2);

        trace!("Instruction: f64.copysign [{v1} {v2}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i32_wrap_i64,
    opcode::I32_WRAP_I64,
    |Args { stack, .. }| {
        let v: i64 = stack.pop_value().try_into().unwrap_validated();
        let res: i32 = v as i32;

        trace!("Instruction: i32.wrap_i64 [{v}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i32_trunc_f32_s,
    opcode::I32_TRUNC_F32_S,
    |Args { stack, .. }| {
        let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F32(2147483648.0) || v <= value::F32(-2147483904.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_i32();

        trace!("Instruction: i32.trunc_f32_s [{v:.7}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i32_trunc_f32_u,
    opcode::I32_TRUNC_F32_U,
    |Args { stack, .. }| {
        let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F32(4294967296.0) || v <= value::F32(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_u32() as i32;

        trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i32_trunc_f64_s,
    opcode::I32_TRUNC_F64_S,
    |Args { stack, .. }| {
        let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F64(2147483648.0) || v <= value::F64(-2147483649.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_i32();

        trace!("Instruction: i32.trunc_f64_s [{v:.7}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i32_trunc_f64_u,
    opcode::I32_TRUNC_F64_U,
    |Args { stack, .. }| {
        let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F64(4294967296.0) || v <= value::F64(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_u32() as i32;

        trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i64_extend_i32_s,
    opcode::I64_EXTEND_I32_S,
    |Args { stack, .. }| {
        let v: i32 = stack.pop_value().try_into().unwrap_validated();

        let res: i64 = v as i64;

        trace!("Instruction: i64.extend_i32_s [{v}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i64_extend_i32_u,
    opcode::I64_EXTEND_I32_U,
    |Args { stack, .. }| {
        let v: i32 = stack.pop_value().try_into().unwrap_validated();

        let res: i64 = v as u32 as i64;

        trace!("Instruction: i64.extend_i32_u [{v}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i64_trunc_f32_s,
    opcode::I64_TRUNC_F32_S,
    |Args { stack, .. }| {
        let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F32(9223372036854775808.0) || v <= value::F32(-9223373136366403584.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_i64();

        trace!("Instruction: i64.trunc_f32_s [{v:.7}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i64_trunc_f32_u,
    opcode::I64_TRUNC_F32_U,
    |Args { stack, .. }| {
        let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F32(18446744073709551616.0) || v <= value::F32(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_u64() as i64;

        trace!("Instruction: i64.trunc_f32_u [{v:.7}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i64_trunc_f64_s,
    opcode::I64_TRUNC_F64_S,
    |Args { stack, .. }| {
        let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F64(9223372036854775808.0) || v <= value::F64(-9223372036854777856.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_i64();

        trace!("Instruction: i64.trunc_f64_s [{v:.17}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i64_trunc_f64_u,
    opcode::I64_TRUNC_F64_U,
    |Args { stack, .. }| {
        let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= value::F64(18446744073709551616.0) || v <= value::F64(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_u64() as i64;

        trace!("Instruction: i64.trunc_f64_u [{v:.17}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f32_convert_i32_s,
    opcode::F32_CONVERT_I32_S,
    |Args { stack, .. }| {
        let v: i32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = value::F32(v as f32);

        trace!("Instruction: f32.convert_i32_s [{v}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f32_convert_i32_u,
    opcode::F32_CONVERT_I32_U,
    |Args { stack, .. }| {
        let v: i32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = value::F32(v as u32 as f32);

        trace!("Instruction: f32.convert_i32_u [{v}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f32_convert_i64_s,
    opcode::F32_CONVERT_I64_S,
    |Args { stack, .. }| {
        let v: i64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = value::F32(v as f32);

        trace!("Instruction: f32.convert_i64_s [{v}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f32_convert_i64_u,
    opcode::F32_CONVERT_I64_U,
    |Args { stack, .. }| {
        let v: i64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = value::F32(v as u64 as f32);

        trace!("Instruction: f32.convert_i64_u [{v}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f32_demote_f64,
    opcode::F32_DEMOTE_F64,
    |Args { stack, .. }| {
        let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = v.as_f32();

        trace!("Instruction: f32.demote_f64 [{v:.17}] -> [{res:.7}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f64_convert_i32_s,
    opcode::F64_CONVERT_I32_S,
    |Args { stack, .. }| {
        let v: i32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = value::F64(v as f64);

        trace!("Instruction: f64.convert_i32_s [{v}] -> [{res:.17}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f64_convert_i32_u,
    opcode::F64_CONVERT_I32_U,
    |Args { stack, .. }| {
        let v: i32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = value::F64(v as u32 as f64);

        trace!("Instruction: f64.convert_i32_u [{v}] -> [{res:.17}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f64_convert_i64_s,
    opcode::F64_CONVERT_I64_S,
    |Args { stack, .. }| {
        let v: i64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = value::F64(v as f64);

        trace!("Instruction: f64.convert_i64_s [{v}] -> [{res:.17}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f64_convert_i64_u,
    opcode::F64_CONVERT_I64_U,
    |Args { stack, .. }| {
        let v: i64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = value::F64(v as u64 as f64);

        trace!("Instruction: f64.convert_i64_u [{v}] -> [{res:.17}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f64_promote_f32,
    opcode::F64_PROMOTE_F32,
    |Args { stack, .. }| {
        let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = v.as_f64();

        trace!("Instruction: f64.promote_f32 [{v:.7}] -> [{res:.17}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i32_reinterpret_f32,
    opcode::I32_REINTERPRET_F32,
    |Args { stack, .. }| {
        let v: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res: i32 = v.reinterpret_as_i32();

        trace!("Instruction: i32.reinterpret_f32 [{v:.7}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    i64_reinterpret_f64,
    opcode::I64_REINTERPRET_F64,
    |Args { stack, .. }| {
        let v: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res: i64 = v.reinterpret_as_i64();

        trace!("Instruction: i64.reinterpret_f64 [{v:.17}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f32_reinterpret_i32,
    opcode::F32_REINTERPRET_I32,
    |Args { stack, .. }| {
        let v1: i32 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F32 = value::F32::from_bits(v1 as u32);

        trace!("Instruction: f32.reinterpret_i32 [{v1}] -> [{res:.7}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    f64_reinterpret_i64,
    opcode::F64_REINTERPRET_I64,
    |Args { stack, .. }| {
        let v1: i64 = stack.pop_value().try_into().unwrap_validated();
        let res: value::F64 = value::F64::from_bits(v1 as u64);

        trace!("Instruction: f64.reinterpret_i64 [{v1}] -> [{res:.17}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    ref_null,
    opcode::REF_NULL,
    |Args { wasm, stack, .. }| {
        let reftype = RefType::read(wasm).unwrap_validated();

        stack.push_value::<T>(Value::Ref(Ref::Null(reftype)))?;
        trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
        Ok(None)
    }
);

define_instruction!(
    ref_is_null,
    opcode::REF_IS_NULL,
    |Args { stack, .. }| {
        let rref: Ref = stack.pop_value().try_into().unwrap_validated();
        let is_null = matches!(rref, Ref::Null(_));

        let res = if is_null { 1 } else { 0 };
        trace!("Instruction: ref.is_null [{}] -> [{}]", rref, res);
        stack.push_value::<T>(Value::I32(res))?;
        Ok(None)
    }
);

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
define_instruction!(
    ref_func,
    opcode::REF_FUNC,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees a valid function index to be
        // next.
        let func_idx = unsafe { FuncIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let current_module = unsafe { modules.get(*current_module) };
        // SAFETY: Validation guarantees the function index to be valid
        // in the current module.
        let func_addr = unsafe { current_module.func_addrs.get(func_idx) };
        stack.push_value::<T>(Value::Ref(Ref::Func(*func_addr)))?;
        Ok(None)
    }
);

define_instruction!(
    i32_extend8_s,
    opcode::I32_EXTEND8_S,
    |Args { stack, .. }| {
        let mut v: u32 = stack.pop_value().try_into().unwrap_validated();

        if v | 0xFF != 0xFF {
            trace!("Number v ({}) not contained in 8 bits, truncating", v);
            v &= 0xFF;
        }

        let res = if v | 0x7F != 0x7F { v | 0xFFFFFF00 } else { v };

        stack.push_value::<T>(res.into())?;

        trace!("Instruction i32.extend8_s [{}] -> [{}]", v, res);
        Ok(None)
    }
);

define_instruction!(
    i32_extend16_s,
    opcode::I32_EXTEND16_S,
    |Args { stack, .. }| {
        let mut v: u32 = stack.pop_value().try_into().unwrap_validated();

        if v | 0xFFFF != 0xFFFF {
            trace!("Number v ({}) not contained in 16 bits, truncating", v);
            v &= 0xFFFF;
        }

        let res = if v | 0x7FFF != 0x7FFF {
            v | 0xFFFF0000
        } else {
            v
        };

        stack.push_value::<T>(res.into())?;

        trace!("Instruction i32.extend16_s [{}] -> [{}]", v, res);
        Ok(None)
    }
);

define_instruction!(
    i64_extend8_s,
    opcode::I64_EXTEND8_S,
    |Args { stack, .. }| {
        let mut v: u64 = stack.pop_value().try_into().unwrap_validated();

        if v | 0xFF != 0xFF {
            trace!("Number v ({}) not contained in 8 bits, truncating", v);
            v &= 0xFF;
        }

        let res = if v | 0x7F != 0x7F {
            v | 0xFFFFFFFF_FFFFFF00
        } else {
            v
        };

        stack.push_value::<T>(res.into())?;

        trace!("Instruction i64.extend8_s [{}] -> [{}]", v, res);
        Ok(None)
    }
);

define_instruction!(
    i64_extend16_s,
    opcode::I64_EXTEND16_S,
    |Args { stack, .. }| {
        let mut v: u64 = stack.pop_value().try_into().unwrap_validated();

        if v | 0xFFFF != 0xFFFF {
            trace!("Number v ({}) not contained in 16 bits, truncating", v);
            v &= 0xFFFF;
        }

        let res = if v | 0x7FFF != 0x7FFF {
            v | 0xFFFFFFFF_FFFF0000
        } else {
            v
        };

        stack.push_value::<T>(res.into())?;

        trace!("Instruction i64.extend16_s [{}] -> [{}]", v, res);
        Ok(None)
    }
);

define_instruction!(
    i64_extend32_s,
    opcode::I64_EXTEND32_S,
    |Args { stack, .. }| {
        let mut v: u64 = stack.pop_value().try_into().unwrap_validated();

        if v | 0xFFFF_FFFF != 0xFFFF_FFFF {
            trace!("Number v ({}) not contained in 32 bits, truncating", v);
            v &= 0xFFFF_FFFF;
        }

        let res = if v | 0x7FFF_FFFF != 0x7FFF_FFFF {
            v | 0xFFFFFFFF_00000000
        } else {
            v
        };

        stack.push_value::<T>(res.into())?;

        trace!("Instruction i64.extend32_s [{}] -> [{}]", v, res);
        Ok(None)
    }
);

define_instruction!(
    no_fuel_check,
    fc_extensions,
    opcode::FC_EXTENSIONS,
    |args: Args| {
        // should we call instruction hook here as well? multibyte instruction
        let second_instr = args.wasm.read_var_u32().unwrap_validated();

        use crate::core::reader::types::opcode::fc_extensions::*;
        let instruction_fn = match second_instr {
            I32_TRUNC_SAT_F32_S => i32_trunc_sat_f32_s::<T>,
            I32_TRUNC_SAT_F32_U => i32_trunc_sat_f32_u::<T>,
            I32_TRUNC_SAT_F64_S => i32_trunc_sat_f64_s::<T>,
            I32_TRUNC_SAT_F64_U => i32_trunc_sat_f64_u::<T>,
            I64_TRUNC_SAT_F32_S => i64_trunc_sat_f32_s::<T>,
            I64_TRUNC_SAT_F32_U => i64_trunc_sat_f32_u::<T>,
            I64_TRUNC_SAT_F64_S => i64_trunc_sat_f64_s::<T>,
            I64_TRUNC_SAT_F64_U => i64_trunc_sat_f64_u::<T>,
            MEMORY_INIT => memory_init_fn::<T>,
            DATA_DROP => data_drop_fn::<T>,
            MEMORY_COPY => memory_copy::<T>,
            MEMORY_FILL => memory_fill::<T>,
            TABLE_INIT => table_init_fn::<T>,
            ELEM_DROP => elem_drop_fn::<T>,
            TABLE_COPY => table_copy::<T>,
            TABLE_GROW => table_grow::<T>,
            TABLE_SIZE => table_size::<T>,
            TABLE_FILL => table_fill::<T>,
            _ => unreachable!(),
        };

        // SAFETY: The caller of the current fc_extensions instruction handler ensures the same
        // safety requirements that are also required by the handler we are calling now.
        unsafe { instruction_fn(args) }
    }
);

define_instruction!(
    fc_fuel_check,
    i32_trunc_sat_f32_s,
    opcode::fc_extensions::I32_TRUNC_SAT_F32_S,
    |Args { stack, .. }| {
        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i32::MIN
            } else if v1.is_infinity() {
                i32::MAX
            } else {
                v1.as_i32()
            }
        };

        trace!("Instruction: i32.trunc_sat_f32_s [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    i32_trunc_sat_f32_u,
    opcode::fc_extensions::I32_TRUNC_SAT_F32_U,
    |Args { stack, .. }| {
        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u32::MAX as i32
            } else {
                v1.as_u32() as i32
            }
        };

        trace!("Instruction: i32.trunc_sat_f32_u [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    i32_trunc_sat_f64_s,
    opcode::fc_extensions::I32_TRUNC_SAT_F64_S,
    |Args { stack, .. }| {
        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i32::MIN
            } else if v1.is_infinity() {
                i32::MAX
            } else {
                v1.as_i32()
            }
        };

        trace!("Instruction: i32.trunc_sat_f64_s [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    i32_trunc_sat_f64_u,
    opcode::fc_extensions::I32_TRUNC_SAT_F64_U,
    |Args { stack, .. }| {
        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u32::MAX as i32
            } else {
                v1.as_u32() as i32
            }
        };

        trace!("Instruction: i32.trunc_sat_f64_u [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    i64_trunc_sat_f32_s,
    opcode::fc_extensions::I64_TRUNC_SAT_F32_S,
    |Args { stack, .. }| {
        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i64::MIN
            } else if v1.is_infinity() {
                i64::MAX
            } else {
                v1.as_i64()
            }
        };

        trace!("Instruction: i64.trunc_sat_f32_s [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    i64_trunc_sat_f32_u,
    opcode::fc_extensions::I64_TRUNC_SAT_F32_U,
    |Args { stack, .. }| {
        let v1: value::F32 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u64::MAX as i64
            } else {
                v1.as_u64() as i64
            }
        };

        trace!("Instruction: i64.trunc_sat_f32_u [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    i64_trunc_sat_f64_s,
    opcode::fc_extensions::I64_TRUNC_SAT_F64_S,
    |Args { stack, .. }| {
        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i64::MIN
            } else if v1.is_infinity() {
                i64::MAX
            } else {
                v1.as_i64()
            }
        };

        trace!("Instruction: i64.trunc_sat_f64_s [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    i64_trunc_sat_f64_u,
    opcode::fc_extensions::I64_TRUNC_SAT_F64_U,
    |Args { stack, .. }| {
        let v1: value::F64 = stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u64::MAX as i64
            } else {
                v1.as_u64() as i64
            }
        };

        trace!("Instruction: i64.trunc_sat_f64_u [{v1}] -> [{res}]");
        stack.push_value::<T>(res.into())?;
        Ok(None)
    }
);

// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x
// Copy a region from a data segment into memory
define_instruction!(
    no_fuel_check,
    memory_init_fn,
    opcode::fc_extensions::MEMORY_INIT,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         maybe_fuel,
         ..
     }| {
        //  mappings:
        //      n => number of bytes to copy
        //      s =            }> starting pointer in the data segment
        //      d => destination address to copy to
        // SAFETY: Validation guarantees there to be a valid
        // data index next.
        let data_idx = unsafe { DataIdx::read_unchecked(wasm) };

        // Note: This zero byte is reserved for the multiple memories
        // proposal.
        let _zero = wasm.read_u8().unwrap_validated();

        let n: u32 = stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::MEMORY_INIT)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::MEMORY_INIT);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: u32 = stack.pop_value().try_into().unwrap_validated();
        let d: u32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: All requirements are met:
        // 1. The current module address must come from the
        //    current store, because it is the only parameter to
        //    this function that can contain module addresses. All
        //    stores guarantee all addresses in them to be valid
        //    within themselves.
        // 2. Validation guarantees at least one memory to exist.
        // 3./5. The memory and data addresses are valid for a
        //       similar reason that the module address is valid:
        //       they are stored in the current module instance,
        //       which is also part of the current store.
        // 4. Validation gurantees this data index to be valid
        //    for the current module instance.
        unsafe {
            memory_init(
                modules,
                &mut store_inner.memories,
                &store_inner.data,
                *current_module,
                data_idx,
                MemIdx::new(0),
                n,
                s,
                d,
            )?
        };
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    data_drop_fn,
    opcode::fc_extensions::DATA_DROP,
    |Args {
         wasm,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // data index next.
        let data_idx = unsafe { DataIdx::read_unchecked(wasm) };
        // SAFETY: All requirements are met:
        // 1. The current module address must come from the
        //    current store, because it is the only parameter to
        //    this function that can contain module addresses. All
        //    stores guarantee all addresses in them to be valid
        //    within themselves.
        // 2. Validation guarantees the data index to be valid
        //    for the current module instance.
        // 3. The data address is valid for a similar reason that
        //    the module address is valid: it is stored in the
        //    current module instance, which is also part of the
        //    current store.
        unsafe { data_drop(modules, &mut store_inner.data, *current_module, data_idx) };
        Ok(None)
    }
);

// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy
define_instruction!(
    no_fuel_check,
    memory_copy,
    opcode::fc_extensions::MEMORY_COPY,
    |Args {
         stack,
         wasm,
         store_inner,
         modules,
         current_module,
         maybe_fuel,
         ..
     }| {
        //  mappings:
        //      n => number of bytes to copy
        //      s => source address to copy from
        //      d => destination address to copy to
        // Note: These zero bytes are reserved for the multiple
        // memories proposal.
        let _zero = wasm.read_u8().unwrap_validated();
        let _zero = wasm.read_u8().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let src_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let dst_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };

        let n: u32 = stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::MEMORY_COPY)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::MEMORY_COPY);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: i32 = stack.pop_value().try_into().unwrap_validated();
        let d: i32 = stack.pop_value().try_into().unwrap_validated();

        // SAFETY: This source memory address was just read from
        // the current store. Therefore, it must also be valid
        // in the current store.
        let src_mem = unsafe { store_inner.memories.get(src_addr) };
        // SAFETY: This destination memory address was just read
        // from the current store. Therefore, it must also be
        // valid in the current store.
        let dest_mem = unsafe { store_inner.memories.get(dst_addr) };

        dest_mem.mem.copy(
            d.cast_unsigned().into_usize(),
            &src_mem.mem,
            s.cast_unsigned().into_usize(),
            n.into_usize(),
        )?;
        trace!("Instruction: memory.copy");
        Ok(None)
    }
);

// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill
define_instruction!(
    no_fuel_check,
    memory_fill,
    opcode::fc_extensions::MEMORY_FILL,
    |Args {
         stack,
         wasm,
         store_inner,
         modules,
         current_module,
         maybe_fuel,
         ..
     }| {
        //  mappings:
        //      n => number of bytes to update
        //      val => the value to set each byte to (must be < 256)
        //      d => the pointer to the region to update

        // Note: This zero byte is reserved for the multiple
        // memories proposal.
        let _zero = wasm.read_u8().unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let mem = unsafe { store_inner.memories.get(mem_addr) };

        let n: u32 = stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::MEMORY_FILL)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::MEMORY_FILL);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let val: i32 = stack.pop_value().try_into().unwrap_validated();

        if !(0..=255).contains(&val) {
            warn!("Value for memory.fill does not fit in a byte ({val})");
        }

        let d: i32 = stack.pop_value().try_into().unwrap_validated();

        mem.mem
            .fill(d.cast_unsigned().into_usize(), val as u8, n.into_usize())?;

        trace!("Instruction: memory.fill");
        Ok(None)
    }
);

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-init-x-y
// https://webassembly.github.io/spec/core/binary/instructions.html#table-instructions
// in binary format it seems that elemidx is first ???????
// this is ONLY for passive elements
define_instruction!(
    no_fuel_check,
    table_init_fn,
    opcode::fc_extensions::TABLE_INIT,
    |Args {
         stack,
         wasm,
         store_inner,
         modules,
         current_module,
         maybe_fuel,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // element index next.
        let elem_idx = unsafe { ElemIdx::read_unchecked(wasm) };
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        let n: u32 = stack.pop_value().try_into().unwrap_validated(); // size
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_INIT)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_INIT);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: i32 = stack.pop_value().try_into().unwrap_validated(); // offset
        let d: i32 = stack.pop_value().try_into().unwrap_validated(); // dst

        // SAFETY: All requirements are met:
        // 1. The current module address must come from the
        //    current store, because it is the only parameter to
        //    this function that can contain module addresses. All
        //    stores guarantee all addresses in them to be valid
        //    within themselves.
        // 2. Validation guarantees the table index to be valid
        //    in the current module instance.
        // 3./5. The table/element addresses are valid for a
        //       similar reason that the module address is valid:
        //       they are stored in the current module instance,
        //       which is also part of the current store.
        // 4. Validation guarantees the element index to be
        //    valid in the current module instance.
        unsafe {
            table_init(
                modules,
                &mut store_inner.tables,
                &store_inner.elements,
                *current_module,
                elem_idx,
                table_idx,
                n,
                s,
                d,
            )?
        };
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    elem_drop_fn,
    opcode::fc_extensions::ELEM_DROP,
    |Args {
         wasm,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there a valid element
        // index next.
        let elem_idx = unsafe { ElemIdx::read_unchecked(wasm) };

        // SAFETY: All requirements are met:
        // 1. The current module address must come from the
        //    current store, because it is the only parameter to
        //    this function that can contain module addresses. All
        //    stores guarantee all addresses in them to be valid
        //    within themselves.
        // 2. Validation guarantees the element index to be
        //    valid in the current module instance.
        // 3. The element address is valid for a similar reason
        //    that the module address is valid: it is stored in the
        //    current module instance, which is also part of the
        //    current store.
        unsafe {
            elem_drop(
                modules,
                &mut store_inner.elements,
                *current_module,
                elem_idx,
            );
        }
        Ok(None)
    }
);

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-copy-x-y
define_instruction!(
    no_fuel_check,
    table_copy,
    opcode::fc_extensions::TABLE_COPY,
    |Args {
         stack,
         wasm,
         modules,
         current_module,
         maybe_fuel,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_x_idx = unsafe { TableIdx::read_unchecked(wasm) };
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_y_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr_x = *unsafe { module.table_addrs.get(table_x_idx) };
        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr_y = *unsafe { module.table_addrs.get(table_y_idx) };

        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab_x_elem_len = unsafe { store_inner.tables.get(table_addr_x) }.elem.len();
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab_y_elem_len = unsafe { store_inner.tables.get(table_addr_y) }.elem.len();

        let n: u32 = stack.pop_value().try_into().unwrap_validated(); // size
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_COPY)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_COPY);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: u32 = stack.pop_value().try_into().unwrap_validated(); // source
        let d: u32 = stack.pop_value().try_into().unwrap_validated(); // destination

        let src_res = match s.checked_add(n) {
            Some(res) => {
                if res > tab_y_elem_len as u32 {
                    return Err(TrapError::TableOrElementAccessOutOfBounds.into());
                } else {
                    res.into_usize()
                }
            }
            _ => return Err(TrapError::TableOrElementAccessOutOfBounds.into()),
        };

        let dst_res = match d.checked_add(n) {
            Some(res) => {
                if res > tab_x_elem_len as u32 {
                    return Err(TrapError::TableOrElementAccessOutOfBounds.into());
                } else {
                    res.into_usize()
                }
            }
            _ => return Err(TrapError::TableOrElementAccessOutOfBounds.into()),
        };

        if table_addr_x == table_addr_y {
            // SAFETY: This table address was just read from the
            // current store. Therefore, it is valid in the
            // current store.
            let table = unsafe { store_inner.tables.get_mut(table_addr_x) };

            table.elem.copy_within(s as usize..src_res, d as usize);
        } else {
            let dst_addr = table_addr_x;
            let src_addr = table_addr_y;

            // SAFETY: These table addresses were just read from
            // the current store. Therefore, they are valid in
            // the current store.
            let (src_table, dst_table) =
                unsafe { store_inner.tables.get_two_mut(src_addr, dst_addr) }
                    .expect("both addrs to never be equal");

            dst_table.elem[d.into_usize()..dst_res]
                .copy_from_slice(&src_table.elem[s.into_usize()..src_res]);
        }

        trace!(
            "Instruction: table.copy '{}' '{}' [{} {} {}] -> []",
            table_x_idx,
            table_y_idx,
            d,
            s,
            n
        );
        Ok(None)
    }
);

define_instruction!(
    no_fuel_check,
    table_grow,
    opcode::fc_extensions::TABLE_GROW,
    |Args {
         stack,
         wasm,
         modules,
         current_module,
         maybe_fuel,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let sz = tab.elem.len() as u32;

        let n: u32 = stack.pop_value().try_into().unwrap_validated();
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_GROW)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_GROW);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let val: Ref = stack.pop_value().try_into().unwrap_validated();

        // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
        // for now we execute it always according to the following match expr.
        // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the stack per spec
        match tab.grow(n, val) {
            Ok(_) => {
                stack.push_value::<T>(Value::I32(sz))?;
            }
            Err(_) => {
                stack.push_value::<T>(Value::I32(u32::MAX))?;
            }
        }
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    table_size,
    opcode::fc_extensions::TABLE_SIZE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be valid table
        // index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let sz = tab.elem.len() as u32;

        stack.push_value::<T>(Value::I32(sz))?;

        trace!("Instruction: table.size '{}' [] -> [{}]", table_idx, sz);
        Ok(None)
    }
);

define_instruction!(
    no_fuel_check,
    table_fill,
    opcode::fc_extensions::TABLE_FILL,
    |Args {
         stack,
         wasm,
         modules,
         current_module,
         maybe_fuel,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let len: u32 = stack.pop_value().try_into().unwrap_validated();
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_FILL)
            + u64::from(len)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_FILL);
        if let Some(fuel) = maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                stack.push_value::<T>(Value::I32(len)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let val: Ref = stack.pop_value().try_into().unwrap_validated();
        let dst: u32 = stack.pop_value().try_into().unwrap_validated();

        let end = (dst.into_usize())
            .checked_add(len.into_usize())
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

        tab.elem
            .get_mut(dst.into_usize()..end)
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?
            .fill(val);

        trace!(
            "Instruction table.fill '{}' [{} {} {}] -> []",
            table_idx,
            dst,
            val,
            len
        );
        Ok(None)
    }
);

define_instruction!(
    no_fuel_check,
    fd_extensions,
    opcode::FD_EXTENSIONS,
    |args: Args| {
        // Should we call instruction hook here as well? Multibyte instruction
        let second_instr = args.wasm.read_var_u32().unwrap_validated();

        use crate::core::reader::types::opcode::fd_extensions::*;
        let instruction_fn = match second_instr {
            V128_LOAD => v128_load::<T>,
            V128_STORE => v128_store::<T>,
            V128_LOAD8X8_S => v128_load8x8_s::<T>,
            V128_LOAD8X8_U => v128_load8x8_u::<T>,
            V128_LOAD16X4_S => v128_load16x4_s::<T>,
            V128_LOAD16X4_U => v128_load16x4_u::<T>,
            V128_LOAD32X2_S => v128_load32x2_s::<T>,
            V128_LOAD32X2_U => v128_load32x2_u::<T>,
            V128_LOAD8_SPLAT => v128_load8_splat::<T>,
            V128_LOAD16_SPLAT => v128_load16_splat::<T>,
            V128_LOAD32_SPLAT => v128_load32_splat::<T>,
            V128_LOAD64_SPLAT => v128_load64_splat::<T>,
            V128_LOAD32_ZERO => v128_load32_zero::<T>,
            V128_LOAD64_ZERO => v128_load64_zero::<T>,
            V128_LOAD8_LANE => v128_load8_lane::<T>,
            V128_LOAD16_LANE => v128_load16_lane::<T>,
            V128_LOAD32_LANE => v128_load32_lane::<T>,
            V128_LOAD64_LANE => v128_load64_lane::<T>,
            V128_STORE8_LANE => v128_store8_lane::<T>,
            V128_STORE16_LANE => v128_store16_lane::<T>,
            V128_STORE32_LANE => v128_store32_lane::<T>,
            V128_STORE64_LANE => v128_store64_lane::<T>,
            V128_CONST => v128_const::<T>,
            V128_NOT => v128_not::<T>,
            V128_AND => v128_and::<T>,
            V128_ANDNOT => v128_andnot::<T>,
            V128_OR => v128_or::<T>,
            V128_XOR => v128_xor::<T>,
            V128_BITSELECT => v128_bitselect::<T>,
            V128_ANY_TRUE => v128_any_true::<T>,
            I8X16_SWIZZLE => i8x16_swizzle::<T>,
            I8X16_SHUFFLE => i8x16_shuffle::<T>,
            I8X16_SPLAT => i8x16_splat::<T>,
            I16X8_SPLAT => i16x8_splat::<T>,
            I32X4_SPLAT => i32x4_splat::<T>,
            I64X2_SPLAT => i64x2_splat::<T>,
            F32X4_SPLAT => f32x4_splat::<T>,
            F64X2_SPLAT => f64x2_splat::<T>,
            I8X16_EXTRACT_LANE_S => i8x16_extract_lane_s::<T>,
            I8X16_EXTRACT_LANE_U => i8x16_extract_lane_u::<T>,
            I16X8_EXTRACT_LANE_S => i16x8_extract_lane_s::<T>,
            I16X8_EXTRACT_LANE_U => i16x8_extract_lane_u::<T>,
            I32X4_EXTRACT_LANE => i32x4_extract_lane::<T>,
            I64X2_EXTRACT_LANE => i64x2_extract_lane::<T>,
            F32X4_EXTRACT_LANE => f32x4_extract_lane::<T>,
            F64X2_EXTRACT_LANE => f64x2_extract_lane::<T>,
            I8X16_REPLACE_LANE => i8x16_replace_lane::<T>,
            I16X8_REPLACE_LANE => i16x8_replace_lane::<T>,
            I32X4_REPLACE_LANE => i32x4_replace_lane::<T>,
            I64X2_REPLACE_LANE => i64x2_replace_lane::<T>,
            F32X4_REPLACE_LANE => f32x4_replace_lane::<T>,
            F64X2_REPLACE_LANE => f64x2_replace_lane::<T>,
            I8X16_ABS => i8x16_abs::<T>,
            I16X8_ABS => i16x8_abs::<T>,
            I32X4_ABS => i32x4_abs::<T>,
            I64X2_ABS => i64x2_abs::<T>,
            I8X16_NEG => i8x16_neg::<T>,
            I16X8_NEG => i16x8_neg::<T>,
            I32X4_NEG => i32x4_neg::<T>,
            I64X2_NEG => i64x2_neg::<T>,
            F32X4_ABS => f32x4_abs::<T>,
            F64X2_ABS => f64x2_abs::<T>,
            F32X4_NEG => f32x4_neg::<T>,
            F64X2_NEG => f64x2_neg::<T>,
            F32X4_SQRT => f32x4_sqrt::<T>,
            F64X2_SQRT => f64x2_sqrt::<T>,
            F32X4_CEIL => f32x4_ceil::<T>,
            F64X2_CEIL => f64x2_ceil::<T>,
            F32X4_FLOOR => f32x4_floor::<T>,
            F64X2_FLOOR => f64x2_floor::<T>,
            F32X4_TRUNC => f32x4_trunc::<T>,
            F64X2_TRUNC => f64x2_trunc::<T>,
            F32X4_NEAREST => f32x4_nearest::<T>,
            F64X2_NEAREST => f64x2_nearest::<T>,
            I8X16_POPCNT => i8x16_popcnt::<T>,
            I8X16_ADD => i8x16_add::<T>,
            I16X8_ADD => i16x8_add::<T>,
            I32X4_ADD => i32x4_add::<T>,
            I64X2_ADD => i64x2_add::<T>,
            I8X16_SUB => i8x16_sub::<T>,
            I16X8_SUB => i16x8_sub::<T>,
            I32X4_SUB => i32x4_sub::<T>,
            I64X2_SUB => i64x2_sub::<T>,
            F32X4_ADD => f32x4_add::<T>,
            F64X2_ADD => f64x2_add::<T>,
            F32X4_SUB => f32x4_sub::<T>,
            F64X2_SUB => f64x2_sub::<T>,
            F32X4_MUL => f32x4_mul::<T>,
            F64X2_MUL => f64x2_mul::<T>,
            F32X4_DIV => f32x4_div::<T>,
            F64X2_DIV => f64x2_div::<T>,
            F32X4_MIN => f32x4_min::<T>,
            F64X2_MIN => f64x2_min::<T>,
            F32X4_MAX => f32x4_max::<T>,
            F64X2_MAX => f64x2_max::<T>,
            F32X4_PMIN => f32x4_pmin::<T>,
            F64X2_PMIN => f64x2_pmin::<T>,
            F32X4_PMAX => f32x4_pmax::<T>,
            F64X2_PMAX => f64x2_pmax::<T>,
            I8X16_MIN_S => i8x16_min_s::<T>,
            I16X8_MIN_S => i16x8_min_s::<T>,
            I32X4_MIN_S => i32x4_min_s::<T>,
            I8X16_MIN_U => i8x16_min_u::<T>,
            I16X8_MIN_U => i16x8_min_u::<T>,
            I32X4_MIN_U => i32x4_min_u::<T>,
            I8X16_MAX_S => i8x16_max_s::<T>,
            I16X8_MAX_S => i16x8_max_s::<T>,
            I32X4_MAX_S => i32x4_max_s::<T>,
            I8X16_MAX_U => i8x16_max_u::<T>,
            I16X8_MAX_U => i16x8_max_u::<T>,
            I32X4_MAX_U => i32x4_max_u::<T>,
            I8X16_ADD_SAT_S => i8x16_add_sat_s::<T>,
            I16X8_ADD_SAT_S => i16x8_add_sat_s::<T>,
            I8X16_ADD_SAT_U => i8x16_add_sat_u::<T>,
            I16X8_ADD_SAT_U => i16x8_add_sat_u::<T>,
            I8X16_SUB_SAT_S => i8x16_sub_sat_s::<T>,
            I16X8_SUB_SAT_S => i16x8_sub_sat_s::<T>,
            I8X16_SUB_SAT_U => i8x16_sub_sat_u::<T>,
            I16X8_SUB_SAT_U => i16x8_sub_sat_u::<T>,
            I16X8_MUL => i16x8_mul::<T>,
            I32X4_MUL => i32x4_mul::<T>,
            I64X2_MUL => i64x2_mul::<T>,
            I8X16_AVGR_U => i8x16_avgr_u::<T>,
            I16X8_AVGR_U => i16x8_avgr_u::<T>,
            I16X8_Q15MULRSAT_S => i16x8_q15mulrsat_s::<T>,
            I8X16_EQ => i8x16_eq::<T>,
            I16X8_EQ => i16x8_eq::<T>,
            I32X4_EQ => i32x4_eq::<T>,
            I64X2_EQ => i64x2_eq::<T>,
            I8X16_NE => i8x16_ne::<T>,
            I16X8_NE => i16x8_ne::<T>,
            I32X4_NE => i32x4_ne::<T>,
            I64X2_NE => i64x2_ne::<T>,
            I8X16_LT_S => i8x16_lt_s::<T>,
            I16X8_LT_S => i16x8_lt_s::<T>,
            I32X4_LT_S => i32x4_lt_s::<T>,
            I64X2_LT_S => i64x2_lt_s::<T>,
            I8X16_LT_U => i8x16_lt_u::<T>,
            I16X8_LT_U => i16x8_lt_u::<T>,
            I32X4_LT_U => i32x4_lt_u::<T>,
            I8X16_GT_S => i8x16_gt_s::<T>,
            I16X8_GT_S => i16x8_gt_s::<T>,
            I32X4_GT_S => i32x4_gt_s::<T>,
            I64X2_GT_S => i64x2_gt_s::<T>,
            I8X16_GT_U => i8x16_gt_u::<T>,
            I16X8_GT_U => i16x8_gt_u::<T>,
            I32X4_GT_U => i32x4_gt_u::<T>,
            I8X16_LE_S => i8x16_le_s::<T>,
            I16X8_LE_S => i16x8_le_s::<T>,
            I32X4_LE_S => i32x4_le_s::<T>,
            I64X2_LE_S => i64x2_le_s::<T>,
            I8X16_LE_U => i8x16_le_u::<T>,
            I16X8_LE_U => i16x8_le_u::<T>,
            I32X4_LE_U => i32x4_le_u::<T>,
            I8X16_GE_S => i8x16_ge_s::<T>,
            I16X8_GE_S => i16x8_ge_s::<T>,
            I32X4_GE_S => i32x4_ge_s::<T>,
            I64X2_GE_S => i64x2_ge_s::<T>,
            I8X16_GE_U => i8x16_ge_u::<T>,
            I16X8_GE_U => i16x8_ge_u::<T>,
            I32X4_GE_U => i32x4_ge_u::<T>,
            F32X4_EQ => f32x4_eq::<T>,
            F64X2_EQ => f64x2_eq::<T>,
            F32X4_NE => f32x4_ne::<T>,
            F64X2_NE => f64x2_ne::<T>,
            F32X4_LT => f32x4_lt::<T>,
            F64X2_LT => f64x2_lt::<T>,
            F32X4_GT => f32x4_gt::<T>,
            F64X2_GT => f64x2_gt::<T>,
            F32X4_LE => f32x4_le::<T>,
            F64X2_LE => f64x2_le::<T>,
            F32X4_GE => f32x4_ge::<T>,
            F64X2_GE => f64x2_ge::<T>,
            I8X16_SHL => i8x16_shl::<T>,
            I16X8_SHL => i16x8_shl::<T>,
            I32X4_SHL => i32x4_shl::<T>,
            I64X2_SHL => i64x2_shl::<T>,
            I8X16_SHR_S => i8x16_shr_s::<T>,
            I8X16_SHR_U => i8x16_shr_u::<T>,
            I16X8_SHR_S => i16x8_shr_s::<T>,
            I16X8_SHR_U => i16x8_shr_u::<T>,
            I32X4_SHR_S => i32x4_shr_s::<T>,
            I32X4_SHR_U => i32x4_shr_u::<T>,
            I64X2_SHR_S => i64x2_shr_s::<T>,
            I64X2_SHR_U => i64x2_shr_u::<T>,
            I8X16_ALL_TRUE => i8x16_all_true::<T>,
            I16X8_ALL_TRUE => i16x8_all_true::<T>,
            I32X4_ALL_TRUE => i32x4_all_true::<T>,
            I64X2_ALL_TRUE => i64x2_all_true::<T>,
            I16X8_EXTEND_HIGH_I8X16_S => i16x8_extend_high_i8x16_s::<T>,
            I16X8_EXTEND_HIGH_I8X16_U => i16x8_extend_high_i8x16_u::<T>,
            I16X8_EXTEND_LOW_I8X16_S => i16x8_extend_low_i8x16_s::<T>,
            I16X8_EXTEND_LOW_I8X16_U => i16x8_extend_low_i8x16_u::<T>,
            I32X4_EXTEND_HIGH_I16X8_S => i32x4_extend_high_i16x8_s::<T>,
            I32X4_EXTEND_HIGH_I16X8_U => i32x4_extend_high_i16x8_u::<T>,
            I32X4_EXTEND_LOW_I16X8_S => i32x4_extend_low_i16x8_s::<T>,
            I32X4_EXTEND_LOW_I16X8_U => i32x4_extend_low_i16x8_u::<T>,
            I64X2_EXTEND_HIGH_I32X4_S => i64x2_extend_high_i32x4_s::<T>,
            I64X2_EXTEND_HIGH_I32X4_U => i64x2_extend_high_i32x4_u::<T>,
            I64X2_EXTEND_LOW_I32X4_S => i64x2_extend_low_i32x4_s::<T>,
            I64X2_EXTEND_LOW_I32X4_U => i64x2_extend_low_i32x4_u::<T>,
            I32X4_TRUNC_SAT_F32X4_S => i32x4_trunc_sat_f32x4_s::<T>,
            I32X4_TRUNC_SAT_F32X4_U => i32x4_trunc_sat_f32x4_u::<T>,
            I32X4_TRUNC_SAT_F64X2_S_ZERO => i32x4_trunc_sat_f64x2_s_zero::<T>,
            I32X4_TRUNC_SAT_F64X2_U_ZERO => i32x4_trunc_sat_f64x2_u_zero::<T>,
            F32X4_CONVERT_I32X4_S => f32x4_convert_i32x4_s::<T>,
            F32X4_CONVERT_I32X4_U => f32x4_convert_i32x4_u::<T>,
            F64X2_CONVERT_LOW_I32X4_S => f64x2_convert_low_i32x4_s::<T>,
            F64X2_CONVERT_LOW_I32X4_U => f64x2_convert_low_i32x4_u::<T>,
            F32X4_DEMOTE_F64X2_ZERO => f32x4_demote_f64x2_zero::<T>,
            F64X2_PROMOTE_LOW_F32X4 => f64x2_promote_low_f32x4::<T>,
            I8X16_NARROW_I16X8_S => i8x16_narrow_i16x8_s::<T>,
            I8X16_NARROW_I16X8_U => i8x16_narrow_i16x8_u::<T>,
            I16X8_NARROW_I32X4_S => i16x8_narrow_i32x4_s::<T>,
            I16X8_NARROW_I32X4_U => i16x8_narrow_i32x4_u::<T>,
            I8X16_BITMASK => i8x16_bitmask::<T>,
            I16X8_BITMASK => i16x8_bitmask::<T>,
            I32X4_BITMASK => i32x4_bitmask::<T>,
            I64X2_BITMASK => i64x2_bitmask::<T>,
            I32X4_DOT_I16X8_S => i32x4_dot_i16x8_s::<T>,
            I16X8_EXTMUL_HIGH_I8X16_S => i16x8_extmul_high_i8x16_s::<T>,
            I16X8_EXTMUL_HIGH_I8X16_U => i16x8_extmul_high_i8x16_u::<T>,
            I16X8_EXTMUL_LOW_I8X16_S => i16x8_extmul_low_i8x16_s::<T>,
            I16X8_EXTMUL_LOW_I8X16_U => i16x8_extmul_low_i8x16_u::<T>,
            I32X4_EXTMUL_HIGH_I16X8_S => i32x4_extmul_high_i16x8_s::<T>,
            I32X4_EXTMUL_HIGH_I16X8_U => i32x4_extmul_high_i16x8_u::<T>,
            I32X4_EXTMUL_LOW_I16X8_S => i32x4_extmul_low_i16x8_s::<T>,
            I32X4_EXTMUL_LOW_I16X8_U => i32x4_extmul_low_i16x8_u::<T>,
            I64X2_EXTMUL_HIGH_I32X4_S => i64x2_extmul_high_i32x4_s::<T>,
            I64X2_EXTMUL_HIGH_I32X4_U => i64x2_extmul_high_i32x4_u::<T>,
            I64X2_EXTMUL_LOW_I32X4_S => i64x2_extmul_low_i32x4_s::<T>,
            I64X2_EXTMUL_LOW_I32X4_U => i64x2_extmul_low_i32x4_u::<T>,
            I16X8_EXTADD_PAIRWISE_I8X16_S => i16x8_extadd_pairwise_i8x16_s::<T>,
            I16X8_EXTADD_PAIRWISE_I8X16_U => i16x8_extadd_pairwise_i8x16_u::<T>,
            I32X4_EXTADD_PAIRWISE_I16X8_S => i32x4_extadd_pairwise_i16x8_s::<T>,
            I32X4_EXTADD_PAIRWISE_I16X8_U => i32x4_extadd_pairwise_i16x8_u::<T>,

            F32X4_RELAXED_MADD
            | F32X4_RELAXED_MAX
            | F32X4_RELAXED_MIN
            | F32X4_RELAXED_NMADD
            | F64X2_RELAXED_MADD
            | F64X2_RELAXED_MAX
            | F64X2_RELAXED_MIN
            | F64X2_RELAXED_NMADD
            | I16X8_RELAXED_LANESELECT
            | I32X4_RELAXED_LANESELECT
            | I32X4_RELAXED_TRUNC_F32X4_S
            | I32X4_RELAXED_TRUNC_F32X4_U
            | I32X4_RELAXED_TRUNC_F64X2_S_ZERO
            | I32X4_RELAXED_TRUNC_F64X2_U_ZERO
            | I64X2_RELAXED_LANESELECT
            | I8X16_RELAXED_LANESELECT
            | I8X16_RELAXED_SWIZZLE
            | 154
            | 187
            | 194
            | 256.. => unreachable_validated!(),
        };

        // SAFETY: The caller of the current fd_extension instruction handler ensures the same
        // safety requirements that are also required by the handler we are calling now.
        unsafe { instruction_fn(args) }
    }
);

define_instruction!(
    fd_fuel_check,
    v128_load,
    opcode::fd_extensions::V128_LOAD,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data: u128 = memory.mem.load(idx)?;
        stack.push_value::<T>(data.to_le_bytes().into())?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_store,
    opcode::fd_extensions::V128_STORE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        memory.mem.store(idx, u128::from_le_bytes(data))?;
        Ok(None)
    }
);

// v128.loadNxM_sx
define_instruction!(
    fd_fuel_check,
    v128_load8x8_s,
    opcode::fd_extensions::V128_LOAD8X8_S,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i8; 8] = to_lanes::<1, 16, i8>(data)[..8].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i16);

        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load8x8_u,
    opcode::fd_extensions::V128_LOAD8X8_U,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u8; 8] = to_lanes::<1, 16, u8>(data)[..8].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u16);

        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load16x4_s,
    opcode::fd_extensions::V128_LOAD16X4_S,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i16; 4] = to_lanes::<2, 8, i16>(data)[..4].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i32);

        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load16x4_u,
    opcode::fd_extensions::V128_LOAD16X4_U,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u16; 4] = to_lanes::<2, 8, u16>(data)[..4].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u32);

        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32x2_s,
    opcode::fd_extensions::V128_LOAD32X2_S,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i32; 2] = to_lanes::<4, 4, i32>(data)[..2].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i64);

        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32x2_u,
    opcode::fd_extensions::V128_LOAD32X2_U,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u32; 2] = to_lanes::<4, 4, u32>(data)[..2].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u64);

        stack.push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);

// v128.loadN_splat
define_instruction!(
    fd_fuel_check,
    v128_load8_splat,
    opcode::fd_extensions::V128_LOAD8_SPLAT,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<1, u8>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes([lane; 16])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load16_splat,
    opcode::fd_extensions::V128_LOAD16_SPLAT,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<2, u16>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes([lane; 8])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32_splat,
    opcode::fd_extensions::V128_LOAD32_SPLAT,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<4, u32>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes([lane; 4])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load64_splat,
    opcode::fd_extensions::V128_LOAD64_SPLAT,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<8, u64>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes([lane; 2])))?;
        Ok(None)
    }
);

// v128.loadN_zero
define_instruction!(
    fd_fuel_check,
    v128_load32_zero,
    opcode::fd_extensions::V128_LOAD32_ZERO,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data = memory.mem.load::<4, u32>(idx)? as u128;
        stack.push_value::<T>(Value::V128(data.to_le_bytes()))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load64_zero,
    opcode::fd_extensions::V128_LOAD64_ZERO,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };

        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data = memory.mem.load::<8, u64>(idx)? as u128;
        stack.push_value::<T>(Value::V128(data.to_le_bytes()))?;
        Ok(None)
    }
);

// v128.loadN_lane
define_instruction!(
    fd_fuel_check,
    v128_load8_lane,
    opcode::fd_extensions::V128_LOAD8_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let mut lanes: [u8; 16] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<1, u8>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    v128_load16_lane,
    opcode::fd_extensions::V128_LOAD16_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let mut lanes: [u16; 8] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<2, u16>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32_lane,
    opcode::fd_extensions::V128_LOAD32_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let mut lanes: [u32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<4, u32>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load64_lane,
    opcode::fd_extensions::V128_LOAD64_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let mut lanes: [u64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<8, u64>(idx)?;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);

// v128.storeN_lane
define_instruction!(
    fd_fuel_check,
    v128_store8_lane,
    opcode::fd_extensions::V128_STORE8_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());

        let lane = *to_lanes::<1, 16, u8>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<1, u8>(idx, lane)?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_store16_lane,
    opcode::fd_extensions::V128_STORE16_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());

        let lane = *to_lanes::<2, 8, u16>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<2, u16>(idx, lane)?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_store32_lane,
    opcode::fd_extensions::V128_STORE32_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());

        let lane = *to_lanes::<4, 4, u32>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<4, u32>(idx, lane)?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_store64_lane,
    opcode::fd_extensions::V128_STORE64_LANE,
    |Args {
         wasm,
         stack,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = stack.pop_value().try_into().unwrap_validated();
        let memarg = MemArg::read(wasm).unwrap_validated();
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees at least one memory to
        // exist.
        let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
        // SAFETY: This memory address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let memory = unsafe { store_inner.memories.get(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());

        let lane = *to_lanes::<8, 2, u64>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<8, u64>(idx, lane)?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    v128_const,
    opcode::fd_extensions::V128_CONST,
    |Args { wasm, stack, .. }| {
        let mut data = [0; 16];
        for byte_ref in &mut data {
            *byte_ref = wasm.read_u8().unwrap_validated();
        }

        stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);

// vvunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvunop>
define_instruction!(
    fd_fuel_check,
    v128_not,
    opcode::fd_extensions::V128_NOT,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        stack.push_value::<T>(Value::V128(data.map(|byte| !byte)))?;
        Ok(None)
    }
);

// vvbinop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvbinop>
define_instruction!(
    fd_fuel_check,
    v128_and,
    opcode::fd_extensions::V128_AND,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] & data2[i]);
        stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_andnot,
    opcode::fd_extensions::V128_ANDNOT,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] & !data2[i]);
        stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_or,
    opcode::fd_extensions::V128_OR,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] | data2[i]);
        stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_xor,
    opcode::fd_extensions::V128_XOR,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] ^ data2[i]);
        stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

// vvternop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvternop>
define_instruction!(
    fd_fuel_check,
    v128_bitselect,
    opcode::fd_extensions::V128_BITSELECT,
    |Args { stack, .. }| {
        let data3: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| (data1[i] & data3[i]) | (data2[i] & !data3[i]));
        stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

// vvtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvtestop>
define_instruction!(
    fd_fuel_check,
    v128_any_true,
    opcode::fd_extensions::V128_ANY_TRUE,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let any_true = data.into_iter().any(|byte| byte > 0);
        stack.push_value::<T>(Value::I32(any_true as u32))?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    i8x16_swizzle,
    opcode::fd_extensions::I8X16_SWIZZLE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| *data1.get(usize::from(data2[i])).unwrap_or(&0));
        stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    i8x16_shuffle,
    opcode::fd_extensions::I8X16_SHUFFLE,
    |Args { wasm, stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();

        let lane_selector_indices: [u8; 16] = array::from_fn(|_| wasm.read_u8().unwrap_validated());

        let result = lane_selector_indices.map(|i| {
            *data1
                .get(usize::from(i))
                .or_else(|| data2.get(usize::from(i) - 16))
                .unwrap_validated()
        });

        stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

// shape.splat
define_instruction!(
    fd_fuel_check,
    i8x16_splat,
    opcode::fd_extensions::I8X16_SPLAT,
    |Args { stack, .. }| {
        let value: u32 = stack.pop_value().try_into().unwrap_validated();
        let lane = value as u8;
        let data = from_lanes([lane; 16]);
        stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_splat,
    opcode::fd_extensions::I16X8_SPLAT,
    |Args { stack, .. }| {
        let value: u32 = stack.pop_value().try_into().unwrap_validated();
        let lane = value as u16;
        let data = from_lanes([lane; 8]);
        stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_splat,
    opcode::fd_extensions::I32X4_SPLAT,
    |Args { stack, .. }| {
        let lane: u32 = stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 4]);
        stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_splat,
    opcode::fd_extensions::I64X2_SPLAT,
    |Args { stack, .. }| {
        let lane: u64 = stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 2]);
        stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_splat,
    opcode::fd_extensions::F32X4_SPLAT,
    |Args { stack, .. }| {
        let lane: F32 = stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 4]);
        stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_splat,
    opcode::fd_extensions::F64X2_SPLAT,
    |Args { stack, .. }| {
        let lane: F64 = stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 2]);
        stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);

// shape.extract_lane
define_instruction!(
    fd_fuel_check,
    i8x16_extract_lane_s,
    opcode::fd_extensions::I8X16_EXTRACT_LANE_S,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_extract_lane_u,
    opcode::fd_extensions::I8X16_EXTRACT_LANE_U,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extract_lane_s,
    opcode::fd_extensions::I16X8_EXTRACT_LANE_S,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extract_lane_u,
    opcode::fd_extensions::I16X8_EXTRACT_LANE_U,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extract_lane,
    opcode::fd_extensions::I32X4_EXTRACT_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::I32(lane))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extract_lane,
    opcode::fd_extensions::I64X2_EXTRACT_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::I64(lane))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_extract_lane,
    opcode::fd_extensions::F32X4_EXTRACT_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::F32(lane))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_extract_lane,
    opcode::fd_extensions::F64X2_EXTRACT_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        stack.push_value::<T>(Value::F64(lane))?;
        Ok(None)
    }
);

// shape.replace_lane
define_instruction!(
    fd_fuel_check,
    i8x16_replace_lane,
    opcode::fd_extensions::I8X16_REPLACE_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let value: u32 = stack.pop_value().try_into().unwrap_validated();
        let new_lane = value as u8;
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u8; 16] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_replace_lane,
    opcode::fd_extensions::I16X8_REPLACE_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let value: u32 = stack.pop_value().try_into().unwrap_validated();
        let new_lane = value as u16;
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u16; 8] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_replace_lane,
    opcode::fd_extensions::I32X4_REPLACE_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_replace_lane,
    opcode::fd_extensions::I64X2_REPLACE_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: u64 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_replace_lane,
    opcode::fd_extensions::F32X4_REPLACE_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: F32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [F32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_replace_lane,
    opcode::fd_extensions::F64X2_REPLACE_LANE,
    |Args { wasm, stack, .. }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: F64 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [F64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        stack.push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);

// Group vunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vunop>
// viunop
define_instruction!(
    fd_fuel_check,
    i8x16_abs,
    opcode::fd_extensions::I8X16_ABS,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(i8::wrapping_abs);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_abs,
    opcode::fd_extensions::I16X8_ABS,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(i16::wrapping_abs);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_abs,
    opcode::fd_extensions::I32X4_ABS,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(i32::wrapping_abs);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_abs,
    opcode::fd_extensions::I64X2_ABS,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(i64::wrapping_abs);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_neg,
    opcode::fd_extensions::I8X16_NEG,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(i8::wrapping_neg);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_neg,
    opcode::fd_extensions::I16X8_NEG,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(i16::wrapping_neg);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_neg,
    opcode::fd_extensions::I32X4_NEG,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(i32::wrapping_neg);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_neg,
    opcode::fd_extensions::I64X2_NEG,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(i64::wrapping_neg);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
// vfunop
define_instruction!(
    fd_fuel_check,
    f32x4_abs,
    opcode::fd_extensions::F32X4_ABS,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.abs());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_abs,
    opcode::fd_extensions::F64X2_ABS,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.abs());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_neg,
    opcode::fd_extensions::F32X4_NEG,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_neg,
    opcode::fd_extensions::F64X2_NEG,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_sqrt,
    opcode::fd_extensions::F32X4_SQRT,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.sqrt());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_sqrt,
    opcode::fd_extensions::F64X2_SQRT,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.sqrt());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_ceil,
    opcode::fd_extensions::F32X4_CEIL,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.ceil());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_ceil,
    opcode::fd_extensions::F64X2_CEIL,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.ceil());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_floor,
    opcode::fd_extensions::F32X4_FLOOR,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.floor());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_floor,
    opcode::fd_extensions::F64X2_FLOOR,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.floor());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_trunc,
    opcode::fd_extensions::F32X4_TRUNC,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.trunc());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_trunc,
    opcode::fd_extensions::F64X2_TRUNC,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.trunc());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_nearest,
    opcode::fd_extensions::F32X4_NEAREST,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.nearest());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_nearest,
    opcode::fd_extensions::F64X2_NEAREST,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.nearest());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
// others
define_instruction!(
    fd_fuel_check,
    i8x16_popcnt,
    opcode::fd_extensions::I8X16_POPCNT,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.count_ones() as u8);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// Group vbinop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vbinop>
// vibinop
define_instruction!(
    fd_fuel_check,
    i8x16_add,
    opcode::fd_extensions::I8X16_ADD,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_add,
    opcode::fd_extensions::I16X8_ADD,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_add,
    opcode::fd_extensions::I32X4_ADD,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_add,
    opcode::fd_extensions::I64X2_ADD,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_sub,
    opcode::fd_extensions::I8X16_SUB,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_sub,
    opcode::fd_extensions::I16X8_SUB,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_sub,
    opcode::fd_extensions::I32X4_SUB,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_sub,
    opcode::fd_extensions::I64X2_SUB,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
// vfbinop
define_instruction!(
    fd_fuel_check,
    f32x4_add,
    opcode::fd_extensions::F32X4_ADD,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_add,
    opcode::fd_extensions::F64X2_ADD,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_sub,
    opcode::fd_extensions::F32X4_SUB,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_sub,
    opcode::fd_extensions::F64X2_SUB,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_mul,
    opcode::fd_extensions::F32X4_MUL,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_mul,
    opcode::fd_extensions::F64X2_MUL,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_div,
    opcode::fd_extensions::F32X4_DIV,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_div,
    opcode::fd_extensions::F64X2_DIV,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_min,
    opcode::fd_extensions::F32X4_MIN,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_min,
    opcode::fd_extensions::F64X2_MIN,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_max,
    opcode::fd_extensions::F32X4_MAX,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_max,
    opcode::fd_extensions::F64X2_MAX,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_pmin,
    opcode::fd_extensions::F32X4_PMIN,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| {
            let v1 = lanes1[i];
            let v2 = lanes2[i];
            if v2 < v1 {
                v2
            } else {
                v1
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_pmin,
    opcode::fd_extensions::F64X2_PMIN,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| {
            let v1 = lanes1[i];
            let v2 = lanes2[i];
            if v2 < v1 {
                v2
            } else {
                v1
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_pmax,
    opcode::fd_extensions::F32X4_PMAX,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| {
            let v1 = lanes1[i];
            let v2 = lanes2[i];
            if v1 < v2 {
                v2
            } else {
                v1
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_pmax,
    opcode::fd_extensions::F64X2_PMAX,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| {
            let v1 = lanes1[i];
            let v2 = lanes2[i];
            if v1 < v2 {
                v2
            } else {
                v1
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
// viminmaxop
define_instruction!(
    fd_fuel_check,
    i8x16_min_s,
    opcode::fd_extensions::I8X16_MIN_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_min_s,
    opcode::fd_extensions::I16X8_MIN_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_min_s,
    opcode::fd_extensions::I32X4_MIN_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_min_u,
    opcode::fd_extensions::I8X16_MIN_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_min_u,
    opcode::fd_extensions::I16X8_MIN_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_min_u,
    opcode::fd_extensions::I32X4_MIN_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_max_s,
    opcode::fd_extensions::I8X16_MAX_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_max_s,
    opcode::fd_extensions::I16X8_MAX_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_max_s,
    opcode::fd_extensions::I32X4_MAX_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_max_u,
    opcode::fd_extensions::I8X16_MAX_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_max_u,
    opcode::fd_extensions::I16X8_MAX_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_max_u,
    opcode::fd_extensions::I32X4_MAX_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// visatbinop
define_instruction!(
    fd_fuel_check,
    i8x16_add_sat_s,
    opcode::fd_extensions::I8X16_ADD_SAT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_add_sat_s,
    opcode::fd_extensions::I16X8_ADD_SAT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_add_sat_u,
    opcode::fd_extensions::I8X16_ADD_SAT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_add_sat_u,
    opcode::fd_extensions::I16X8_ADD_SAT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_sub_sat_s,
    opcode::fd_extensions::I8X16_SUB_SAT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_sub_sat_s,
    opcode::fd_extensions::I16X8_SUB_SAT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_sub_sat_u,
    opcode::fd_extensions::I8X16_SUB_SAT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_sub_sat_u,
    opcode::fd_extensions::I16X8_SUB_SAT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
// others
define_instruction!(
    fd_fuel_check,
    i16x8_mul,
    opcode::fd_extensions::I16X8_MUL,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_mul,
    opcode::fd_extensions::I32X4_MUL,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_mul,
    opcode::fd_extensions::I64X2_MUL,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_avgr_u,
    opcode::fd_extensions::I8X16_AVGR_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] =
            array::from_fn(|i| (lanes1[i] as u16 + lanes2[i] as u16).div_ceil(2) as u8);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_avgr_u,
    opcode::fd_extensions::I16X8_AVGR_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] =
            array::from_fn(|i| (lanes1[i] as u32 + lanes2[i] as u32).div_ceil(2) as u16);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_q15mulrsat_s,
    opcode::fd_extensions::I16X8_Q15MULRSAT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| {
            (((lanes1[i] as i64).mul(lanes2[i] as i64) + 2i64.pow(14)) >> 15i64)
                .clamp(i16::MIN as i64, i16::MAX as i64) as i16
        });
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// Group vrelop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vrelop>
// virelop
define_instruction!(
    fd_fuel_check,
    i8x16_eq,
    opcode::fd_extensions::I8X16_EQ,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_eq,
    opcode::fd_extensions::I16X8_EQ,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_eq,
    opcode::fd_extensions::I32X4_EQ,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_eq,
    opcode::fd_extensions::I64X2_EQ,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_ne,
    opcode::fd_extensions::I8X16_NE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_ne,
    opcode::fd_extensions::I16X8_NE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_ne,
    opcode::fd_extensions::I32X4_NE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_ne,
    opcode::fd_extensions::I64X2_NE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_lt_s,
    opcode::fd_extensions::I8X16_LT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_lt_s,
    opcode::fd_extensions::I16X8_LT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_lt_s,
    opcode::fd_extensions::I32X4_LT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_lt_s,
    opcode::fd_extensions::I64X2_LT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_lt_u,
    opcode::fd_extensions::I8X16_LT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_lt_u,
    opcode::fd_extensions::I16X8_LT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_lt_u,
    opcode::fd_extensions::I32X4_LT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_gt_s,
    opcode::fd_extensions::I8X16_GT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_gt_s,
    opcode::fd_extensions::I16X8_GT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_gt_s,
    opcode::fd_extensions::I32X4_GT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_gt_s,
    opcode::fd_extensions::I64X2_GT_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_gt_u,
    opcode::fd_extensions::I8X16_GT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_gt_u,
    opcode::fd_extensions::I16X8_GT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_gt_u,
    opcode::fd_extensions::I32X4_GT_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_le_s,
    opcode::fd_extensions::I8X16_LE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_le_s,
    opcode::fd_extensions::I16X8_LE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_le_s,
    opcode::fd_extensions::I32X4_LE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_le_s,
    opcode::fd_extensions::I64X2_LE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_le_u,
    opcode::fd_extensions::I8X16_LE_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_le_u,
    opcode::fd_extensions::I16X8_LE_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_le_u,
    opcode::fd_extensions::I32X4_LE_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    i8x16_ge_s,
    opcode::fd_extensions::I8X16_GE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_ge_s,
    opcode::fd_extensions::I16X8_GE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_ge_s,
    opcode::fd_extensions::I32X4_GE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_ge_s,
    opcode::fd_extensions::I64X2_GE_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_ge_u,
    opcode::fd_extensions::I8X16_GE_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_ge_u,
    opcode::fd_extensions::I16X8_GE_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_ge_u,
    opcode::fd_extensions::I32X4_GE_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
// vfrelop
define_instruction!(
    fd_fuel_check,
    f32x4_eq,
    opcode::fd_extensions::F32X4_EQ,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_eq,
    opcode::fd_extensions::F64X2_EQ,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_ne,
    opcode::fd_extensions::F32X4_NE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_ne,
    opcode::fd_extensions::F64X2_NE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_lt,
    opcode::fd_extensions::F32X4_LT,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_lt,
    opcode::fd_extensions::F64X2_LT,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_gt,
    opcode::fd_extensions::F32X4_GT,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_gt,
    opcode::fd_extensions::F64X2_GT,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_le,
    opcode::fd_extensions::F32X4_LE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_le,
    opcode::fd_extensions::F64X2_LE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_ge,
    opcode::fd_extensions::F32X4_GE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_ge,
    opcode::fd_extensions::F64X2_GE,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// Group vishiftop
define_instruction!(
    fd_fuel_check,
    i8x16_shl,
    opcode::fd_extensions::I8X16_SHL,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shl(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_shl,
    opcode::fd_extensions::I16X8_SHL,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shl(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_shl,
    opcode::fd_extensions::I32X4_SHL,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shl(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_shl,
    opcode::fd_extensions::I64X2_SHL,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shl(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_shr_s,
    opcode::fd_extensions::I8X16_SHR_S,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_shr_u,
    opcode::fd_extensions::I8X16_SHR_U,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_shr_s,
    opcode::fd_extensions::I16X8_SHR_S,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_shr_u,
    opcode::fd_extensions::I16X8_SHR_U,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_shr_s,
    opcode::fd_extensions::I32X4_SHR_S,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_shr_u,
    opcode::fd_extensions::I32X4_SHR_U,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_shr_s,
    opcode::fd_extensions::I64X2_SHR_S,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_shr_u,
    opcode::fd_extensions::I64X2_SHR_U,
    |Args { stack, .. }| {
        let shift: u32 = stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// Group vtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vtestop>
// vitestop
define_instruction!(
    fd_fuel_check,
    i8x16_all_true,
    opcode::fd_extensions::I8X16_ALL_TRUE,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        stack.push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_all_true,
    opcode::fd_extensions::I16X8_ALL_TRUE,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        stack.push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_all_true,
    opcode::fd_extensions::I32X4_ALL_TRUE,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        stack.push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_all_true,
    opcode::fd_extensions::I64X2_ALL_TRUE,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        stack.push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);

// Group vcvtop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vcvtop>
define_instruction!(
    fd_fuel_check,
    i16x8_extend_high_i8x16_s,
    opcode::fd_extensions::I16X8_EXTEND_HIGH_I8X16_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let high_lanes: [i8; 8] = lanes[8..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i16);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extend_high_i8x16_u,
    opcode::fd_extensions::I16X8_EXTEND_HIGH_I8X16_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let high_lanes: [u8; 8] = lanes[8..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u16);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extend_low_i8x16_s,
    opcode::fd_extensions::I16X8_EXTEND_LOW_I8X16_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let low_lanes: [i8; 8] = lanes[..8].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i16);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extend_low_i8x16_u,
    opcode::fd_extensions::I16X8_EXTEND_LOW_I8X16_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let low_lanes: [u8; 8] = lanes[..8].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u16);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_high_i16x8_s,
    opcode::fd_extensions::I32X4_EXTEND_HIGH_I16X8_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let high_lanes: [i16; 4] = lanes[4..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i32);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_high_i16x8_u,
    opcode::fd_extensions::I32X4_EXTEND_HIGH_I16X8_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let high_lanes: [u16; 4] = lanes[4..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u32);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_low_i16x8_s,
    opcode::fd_extensions::I32X4_EXTEND_LOW_I16X8_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let low_lanes: [i16; 4] = lanes[..4].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i32);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_low_i16x8_u,
    opcode::fd_extensions::I32X4_EXTEND_LOW_I16X8_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let low_lanes: [u16; 4] = lanes[..4].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u32);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_high_i32x4_s,
    opcode::fd_extensions::I64X2_EXTEND_HIGH_I32X4_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let high_lanes: [i32; 2] = lanes[2..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i64);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_high_i32x4_u,
    opcode::fd_extensions::I64X2_EXTEND_HIGH_I32X4_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let high_lanes: [u32; 2] = lanes[2..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u64);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_low_i32x4_s,
    opcode::fd_extensions::I64X2_EXTEND_LOW_I32X4_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i64);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_low_i32x4_u,
    opcode::fd_extensions::I64X2_EXTEND_LOW_I32X4_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u64);
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f32x4_s,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F32X4_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result = lanes.map(|lane| {
            if lane.is_nan() {
                0
            } else if lane.is_negative_infinity() {
                i32::MIN
            } else if lane.is_infinity() {
                i32::MAX
            } else {
                lane.as_i32()
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f32x4_u,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F32X4_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result = lanes.map(|lane| {
            if lane.is_nan() || lane.is_negative_infinity() {
                u32::MIN
            } else if lane.is_infinity() {
                u32::MAX
            } else {
                lane.as_u32()
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f64x2_s_zero,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F64X2_S_ZERO,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result = lanes.map(|lane| {
            if lane.is_nan() {
                0
            } else if lane.is_negative_infinity() {
                i32::MIN
            } else if lane.is_infinity() {
                i32::MAX
            } else {
                lane.as_i32()
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f64x2_u_zero,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F64X2_U_ZERO,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result = lanes.map(|lane| {
            if lane.is_nan() || lane.is_negative_infinity() {
                u32::MIN
            } else if lane.is_infinity() {
                u32::MAX
            } else {
                lane.as_u32()
            }
        });
        stack.push_value::<T>(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_convert_i32x4_s,
    opcode::fd_extensions::F32X4_CONVERT_I32X4_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_convert_i32x4_u,
    opcode::fd_extensions::F32X4_CONVERT_I32X4_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_convert_low_i32x4_s,
    opcode::fd_extensions::F64X2_CONVERT_LOW_I32X4_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| F64(lane as f64));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_convert_low_i32x4_u,
    opcode::fd_extensions::F64X2_CONVERT_LOW_I32X4_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| F64(lane as f64));
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_demote_f64x2_zero,
    opcode::fd_extensions::F32X4_DEMOTE_F64X2_ZERO,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes = to_lanes::<8, 2, F64>(data);
        let half_lanes = lanes.map(|lane| lane.as_f32());
        let result = [half_lanes[0], half_lanes[1], F32(0.0), F32(0.0)];
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_promote_low_f32x4,
    opcode::fd_extensions::F64X2_PROMOTE_LOW_F32X4,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let half_lanes: [F32; 2] = lanes[..2].try_into().unwrap();
        let result = half_lanes.map(|lane| lane.as_f64());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// ishape.narrow_ishape_sx
define_instruction!(
    fd_fuel_check,
    i8x16_narrow_i16x8_s,
    opcode::fd_extensions::I8X16_NARROW_I16X8_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(i8::MIN as i16, i8::MAX as i16) as i8);
        let result: [i8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_narrow_i16x8_u,
    opcode::fd_extensions::I8X16_NARROW_I16X8_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(u8::MIN as i16, u8::MAX as i16) as u8);
        let result: [u8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_narrow_i32x4_s,
    opcode::fd_extensions::I16X8_NARROW_I32X4_S,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(i16::MIN as i32, i16::MAX as i32) as i16);
        let result: [i16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_narrow_i32x4_u,
    opcode::fd_extensions::I16X8_NARROW_I32X4_U,
    |Args { stack, .. }| {
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(u16::MIN as i32, u16::MAX as i32) as u16);
        let result: [u16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        stack.push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// ishape.bitmask
define_instruction!(
    fd_fuel_check,
    i8x16_bitmask,
    opcode::fd_extensions::I8X16_BITMASK,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_bitmask,
    opcode::fd_extensions::I16X8_BITMASK,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_bitmask,
    opcode::fd_extensions::I32X4_BITMASK,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_bitmask,
    opcode::fd_extensions::I64X2_BITMASK,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);

// ishape.dot_ishape_s
define_instruction!(
    fd_fuel_check,
    i32x4_dot_i16x8_s,
    opcode::fd_extensions::I32X4_DOT_I16X8_S,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i16; 8] = to_lanes(data1);
        let lanes2: [i16; 8] = to_lanes(data2);
        let multiplied: [i32; 8] = array::from_fn(|i| {
            let v1 = lanes1[i] as i32;
            let v2 = lanes2[i] as i32;
            v1.wrapping_mul(v2)
        });
        let added: [i32; 4] = array::from_fn(|i| {
            let v1 = multiplied[2 * i];
            let v2 = multiplied[2 * i + 1];
            v1.wrapping_add(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(added)))?;
        Ok(None)
    }
);

// ishape.extmul_half_ishape_sx
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_high_i8x16_s,
    opcode::fd_extensions::I16X8_EXTMUL_HIGH_I8X16_S,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i8; 16] = to_lanes(data1);
        let lanes2: [i8; 16] = to_lanes(data2);
        let high_lanes1: [i8; 8] = lanes1[8..].try_into().unwrap();
        let high_lanes2: [i8; 8] = lanes2[8..].try_into().unwrap();
        let multiplied: [i16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i16;
            let v2 = high_lanes2[i] as i16;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_high_i8x16_u,
    opcode::fd_extensions::I16X8_EXTMUL_HIGH_I8X16_U,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u8; 16] = to_lanes(data1);
        let lanes2: [u8; 16] = to_lanes(data2);
        let high_lanes1: [u8; 8] = lanes1[8..].try_into().unwrap();
        let high_lanes2: [u8; 8] = lanes2[8..].try_into().unwrap();
        let multiplied: [u16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u16;
            let v2 = high_lanes2[i] as u16;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_low_i8x16_s,
    opcode::fd_extensions::I16X8_EXTMUL_LOW_I8X16_S,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i8; 16] = to_lanes(data1);
        let lanes2: [i8; 16] = to_lanes(data2);
        let high_lanes1: [i8; 8] = lanes1[..8].try_into().unwrap();
        let high_lanes2: [i8; 8] = lanes2[..8].try_into().unwrap();
        let multiplied: [i16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i16;
            let v2 = high_lanes2[i] as i16;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_low_i8x16_u,
    opcode::fd_extensions::I16X8_EXTMUL_LOW_I8X16_U,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u8; 16] = to_lanes(data1);
        let lanes2: [u8; 16] = to_lanes(data2);
        let high_lanes1: [u8; 8] = lanes1[..8].try_into().unwrap();
        let high_lanes2: [u8; 8] = lanes2[..8].try_into().unwrap();
        let multiplied: [u16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u16;
            let v2 = high_lanes2[i] as u16;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_high_i16x8_s,
    opcode::fd_extensions::I32X4_EXTMUL_HIGH_I16X8_S,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i16; 8] = to_lanes(data1);
        let lanes2: [i16; 8] = to_lanes(data2);
        let high_lanes1: [i16; 4] = lanes1[4..].try_into().unwrap();
        let high_lanes2: [i16; 4] = lanes2[4..].try_into().unwrap();
        let multiplied: [i32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i32;
            let v2 = high_lanes2[i] as i32;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_high_i16x8_u,
    opcode::fd_extensions::I32X4_EXTMUL_HIGH_I16X8_U,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u16; 8] = to_lanes(data1);
        let lanes2: [u16; 8] = to_lanes(data2);
        let high_lanes1: [u16; 4] = lanes1[4..].try_into().unwrap();
        let high_lanes2: [u16; 4] = lanes2[4..].try_into().unwrap();
        let multiplied: [u32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u32;
            let v2 = high_lanes2[i] as u32;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_low_i16x8_s,
    opcode::fd_extensions::I32X4_EXTMUL_LOW_I16X8_S,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i16; 8] = to_lanes(data1);
        let lanes2: [i16; 8] = to_lanes(data2);
        let high_lanes1: [i16; 4] = lanes1[..4].try_into().unwrap();
        let high_lanes2: [i16; 4] = lanes2[..4].try_into().unwrap();
        let multiplied: [i32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i32;
            let v2 = high_lanes2[i] as i32;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_low_i16x8_u,
    opcode::fd_extensions::I32X4_EXTMUL_LOW_I16X8_U,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u16; 8] = to_lanes(data1);
        let lanes2: [u16; 8] = to_lanes(data2);
        let high_lanes1: [u16; 4] = lanes1[..4].try_into().unwrap();
        let high_lanes2: [u16; 4] = lanes2[..4].try_into().unwrap();
        let multiplied: [u32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u32;
            let v2 = high_lanes2[i] as u32;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_high_i32x4_s,
    opcode::fd_extensions::I64X2_EXTMUL_HIGH_I32X4_S,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i32; 4] = to_lanes(data1);
        let lanes2: [i32; 4] = to_lanes(data2);
        let high_lanes1: [i32; 2] = lanes1[2..].try_into().unwrap();
        let high_lanes2: [i32; 2] = lanes2[2..].try_into().unwrap();
        let multiplied: [i64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i64;
            let v2 = high_lanes2[i] as i64;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_high_i32x4_u,
    opcode::fd_extensions::I64X2_EXTMUL_HIGH_I32X4_U,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u32; 4] = to_lanes(data1);
        let lanes2: [u32; 4] = to_lanes(data2);
        let high_lanes1: [u32; 2] = lanes1[2..].try_into().unwrap();
        let high_lanes2: [u32; 2] = lanes2[2..].try_into().unwrap();
        let multiplied: [u64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u64;
            let v2 = high_lanes2[i] as u64;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_low_i32x4_s,
    opcode::fd_extensions::I64X2_EXTMUL_LOW_I32X4_S,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i32; 4] = to_lanes(data1);
        let lanes2: [i32; 4] = to_lanes(data2);
        let high_lanes1: [i32; 2] = lanes1[..2].try_into().unwrap();
        let high_lanes2: [i32; 2] = lanes2[..2].try_into().unwrap();
        let multiplied: [i64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i64;
            let v2 = high_lanes2[i] as i64;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_low_i32x4_u,
    opcode::fd_extensions::I64X2_EXTMUL_LOW_I32X4_U,
    |Args { stack, .. }| {
        let data1: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u32; 4] = to_lanes(data1);
        let lanes2: [u32; 4] = to_lanes(data2);
        let high_lanes1: [u32; 2] = lanes1[..2].try_into().unwrap();
        let high_lanes2: [u32; 2] = lanes2[..2].try_into().unwrap();
        let multiplied: [u64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u64;
            let v2 = high_lanes2[i] as u64;
            v1.wrapping_mul(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);

// ishape.extadd_pairwise_ishape_sx
define_instruction!(
    fd_fuel_check,
    i16x8_extadd_pairwise_i8x16_s,
    opcode::fd_extensions::I16X8_EXTADD_PAIRWISE_I8X16_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let added_pairwise: [i16; 8] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as i16;
            let v2 = lanes[2 * i + 1] as i16;
            v1.wrapping_add(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extadd_pairwise_i8x16_u,
    opcode::fd_extensions::I16X8_EXTADD_PAIRWISE_I8X16_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let added_pairwise: [u16; 8] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as u16;
            let v2 = lanes[2 * i + 1] as u16;
            v1.wrapping_add(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extadd_pairwise_i16x8_s,
    opcode::fd_extensions::I32X4_EXTADD_PAIRWISE_I16X8_S,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let added_pairwise: [i32; 4] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as i32;
            let v2 = lanes[2 * i + 1] as i32;
            v1.wrapping_add(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extadd_pairwise_i16x8_u,
    opcode::fd_extensions::I32X4_EXTADD_PAIRWISE_I16X8_U,
    |Args { stack, .. }| {
        let data: [u8; 16] = stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let added_pairwise: [u32; 4] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as u32;
            let v2 = lanes[2 * i + 1] as u32;
            v1.wrapping_add(v2)
        });
        stack.push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
