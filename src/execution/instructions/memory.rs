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
            calculate_mem_address, data_drop, define_instruction_fn, from_lanes, memory_init,
            to_lanes, Args, InterpreterLoopOutcome,
        },
    },
    trace, warn, RuntimeError, Value, F32, F64,
};

// t.load
define_instruction_fn! {
    i32_load,
    fuel_check = flat(instructions::I32_LOAD),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_i32() as u32 };

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

        resumable.stack.push_value(Value::I32(data))?;
        trace!("Instruction: i32.load [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_load,
    fuel_check = flat(instructions::I64_LOAD),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I64(data))?;
        trace!("Instruction: i64.load [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_load,
    fuel_check = flat(instructions::F32_LOAD),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::F32(data))?;
        trace!("Instruction: f32.load [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_load,
    fuel_check = flat(instructions::F64_LOAD),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::F64(data))?;
        trace!("Instruction: f64.load [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    v128_load,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data: u128 = memory.mem.load(idx)?;
        resumable.stack.push_value(data.to_le_bytes().into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// t.loadN_sx
define_instruction_fn! {
    i32_load8_s,
    fuel_check = flat(instructions::I32_LOAD8_S),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: i8 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I32(data as u32))?;
        trace!("Instruction: i32.load8_s [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_load8_u,
    fuel_check = flat(instructions::I32_LOAD8_U),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: u8 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I32(data as u32))?;
        trace!("Instruction: i32.load8_u [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_load16_s,
    fuel_check = flat(instructions::I32_LOAD16_S),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: i16 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I32(data as u32))?;
        trace!("Instruction: i32.load16_s [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_load16_u,
    fuel_check = flat(instructions::I32_LOAD16_U),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: u16 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I32(data as u32))?;
        trace!("Instruction: i32.load16_u [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_load8_s,
    fuel_check = flat(instructions::I64_LOAD8_S),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: i8 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I64(data as u64))?;
        trace!("Instruction: i64.load8_s [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_load8_u,
    fuel_check = flat(instructions::I64_LOAD8_U),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: u8 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I64(data as u64))?;
        trace!("Instruction: i64.load8_u [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_load16_s,
    fuel_check = flat(instructions::I64_LOAD16_S),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: i16 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I64(data as u64))?;
        trace!("Instruction: i64.load16_s [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_load16_u,
    fuel_check = flat(instructions::I64_LOAD16_U),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: u16 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I64(data as u64))?;
        trace!("Instruction: i64.load16_u [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_load32_s,
    fuel_check = flat(instructions::I64_LOAD32_S),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: i32 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I64(data as u64))?;
        trace!("Instruction: i64.load32_s [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_load32_u,
    fuel_check = flat(instructions::I64_LOAD32_U),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let data: u32 = mem.mem.load(idx)?;

        resumable.stack.push_value(Value::I64(data as u64))?;
        trace!("Instruction: i64.load32_u [{relative_address}] -> [{data}]");
        Ok(ControlFlow::Continue(()))
    }
}

// v128.loadNxM_sx
define_instruction_fn! {
    v128_load8x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD8X8_S),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i8; 8] = to_lanes::<1, 16, i8>(data)[..8].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i16);

        resumable
            .stack
            .push_value(Value::V128(from_lanes(extended_lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load8x8_u,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD8X8_U),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u8; 8] = to_lanes::<1, 16, u8>(data)[..8].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u16);

        resumable
            .stack
            .push_value(Value::V128(from_lanes(extended_lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load16x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD16X4_S),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i16; 4] = to_lanes::<2, 8, i16>(data)[..4].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i32);

        resumable
            .stack
            .push_value(Value::V128(from_lanes(extended_lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load16x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD16X4_U),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u16; 4] = to_lanes::<2, 8, u16>(data)[..4].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u32);

        resumable
            .stack
            .push_value(Value::V128(from_lanes(extended_lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load32x2_s,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD32X2_S),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [i32; 2] = to_lanes::<4, 4, i32>(data)[..2].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as i64);

        resumable
            .stack
            .push_value(Value::V128(from_lanes(extended_lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load32x2_u,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD32X2_U),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let half_data: [u8; 8] = memory.mem.load_bytes::<8>(idx)?; // v128 load always loads half of a v128

        // Special case where we have only half of a v128. To convert it to lanes via `to_lanes`, pad the data with zeros
        let data: [u8; 16] = array::from_fn(|i| *half_data.get(i).unwrap_or(&0));
        let half_lanes: [u32; 2] = to_lanes::<4, 4, u32>(data)[..2].try_into().unwrap();

        let extended_lanes = half_lanes.map(|lane| lane as u64);

        resumable
            .stack
            .push_value(Value::V128(from_lanes(extended_lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// v128.loadN_splat
define_instruction_fn! {
    v128_load8_splat,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD8_SPLAT),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<1, u8>(idx)?;
        resumable
            .stack
            .push_value(Value::V128(from_lanes([lane; 16])))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load16_splat,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD16_SPLAT),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<2, u16>(idx)?;
        resumable
            .stack
            .push_value(Value::V128(from_lanes([lane; 8])))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load32_splat,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD32_SPLAT),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<4, u32>(idx)?;
        resumable
            .stack
            .push_value(Value::V128(from_lanes([lane; 4])))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load64_splat,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD64_SPLAT),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let lane = memory.mem.load::<8, u64>(idx)?;
        resumable
            .stack
            .push_value(Value::V128(from_lanes([lane; 2])))?;
        Ok(ControlFlow::Continue(()))
    }
}

// v128.loadN_zero
define_instruction_fn! {
    v128_load32_zero,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD32_ZERO),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data = memory.mem.load::<4, u32>(idx)? as u128;
        resumable
            .stack
            .push_value(Value::V128(data.to_le_bytes()))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load64_zero,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD64_ZERO),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        let data = memory.mem.load::<8, u64>(idx)? as u128;
        resumable
            .stack
            .push_value(Value::V128(data.to_le_bytes()))?;
        Ok(ControlFlow::Continue(()))
    }
}

