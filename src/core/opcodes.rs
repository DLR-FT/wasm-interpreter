pub const NOP: u8 = 0x01;
pub const END: u8 = 0x0B;
pub const LOCAL_GET: u8 = 0x20;
pub const LOCAL_SET: u8 = 0x21;
pub const GLOBAL_GET: u8 = 0x23;
pub const GLOBAL_SET: u8 = 0x24;
pub const I32_LOAD: u8 = 0x28;
pub const I32_STORE: u8 = 0x36;
pub const I32_CONST: u8 = 0x41;
pub const I32_ADD: u8 = 0x6A;
pub const I32_MUL: u8 = 0x6C;
pub const I32_DIV_S: u8 = 0x6D;
pub const I32_DIV_U: u8 = 0x6E;
pub const FB_INSTRUCTIONS: u8 = 0xFB;
pub const FC_INSTRUCTIONS: u8 = 0xFC;
pub const FD_INSTRUCTIONS: u8 = 0xFD;
pub const FE_INSTRUCTIONS: u8 = 0xFE;

pub mod fc_opcodes {
    pub const I32_TRUNC_SAT_F32S: u8 = 0x00;
}
