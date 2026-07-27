use core::ops::ControlFlow;

use crate::{
    core::structure::{
        instructions,
        modules::indices::{GlobalIdx, LocalIdx},
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction_fn, Args},
    },
    trace,
};

define_instruction_fn! {
    local_get,
    fuel_check = flat(instructions::LOCAL_GET),
    |args: Args| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::decode_unchecked(args.wasm) };
        let value = *args.resumable.stack.get_local(local_idx);
        args.resumable.stack.push_value(value)?;
        trace!("Instruction: local.get {} [] -> [t]", local_idx);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    local_set,
    fuel_check = flat(instructions::LOCAL_SET),
    |args: Args| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::decode_unchecked(args.wasm) };
        let value = args.resumable.stack.pop_value();
        *args.resumable.stack.get_local_mut(local_idx) = value;
        trace!("Instruction: local.set {} [t] -> []", local_idx);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    local_tee,
    fuel_check = flat(instructions::LOCAL_TEE),
    |args: Args| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::decode_unchecked(args.wasm) };
        let value = args.resumable.stack.peek_value().unwrap_validated();
        *args.resumable.stack.get_local_mut(local_idx) = value;
        trace!("Instruction: local.tee {} [t] -> [t]", local_idx);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    global_get,
    fuel_check = flat(instructions::GLOBAL_GET),
    |args: Args| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::decode_unchecked(args.wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { args.modules.get(*args.current_module) };

        // SAFETY: Validation guarantees the global index to be valid in
        // the current module.
        let global_addr = *unsafe { module.global_addrs.get(global_idx) };
        // SAFETY: This global address was just read from the current
        // store. Therefore, it is valid in the current store.
        let global = unsafe { args.store_inner.globals.get(global_addr) };

        args.resumable.stack.push_value(global.value)?;

        trace!(
            "Instruction: global.get '{}' [<GLOBAL>] -> [{:?}]",
            global_idx,
            global.value
        );
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    global_set,
    fuel_check = flat(instructions::GLOBAL_SET),
    |args: Args| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::decode_unchecked(args.wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { args.modules.get(*args.current_module) };
        // SAFETY: Validation guarantees the global index to be valid in
        // the current module.
        let global_addr = *unsafe { module.global_addrs.get(global_idx) };
        // SAFETY: This global address was just read from the current
        // store. Therefore, it is valid in the current store.
        let global = unsafe { args.store_inner.globals.get_mut(global_addr) };

        global.value = args.resumable.stack.pop_value();
        trace!("Instruction: GLOBAL_SET");
        Ok(ControlFlow::Continue(()))
    }
}
