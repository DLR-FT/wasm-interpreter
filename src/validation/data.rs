use alloc::vec::Vec;

use crate::{
    core::{
        indices::MemIdx,
        reader::{
            section_header::{SectionHeader, SectionTy},
            types::{
                data::{DataMode, DataModeActive, DataSegment},
                global::GlobalType,
            },
            WasmReader,
        },
    },
    read_constant_expression::read_constant_expression,
    validation_stack::ValidationStack,
    ValidationError,
};

/// Validate the data section.
pub(super) fn validate_data_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
    imported_global_types: &[GlobalType],
    no_of_total_memories: usize,
    num_funcs: usize,
) -> Result<Vec<DataSegment>, ValidationError> {
    assert_eq!(section_header.ty, SectionTy::Data);

    wasm.read_vec(|wasm| {
        use crate::{NumType, ValType};
        let mode = wasm.read_var_u32()?;
        let data_sec: DataSegment = match mode {
            0 => {
                // active { memory 0, offset e }
                trace!("Data section: active {{ memory 0, offset e }}");

                if no_of_total_memories == 0 {
                    return Err(ValidationError::InvalidMemIdx(0));
                }

                let mut valid_stack = ValidationStack::new();
                let (offset, _) = {
                    read_constant_expression(
                        wasm,
                        &mut valid_stack,
                        imported_global_types,
                        num_funcs,
                    )?
                };

                valid_stack.assert_val_types(&[ValType::NumType(NumType::I32)], true)?;

                let byte_vec = wasm.read_vec(|el| el.read_u8())?;

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
                    init: wasm.read_vec(|el| el.read_u8())?,
                }
            }
            2 => {
                trace!("Data section: active {{ memory x, offset e }}");
                let mem_idx = wasm.read_var_u32()? as MemIdx;
                if mem_idx >= no_of_total_memories {
                    return Err(crate::ValidationError::InvalidMemIdx(mem_idx));
                }
                assert!(
                    mem_idx == 0,
                    "Memory index is not 0 - it's {mem_idx}! Multiple memories are NOT supported"
                );

                let mut valid_stack = ValidationStack::new();
                let (offset, _) = {
                    read_constant_expression(
                        wasm,
                        &mut valid_stack,
                        imported_global_types,
                        num_funcs,
                    )?
                };

                valid_stack.assert_val_types(&[ValType::NumType(NumType::I32)], true)?;

                let byte_vec = wasm.read_vec(|el| el.read_u8())?;

                DataSegment {
                    mode: DataMode::Active(DataModeActive {
                        memory_idx: 0,
                        offset,
                    }),
                    init: byte_vec,
                }
                // mode active { memory x, offset e }
                // this hasn't been yet implemented in wasm
                // as per docs:

                // https://webassembly.github.io/spec/core/binary/modules.html#data-section
                // The initial integer can be interpreted as a bitfield. Bit 0 indicates a passive segment, bit 1 indicates the presence of an explicit memory index for an active segment.
                // In the current version of WebAssembly, at most one memory may be defined or imported in a single module, so all valid active data segments have a memory value of 0
            }
            _ => unreachable!(),
        };

        trace!("{:?}", data_sec.init);
        Ok(data_sec)
    })
}
