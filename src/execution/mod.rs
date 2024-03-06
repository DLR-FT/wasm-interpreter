use crate::core::indices::{FuncIdx, TypeIdx};
use crate::core::reader::section_header::SectionHeader;
use crate::core::reader::section_header::SectionTy;
use crate::core::reader::span::Span;
use crate::core::reader::types::export::Export;
use crate::core::reader::types::import::{Import, ImportDesc};
use crate::core::reader::types::{FuncType, GlobalType, MemType, TableType, ValType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::sections::export::read_export_section;
use crate::execution::sections::function::read_function_section;
use crate::execution::sections::global::read_global_section;
use crate::execution::sections::import::read_import_section;
use crate::execution::sections::memory::read_memory_section;
use crate::execution::sections::r#type::read_type_section;
use crate::execution::sections::table::read_table_section;
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use crate::{Result, ValidationInfo};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::iter;

// TODO
mod sections;
pub(crate) mod unwrap_validated;

pub struct InstantiatedInstance<'a> {
    wasm: &'a [u8],
    types: Vec<FuncType>,
    imports: Vec<Import>,
    functions: Vec<TypeIdx>,
    tables: Vec<TableType>,
    memories: Vec<MemType>,
    globals: Vec<GlobalType>,
    exports: Vec<Export>,
    code_blocks: Vec<Span>,
}

pub fn instantiate(wasm: &[u8], _validation_info: ValidationInfo) -> Result<InstantiatedInstance> {
    let mut wasm = WasmReader::new(wasm);
    trace!("Starting instantiation of bytecode");

    // Skip magic value(4b) and version number(4b)
    wasm.skip(8)?;

    let mut header = None;
    read_next_header(&mut wasm, &mut header)?;

    macro_rules! handle_section {
        ($section_ty:pat, $then:expr) => {
            match &header {
                Some(SectionHeader {
                    ty: $section_ty, ..
                }) => {
                    let h = header.take().unwrap();
                    trace!("Handling section {:?}", h.ty);
                    let ret = $then(h);
                    read_next_header(&mut wasm, &mut header)?;
                    Some(ret)
                }
                _ => None,
            }
        };
    }
    macro_rules! skip_custom_sections {
        () => {
            let mut skip_section = || {
                handle_section!(SectionTy::Custom, |h: SectionHeader| {
                    wasm.skip(h.contents.len())
                })
                .transpose()
            };

            while let Some(_) = skip_section()? {}
        };
    }

    skip_custom_sections!();

    let types =
        handle_section!(SectionTy::Type, |h| { read_type_section(&mut wasm) }).unwrap_or_default();

    skip_custom_sections!();

    let imports = handle_section!(SectionTy::Import, |_| { read_import_section(&mut wasm) })
        .unwrap_or_default();

    let mut current_funcidx = imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Func(_)))
        .count();
    let mut current_tableidx = imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Table(_)))
        .count();
    let mut current_memidx = imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Mem(_)))
        .count();
    let mut current_globalidx = imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Global(_)))
        .count();

    skip_custom_sections!();

    let functions = handle_section!(SectionTy::Function, |h| {
        read_function_section(&mut wasm)
    })
    .unwrap_or_default();
    // Now we have collected all available functions:
    // - some may be in `imports`
    // - and the ones in `functions`
    // Note that these two share their index space [FuncIdx].
    // Same for the following indices for tables, memories and globals
    current_funcidx += functions.len();

    skip_custom_sections!();

    let tables = handle_section!(SectionTy::Table, |_| { read_table_section(&mut wasm) })
        .unwrap_or_default();
    current_tableidx += tables.len();

    skip_custom_sections!();

    let memories = handle_section!(SectionTy::Memory, |_| { read_memory_section(&mut wasm) })
        .unwrap_or_default();
    current_memidx += memories.len();

    skip_custom_sections!();

    let globals = handle_section!(SectionTy::Global, |_| { read_global_section(&mut wasm) })
        .unwrap_or_default();
    current_globalidx += globals.len();

    skip_custom_sections!();

    let exports = handle_section!(SectionTy::Export, |h| { read_export_section(&mut wasm) })
        .unwrap_or_default();

    skip_custom_sections!();

    let _start = handle_section!(SectionTy::Start, |h| {
        wasm.read_var_u32().unwrap_validated() as FuncIdx
    });

    skip_custom_sections!();

    handle_section!(SectionTy::Element, |h: SectionHeader| {
        // TODO element
        wasm.skip(h.contents.len()).unwrap()
    });

    skip_custom_sections!();

    handle_section!(SectionTy::DataCount, |h: SectionHeader| {
        // data count is not necessary for execution
        wasm.skip(h.contents.len()).unwrap()
    });

    skip_custom_sections!();

    let code_blocks = handle_section!(SectionTy::Code, |h: SectionHeader| {
        wasm.read_vec(|wasm| {
            let size = wasm.read_var_u32()?;
            Ok(wasm.make_span(size as usize))
        })
    })
    .transpose()?
    .unwrap_or_default();

    skip_custom_sections!();

    handle_section!(SectionTy::Data, |h: SectionHeader| {
        // TODO data
        wasm.skip(h.contents.len()).unwrap()
    });

    skip_custom_sections!();

    // VALIDATION_ASSERT: No sections are left

    // TODO execute start function

    Ok(InstantiatedInstance {
        wasm: wasm.into_inner(),
        types,
        imports,
        functions,
        tables,
        memories,
        globals,
        exports,
        code_blocks,
    })
}

/// Can only invocate functions with signature `[i32] -> [i32]` as of now.
/// Also this panics if a trap is received.
pub fn invocate_fn(instantiation: &mut InstantiatedInstance, fn_idx: usize, param: i32) -> u32 {
    let fn_code_span = *instantiation.code_blocks.get(fn_idx).expect("valid fn_idx");

    let mut wasm = WasmReader::new(instantiation.wasm);
    wasm.move_to(fn_code_span);

    let _locals: Vec<ValType> = wasm
        .read_vec(|wasm| {
            let n = wasm.read_var_u32().unwrap_validated();
            let ty = ValType::read_unvalidated(wasm);
            Ok((n, ty))
        })
        .unwrap_validated()
        .into_iter()
        .flat_map(|(n, ty)| iter::repeat(ty).take(n as usize))
        .collect();

    todo!("setup value stack, push params, interpret code, return result")
}

fn read_next_header(wasm: &mut WasmReader, header: &mut Option<SectionHeader>) -> Result<()> {
    if header.is_none() && wasm.remaining_bytes().len() > 0 {
        *header = Some(SectionHeader::read_unvalidated(wasm));
    }
    Ok(())
}
