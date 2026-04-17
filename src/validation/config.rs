pub trait ValidationConfig {
    #[inline(always)]
    fn instruction_hook(&mut self, _bytecode: &[u8], _pc: usize) {}
}

impl ValidationConfig for () {}
