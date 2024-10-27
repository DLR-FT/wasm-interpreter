use alloc::vec::Vec;
use read_constant_expression::read_constant_instructions;

use crate::const_interpreter_loop::run_const;
use crate::core::indices::{FuncIdx, TypeIdx};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::span::Span;
use crate::core::reader::types::data::{DataMode, DataModeActive, DataSegment};
use crate::core::reader::types::export::Export;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::Import;
use crate::core::reader::types::{FuncType, MemType, TableType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::value_stack::Stack;
use crate::{Error, Limits, Result, Value};

pub(crate) mod code;
pub(crate) mod globals;
pub(crate) mod read_constant_expression;
pub(crate) mod validation_stack;

/// Information collected from validating a module.
/// This can be used to create a [crate::RuntimeInstance].
pub struct ValidationInfo<'bytecode> {
    pub(crate) wasm: &'bytecode [u8],
    pub(crate) types: Vec<FuncType>,
    #[allow(dead_code)]
    pub(crate) imports: Vec<Import>,
    pub(crate) functions: Vec<TypeIdx>,
    #[allow(dead_code)]
    pub(crate) tables: Vec<TableType>,
    pub(crate) memories: Vec<MemType>,
    pub(crate) globals: Vec<Global>,
    #[allow(dead_code)]
    pub(crate) exports: Vec<Export>,
    pub(crate) func_blocks: Vec<Span>,
    pub(crate) data: Vec<DataSegment>,
    /// The start function which is automatically executed during instantiation
    pub(crate) start: Option<FuncIdx>,
}

