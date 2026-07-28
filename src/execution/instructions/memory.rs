use core::{array, num::NonZeroU64, ops::ControlFlow};

use crate::{
    core::{
        structure::{
            instructions,
            modules::indices::{DataIdx, Idx, MemIdx},
            types::MemArg,
        },
        utils::ToUsizeExt,
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{
            calculate_mem_address, data_drop, define_instruction, from_lanes, memory_init,
            to_lanes, InterpreterLoopOutcome, State,
        },
    },
    trace, warn, Config, RuntimeError, Value, F32, F64,
};

// t.load
define_instruction!(super::i32_load, i32_load_mod, fuel_check = flat(I32_LOAD));
#[inline(always)]
pub unsafe fn i32_load(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem_inst = unsafe { state.store_inner.memories.get(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data = mem_inst.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I32(data))?;
    trace!("Instruction: i32.load [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_load, i64_load_mod, fuel_check = flat(I64_LOAD));
#[inline(always)]
pub unsafe fn i64_load(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I64(data))?;
    trace!("Instruction: i64.load [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_load, f32_load_mod, fuel_check = flat(F32_LOAD));
#[inline(always)]
pub unsafe fn f32_load(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::F32(data))?;
    trace!("Instruction: f32.load [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_load, f64_load_mod, fuel_check = flat(F64_LOAD));
#[inline(always)]
pub unsafe fn f64_load(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::F64(data))?;
    trace!("Instruction: f64.load [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::v128_load,
    v128_load_mod,
    fuel_check = flat_fd(V128_LOAD)
);
#[inline(always)]
pub unsafe fn v128_load(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let data: u128 = memory.mem.load(idx)?;
    state
        .resumable
        .stack
        .push_value(data.to_le_bytes().into())?;
    Ok(ControlFlow::Continue(()))
}

// t.loadN_sx
define_instruction!(
    super::i32_load8_s,
    i32_load8_s_mod,
    fuel_check = flat(I32_LOAD8_S)
);
#[inline(always)]
pub unsafe fn i32_load8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: i8 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I32(data as u32))?;
    trace!("Instruction: i32.load8_s [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_load8_u,
    i32_load8_u_mod,
    fuel_check = flat(I32_LOAD8_U)
);
#[inline(always)]
pub unsafe fn i32_load8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u8 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I32(data as u32))?;
    trace!("Instruction: i32.load8_u [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_load16_s,
    i32_load16_s_mod,
    fuel_check = flat(I32_LOAD16_S)
);
#[inline(always)]
pub unsafe fn i32_load16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: i16 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I32(data as u32))?;
    trace!("Instruction: i32.load16_s [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_load16_u,
    i32_load16_u_mod,
    fuel_check = flat(I32_LOAD16_U)
);
#[inline(always)]
pub unsafe fn i32_load16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u16 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I32(data as u32))?;
    trace!("Instruction: i32.load16_u [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_load8_s,
    i64_load8_s_mod,
    fuel_check = flat(I64_LOAD8_S)
);
#[inline(always)]
pub unsafe fn i64_load8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: i8 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I64(data as u64))?;
    trace!("Instruction: i64.load8_s [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_load8_u,
    i64_load8_u_mod,
    fuel_check = flat(I64_LOAD8_U)
);
#[inline(always)]
pub unsafe fn i64_load8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u8 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I64(data as u64))?;
    trace!("Instruction: i64.load8_u [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_load16_s,
    i64_load16_s_mod,
    fuel_check = flat(I64_LOAD16_S)
);
#[inline(always)]
pub unsafe fn i64_load16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: i16 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I64(data as u64))?;
    trace!("Instruction: i64.load16_s [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_load16_u,
    i64_load16_u_mod,
    fuel_check = flat(I64_LOAD16_U)
);
#[inline(always)]
pub unsafe fn i64_load16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u16 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I64(data as u64))?;
    trace!("Instruction: i64.load16_u [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_load32_s,
    i64_load32_s_mod,
    fuel_check = flat(I64_LOAD32_S)
);
#[inline(always)]
pub unsafe fn i64_load32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: i32 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I64(data as u64))?;
    trace!("Instruction: i64.load32_s [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_load32_u,
    i64_load32_u_mod,
    fuel_check = flat(I64_LOAD32_U)
);
#[inline(always)]
pub unsafe fn i64_load32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    let data: u32 = mem.mem.load(idx)?;

    state.resumable.stack.push_value(Value::I64(data as u64))?;
    trace!("Instruction: i64.load32_u [{relative_address}] -> [{data}]");
    Ok(ControlFlow::Continue(()))
}

