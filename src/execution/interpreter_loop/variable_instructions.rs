use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{GlobalIdx, LocalIdx},
        reader::types::opcode,
    },
    execution::interpreter_loop::{define_instruction, Args},
};

define_instruction!(
    local_get,
    opcode::LOCAL_GET,
    |Args { wasm, .. }: &mut Args<T>| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::read_unchecked(&mut *wasm.get_reader()) };
        let value = *wasm.resumable.stack.get_local(local_idx);
        wasm.resumable.stack.push_value::<T>(value)?;
        trace!("Instruction: local.get {} [] -> [t]", local_idx);
        Ok(None)
    }
);

define_instruction!(
    local_set,
    opcode::LOCAL_SET,
    |Args { wasm, .. }: &mut Args<T>| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::read_unchecked(&mut *wasm.get_reader()) };
        let value = wasm.resumable.stack.pop_value();
        *wasm.resumable.stack.get_local_mut(local_idx) = value;
        trace!("Instruction: local.set {} [t] -> []", local_idx);
        Ok(None)
    }
);

define_instruction!(
    local_tee,
    opcode::LOCAL_TEE,
    |Args { wasm, .. }: &mut Args<T>| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::read_unchecked(&mut *wasm.get_reader()) };
        let value = wasm.resumable.stack.peek_value().unwrap_validated();
        *wasm.resumable.stack.get_local_mut(local_idx) = value;
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

         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::read_unchecked(&mut *wasm.get_reader()) };
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

        wasm.resumable.stack.push_value::<T>(global.value)?;

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

         wasm,
         current_module,
         ..
     }: &mut Args<T>| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::read_unchecked(&mut *wasm.get_reader()) };
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

        global.value = wasm.resumable.stack.pop_value();
        trace!("Instruction: GLOBAL_SET");
        Ok(None)
    }
);