// v128.loadN_lane
define_instruction_fn! {
    v128_load8_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD8_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());
        let mut lanes: [u8; 16] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<1, u8>(idx)?;
        resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    v128_load16_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD16_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());
        let mut lanes: [u16; 8] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<2, u16>(idx)?;
        resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load32_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD32_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());
        let mut lanes: [u32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<4, u32>(idx)?;
        resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_load64_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_LOAD64_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());
        let mut lanes: [u64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = memory.mem.load::<8, u64>(idx)?;
        resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// t.store
define_instruction_fn! {
    i32_store,
    fuel_check = flat(instructions::I32_STORE),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: i32.store [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_store,
    fuel_check = flat(instructions::I64_STORE),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: u64 = unsafe { resumable.stack.pop_value().as_u64() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: i64.store [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_store,
    fuel_check = flat(instructions::F32_STORE),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: F32 = unsafe { resumable.stack.pop_value().as_f32() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: f32.store [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_store,
    fuel_check = flat(instructions::F64_STORE),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: F64 = unsafe { resumable.stack.pop_value().as_f64() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        mem.mem.store(idx, data_to_store)?;

        trace!("Instruction: f64.store [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    v128_store,
    fuel_check = flat_fc(instructions::fd_extensions::V128_STORE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };

        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let idx = calculate_mem_address(&memarg, relative_address)?;

        memory.mem.store(idx, u128::from_le_bytes(data))?;
        Ok(ControlFlow::Continue(()))
    }
}

// t.storeN
define_instruction_fn! {
    i32_store8,
    fuel_check = flat(instructions::I32_STORE8),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: i32 = unsafe { resumable.stack.pop_value().as_i32() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let mem = unsafe { store_inner.memories.get_mut(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i32.store8 [{relative_address} {wrapped_data}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_store16,
    fuel_check = flat(instructions::I32_STORE16),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: i32 = unsafe { resumable.stack.pop_value().as_i32() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let mem = unsafe { store_inner.memories.get_mut(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i32.store16 [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_store8,
    fuel_check = flat(instructions::I64_STORE8),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: i64 = unsafe { resumable.stack.pop_value().as_i64() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let mem = unsafe { store_inner.memories.get_mut(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i64.store8 [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_store16,
    fuel_check = flat(instructions::I64_STORE16),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: i64 = unsafe { resumable.stack.pop_value().as_i64() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let mem = unsafe { store_inner.memories.get_mut(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i64.store16 [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_store32,
    fuel_check = flat(instructions::I64_STORE32),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        let memarg = MemArg::decode_ptr(wasm);

        let data_to_store: i64 = unsafe { resumable.stack.pop_value().as_i64() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        let mem = unsafe { store_inner.memories.get_mut(mem_addr) };

        let idx = calculate_mem_address(&memarg, relative_address)?;
        mem.mem.store(idx, wrapped_data)?;

        trace!("Instruction: i64.store32 [{relative_address} {data_to_store}] -> []");
        Ok(ControlFlow::Continue(()))
    }
}

// v128.storeN_lane
define_instruction_fn! {
    v128_store8_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_STORE8_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());

        let lane = *to_lanes::<1, 16, u8>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<1, u8>(idx, lane)?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_store16_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_STORE16_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());

        let lane = *to_lanes::<2, 8, u16>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<2, u16>(idx, lane)?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_store32_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_STORE32_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());

        let lane = *to_lanes::<4, 4, u32>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<4, u32>(idx, lane)?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_store64_lane,
    fuel_check = flat_fc(instructions::fd_extensions::V128_STORE64_LANE),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        let data: [u8; 16] = unsafe { resumable.stack.pop_value().as_vec() };
        let relative_address: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let memarg = MemArg::decode_ptr(wasm);
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
        let memory = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let lane_idx = usize::from(wasm.decode_u8());

        let lane = *to_lanes::<8, 2, u64>(data).get(lane_idx).unwrap_validated();

        memory.mem.store::<8, u64>(idx, lane)?;
        Ok(ControlFlow::Continue(()))
    }
}

// memory.size
define_instruction_fn! {
    memory_size,
    fuel_check = flat(instructions::MEMORY_SIZE),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        // Note: This zero byte is reserved for the multiple memories
        // proposal.
        let _zero = wasm.decode_u8();
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
        let size = mem.size() as u32;
        resumable.stack.push_value(Value::I32(size))?;
        trace!("Instruction: memory.size [] -> [{}]", size);
        Ok(ControlFlow::Continue(()))
    }
}

// memory.grow
define_instruction_fn! {
    memory_grow,
    fuel_check = omit,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        // Note: This zero byte is reserved for the multiple memories
        // proposal.
        let _zero = wasm.decode_u8();
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

        let n: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        // decrement fuel, but push n back if it fails
        let cost = T::get_flat_cost(instructions::MEMORY_GROW)
            + u64::from(n) * T::get_cost_per_element(instructions::MEMORY_GROW);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable.stack.push_value(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.

                return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
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
            Err(
                RuntimeError::MemoryGrowOverflowed | RuntimeError::MemoryGrowExceededLimit,
            ) => u32::MAX,
            Err(_) => unreachable!("growing memory cannot return any other errors"),
        };
        resumable.stack.push_value(Value::I32(pushed_value))?;
        trace!("Instruction: memory.grow [{}] -> [{}]", n, pushed_value);
        Ok(ControlFlow::Continue(()))
    }
}

// memory.fill
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill
define_instruction_fn! {
    memory_fill,
    fuel_check = omit,
    |Args {
         resumable,
         wasm,
         store_inner,
         modules,
         current_module,
         ..
     }| {
        //  mappings:
        //      n => number of bytes to update
        //      val => the value to set each byte to (must be < 256)
        //      d => the pointer to the region to update

        // Note: This zero byte is reserved for the multiple
        // memories proposal.
        let _zero = wasm.decode_u8();

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
        let mem = unsafe { store_inner.memories.get_mut(mem_addr) };

        let n: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::MEMORY_FILL)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(instructions::fc_extensions::MEMORY_FILL);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable.stack.push_value(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let val: i32 = unsafe { resumable.stack.pop_value().as_i32() };

        if !(0..=255).contains(&val) {
            warn!("Value for memory.fill does not fit in a byte ({val})");
        }

        let d: i32 = unsafe { resumable.stack.pop_value().as_i32() };

        mem.mem
            .fill(d.cast_unsigned().into_usize(), val as u8, n.into_usize())?;

        trace!("Instruction: memory.fill");
        Ok(ControlFlow::Continue(()))
    }
}

// memory.copy
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy
define_instruction_fn! {
    memory_copy,
    fuel_check = omit,
    |Args {
         resumable,
         wasm,
         store_inner,
         modules,
         current_module,
         ..
     }| {
        //  mappings:
        //      n => number of bytes to copy
        //      s => source address to copy from
        //      d => destination address to copy to
        // Note: These zero bytes are reserved for the multiple
        // memories proposal.
        let _zero = wasm.decode_u8();
        let _zero = wasm.decode_u8();

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

        let n: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::MEMORY_COPY)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(instructions::fc_extensions::MEMORY_COPY);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable.stack.push_value(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        assert_eq!(src_addr, dst_addr, "the multiple memories proposal is not yet supported");
        let src_dst_addr = src_addr;

        let s: i32 = unsafe { resumable.stack.pop_value().as_i32() };
        let d: i32 = unsafe { resumable.stack.pop_value().as_i32() };

        // SAFETY: The source and destination addresses (which must be the same as of now!) were
        // just read from the current store. Therefore, it must also be valid in the current store.
        let src_dst_memory = unsafe { store_inner.memories.get_mut(src_dst_addr) };

        src_dst_memory.mem.copy_within(d.cast_unsigned().into_usize(), s.cast_unsigned().into_usize(), n.into_usize())?;

        trace!("Instruction: memory.copy");
        Ok(ControlFlow::Continue(()))
    }
}

// memory.init
// See https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x
// Copy a region from a data segment into memory
define_instruction_fn! {
    memory_init_fn,
    fuel_check = omit,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        //  mappings:
        //      n => number of bytes to copy
        //      s =            }> starting pointer in the data segment
        //      d => destination address to copy to
        // SAFETY: Validation guarantees there to be a valid
        // data index next.
        let data_idx = unsafe { DataIdx::decode_unchecked_ptr(wasm) };

        // Note: This zero byte is reserved for the multiple memories
        // proposal.
        let _zero = wasm.decode_u8();

        let n: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        // decrement fuel, but push n back if it fails
        let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::MEMORY_INIT)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(instructions::fc_extensions::MEMORY_INIT);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable.stack.push_value(Value::I32(n)).unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: u32 = unsafe { resumable.stack.pop_value().as_u32() };
        let d: u32 = unsafe { resumable.stack.pop_value().as_u32() };

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
        Ok(ControlFlow::Continue(()))
    }
}

// data.drop
define_instruction_fn! {
    data_drop_fn,
    fuel_check = flat_fc(instructions::fc_extensions::DATA_DROP),
    |Args {
         wasm,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // data index next.
        let data_idx = unsafe { DataIdx::decode_unchecked_ptr(wasm) };
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
        Ok(ControlFlow::Continue(()))
    }
}