// v128.loadNxM_sx
define_instruction!(
    super::v128_load8x8_s,
    v128_load8x8_s_mod,
    fuel_check = flat_fd(V128_LOAD8X8_S)
);
#[inline(always)]
pub unsafe fn v128_load8x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

    // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
    let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
    let half_lanes: [i8; 8] = to_lanes::<1, 16, i8>(data)[..8].try_into().unwrap();

    let extended_lanes = half_lanes.map(|lane| lane as i16);

    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(extended_lanes)))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load8x8_u,
    v128_load8x8_u_mod,
    fuel_check = flat_fd(V128_LOAD8X8_U)
);
#[inline(always)]
pub unsafe fn v128_load8x8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

    // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
    let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
    let half_lanes: [u8; 8] = to_lanes::<1, 16, u8>(data)[..8].try_into().unwrap();

    let extended_lanes = half_lanes.map(|lane| lane as u16);

    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(extended_lanes)))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load16x4_s,
    v128_load16x4_s_mod,
    fuel_check = flat_fd(V128_LOAD16X4_S)
);
#[inline(always)]
pub unsafe fn v128_load16x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

    // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
    let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
    let half_lanes: [i16; 4] = to_lanes::<2, 8, i16>(data)[..4].try_into().unwrap();

    let extended_lanes = half_lanes.map(|lane| lane as i32);

    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(extended_lanes)))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load16x4_u,
    v128_load16x4_u_mod,
    fuel_check = flat_fd(V128_LOAD16X4_U)
);
#[inline(always)]
pub unsafe fn v128_load16x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

    // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
    let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
    let half_lanes: [u16; 4] = to_lanes::<2, 8, u16>(data)[..4].try_into().unwrap();

    let extended_lanes = half_lanes.map(|lane| lane as u32);

    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(extended_lanes)))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load32x2_s,
    v128_load32x2_s_mod,
    fuel_check = flat_fd(V128_LOAD32X2_S)
);
#[inline(always)]
pub unsafe fn v128_load32x2_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

    // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
    let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
    let half_lanes: [i32; 2] = to_lanes::<4, 4, i32>(data)[..2].try_into().unwrap();

    let extended_lanes = half_lanes.map(|lane| lane as i64);

    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(extended_lanes)))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load32x2_u,
    v128_load32x2_u_mod,
    fuel_check = flat_fd(V128_LOAD32X2_U)
);
#[inline(always)]
pub unsafe fn v128_load32x2_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

    // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
    let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
    let half_lanes: [u32; 2] = to_lanes::<4, 4, u32>(data)[..2].try_into().unwrap();

    let extended_lanes = half_lanes.map(|lane| lane as u64);

    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(extended_lanes)))?;
    Ok(ControlFlow::Continue(()))
}

// v128.loadN_splat
define_instruction!(
    super::v128_load8_splat,
    v128_load8_splat_mod,
    fuel_check = flat_fd(V128_LOAD8_SPLAT)
);
#[inline(always)]
pub unsafe fn v128_load8_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let lane = memory.mem.load::<1, u8>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes([lane; 16])))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load16_splat,
    v128_load16_splat_mod,
    fuel_check = flat_fd(V128_LOAD16_SPLAT)
);
#[inline(always)]
pub unsafe fn v128_load16_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let lane = memory.mem.load::<2, u16>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes([lane; 8])))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load32_splat,
    v128_load32_splat_mod,
    fuel_check = flat_fd(V128_LOAD32_SPLAT)
);
#[inline(always)]
pub unsafe fn v128_load32_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let lane = memory.mem.load::<4, u32>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes([lane; 4])))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load64_splat,
    v128_load64_splat_mod,
    fuel_check = flat_fd(V128_LOAD64_SPLAT)
);
#[inline(always)]
pub unsafe fn v128_load64_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let lane = memory.mem.load::<8, u64>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes([lane; 2])))?;
    Ok(ControlFlow::Continue(()))
}

