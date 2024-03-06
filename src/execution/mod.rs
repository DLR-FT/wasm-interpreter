use alloc::vec;
use alloc::vec::Vec;

use crate::{Result, unreachable_validated, ValidationInfo};
use crate::core::indices::{FuncIdx, LocalIdx, TypeIdx};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::reader::section_header::SectionHeader;
use crate::core::reader::section_header::SectionTy;
use crate::core::reader::span::Span;
use crate::core::reader::types::{FuncType, GlobalType, MemType, TableType};
use crate::core::reader::types::export::Export;
use crate::core::reader::types::import::{Import, ImportDesc};
use crate::execution::sections::export::read_export_section;
use crate::execution::sections::function::read_function_section;
use crate::execution::sections::global::read_global_section;
use crate::execution::sections::import::read_import_section;
use crate::execution::sections::memory::read_memory_section;
use crate::execution::sections::r#type::read_type_section;
use crate::execution::sections::table::read_table_section;
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use crate::execution::value::Value;
use crate::validation::sections::read_declared_locals;

// TODO
mod sections;
pub(crate) mod unwrap_validated;
pub mod value;

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

/// Can only invocate functions with signature `[t1] -> [t2]` as of now.
/// Also this panics if a trap is received.
pub fn invocate_fn(
    instantiation: &mut InstantiatedInstance,
    fn_idx: usize,
    mut param: Value,
) -> Value {
    let fn_code_span = *instantiation.code_blocks.get(fn_idx).expect("valid fn_idx");

    let func_ty = instantiation
        .types
        .get(*instantiation.functions.get(fn_idx).expect("valid fn_idx"))
        .unwrap();

    let mut wasm = WasmReader::new(instantiation.wasm);
    wasm.move_to(fn_code_span);

    let params = &func_ty.params.valtypes;
    assert_eq!(
        params.len(),
        1,
        "just one parameter is allowed at this time"
    );
    let declared_locals = read_declared_locals(&mut wasm).unwrap_validated();

    let mut locals: Vec<Value> = vec![Value::Uninitialized; declared_locals.len()];

    fn get_local_mut<'a>(
        param: &'a mut Value,
        locals: &'a mut [Value],
        idx: LocalIdx,
    ) -> &'a mut Value {
        if idx == 0 {
            param
        } else {
            &mut locals[idx - 1]
        }
    }

    let mut value_stack: Vec<Value> = Vec::new();

    loop {
        match wasm.read_u8().unwrap_validated() {
            // end
            0x0B => {
                break;
            }
            // local.get: [] -> [t]
            0x20 => {
                let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                let local_ty = *get_local_mut(&mut param, &mut locals, local_idx);
                value_stack.push(local_ty);
            }
            // local.set [t] -> []
            0x21 => {
                let local_idx = wasm.read_var_u32().unwrap_validated() as LocalIdx;
                let local_ty = get_local_mut(&mut param, &mut locals, local_idx);
                *local_ty = value_stack.pop().unwrap_validated();
            }
            // i32.add: [i32 i32] -> [i32]
            0x6A => {
                let Some(Value::I32(v1)) = value_stack.pop() else {
                    unreachable_validated!();
                };
                let Some(Value::I32(v2)) = value_stack.pop() else {
                    unreachable_validated!();
                };

                let v1 = i32::from_le_bytes(v1.to_le_bytes());
                let v2 = i32::from_le_bytes(v2.to_le_bytes());

                let res = v1 + v2;
                let res = u32::from_le_bytes(res.to_le_bytes());
                value_stack.push(Value::I32(res));
            }
            // i32.const: [] -> [i32]
            0x41 => {
                let constant = wasm.read_var_i32().unwrap_validated();
                let constant = u32::from_le_bytes(constant.to_le_bytes());

                value_stack.push(Value::I32(constant));
            }
            _ => {}
        }
    }

    assert_eq!(
        value_stack.len(),
        1,
        "just one return value is allowed at this time"
    );
    value_stack.pop().unwrap_validated()
}

fn read_next_header(wasm: &mut WasmReader, header: &mut Option<SectionHeader>) -> Result<()> {
    if header.is_none() && wasm.remaining_bytes().len() > 0 {
        *header = Some(SectionHeader::read_unvalidated(wasm));
    }
    Ok(())
}
