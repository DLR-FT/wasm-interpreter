use core::ops::ControlFlow;

use crate::{
    core::structure::instructions,
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction_fn, Args},
    },
    trace, ValType,
};

define_instruction_fn! {
    drop,
    fuel_check = flat(instructions::DROP),
    |args: Args| {
        args.resumable.stack.pop_value();
        trace!("Instruction: DROP");

        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    select,
    fuel_check = flat(instructions::SELECT),
    |args: Args| {
        let test_val: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let val2 = args.resumable.stack.pop_value();
        let val1 = args.resumable.stack.pop_value();
        if test_val != 0 {
            args.resumable.stack.push_value(val1)?;
        } else {
            args.resumable.stack.push_value(val2)?;
        }
        trace!("Instruction: SELECT");
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    select_t,
    fuel_check = flat(instructions::SELECT_T),
    |args: Args| {
        let _type_vec = args.wasm.decode_vec(ValType::decode).unwrap_validated();
        let test_val: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let val2 = args.resumable.stack.pop_value();
        let val1 = args.resumable.stack.pop_value();
        if test_val != 0 {
            args.resumable.stack.push_value(val1)?;
        } else {
            args.resumable.stack.push_value(val2)?;
        }
        trace!("Instruction: SELECT_T");
        Ok(ControlFlow::Continue(()))
    }
}