// v128.loadN_zero
define_instruction!(
    super::v128_load32_zero,
    v128_load32_zero_mod,
    fuel_check = flat_fd(V128_LOAD32_ZERO)
);
#[inline(always)]
pub unsafe fn v128_load32_zero(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let data = memory.mem.load::<4, u32>(idx)? as u128;
    state
        .resumable
        .stack
        .push_value(Value::V128(data.to_le_bytes()))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load64_zero,
    v128_load64_zero_mod,
    fuel_check = flat_fd(V128_LOAD64_ZERO)
);
#[inline(always)]
pub unsafe fn v128_load64_zero(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    let data = memory.mem.load::<8, u64>(idx)? as u128;
    state
        .resumable
        .stack
        .push_value(Value::V128(data.to_le_bytes()))?;
    Ok(ControlFlow::Continue(()))
}

// v128.loadN_lane
define_instruction!(
    super::v128_load8_lane,
    v128_load8_lane_mod,
    fuel_check = flat_fd(V128_LOAD8_LANE)
);
#[inline(always)]
pub unsafe fn v128_load8_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    let mut lanes: [u8; 16] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<1, u8>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::v128_load16_lane,
    v128_load16_lane_mod,
    fuel_check = flat_fd(V128_LOAD16_LANE)
);
#[inline(always)]
pub unsafe fn v128_load16_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    let mut lanes: [u16; 8] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<2, u16>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load32_lane,
    v128_load32_lane_mod,
    fuel_check = flat_fd(V128_LOAD32_LANE)
);
#[inline(always)]
pub unsafe fn v128_load32_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    let mut lanes: [u32; 4] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<4, u32>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_load64_lane,
    v128_load64_lane_mod,
    fuel_check = flat_fd(V128_LOAD64_LANE)
);
#[inline(always)]
pub unsafe fn v128_load64_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    let mut lanes: [u64; 2] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<8, u64>(idx)?;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

// t.store
define_instruction!(
    super::i32_store,
    i32_store_mod,
    fuel_check = flat(I32_STORE)
);
#[inline(always)]
pub unsafe fn i32_store(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, data_to_store)?;

    trace!("Instruction: i32.store [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_store,
    i64_store_mod,
    fuel_check = flat(I64_STORE)
);
#[inline(always)]
pub unsafe fn i64_store(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: u64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, data_to_store)?;

    trace!("Instruction: i64.store [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_store,
    f32_store_mod,
    fuel_check = flat(F32_STORE)
);
#[inline(always)]
pub unsafe fn f32_store(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, data_to_store)?;

    trace!("Instruction: f32.store [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_store,
    f64_store_mod,
    fuel_check = flat(F64_STORE)
);
#[inline(always)]
pub unsafe fn f64_store(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, data_to_store)?;

    trace!("Instruction: f64.store [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::v128_store,
    v128_store_mod,
    fuel_check = flat_fd(V128_STORE)
);
#[inline(always)]
pub unsafe fn v128_store(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let idx = calculate_mem_address(&memarg, relative_address)?;

    memory.mem.store(idx, u128::from_le_bytes(data))?;
    Ok(ControlFlow::Continue(()))
}

// t.storeN
define_instruction!(
    super::i32_store8,
    i32_store8_mod,
    fuel_check = flat(I32_STORE8)
);
#[inline(always)]
pub unsafe fn i32_store8(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let wrapped_data = data_to_store as i8;

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, wrapped_data)?;

    trace!("Instruction: i32.store8 [{relative_address} {wrapped_data}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_store16,
    i32_store16_mod,
    fuel_check = flat(I32_STORE16)
);
#[inline(always)]
pub unsafe fn i32_store16(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let wrapped_data = data_to_store as i16;

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, wrapped_data)?;

    trace!("Instruction: i32.store16 [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_store8,
    i64_store8_mod,
    fuel_check = flat(I64_STORE8)
);
#[inline(always)]
pub unsafe fn i64_store8(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let wrapped_data = data_to_store as i8;

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, wrapped_data)?;

    trace!("Instruction: i64.store8 [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_store16,
    i64_store16_mod,
    fuel_check = flat(I64_STORE16)
);
#[inline(always)]
pub unsafe fn i64_store16(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let wrapped_data = data_to_store as i16;

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, wrapped_data)?;

    trace!("Instruction: i64.store16 [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_store32,
    i64_store32_mod,
    fuel_check = flat(I64_STORE32)
);
#[inline(always)]
pub unsafe fn i64_store32(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let memarg = MemArg::decode(state.wasm).unwrap_validated();

    let data_to_store: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let wrapped_data = data_to_store as i32;

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let idx = calculate_mem_address(&memarg, relative_address)?;
    mem.mem.store(idx, wrapped_data)?;

    trace!("Instruction: i64.store32 [{relative_address} {data_to_store}] -> []");
    Ok(ControlFlow::Continue(()))
}