pub fn validate(wasm: &[u8]) -> Result<ValidationInfo> {
    let mut wasm = WasmReader::new(wasm);
    trace!("Starting validation of bytecode");

    trace!("Validating magic value");
    let [0x00, 0x61, 0x73, 0x6d] = wasm.strip_bytes::<4>()? else {
        return Err(Error::InvalidMagic);
    };

    trace!("Validating version number");
    let [0x01, 0x00, 0x00, 0x00] = wasm.strip_bytes::<4>()? else {
        return Err(Error::InvalidVersion);
    };
    debug!("Header ok");

    let mut header = None;
    read_next_header(&mut wasm, &mut header)?;

    let skip_section = |wasm: &mut WasmReader, section_header: &mut Option<SectionHeader>| {
        // trace!("Header length: {}", if section_header.is_some() {
        //     format!("{:#?}",section_header.as_ref().unwrap().ty)
        //     // section_header.as_ref().unwrap().contents.len()
        // } else {
        //     String::new()
        // });

        handle_section(wasm, section_header, SectionTy::Custom, |wasm, h| {
            // trace!("Hello!");
            /*
            customsec   ::= section_0(custom)
            custom      ::= name byte^*
             */

            // trace!("Custom section length: {}", h.contents.len());
            // let initial_pc = wasm.pc;
            // let remaining_bytes = wasm.remaining_bytes().len();

            // let name_length = wasm.read_var_u32().unwrap();
            // // final_skip
            // // // let name_span = Span::new(wasm.pc, name_length as usize);
            // let mut name_vec = Vec::with_capacity(name_length as usize);
            // (0..name_length as usize).for_each(|_| {
            //     name_vec.push(wasm.read_u8().unwrap());
            // });
            // let name = unsafe { String::from_utf8_unchecked(name_vec) };
            // match name.as_str() {
            //     "name" => {
            //         let id = wasm.read_u8().unwrap();
            //         let size = wasm.read_var_u32().unwrap();
            //         trace!("Name section id: {} - size: {}", id, size);
            //     }
            //     _ => {
            //         trace!("Custom Section \"{}\" not implemented! Skipping...", name);
            //         // wasm.move_start_to(Span::new(initial_pc + h.contents.len(), 0))?;
            //         // wasm.skip(h.contents.len() - (wasm.remaining_bytes().len() - remaining_bytes))?;
            //     }
            // }
            // // trace!("Found Custom Section: \"{}\"", name);

            // // wasm.skip(h.contents.len() - name_length as usize - 1)
            // wasm.skip(h.contents.len() - (wasm.remaining_bytes().len() - remaining_bytes))
            wasm.skip(h.contents.len())
        })
    };

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let types = handle_section(&mut wasm, &mut header, SectionTy::Type, |wasm, _| {
        wasm.read_vec(FuncType::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let imports = handle_section(&mut wasm, &mut header, SectionTy::Import, |wasm, _| {
        wasm.read_vec(Import::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let functions = handle_section(&mut wasm, &mut header, SectionTy::Function, |wasm, _| {
        wasm.read_vec(|wasm| wasm.read_var_u32().map(|u| u as usize))
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let tables = handle_section(&mut wasm, &mut header, SectionTy::Table, |wasm, _| {
        wasm.read_vec(TableType::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let memories = handle_section(&mut wasm, &mut header, SectionTy::Memory, |wasm, _| {
        wasm.read_vec(MemType::read)
    })?
    .unwrap_or_default();
    if memories.len() > 1 {
        return Err(Error::MoreThanOneMemory);
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let globals = handle_section(&mut wasm, &mut header, SectionTy::Global, |wasm, h| {
        globals::validate_global_section(wasm, h)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let exports = handle_section(&mut wasm, &mut header, SectionTy::Export, |wasm, _| {
        wasm.read_vec(Export::read)
    })?
    .unwrap_or_default();

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let start = handle_section(&mut wasm, &mut header, SectionTy::Start, |wasm, _| {
        wasm.read_var_u32().map(|idx| idx as FuncIdx)
    })?;

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    // #region element
    // let _: Option<()> = handle_section(&mut wasm, &mut header, SectionTy::Element, |wasm, _| {
    //     let mut elem_vec: Vec<()> = Vec::new();
    //     // https://webassembly.github.io/spec/core/binary/modules.html#element-section
    //      // TODO: replace with wasm.read_vec in the future
    //      let vec_length = wasm.read_var_u32().unwrap();
    //      trace!("Element sections no.: {}", vec_length);
    //      for i in 0..vec_length {
    //         let ttype = wasm.read_var_u32().unwrap();
    //         // https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
    //         // https://webassembly.github.io/spec/core/binary/modules.html#element-section
    //         // We can treat the ttype as a 3bit integer
    //         // If it's not 3 bits I am not sure what to do
    //         // bit 0 => diff between passive|declartive and active segment
    //         // bit 1 => presence of an explicit table index for an active segment
    //         // bit 2 => use of element type and element expressions instead of element kind and element indices
    //         if (ttype & 0b111) > 0b111 {
    //             // what should we do?
    //             // error or is fine?
    //             // should be unspecified
    //         }
    //         // decide if we should
    //         // let elem_mode = if ttype & 0b001 == 0b001 {
    //         //     // passive or declarative
    //         //     if ttype & 0b010 == 0b010 {
    //         //         ElemMode::Declarative
    //         //     } else {
    //         //         ElemMode::Passive
    //         //     }
    //         // } else {
    //         //     if ttype & 0b010 == 0b010 {
    //         //         let table_idx = wasm.read_var_u32().unwrap();
    //         //         let bytes = Vec::new();
    //         //         bytes.push(wasm.read_var_u32().unwrap());
    //         //         while bytes.last().unwrap() != END {
    //         //             bytes.push(wasm.read_var_u32().unwrap());
    //         //         }
    //         //         ElemMode::Active(ActiveElem {
    //         //             table: table_idx,
    //         //             offset: bytes
    //         //         })
    //         //     } else {
    //         //     }
    //         // }
    //         match ttype {
    //             0 => {
    //                 let expr = {
    //                     // TODO: actually verify this expression
    //                     let mut const_expr = read_constant_expression(wasm).unwrap();
    //                 };
    //                 let func_idxs: Vec<u32> = wasm.read_vec(|w| {
    //                     w.read_var_u32()
    //                 }).unwrap();
    //                 // type funcref
    //             }
    //             1 => {
    //                 // type elemkind
    //             }
    //             2 => {
    //                 // type elemkind
    //             }
    //             3 => {
    //                 // type elemkind
    //             }
    //             4 => {
    //                 // type funcref
    //             }
    //             5 => {
    //                 // type reftype
    //             }
    //             6 => {
    //                 // type reftype
    //             }
    //             7 => {
    //                 // type reftype
    //             }
    //             _ => unimplemented!()
    //         }
    //      }
    //     todo!("element section not yet supported")
    // })?;
    // #endregion

    let _element: Option<()> =
        handle_section(&mut wasm, &mut header, SectionTy::Element, |_, _| {
            todo!("element section not yet supported")
        })?;
    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    // As per the official documentation:
    // 
    // The data count section is used to simplify single-pass validation. Since the data section occurs after the code section, the `memory.init` and `data.drop` and instructions would not be able to check whether the data segment index is valid until the data section is read. The data count section occurs before the code section, so a single-pass validator can use this count instead of deferring validation.
    let data_count: Option<u32> =
        handle_section(&mut wasm, &mut header, SectionTy::DataCount, |wasm, _| {
            wasm.read_var_u32()
        })?;
    if data_count.is_some() {
        trace!("data count: {}", data_count.unwrap());
    }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let func_blocks = handle_section(&mut wasm, &mut header, SectionTy::Code, |wasm, h| {
        code::validate_code_section(
            wasm,
            h,
            &types,
            &functions,
            &globals,
            &memories,
            &data_count,
        )
    })?
    .unwrap_or_default();

    assert_eq!(func_blocks.len(), functions.len(), "these should be equal"); // TODO check if this is in the spec

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    let data_section = handle_section(&mut wasm, &mut header, SectionTy::Data, |wasm, _| {
        let mut data_vec: Vec<DataSegment> = Vec::new();

        wasm.read_vec(|wasm| {
            let mode = wasm.read_var_u32().unwrap();
            let data_sec: DataSegment = match mode {
                0 => {
                    // active { memory 0, offset e }
                    trace!("Data section: active");
                    let offset = { read_constant_instructions(wasm, None, None).unwrap() };

                    let byte_vec = wasm.read_vec(|el| Ok(el.read_u8().unwrap())).unwrap();

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
                        init: wasm.read_vec(|el| Ok(el.read_u8().unwrap())).unwrap(),
                    }
                }
                2 => {
                    // mode active { memory x, offset e }
                    // this hasn't been yet implemented in wasm
                    // as per docs:

                    // https://webassembly.github.io/spec/core/binary/modules.html#data-section
                    // The initial integer can be interpreted as a bitfield. Bit 0 indicates a passive segment, bit 1 indicates the presence of an explicit memory index for an active segment.
                    // In the current version of WebAssembly, at most one memory may be defined or imported in a single module, so all valid active data segments have a memory value of 0
                    unimplemented!();
                }
                _ => unreachable!(),
            };

            trace!("{:?}", data_sec.init);
            data_vec.push(data_sec);
            Ok(())
        })?;

        Ok(data_vec)
    })?
    .unwrap_or_default();

    // if data_count.is_some() {
    //     assert_eq!(data_count.unwrap() as usize, data_section.len());
    // }

    while (skip_section(&mut wasm, &mut header)?).is_some() {}

    // All sections should have been handled
    if let Some(header) = header {
        return Err(Error::SectionOutOfOrder(header.ty));
    }

    debug!("Validation was successful");
    Ok(ValidationInfo {
        wasm: wasm.into_inner(),
        types,
        imports,
        functions,
        tables,
        memories,
        globals,
        exports,
        func_blocks,
        data: data_section,
        start,
    })
}

fn read_next_header(wasm: &mut WasmReader, header: &mut Option<SectionHeader>) -> Result<()> {
    if header.is_none() && !wasm.remaining_bytes().is_empty() {
        *header = Some(SectionHeader::read(wasm)?);
    }
    if header.is_some() {
        trace!("Read next header: {:#?}", header.as_ref().unwrap().ty);
    } else {
        trace!("Couldn't read next header. Done!");
    }
    Ok(())
}

#[inline(always)]
fn handle_section<T, F: FnOnce(&mut WasmReader, SectionHeader) -> Result<T>>(
    wasm: &mut WasmReader,
    header: &mut Option<SectionHeader>,
    section_ty: SectionTy,
    handler: F,
) -> Result<Option<T>> {
    match &header {
        Some(SectionHeader { ty, .. }) if *ty == section_ty => {
            let h = header.take().unwrap();
            trace!("Handling section {:?}", h.ty);
            let ret = handler(wasm, h)?;
            read_next_header(wasm, header)?;
            Ok(Some(ret))
        }
        _ => Ok(None),
    }
}
