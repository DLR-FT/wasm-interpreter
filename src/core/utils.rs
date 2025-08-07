#[cfg(debug_assertions)]
pub fn print_beautiful_instruction_name_1_byte(first_byte: u8, pc: usize) {
    use crate::core::reader::types::opcode::opcode_byte_to_str;

    trace!(
        "Read instruction {} at wasm_binary[{}]",
        opcode_byte_to_str(first_byte),
        pc
    );
}

#[cfg(debug_assertions)]
pub fn print_beautiful_fc_extension(second_byte: u32, pc: usize) {
    use crate::core::reader::types::opcode::fc_extension_opcode_to_str;

    trace!(
        "Read instruction {} at wasm_binary[{}]",
        fc_extension_opcode_to_str(second_byte),
        pc,
    );
}
