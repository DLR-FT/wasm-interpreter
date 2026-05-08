use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{Idx, MemIdx},
        reader::types::{memarg::MemArg, opcode},
    },
    execution::{
        interpreter_loop::{calculate_mem_address, define_instruction, Args},
        store::linear_memory::Ord,
    },
    instances::MemInst,
    Value,
};

define_instruction!(
    fe_fuel_check,
    memory_atomic_notify,
    opcode::fe_extensions::MEMORY_ATOMIC_NOTIFY,
    |Args { .. }| { todo!() }
);

define_instruction!(
    fe_fuel_check,
    memory_atomic_wait32,
    opcode::fe_extensions::MEMORY_ATOMIC_WAIT32,
    |Args { .. }| { todo!() }
);

define_instruction!(
    fe_fuel_check,
    memory_atomic_wait64,
    opcode::fe_extensions::MEMORY_ATOMIC_WAIT64,
    |Args { .. }| { todo!() }
);

define_instruction!(
    fe_fuel_check,
    i32_atomic_load,
    opcode::fe_extensions::I32_ATOMIC_LOAD,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
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
        let mem_inst = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: u32 = match mem_inst {
            MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
            MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        };

        resumable.stack.push_value::<T>(Value::I32(data))?;
        trace!("Instruction: i32.load [{data}] -> [{data}]");
        Ok(None)
    }
);

define_instruction!(
    fe_fuel_check,
    i64_atomic_load,
    opcode::fe_extensions::I64_ATOMIC_LOAD,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
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
        let mem_inst = unsafe { store_inner.memories.get_mut(mem_addr) };
        let idx = calculate_mem_address(&memarg, relative_address)?;
        let data: u64 = match mem_inst {
            MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.mem.load(idx)?,
            MemInst::Shared(shared_mem_inst) => shared_mem_inst.mem.load(idx, Ord::SeqCst)?,
        };

        resumable.stack.push_value::<T>(Value::I64(data))?;
        trace!("Instruction: i32.load [{data}] -> [{data}]");
        Ok(None)
    }
);
