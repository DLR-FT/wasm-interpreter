#[cfg(debug_assertions)]
pub fn print_beautiful_instruction_name_1_byte(first_byte: u8, pc: usize) {
    use crate::core::reader::types::opcode::opcode_byte_to_str;

    trace!(
        "Read instruction {} at wasm_binary[{}]",
        opcode_byte_to_str(first_byte),
        pc
    );
}
