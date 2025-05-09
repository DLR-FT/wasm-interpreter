use core::fmt::{Debug, Formatter};

use alloc::{format, vec::Vec};

use crate::core::{
    indices::MemIdx,
    reader::{span::Span, types::opcode, WasmReadable},
};

use super::UnwrapValidatedExt;

#[derive(Clone)]
pub struct DataSegment {
    pub init: Vec<u8>,
    pub mode: DataMode,
}

#[derive(Clone)]
pub enum DataMode {
    Passive,
    Active(DataModeActive),
}

#[derive(Clone)]
pub struct DataModeActive {
    pub memory_idx: MemIdx,
    pub offset: Span,
}

impl WasmReadable for DataSegment {
    fn read(wasm: &mut crate::core::reader::WasmReader) -> crate::Result<Self> {
        let mode = wasm.read_var_u32()?;
        let data_sec: DataSegment = match mode {
            0 => {
                // active { memory 0, offset e }
                trace!("Data section: active");

                let from = wasm.pc;
                let len = loop {
                    let instr = wasm.read_u8()?;
                    if instr == opcode::END {
                        break wasm.pc;
                    }
                } - from;
                let offset = Span { from, len };

                let byte_vec = wasm.read_vec(|el| el.read_u8())?;

                DataSegment {
                    mode: DataMode::Active(DataModeActive {
                        memory_idx: 0,
                        offset,
                    }),
                    init: byte_vec,
                }
            }
            1 => {
                // passive
                // A passive data segment's contents can be copied into a memory using the `memory.init` instruction
                trace!("Data section: passive");
                DataSegment {
                    mode: DataMode::Passive,
                    init: wasm.read_vec(|el| el.read_u8())?,
                }
            }
            2 => {
                // mode active { memory x, offset e }
                // this hasn't been yet implemented in wasm
                // as per docs:

                // https://webassembly.github.io/spec/core/binary/modules.html#data-section
                // The initial integer can be interpreted as a bitfield. Bit 0 indicates a passive segment, bit 1 indicates the presence of an explicit memory index for an active segment.
                // In the current version of WebAssembly, at most one memory may be defined or imported in a single module, so all valid active data segments have a memory value of 0
                todo!("Data section: active - with multiple memories - NOT YET IMPLEMENTED!");
            }
            _ => unreachable!(),
        };

        trace!("{:?}", data_sec.init);
        Ok(data_sec)
    }

    fn read_unvalidated(wasm: &mut crate::core::reader::WasmReader) -> Self {
        let mode = wasm.read_var_u32().unwrap_validated();
        let data_sec: DataSegment = match mode {
            0 => {
                // active { memory 0, offset e }
                trace!("Data section: active");

                let from = wasm.pc;
                let len = loop {
                    let instr = wasm.read_u8().unwrap_validated();
                    if instr == opcode::END {
                        break wasm.pc;
                    }
                } - from;
                let offset = Span { from, len };

                let byte_vec = wasm
                    .read_vec(|el| Ok(el.read_u8().unwrap_validated()))
                    .unwrap_validated();

                // WARN: we currently don't take into consideration how we act when we are dealing with globals here
                DataSegment {
                    mode: DataMode::Active(DataModeActive {
                        memory_idx: 0,
                        offset,
                    }),
                    init: byte_vec,
                }
            }
            1 => {
                // passive
                // A passive data segment's contents can be copied into a memory using the `memory.init` instruction
                trace!("Data section: passive");
                DataSegment {
                    mode: DataMode::Passive,
                    init: wasm
                        .read_vec(|el| Ok(el.read_u8().unwrap_validated()))
                        .unwrap_validated(),
                }
            }
            2 => {
                // mode active { memory x, offset e }
                // this hasn't been yet implemented in wasm
                // as per docs:

                // https://webassembly.github.io/spec/core/binary/modules.html#data-section
                // The initial integer can be interpreted as a bitfield. Bit 0 indicates a passive segment, bit 1 indicates the presence of an explicit memory index for an active segment.
                // In the current version of WebAssembly, at most one memory may be defined or imported in a single module, so all valid active data segments have a memory value of 0
                todo!("Data section: active - with multiple memories - NOT YET IMPLEMENTED!");
            }
            _ => unreachable!(),
        };

        data_sec
    }
}

impl Debug for DataSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut init_str = alloc::string::String::new();

        let iter = self.init.iter().peekable();
        // only if it's valid do we print is as a normal utf-8 char, otherwise, hex
        for &byte in iter {
            if let Ok(valid_char) = alloc::string::String::from_utf8(Vec::from(&[byte])) {
                init_str.push_str(valid_char.as_str());
            } else {
                init_str.push_str(&format!("\\x{:02x}", byte));
            }
        }

        f.debug_struct("DataSegment")
            .field("init", &init_str)
            .field("mode", &self.mode)
            .finish()
    }
}

///
///  Usually, we'd have something like this:
/// ```wasm
/// (module
///  (memory 1) ;; memory starting with 1 page
///  (data (i32.const 0) "abc")  ;; writing the array of byte "abc" in the first memory (0) at offset 0
///                             ;; for hardcoded offsets, we'll usually use i32.const because of wasm being x86 arch
/// )
/// ```
///
/// Since the span has only the start and length and acts a reference, we print the start and end (both inclusive, notice the '..=')
/// We print it in both decimal and hexadecimal so it's easy to trace in something like <https://webassembly.github.io/wabt/demo/wat2wasm/>
impl Debug for DataMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DataMode::Passive => f.debug_struct("Passive").finish(),
            DataMode::Active(active_data_mode) => {
                let from = active_data_mode.offset.from;
                let to = active_data_mode.offset.from + active_data_mode.offset.len() - 1;
                f.debug_struct("Active")
                    // .field("offset", format_args!("[{}..={}]", from, to))
                    .field(
                        "offset",
                        &format_args!("[{}..={}] (hex = [{:X}..={:X}])", from, to, from, to),
                    )
                    .finish()
                // f.
            }
        }
    }
}

pub struct PassiveData {
    pub init: Vec<u8>,
}

impl Debug for PassiveData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut init_str = alloc::string::String::new();

        let iter = self.init.iter().peekable();
        for &byte in iter {
            if let Ok(valid_char) = alloc::string::String::from_utf8(Vec::from(&[byte])) {
                init_str.push_str(valid_char.as_str());
            } else {
                // If it's not valid UTF-8, print it as hex
                init_str.push_str(&format!("\\x{:02x}", byte));
            }
        }
        f.debug_struct("PassiveData")
            .field("init", &init_str)
            .finish()
    }
}