// v128.storeN_lane
define_instruction!(
    super::v128_store8_lane,
    v128_store8_lane_mod,
    fuel_check = flat_fd(V128_STORE8_LANE)
);
#[inline(always)]
pub unsafe fn v128_store8_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());

    let lane = *to_lanes::<1, 16, u8>(data).get(lane_idx).unwrap_validated();

    memory.mem.store::<1, u8>(idx, lane)?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_store16_lane,
    v128_store16_lane_mod,
    fuel_check = flat_fd(V128_STORE16_LANE)
);
#[inline(always)]
pub unsafe fn v128_store16_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());

    let lane = *to_lanes::<2, 8, u16>(data).get(lane_idx).unwrap_validated();

    memory.mem.store::<2, u16>(idx, lane)?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_store32_lane,
    v128_store32_lane_mod,
    fuel_check = flat_fd(V128_STORE32_LANE)
);
#[inline(always)]
pub unsafe fn v128_store32_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());

    let lane = *to_lanes::<4, 4, u32>(data).get(lane_idx).unwrap_validated();

    memory.mem.store::<4, u32>(idx, lane)?;
    Ok(ControlFlow::Continue(()))
}
define_instruction!(
    super::v128_store64_lane,
    v128_store64_lane_mod,
    fuel_check = flat_fd(V128_STORE64_LANE)
);
#[inline(always)]
pub unsafe fn v128_store64_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let data: [u8; 16] = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let relative_address: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let memarg = MemArg::decode(state.wasm).unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let memory = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let idx = calculate_mem_address(&memarg, relative_address)?;
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());

    let lane = *to_lanes::<8, 2, u64>(data).get(lane_idx).unwrap_validated();

    memory.mem.store::<8, u64>(idx, lane)?;
    Ok(ControlFlow::Continue(()))
}

// memory.size
define_instruction!(
    super::memory_size,
    memory_size_mod,
    fuel_check = flat(MEMORY_SIZE)
);
#[inline(always)]
pub unsafe fn memory_size(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // Note: This zero byte is reserved for the multiple memories
    // proposal.
    let _zero = state.wasm.decode_u8().unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };
    let size = mem.size() as u32;
    state.resumable.stack.push_value(Value::I32(size))?;
    trace!("Instruction: memory.size [] -> [{}]", size);
    Ok(ControlFlow::Continue(()))
}

// memory.grow
define_instruction!(super::memory_grow::<T>, memory_grow_mod, fuel_check = omit);
#[inline(always)]
pub unsafe fn memory_grow<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // Note: This zero byte is reserved for the multiple memories
    // proposal.
    let _zero = state.wasm.decode_u8().unwrap_validated();
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let sz: u32 = mem.size() as u32;

    let n: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    // decrement fuel, but push n back if it fails
    let cost = T::get_flat_cost(instructions::MEMORY_GROW)
        + u64::from(n) * T::get_cost_per_element(instructions::MEMORY_GROW);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(n))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.

            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
    // for now we execute it always according to the following match expr.
    // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the stack per spec
    let pushed_value = match mem.grow(n) {
        Ok(_) => sz,
        Err(RuntimeError::MemoryGrowOverflowed | RuntimeError::MemoryGrowExceededLimit) => u32::MAX,
        Err(_) => unreachable!("growing memory cannot return any other errors"),
    };
    state.resumable.stack.push_value(Value::I32(pushed_value))?;
    trace!("Instruction: memory.grow [{}] -> [{}]", n, pushed_value);
    Ok(ControlFlow::Continue(()))
}

