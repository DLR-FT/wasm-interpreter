use core::{fmt, ops::ControlFlow};

use crate::{
    core::{
        decoding::decoder::WasmDecoder,
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
            memory_instances::{shared_linear_memory::Ordering, MemInst},
            module_instances::ModuleInst,
            store::StoreInner,
        },
    },
    AddrVec, ModuleAddr, RuntimeError, TrapError, Value, WasmResumable,
};

define_instruction!(
    super::atomic_fence,
    atomic_fence_mod,
    fuel_check = flat_fe(ATOMIC_FENCE)
);
#[inline(always)]
pub unsafe fn atomic_fence(_: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    unreachable!("fences not yet implemented")
}

define_instruction!(
    super::memory_atomic_notify,
    memory_atomic_notify_mod,
    fuel_check = flat_fe(MEMORY_ATOMIC_NOTIFY)
);
#[inline(always)]
pub unsafe fn memory_atomic_notify(
    _: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    unreachable!("wait and notify instructions not yet implemented")
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
    unreachable!("wait and notify instructions not yet implemented")
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
    unreachable!("wait and notify instructions not yet implemented")
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

define_instruction_fn! {
    i32_atomic_store8,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_STORE8),
    |Args { wasm, store_inner, modules, current_module, resumable, ..}| {
        let memarg = MemArg::decode(wasm).unwrap_validated();

        let data_to_store: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as u8;

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

        let idx = calculate_mem_address(&memarg, relative_address)?;
        match mem {
            MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.store(idx, wrapped_data, Ord::SeqCst)?,
            MemInst::Unshared(unshared_mem_inst) => {
                unshared_mem_inst.mem.store(idx, wrapped_data)?
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_atomic_store16,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_STORE16),
    |Args { wasm, store_inner, modules, current_module, resumable, ..}| {
        let memarg = MemArg::decode(wasm).unwrap_validated();

        let data_to_store: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as u16;

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

        let idx = calculate_mem_address(&memarg, relative_address)?;
        match mem {
            MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.store(idx, wrapped_data, Ord::SeqCst)?,
            MemInst::Unshared(unshared_mem_inst) => {
                unshared_mem_inst.mem.store(idx, wrapped_data)?
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_atomic_store8,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_STORE8),
    |Args { wasm, store_inner, modules, current_module, resumable, ..}| {
        let memarg = MemArg::decode(wasm).unwrap_validated();

        let data_to_store: u64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as u8;

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

        let idx = calculate_mem_address(&memarg, relative_address)?;
        match mem {
            MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.store(idx, wrapped_data, Ord::SeqCst)?,
            MemInst::Unshared(unshared_mem_inst) => {
                unshared_mem_inst.mem.store(idx, wrapped_data)?
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_atomic_store16,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_STORE16),
    |Args { wasm, store_inner, modules, current_module, resumable, ..}| {
        let memarg = MemArg::decode(wasm).unwrap_validated();

        let data_to_store: u64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as u16;

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

        let idx = calculate_mem_address(&memarg, relative_address)?;
        match mem {
            MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.store(idx, wrapped_data, Ord::SeqCst)?,
            MemInst::Unshared(unshared_mem_inst) => {
                unshared_mem_inst.mem.store(idx, wrapped_data)?
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_atomic_store32,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_STORE32),
    |Args { wasm, store_inner, modules, current_module, resumable, ..}| {
        let memarg = MemArg::decode(wasm).unwrap_validated();

        let data_to_store: u64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

        let wrapped_data = data_to_store as u32;

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

        let idx = calculate_mem_address(&memarg, relative_address)?;
        match mem {
            MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.store(idx, wrapped_data, Ord::SeqCst)?,
            MemInst::Unshared(unshared_mem_inst) => {
                unshared_mem_inst.mem.store(idx, wrapped_data)?
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

#[inline(always)]
fn t_atomic_rmw_atop<const T_N: usize, const STORAGE_T_N: usize, T, StorageT, E, AtopFn, WrapFn>(
    wasm: &mut WasmDecoder,
    store_inner: &mut StoreInner,
    modules: &AddrVec<ModuleAddr, ModuleInst>,
    current_module: ModuleAddr,
    resumable: &mut WasmResumable,

    atop: AtopFn,
    wrap: WrapFn,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError>
where
    AtopFn: Fn(T, T) -> T,
    T: LittleEndianBytes<T_N> + TryFrom<Value, Error = E> + Into<Value>,
    StorageT: LittleEndianBytes<STORAGE_T_N> + Copy,
    E: fmt::Debug,

    T: From<StorageT> + Copy,
    WrapFn: Fn(T) -> StorageT,
{
    let memarg = MemArg::decode(wasm).unwrap_validated();

    // 3. Assert: Due to validation, a value of type t is on top of the stack.
    // 4.
    let c_2: T = resumable.stack.pop_value().try_into().unwrap_validated();

    // 5. Assert: Due to validation, a value of value type i32 is on top of the stack.
    // 6.
    let i: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

    // 7.
    let ea: usize = calculate_mem_address(&memarg, i)?;

    // 8.
    if ea % (STORAGE_T_N / 8) != 0 {
        return Err(TrapError::UnalignedAtomicAccess.into());
    }

    // SAFETY: The current module address must come from the current store, because it is the only
    // parameter to this function that can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { modules.get(current_module) };

    // 11.
    // 12.
    // SAFETY: Validation guarantees at least one memory to exist.
    let a = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };

    // 13.
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { store_inner.memories.get_mut(a) };

    let c_1 = match mem {
        MemInst::Unshared(unshared_mem_inst) => {
            // a. If ea + N/8 is larger than the length of mem.data...
            // Check is performed by `LinearMemory::load_bytes`

            // Rest
            let b_r: [u8; _] = unshared_mem_inst.mem.load_bytes(ea)?;
            let c_r: StorageT = StorageT::from_le_bytes(b_r);
            let c_1: T = T::from(c_r);
            let c: T = atop(c_1, c_2);
            let c_w: StorageT = wrap(c);
            let b_w: [u8; _] = c_w.to_le_bytes();
            unshared_mem_inst.mem.store_bytes(ea, b_w)?;
            c_1
        }
        MemInst::Shared(shared_mem_inst) => {
            // Rest
            let n = shared_mem_inst.mem.rd_len_action();

            // b. If ea + N/8 is larger than n...
            // Check is performed by `LinearMemory::load_bytes`

            // c.
            let c_r = unsafe {
                shared_mem_inst
                    .mem
                    .rmw_data_action::<_, StorageT>(n, |c_r| {
                        let c_1 = T::from(c_r);
                        let c = atop(c_1, c_2);
                        let c_w: StorageT = wrap(c);
                        c_w
                    })
            };

            // We can convert c_r to c_1 a second time to keep the rmw closure pure
            T::from(c_r)
        }
    };

    resumable.stack.push_value(c_1.into())?;

    Ok(ControlFlow::Continue(()))
}

define_instruction_fn! {
    i32_atomic_rmw_add,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW_ADD),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u32::wrapping_add,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw_add,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW_ADD),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u64, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_add,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw8_add_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW8_ADD_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u32::wrapping_add,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw16_add_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW16_ADD_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u32::wrapping_add,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw8_add_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW8_ADD_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_add,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw16_add_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW16_ADD_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_add,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw32_add_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW32_ADD_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_add,
            |a| a as u32,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw_sub,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW_SUB),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u32::wrapping_sub,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw_sub,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW_SUB),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u64, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_sub,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw8_sub_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW8_SUB_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u32::wrapping_sub,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw16_sub_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW16_SUB_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u32::wrapping_sub,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw8_sub_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW8_SUB_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_sub,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw16_sub_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW16_SUB_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_sub,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw32_sub_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW32_SUB_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            u64::wrapping_sub,
            |a| a as u32,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw_and,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW_AND),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitAnd>::bitand,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw_and,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW_AND),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u64, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitAnd>::bitand,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw8_and_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW8_AND_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitAnd>::bitand,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw16_and_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW16_AND_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitAnd>::bitand,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw8_and_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW8_AND_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitAnd>::bitand,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw16_and_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW16_AND_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitAnd>::bitand,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw32_and_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW32_AND_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitAnd>::bitand,
            |a| a as u32,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw_or,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW_OR),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitOr>::bitor,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw_or,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW_OR),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u64, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitOr>::bitor,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw8_or_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW8_OR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitOr>::bitor,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw16_or_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW16_OR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitOr>::bitor,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw8_or_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW8_OR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitOr>::bitor,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw16_or_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW16_OR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitOr>::bitor,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw32_or_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW32_OR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitOr>::bitor,
            |a| a as u32,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw_xor,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW_XOR),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitXor>::bitxor,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw_xor,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW_XOR),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u64, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitXor>::bitxor,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw8_xor_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW8_XOR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitXor>::bitxor,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw16_xor_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW16_XOR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u32 as BitXor>::bitxor,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw8_xor_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW8_XOR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitXor>::bitxor,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw16_xor_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW16_XOR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitXor>::bitxor,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw32_xor_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW32_XOR_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            <u64 as BitXor>::bitxor,
            |a| a as u32,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw_xchg,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW_XCHG),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |_c_1, c_2| c_2,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw_xchg,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW_XCHG),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u64, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |_c_1, c_2| c_2,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw8_xchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW8_XCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |_c_1, c_2| c_2,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw16_xchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW16_XCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u32, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |_c_1, c_2| c_2,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw8_xchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW8_XCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u8, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |_c_1, c_2| c_2,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw16_xchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW16_XCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u16, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |_c_1, c_2| c_2,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw32_xchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW32_XCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_atop::<_, _, u64, u32, _, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |_c_1, c_2| c_2,
            |a| a as u32,
        )
    }
}

