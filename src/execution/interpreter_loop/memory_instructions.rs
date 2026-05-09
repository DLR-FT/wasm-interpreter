use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{DataIdx, Idx, MemIdx},
        reader::types::{memarg::MemArg, opcode},
        utils::ToUsizeExt,
    },
    execution::interpreter_loop::{
        calculate_mem_address, data_drop, define_instruction, from_lanes, memory_init, to_lanes,
        Args, InterpreterLoopOutcome,
    },
    value::{F32, F64},
    Value,
};
use core::{array, num::NonZeroU64};

// t.load
define_instruction!(
    i32_load,
    opcode::I32_LOAD,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I32(data))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I64(data))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::F32(data))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::F64(data))?;
        trace!("Instruction: f64.load [{relative_address}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    v128_load,
    opcode::fd_extensions::V128_LOAD,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data: u128 = memory.mem.load(idx)?;
        resumable.stack.push_value::<T>(data.to_le_bytes().into())?;
        Ok(None)
    }
);

// t.loadN_sx
define_instruction!(
    i32_load8_s,
    opcode::I32_LOAD8_S,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I32(data as u32))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I32(data as u32))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I32(data as u32))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I32(data as u32))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I64(data as u64))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I64(data as u64))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I64(data as u64))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I64(data as u64))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I64(data as u64))?;
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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

        resumable.stack.push_value::<T>(Value::I64(data as u64))?;
        trace!("Instruction: i64.load32_u [{relative_address}] -> [{data}]");
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
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i8; 8] = to_lanes::<1, 16, i8>(data)[..8].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i16);

        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load8x8_u,
    opcode::fd_extensions::V128_LOAD8X8_U,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u8; 8] = to_lanes::<1, 16, u8>(data)[..8].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u16);

        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load16x4_s,
    opcode::fd_extensions::V128_LOAD16X4_S,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i16; 4] = to_lanes::<2, 8, i16>(data)[..4].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i32);

        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load16x4_u,
    opcode::fd_extensions::V128_LOAD16X4_U,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u16; 4] = to_lanes::<2, 8, u16>(data)[..4].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u32);

        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32x2_s,
    opcode::fd_extensions::V128_LOAD32X2_S,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i32; 2] = to_lanes::<4, 4, i32>(data)[..2].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i64);

        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32x2_u,
    opcode::fd_extensions::V128_LOAD32X2_U,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u32; 2] = to_lanes::<4, 4, u32>(data)[..2].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u64);

        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(extended_lanes)))?;
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
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<1, u8>(idx)?;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes([lane; 16])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load16_splat,
    opcode::fd_extensions::V128_LOAD16_SPLAT,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<2, u16>(idx)?;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes([lane; 8])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32_splat,
    opcode::fd_extensions::V128_LOAD32_SPLAT,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<4, u32>(idx)?;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes([lane; 4])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load64_splat,
    opcode::fd_extensions::V128_LOAD64_SPLAT,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<8, u64>(idx)?;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes([lane; 2])))?;
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
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data = memory.mem.load::<4, u32>(idx)? as u128;
        resumable
            .stack
            .push_value::<T>(Value::V128(data.to_le_bytes()))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load64_zero,
    opcode::fd_extensions::V128_LOAD64_ZERO,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data = memory.mem.load::<8, u64>(idx)? as u128;
        resumable
            .stack
            .push_value::<T>(Value::V128(data.to_le_bytes()))?;
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
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    v128_load16_lane,
    opcode::fd_extensions::V128_LOAD16_LANE,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load32_lane,
    opcode::fd_extensions::V128_LOAD32_LANE,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_load64_lane,
    opcode::fd_extensions::V128_LOAD64_LANE,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);

// t.store
define_instruction!(
    i32_store,
    opcode::I32_STORE,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: u64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: F32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: F64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
    fd_fuel_check,
    v128_store,
    opcode::fd_extensions::V128_STORE,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let idx = calculate_mem_address(&memarg, relative_address)?;

        memory.mem.store(idx, u128::from_le_bytes(data))?;
        Ok(None)
    }
);

// t.storeN
define_instruction!(
    i32_store8,
    opcode::I32_STORE8,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        let memarg = MemArg::read(wasm).unwrap_validated();

        let data_to_store: i64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

// v128.storeN_lane
define_instruction!(
    fd_fuel_check,
    v128_store8_lane,
    opcode::fd_extensions::V128_STORE8_LANE,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let relative_address: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
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

// memory.size
define_instruction!(
    memory_size,
    opcode::MEMORY_SIZE,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
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
        resumable.stack.push_value::<T>(Value::I32(size))?;
        trace!("Instruction: memory.size [] -> [{}]", size);
        Ok(None)
    }
);

// memory.grow
define_instruction!(
    no_fuel_check,
    memory_grow,
    opcode::MEMORY_GROW,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
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

        let n: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_flat_cost(opcode::MEMORY_GROW)
            + u64::from(n) * T::get_cost_per_element(opcode::MEMORY_GROW);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(n))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.

                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
        // for now we execute it always according to the following match expr.
        // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the resumable.stack per spec
        let pushed_value = match mem.grow(n) {
            Ok(_) => sz,
            Err(_) => u32::MAX,
        };
        resumable.stack.push_value::<T>(Value::I32(pushed_value))?;
        trace!("Instruction: memory.grow [{}] -> [{}]", n, pushed_value);
        Ok(None)
    }
);

// memory.fill
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill
define_instruction!(
    no_fuel_check,
    memory_fill,
    opcode::fc_extensions::MEMORY_FILL,
    |Args {
         resumable,
         wasm,
         store_inner,
         modules,
         current_module,
         ..
     }: &mut Args<T>| {
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

        let n: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::MEMORY_FILL)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::MEMORY_FILL);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(n))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let val: i32 = resumable.stack.pop_value().try_into().unwrap_validated();

        if !(0..=255).contains(&val) {
            warn!("Value for memory.fill does not fit in a byte ({val})");
        }

        let d: i32 = resumable.stack.pop_value().try_into().unwrap_validated();

        mem.mem
            .fill(d.cast_unsigned().into_usize(), val as u8, n.into_usize())?;

        trace!("Instruction: memory.fill");
        Ok(None)
    }
);

// memory.copy
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy
define_instruction!(
    no_fuel_check,
    memory_copy,
    opcode::fc_extensions::MEMORY_COPY,
    |Args {
         resumable,
         wasm,
         store_inner,
         modules,
         current_module,
         ..
     }: &mut Args<T>| {
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

        let n: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::MEMORY_COPY)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::MEMORY_COPY);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(n))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let d: i32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

// memory.init
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x
// Copy a region from a data segment into memory
define_instruction!(
    no_fuel_check,
    memory_init_fn,
    opcode::fc_extensions::MEMORY_INIT,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }: &mut Args<T>| {
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

        let n: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::MEMORY_INIT)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::MEMORY_INIT);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(n))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let d: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

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

// data.drop
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
     }: &mut Args<T>| {
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
