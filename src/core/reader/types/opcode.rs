#[repr(u8)]
pub enum Op1Byte {
    Nop = 0x01,
    End = 0x0B,
    LocalGet = 0x20,
    LocalSet = 0x21,
    GlobalGet = 0x23,
    GlobalSet = 0x24,
    I32Load = 0x28,
    I32Store = 0x36,
    I32Const = 0x41,
    I32Add = 0x6A,
    I32Mul = 0x6C,
    FCInstructions = 0xFC
}

impl Op1Byte {
    pub fn new(byte: u8) -> Self {
        match byte {
            0x01 => Self::Nop,
            0x0B => Self::End,
            0x20 => Self::LocalGet,
            0x21 => Self::LocalSet,
            0x23 => Self::GlobalGet,
            0x24 => Self::GlobalSet,
            0x28 => Self::I32Load,
            0x36 => Self::I32Store,
            0x41 => Self::I32Const,
            0x6A => Self::I32Add,
            0x6C => Self::I32Mul,
            0xFC => Self::FCInstructions,
            // TODO: replace this with an Error when all codes have been implemented
            _ => {
                unimplemented!("Unimplemented instruction for byte: {byte}")
            }
        }
    }
}

#[repr(u16)]
pub enum Op2Byte {
  I32TruncSatF32S = 0xFC00
}

impl Op2Byte {
  pub fn new(two_bytes: u16) -> Self {
    match two_bytes {
      0xFC00 => Op2Byte::I32TruncSatF32S,
      // TODO: replace this with an Error when all codes have been implemented
      _ => {
        let bytes = two_bytes.to_be_bytes();
        unimplemented!("Unimplemented instruction for bytes: 0x{:X} 0x{:X}", bytes[0], bytes[1])
      }
    }
  }
}