#[inline(always)]
fn t_atomic_rmw_cmpxchg<const T_N: usize, const STORAGE_T_N: usize, T, StorageT, E, WrapFn>(
    wasm: &mut WasmDecoder,
    store_inner: &mut StoreInner,
    modules: &AddrVec<ModuleAddr, ModuleInst>,
    current_module: ModuleAddr,
    resumable: &mut WasmResumable,

    wrap: WrapFn,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError>
where
    T: LittleEndianBytes<T_N> + TryFrom<Value, Error = E> + Into<Value>,
    StorageT: LittleEndianBytes<STORAGE_T_N> + Copy + Eq,
    E: fmt::Debug,

    T: From<StorageT> + Copy,
    WrapFn: Fn(T) -> StorageT,
{
    let memarg = MemArg::decode(wasm).unwrap_validated();

    // 3. Assert: Due to validation, two values of type t are on top of the stack.
    // 4.
    let c_3: T = resumable.stack.pop_value().try_into().unwrap_validated();
    let c_2: T = resumable.stack.pop_value().try_into().unwrap_validated();

    // 6. Assert: Due to validation, a value of value type i32 is on top of the stack.
    // 7.
    let i: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

    // 8.
    let ea: usize = calculate_mem_address(&memarg, i)?;

    // 9.
    if ea % (STORAGE_T_N / 8) != 0 {
        return Err(TrapError::UnalignedAtomicAccess.into());
    }

    // SAFETY: The current module address must come from the current store, because it is the only
    // parameter to this function that can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { modules.get(current_module) };

    // 11.
    // 12.
    // SAFETY: Validation guarantees at least one memory to exist.
    let a = *unsafe { module.mem_addrs.get(MemIdx::new(0)) };

    // 13.
    // SAFETY: This memory address was just read from the current
    // store. Therefore, it is valid in the current store.
    let mem = unsafe { store_inner.memories.get_mut(a) };

    let c_1: T = match mem {
        MemInst::Unshared(unshared_mem_inst) => {
            // 14.

            // a. If ea + N/8 is larger than the length of mem.data...
            // Check is performed by `LinearMemory::load_bytes`

            // b.
            let b_r: [u8; _] = unshared_mem_inst.mem.load_bytes(ea)?;
            // 16.
            let c_r: StorageT = StorageT::from_le_bytes(b_r);
            // 17.
            let c_ex = wrap(c_2);
            // 18.
            if c_r == c_ex {
                let c_w = wrap(c_3);
                let b_w: [u8; _] = c_w.to_le_bytes();
                unshared_mem_inst.mem.store_bytes(ea, b_w)?;
            }

            // 20.
            c_r.into()
        }
        MemInst::Shared(shared_mem_inst) => {
            // 15.
            // a.
            let n = shared_mem_inst.mem.rd_len_action();

            // b. If ea + N/8 is larger than n...
            // Check is performed by `LinearMemory::load_bytes`

            // c.
            // We will execute steps 15..=19 in one action, to guarantee atomicity
            let c_r = unsafe {
                shared_mem_inst
                    .mem
                    .rmw_data_action::<_, StorageT>(n, |c_r| {
                        let c_ex = wrap(c_2);
                        if c_r == c_ex {
                            let c_w = wrap(c_3);
                            c_w
                        } else {
                            c_r
                        }
                    })
            };

            // We can convert c_r to c_1 a second time to keep the rmw closure pure
            T::from(c_r)
        }
    };

    resumable.stack.push_value(c_1.into())?;

    Ok(ControlFlow::Continue(()))
}

define_instruction_fn! {
    i32_atomic_rmw_cmpxchg,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW_CMPXCHG),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_cmpxchg::<_, _, u32, u32, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw_cmpxchg,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW_CMPXCHG),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_cmpxchg::<_, _, u64, u64, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            convert::identity,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw8_cmpxchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW8_CMPXCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_cmpxchg::<_, _, u32, u8, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i32_atomic_rmw16_cmpxchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I32_ATOMIC_RMW16_CMPXCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_cmpxchg::<_, _, u32, u16, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw8_cmpxchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW8_CMPXCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_cmpxchg::<_, _, u64, u8, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |a| a as u8,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw16_cmpxchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW16_CMPXCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_cmpxchg::<_, _, u64, u16, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |a| a as u16,
        )
    }
}

define_instruction_fn! {
    i64_atomic_rmw32_cmpxchg_u,
    fuel_check = flat_fe(instructions::fe_extensions::I64_ATOMIC_RMW32_CMPXCHG_U),
    |Args { wasm, store_inner, modules, current_module, resumable, .. }| {
        t_atomic_rmw_cmpxchg::<_, _, u64, u32, _, _>(
            wasm,
            store_inner,
            modules,
            *current_module,
            resumable,
            |a| a as u32,
        )
    }
}
