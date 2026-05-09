use crate::{
    assert_validated::UnwrapValidatedExt,
    core::reader::types::opcode,
    execution::interpreter_loop::{define_instruction, Args},
    ValType,
};

define_instruction!(drop, opcode::DROP, |Args { wasm, .. }: &mut Args<T>| {
    wasm.resumable.stack.pop_value();
    trace!("Instruction: DROP");
    Ok(None)
});

define_instruction!(select, opcode::SELECT, |Args { wasm, .. }: &mut Args<T>| {
    let test_val: i32 = wasm
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let val2 = wasm.resumable.stack.pop_value();
    let val1 = wasm.resumable.stack.pop_value();
    if test_val != 0 {
        wasm.resumable.stack.push_value::<T>(val1)?;
    } else {
        wasm.resumable.stack.push_value::<T>(val2)?;
    }
    trace!("Instruction: SELECT");
    Ok(None)
});

define_instruction!(
    select_t,
    opcode::SELECT_T,
    |Args { wasm, .. }: &mut Args<T>| {
        let _type_vec = wasm.get_reader().read_vec(ValType::read).unwrap_validated();
        let test_val: i32 = wasm
            .resumable
            .stack
            .pop_value()
            .try_into()
            .unwrap_validated();
        let val2 = wasm.resumable.stack.pop_value();
        let val1 = wasm.resumable.stack.pop_value();
        if test_val != 0 {
            wasm.resumable.stack.push_value::<T>(val1)?;
        } else {
            wasm.resumable.stack.push_value::<T>(val2)?;
        }
        trace!("Instruction: SELECT_T");
        Ok(None)
    }
);
