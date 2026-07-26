use core::{hint::unreachable_unchecked, ops::ControlFlow};

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
        let test_val: i32 = unsafe { resumable.stack.pop_value().as_i32() };
        let val2 = resumable.stack.pop_value();
        let val1 = resumable.stack.pop_value();
        match (val1, val2) {
            (crate::Value::I32(_), crate::Value::I32(_)) => {},
            (crate::Value::I64(_), crate::Value::I64(_)) => {},
            (crate::Value::F32(_), crate::Value::F32(_)) => {}
            (crate::Value::F64(_), crate::Value::F64(_)) => {},
            (crate::Value::V128(_), crate::Value::V128(_)) => {},
            (crate::Value::Ref(_), crate::Value::Ref(_)) => {},
            _ => unsafe { unreachable_unchecked() },
        }
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
        let _type_vec = wasm.decode_vec(|x| x.decode_u8());
        let test_val: i32 = unsafe { resumable.stack.pop_value().as_i32() };
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
