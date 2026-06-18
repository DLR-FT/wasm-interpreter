use core::ops::ControlFlow;

use crate::{
    core::{
        decoding::reader::WasmDecoder,
        structure::{
            modules::indices::{Idx, MemIdx},
            types::MemArg,
        },
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{calculate_mem_address, define_instruction, InterpreterLoopOutcome, State},
        numerics::representations::LittleEndianBytes,
        runtime_structure::{
            memory_instances::{shared_linear_memory::Ord, MemInst},
            module_instances::ModuleInst,
            store::StoreInner,
        },
    },
    instructions, AddrVec, ModuleAddr, RuntimeError, Value, WasmResumable,
};

define_instruction!(
    super::memory_atomic_notify,
    memory_atomic_notify_mod,
    fuel_check = flat_fe(MEMORY_ATOMIC_NOTIFY)
);
#[inline(always)]
pub unsafe fn memory_atomic_notify(
    _: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    todo!()
}

define_instruction!(
    super::memory_atomic_wait32,
    memory_atomic_wait32_mod,
    fuel_check = flat_fe(MEMORY_ATOMIC_WAIT32)
);
#[inline(always)]
pub unsafe fn memory_atomic_wait32(
    _: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    todo!()
}

define_instruction!(
    super::memory_atomic_wait64,
    memory_atomic_wait64_mod,
    fuel_check = flat_fe(MEMORY_ATOMIC_WAIT64)
);
#[inline(always)]
pub unsafe fn memory_atomic_wait64(
    _: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    todo!()
}

#[inline(always)]
fn inn_atomic_load<const N: usize, T: LittleEndianBytes<N> + Into<Value>>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current store, because it is the
    // only parameter to this function that can contain module addresses. All stores guarantee
    // all addresses in them to be valid within themselves.
    let module = unsafe { modules.get(current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current store. Therefore, it is valid
    // in the current store.
    let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let data: T = match memory {
        MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
    };
    resumable.stack.push_value(data.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_atomic_load,
    i32_atomic_load_mod,
    fuel_check = flat_fe(I32_ATOMIC_LOAD)
);
#[inline(always)]
pub unsafe fn i32_atomic_load(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    inn_atomic_load::<_, u32>(state)
}

define_instruction!(
    super::i64_atomic_load,
    i64_atomic_load_mod,
    fuel_check = flat_fe(I64_ATOMIC_LOAD)
);
#[inline(always)]
pub unsafe fn i64_atomic_load(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    inn_atomic_load::<_, u64>(state)
}

define_instruction!(
    super::i32_atomic_load8_u,
    i32_atomic_load8_u_mod,
    fuel_check = flat_fe(I32_ATOMIC_LOAD8_U)
);
#[inline(always)]
pub unsafe fn i32_atomic_load8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current store, because it is the
    // only parameter to this function that can contain module addresses. All stores guarantee
    // all addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current store. Therefore, it is valid
    // in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u8 = match mem {
        MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
    };

    state
        .resumable
        .stack
        .push_value(Value::I32(u32::from(data)))?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_atomic_load16_u,
    i32_atomic_load16_u_mod,
    fuel_check = flat_fe(I32_ATOMIC_LOAD16_U)
);
#[inline(always)]
pub unsafe fn i32_atomic_load16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current store, because it is the
    // only parameter to this function that can contain module addresses. All stores guarantee
    // all addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current store. Therefore, it is valid
    // in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u16 = match mem {
        MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
    };

    state
        .resumable
        .stack
        .push_value(Value::I32(u32::from(data)))?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_atomic_load8_u,
    i64_atomic_load8_u_mod,
    fuel_check = flat_fe(I64_ATOMIC_LOAD8_U)
);
#[inline(always)]
pub unsafe fn i64_atomic_load8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current store, because it is the
    // only parameter to this function that can contain module addresses. All stores guarantee
    // all addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current store. Therefore, it is valid
    // in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u8 = match mem {
        MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
    };

    state
        .resumable
        .stack
        .push_value(Value::I64(u64::from(data)))?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_atomic_load16_u,
    i64_atomic_load16_u_mod,
    fuel_check = flat_fe(I64_ATOMIC_LOAD16_U)
);
#[inline(always)]
pub unsafe fn i64_atomic_load16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current store, because it is the
    // only parameter to this function that can contain module addresses. All stores guarantee
    // all addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current store. Therefore, it is valid
    // in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u16 = match mem {
        MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
    };

    state
        .resumable
        .stack
        .push_value(Value::I64(u64::from(data)))?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_atomic_load32_u,
    i64_atomic_load32_u_mod,
    fuel_check = flat_fe(I64_ATOMIC_LOAD32_U)
);
#[inline(always)]
pub unsafe fn i64_atomic_load32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current store, because it is the
    // only parameter to this function that can contain module addresses. All stores guarantee
    // all addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current store. Therefore, it is valid
    // in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u32 = match mem {
        MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
    };

    state
        .resumable
        .stack
        .push_value(Value::I64(u64::from(data)))?;
    Ok(ControlFlow::Continue(()))
}

pub const I32_ATOMIC_LOAD: u32 = 16;
pub const I64_ATOMIC_LOAD: u32 = 17;
pub const I32_ATOMIC_LOAD8_U: u32 = 18;
pub const I32_ATOMIC_LOAD16_U: u32 = 19;
pub const I64_ATOMIC_LOAD8_U: u32 = 20;
pub const I64_ATOMIC_LOAD16_U: u32 = 21;
pub const I64_ATOMIC_LOAD32_U: u32 = 22;

pub const I32_ATOMIC_STORE: u32 = 23;
pub const I64_ATOMIC_STORE: u32 = 24;
pub const I32_ATOMIC_STORE8: u32 = 25;
pub const I32_ATOMIC_STORE16: u32 = 26;
pub const I64_ATOMIC_STORE8: u32 = 27;
pub const I64_ATOMIC_STORE16: u32 = 28;
pub const I64_ATOMIC_STORE32: u32 = 29;
