use crate::{
    core::{
        decoding::reader::WasmDecoder,
        structure::{
            modules::{
                data_segments::{DataMode, DataModeActive, DataSegment},
                indices::{FuncIdx, IdxVec, MemIdx, TypeIdx},
            },
            types::GlobalType,
        },
    },
    validation::{
        instructions::const_expr::decode_and_validate_constant_expression,
        validation_stack::ValidationStack,
    },
    MemType, ValidationError,
};

impl DataSegment {
    pub fn decode_and_validate(
        wasm: &mut WasmDecoder,
        imported_global_types: &[GlobalType],
        c_funcs: &IdxVec<FuncIdx, TypeIdx>,
        c_mems: &IdxVec<MemIdx, MemType>,
    ) -> Result<Self, ValidationError> {
        use crate::{NumType, ValType};
        let mode = wasm.decode_var_u32()?;
        let data_sec: DataSegment = match mode {
            0 => {
                // active { memory 0, offset e }
                trace!("Data section: active {{ memory 0, offset e }}");

                let _mem_idx = MemIdx::validate(0, c_mems)?;

                let mut valid_stack = ValidationStack::new();
                let (offset, _) = {
                    decode_and_validate_constant_expression(
                        wasm,
                        &mut valid_stack,
                        imported_global_types,
                        c_funcs,
                    )?
                };

                valid_stack.assert_val_types(&[ValType::NumType(NumType::I32)], true)?;

                let byte_vec = wasm.decode_vec(|el| el.decode_u8())?;

                // WARN: we currently don't take into consideration how we act when we are dealing with globals here
                DataSegment {
                    mode: DataMode::Active(DataModeActive {
                        memory_idx: MemIdx::validate(0, c_mems)?,
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
                    init: wasm.decode_vec(|el| el.decode_u8())?,
                }
            }
            2 => {
                trace!("Data section: active {{ memory x, offset e }}");
                let mem_idx = MemIdx::decode_and_validate(wasm, c_mems)?;

                let mut valid_stack = ValidationStack::new();
                let (offset, _) = {
                    decode_and_validate_constant_expression(
                        wasm,
                        &mut valid_stack,
                        imported_global_types,
                        c_funcs,
                    )?
                };

                valid_stack.assert_val_types(&[ValType::NumType(NumType::I32)], true)?;

                let byte_vec = wasm.decode_vec(|el| el.decode_u8())?;

                DataSegment {
                    mode: DataMode::Active(DataModeActive {
                        memory_idx: mem_idx,
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
            invalid_mode @ 3.. => {
                return Err(ValidationError::InvalidDataSegmentMode(invalid_mode))
            }
        };

        trace!("{:?}", data_sec.init);
        Ok(data_sec)
    }
}