// memory.fill
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill
define_instruction!(super::memory_fill::<T>, memory_fill_mod, fuel_check = omit);
#[inline(always)]
pub unsafe fn memory_fill<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    //  mappings:
    //      n => number of bytes to update
    //      val => the value to set each byte to (must be < 256)
    //      d => the pointer to the region to update

    // Note: This zero byte is reserved for the multiple
    // memories proposal.
    let _zero = state.wasm.decode_u8().unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to exist.
    let mem_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: This memory address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let mem = unsafe { state.store_inner.memories.get_mut(mem_addr) };

    let n: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    // decrement fuel, but push n back if it fails
    let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::MEMORY_FILL)
        + u64::from(n)
            * T::get_fc_extension_cost_per_element(instructions::fc_extensions::MEMORY_FILL);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(n))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    let val: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if !(0..=255).contains(&val) {
        warn!("Value for memory.fill does not fit in a byte ({val})");
    }

    let d: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    mem.mem
        .fill(d.cast_unsigned().into_usize(), val as u8, n.into_usize())?;

    trace!("Instruction: memory.fill");
    Ok(ControlFlow::Continue(()))
}

// memory.copy
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy
define_instruction!(super::memory_copy::<T>, memory_copy_mod, fuel_check = omit);
#[inline(always)]
pub unsafe fn memory_copy<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    //  mappings:
    //      n => number of bytes to copy
    //      s => source address to copy from
    //      d => destination address to copy to
    // Note: These zero bytes are reserved for the multiple
    // memories proposal.
    let _zero = state.wasm.decode_u8().unwrap_validated();
    let _zero = state.wasm.decode_u8().unwrap_validated();

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let src_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };
    // SAFETY: Validation guarantees at least one memory to
    // exist.
    let dst_addr = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };

    let n: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    // decrement fuel, but push n back if it fails
    let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::MEMORY_COPY)
        + u64::from(n)
            * T::get_fc_extension_cost_per_element(instructions::fc_extensions::MEMORY_COPY);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(n))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    assert_eq!(
        src_addr, dst_addr,
        "the multiple memories proposal is not yet supported"
    );
    let src_dst_addr = src_addr;

    let s: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let d: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // SAFETY: The source and destination addresses (which must be the same as of now!) were
    // just read from the current store. Therefore, it must also be valid in the current store.
    let src_dst_memory = unsafe { state.store_inner.memories.get_mut(src_dst_addr) };

    src_dst_memory.mem.copy_within(
        d.cast_unsigned().into_usize(),
        s.cast_unsigned().into_usize(),
        n.into_usize(),
    )?;

    trace!("Instruction: memory.copy");
    Ok(ControlFlow::Continue(()))
}

// memory.init
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x
// Copy a region from a data segment into memory
define_instruction!(
    super::memory_init_fn::<T>,
    memory_init_fn_mod,
    fuel_check = omit
);
#[inline(always)]
pub unsafe fn memory_init_fn<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    //  mappings:
    //      n => number of bytes to copy
    //      s =            }> starting pointer in the data segment
    //      d => destination address to copy to
    // SAFETY: Validation guarantees there to be a valid
    // data index next.
    let data_idx = unsafe { DataIdx::decode_unchecked(state.wasm) };

    // Note: This zero byte is reserved for the multiple memories
    // proposal.
    let _zero = state.wasm.decode_u8().unwrap_validated();

    let n: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    // decrement fuel, but push n back if it fails
    let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::MEMORY_INIT)
        + u64::from(n)
            * T::get_fc_extension_cost_per_element(instructions::fc_extensions::MEMORY_INIT);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(n))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    let s: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let d: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

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
            state.modules,
            &mut state.store_inner.memories,
            &state.store_inner.data,
            *state.current_module,
            data_idx,
            MemIdx::new(0),
            n,
            s,
            d,
        )?
    };
    Ok(ControlFlow::Continue(()))
}

// data.drop
define_instruction!(
    super::data_drop_fn,
    data_drop_fn_mod,
    fuel_check = flat_fc(DATA_DROP)
);
#[inline(always)]
pub unsafe fn data_drop_fn(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid
    // data index next.
    let data_idx = unsafe { DataIdx::decode_unchecked(state.wasm) };
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
    unsafe {
        data_drop(
            state.modules,
            &mut state.store_inner.data,
            *state.current_module,
            data_idx,
        )
    };
    Ok(ControlFlow::Continue(()))
}
