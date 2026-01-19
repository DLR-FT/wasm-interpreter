// TODO Reconsider the importance of this module. All of its functions do the same?

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

#[cfg(debug_assertions)]
pub fn print_beautiful_fd_extension(second_byte: u32, pc: usize) {
    use crate::core::reader::types::opcode::fd_extension_opcode_to_str;

    trace!(
        "Read instruction {} at wasm_binary[{}]",
        fd_extension_opcode_to_str(second_byte),
        pc,
    );
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
pub trait ToUsizeExt {
    fn into_usize(self) -> usize;
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl ToUsizeExt for u32 {
    fn into_usize(self) -> usize {
        // SAFETY: The current trait only exists for architectures where
        // pointers are at least 32 bits wide.
        unsafe { usize::try_from(self).unwrap_unchecked() }
    }
}

#[cfg(target_pointer_width = "16")]
compile_error!("targets with 16 bit wide pointers are currently not supported");
