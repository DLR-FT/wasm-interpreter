use core::{hint::unreachable_unchecked, ops::ControlFlow};

use crate::{
    core::structure::{
        instructions,
        modules::indices::{GlobalIdx, LocalIdx},
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction_fn, Args},
    },
    trace, validation, Ref, RefType,
};

define_instruction_fn! {
    local_get,
    fuel_check = flat(instructions::LOCAL_GET),
    |Args {
         resumable, wasm, ..
     }| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::decode_unchecked_ptr(wasm) };
        let value = *resumable.stack.get_local(local_idx);
        resumable.stack.push_value(value)?;
        trace!("Instruction: local.get {} [] -> [t]", local_idx);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    local_set,
    fuel_check = flat(instructions::LOCAL_SET),
    |Args {
         resumable, wasm, current_function_end_marker, ..
     }| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        // let local_idx = unsafe { LocalIdx(wasm.decode_var_u32_branchless(*current_function_end_marker)) };
        let local_idx = unsafe { LocalIdx::decode_unchecked_ptr(wasm) };
        let value = resumable.stack.pop_value();
        let local = unsafe { resumable.stack.get_local_mut_unchecked(local_idx) };

        match local {
            crate::Value::I32(local) => {
                *local = unsafe { value.as_u32() };
            },
            crate::Value::I64(local) => {
                *local = unsafe { value.as_u64() };
            }
            crate::Value::F32(local) => {
                *local = unsafe { value.as_f32()};
            },
            crate::Value::F64(local) => {
                *local = unsafe { value.as_f64()};
            }
            crate::Value::V128(local) => {
                *local = unsafe { value.as_vec()};
            }
            crate::Value::Ref(local_ref @ Ref::Null(RefType::FuncRef) | local_ref @ Ref::Func(_)) => {
                *local_ref = match value {
                    crate::Value::I32(_) => unsafe { unreachable_unchecked() },
                    crate::Value::I64(_) => unsafe { unreachable_unchecked() },
                    crate::Value::F32(_) => unsafe { unreachable_unchecked() },
                    crate::Value::F64(_) => unsafe { unreachable_unchecked() },
                    crate::Value::V128(_) => unsafe { unreachable_unchecked() },
                    crate::Value::Ref(Ref::Null(RefType::FuncRef)) => Ref::Null(RefType::FuncRef),
                    crate::Value::Ref(Ref::Null(RefType::ExternRef)) => unsafe { unreachable_unchecked() },
                    crate::Value::Ref(Ref::Extern(_)) => unsafe { unreachable_unchecked() },
                    crate::Value::Ref(Ref::Func(f)) => Ref::Func(f),
                };
            },

            crate::Value::Ref(local_ref @ Ref::Null(RefType::ExternRef) | local_ref @ Ref::Extern(_)) => {
                *local_ref = match value {
                    crate::Value::I32(_) => unsafe { unreachable_unchecked() },
                    crate::Value::I64(_) => unsafe { unreachable_unchecked() },
                    crate::Value::F32(_) => unsafe { unreachable_unchecked() },
                    crate::Value::F64(_) => unsafe { unreachable_unchecked() },
                    crate::Value::V128(_) => unsafe { unreachable_unchecked() },
                    crate::Value::Ref(Ref::Null(RefType::FuncRef)) => unsafe { unreachable_unchecked() },
                    crate::Value::Ref(Ref::Null(RefType::ExternRef)) => Ref::Null(RefType::ExternRef),
                    crate::Value::Ref(Ref::Extern(e)) => Ref::Extern(e),
                    crate::Value::Ref(Ref::Func(_)) => unsafe { unreachable_unchecked() },
                };
            },
        }


        // match (*local, value) {
        //     (crate::Value::I32(_), crate::Value::I32(x)) => {
        //         *unsafe { local.as_u32_mut() } = x;
        //     },
        //     (crate::Value::I64(_), crate::Value::I64(x)) => {
        //         *unsafe { local.as_u64_mut() } = x;
        //     },
        //     (crate::Value::F32(_), crate::Value::F32(x)) => {
        //         *unsafe { local.as_f32_mut() } = x;
        //     },
        //     (crate::Value::F64(_), crate::Value::F64(x)) => {
        //         *unsafe { local.as_f64_mut() } = x;
        //     },
        //     (crate::Value::V128(_), crate::Value::V128(x)) => {
        //         *unsafe { local.as_vec_mut() } = x;
        //     },
        //     (crate::Value::Ref(Ref::Null(RefType::ExternRef)), crate::Value::Ref(x @ Ref::Extern(_))) => {
        //         *unsafe { local.as_ref_mut() } = x;
        //     },
        //     (crate::Value::Ref(Ref::Null(RefType::ExternRef)), crate::Value::Ref(Ref::Null(x @ RefType::ExternRef))) => {
        //         *unsafe { local.as_ref_null_mut() } = x;
        //     },
        //     (crate::Value::Ref(Ref::Null(RefType::FuncRef)), crate::Value::Ref(x @ Ref::Func(_))) => {
        //         *unsafe { local.as_ref_mut() } = x;
        //     },
        //     (crate::Value::Ref(Ref::Null(RefType::FuncRef)), crate::Value::Ref(Ref::Null(x @ RefType::FuncRef))) => {
        //         *unsafe { local.as_ref_null_mut() } = x;
        //     },

        //     (crate::Value::Ref(Ref::Extern(_)), crate::Value::Ref(Ref::Extern(x))) => {
        //         *unsafe { local.as_ref_extern_mut() } = x;
        //     },
        //     (crate::Value::Ref(Ref::Extern(_)), crate::Value::Ref(x @ Ref::Null(RefType::ExternRef))) => {
        //         *unsafe { local.as_ref_mut() } = x;
        //     },

        //     (crate::Value::Ref(Ref::Func(_)), crate::Value::Ref(Ref::Func(x))) => {
        //         *unsafe { local.as_ref_func_mut() } = x;
        //     },
        //     (crate::Value::Ref(Ref::Func(_)), crate::Value::Ref(x @ Ref::Null(RefType::FuncRef))) => {
        //         *unsafe { local.as_ref_mut() } = x;
        //     },

        //     _ => unsafe { unreachable_unchecked() },
        // }

        trace!("Instruction: local.set {} [t] -> []", local_idx);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    local_tee,
    fuel_check = flat(instructions::LOCAL_TEE),
    |Args {
         resumable, wasm, ..
     }| {
        // SAFETY: Validation guarantees there to be a valid local index
        // next.
        let local_idx = unsafe { LocalIdx::decode_unchecked_ptr(wasm) };
        let value = resumable.stack.peek_value().unwrap_validated();
        let local = resumable.stack.get_local_mut(local_idx);


        *local = value;
        trace!("Instruction: local.tee {} [t] -> [t]", local_idx);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    global_get,
    fuel_check = flat(instructions::GLOBAL_GET),
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::decode_unchecked_ptr(wasm) };
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

        resumable.stack.push_value(global.value)?;

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
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid global
        // index next.
        let global_idx = unsafe { GlobalIdx::decode_unchecked_ptr(wasm) };
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

        global.value = resumable.stack.pop_value();
        trace!("Instruction: GLOBAL_SET");
        Ok(ControlFlow::Continue(()))
    }
}
