use core::ops::ControlFlow;

use crate::{
    core::structure::{instructions, modules::indices::FuncIdx},
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction_fn, Args},
    },
    trace, Ref, RefType, Value,
};

define_instruction_fn! {
    ref_null,
    fuel_check = flat(instructions::REF_NULL),
    |Args {
         wasm, resumable, ..
     }| {
        let reftype = RefType::decode_ptr(wasm);

        resumable.stack.push_value(Value::Ref(Ref::Null(reftype)))?;
        trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    ref_is_null,
    fuel_check = flat(instructions::REF_IS_NULL),
    |Args { resumable, .. }| {
        let rref: Ref = unsafe { resumable.stack.pop_value().as_ref() };
        let is_null = matches!(rref, Ref::Null(_));

        let res = if is_null { 1 } else { 0 };
        trace!("Instruction: ref.is_null [{}] -> [{}]", rref, res);
        resumable.stack.push_value(Value::I32(res))?;
        Ok(ControlFlow::Continue(()))
    }
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
define_instruction_fn! {
    ref_func,
    fuel_check = flat(instructions::REF_FUNC),
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees a valid function index to be
        // next.
        let func_idx = unsafe { FuncIdx::decode_unchecked_ptr(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let current_module = unsafe { modules.get(*current_module) };
        // SAFETY: Validation guarantees the function index to be valid
        // in the current module.
        let func_addr = unsafe { current_module.func_addrs.get(func_idx) };
        resumable
            .stack
            .push_value(Value::Ref(Ref::Func(*func_addr)))?;
        Ok(ControlFlow::Continue(()))
    }
}
