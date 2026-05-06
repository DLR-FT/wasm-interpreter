use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{indices::FuncIdx, reader::types::opcode},
    execution::interpreter_loop::{define_instruction, Args},
    value::Ref,
    RefType, Value,
};

define_instruction!(
    ref_null,
    opcode::REF_NULL,
    |Args {
         wasm, resumable, ..
     }| {
        let reftype = RefType::read(wasm).unwrap_validated();

        resumable
            .stack
            .push_value::<T>(Value::Ref(Ref::Null(reftype)))?;
        trace!("Instruction: ref.null '{:?}' -> [{:?}]", reftype, reftype);
        Ok(None)
    }
);

define_instruction!(
    ref_is_null,
    opcode::REF_IS_NULL,
    |Args { resumable, .. }| {
        let rref: Ref = resumable.stack.pop_value().try_into().unwrap_validated();
        let is_null = matches!(rref, Ref::Null(_));

        let res = if is_null { 1 } else { 0 };
        trace!("Instruction: ref.is_null [{}] -> [{}]", rref, res);
        resumable.stack.push_value::<T>(Value::I32(res))?;
        Ok(None)
    }
);

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-ref-mathsf-ref-func-x
define_instruction!(
    ref_func,
    opcode::REF_FUNC,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees a valid function index to be
        // next.
        let func_idx = unsafe { FuncIdx::read_unchecked(wasm) };

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
            .push_value::<T>(Value::Ref(Ref::Func(*func_addr)))?;
        Ok(None)
    }
);
