use core::ops::ControlFlow;

use crate::{
    assert_validated::UnwrapValidatedExt,
    core::structure::instructions,
    execution::interpreter_loop::{define_instruction_fn, Args},
    ValType,
};

define_instruction_fn! {
    drop,
    fuel_check = flat(instructions::DROP),
    |Args { resumable, .. }| {
        resumable.stack.pop_value();
        trace!("Instruction: DROP");

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    select,
    fuel_check = flat(instructions::SELECT),
    |Args { resumable, .. }| {
        let test_val: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let val2 = resumable.stack.pop_value();
        let val1 = resumable.stack.pop_value();
        if test_val != 0 {
            resumable.stack.push_value(val1)?;
        } else {
            resumable.stack.push_value(val2)?;
        }
        trace!("Instruction: SELECT");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    select_t,
    fuel_check = flat(instructions::SELECT_T),
    |Args {
         resumable, wasm, ..
     }| {
        let _type_vec = wasm.read_vec(ValType::read).unwrap_validated();
        let test_val: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let val2 = resumable.stack.pop_value();
        let val1 = resumable.stack.pop_value();
        if test_val != 0 {
            resumable.stack.push_value(val1)?;
        } else {
            resumable.stack.push_value(val2)?;
        }
        trace!("Instruction: SELECT_T");
        Ok(ControlFlow::Continue(()))
    }
}
