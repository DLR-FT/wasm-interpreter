use crate::{
    assert_validated::UnwrapValidatedExt,
    core::reader::types::opcode,
    execution::interpreter_loop::{define_instruction, Args},
    ValType,
};

define_instruction!(drop, opcode::DROP, |Args { resumable, .. }| {
    resumable.stack.pop_value();
    trace!("Instruction: DROP");
    Ok(None)
});

define_instruction!(select, opcode::SELECT, |Args { resumable, .. }| {
    let test_val: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
    let val2 = resumable.stack.pop_value();
    let val1 = resumable.stack.pop_value();
    if test_val != 0 {
        resumable.stack.push_value::<T>(val1)?;
    } else {
        resumable.stack.push_value::<T>(val2)?;
    }
    trace!("Instruction: SELECT");
    Ok(None)
});

define_instruction!(
    select_t,
    opcode::SELECT_T,
    |Args {
         resumable, wasm, ..
     }| {
        let _type_vec = wasm.read_vec(ValType::read).unwrap_validated();
        let test_val: i32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let val2 = resumable.stack.pop_value();
        let val1 = resumable.stack.pop_value();
        if test_val != 0 {
            resumable.stack.push_value::<T>(val1)?;
        } else {
            resumable.stack.push_value::<T>(val2)?;
        }
        trace!("Instruction: SELECT_T");
        Ok(None)
    }
);
