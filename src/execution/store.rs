use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::iter;

use crate::core::error::{Proposal, Result, StoreInstantiationError};
use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::element::{ElemItems, ElemMode};
use crate::core::reader::types::export::ExportDesc;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::ImportDesc;
use crate::core::reader::types::{MemType, TableType, ValType};
use crate::core::reader::WasmReader;
use crate::core::sidetable::Sidetable;
use crate::execution::value::{Ref, Value};
use crate::execution::{get_address_offset, run_const, run_const_span, Stack};
use crate::value::{ExternAddr, FuncAddr};
use crate::{Error, RefType, ValidationInfo};

use super::execution_info::ExecutionInfo;
use super::UnwrapValidatedExt;

/// The store represents all global state that can be manipulated by WebAssembly programs. It
/// consists of the runtime representation of all instances of functions, tables, memories, and
/// globals, element segments, and data segments that have been allocated during the life time of
/// the abstract machine.
/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
#[derive(Default)]
pub struct Store<'b> {
    pub functions: Vec<FuncInst>,
    pub memories: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub data: Vec<DataInst>,
    pub tables: Vec<TableInst>,
    pub elements: Vec<ElemInst>,
    // pub exports: Vec<Export>,
    pub modules: Vec<ExecutionInfo<'b>>,
    pub module_names: BTreeMap<String, usize>,
}

impl<'b> Store<'b> {
    pub fn add_module(&mut self, name: String, module: ValidationInfo<'b>) -> Result<()> {
        // TODO: we can do validation at linktime such that if another module expects module `name` to export something,
        // and it doesn't, we can reject it here instead of accepting it and failing later.

        let function_inst = module.instantiate_functions()?;
        let mut table_inst = module.instantiate_tables()?;
        let (element_inst, passive_idxs) = module.instantiate_elements(&mut table_inst)?;
        let mut memories = module.instantiate_memories()?;
        let data = module.instantiate_data(&mut memories)?;
        let globals = module.instantiate_globals()?;

        let imported_functions = function_inst
            .iter()
            .filter(|func| matches!(func, FuncInst::Imported(_)))
            .count();
        let imported_memories = 0; // TODO: not yet supported
        let imported_globals = 0; // TODO: not yet supported
        let imported_tables = 0; // TODO: not yet supported

        let functions_offset = self.functions.len();
        let exec_functions = (functions_offset..(functions_offset + function_inst.len())).collect();
        self.functions.extend(function_inst);

        let memories_offset = self.memories.len();
        let exec_memories = (memories_offset..(memories_offset + memories.len())).collect();
        self.memories.extend(memories);

        let globals_offset = self.globals.len();
        let exec_globals = (globals_offset..(globals_offset + globals.len())).collect();
        self.globals.extend(globals);

        let data_offset = self.data.len();
        let exec_data = (data_offset..(data_offset + data.len())).collect();
        self.data.extend(data);

        let tables_offset = self.tables.len();
        let exec_tables = (tables_offset..(tables_offset + table_inst.len())).collect();
        self.tables.extend(table_inst);

        let elements_offset = self.elements.len();
        let exec_elements = (elements_offset..(elements_offset + element_inst.len())).collect();
        self.elements.extend(element_inst);

        let execution_info = ExecutionInfo {
            name: name.clone(),
            wasm_bytecode: module.wasm,
            wasm_reader: WasmReader::new(module.wasm),

            functions: exec_functions,
            functions_offset,
            imported_functions_len: imported_functions,

            memories: exec_memories,
            memories_offset,
            imported_memories_len: imported_memories,

            globals: exec_globals,
            globals_offset,
            imported_globals_len: imported_globals,

            tables: exec_tables,
            tables_offset,
            imported_tables_len: imported_tables,

            data: exec_data,
            data_offset,

            elements: exec_elements,
            elements_offset,

            passive_element_indexes: passive_idxs,
            exports: module.exports,
        };

        self.module_names.insert(name, self.modules.len());
        self.modules.push(execution_info);

        // TODO: At this point of the code, we can continue in two ways with imports/exports:
        // 1. Lazy import resolution: We do the lookup during the interprer loop either directly or via a lookup-table
        // 2. Active import resolution: We resolve the import dependency now, failing if there are unresolved imports.
        //    This limits the order in which modules need to be added.
        // 3. Delayed active import resolution: We resolve the whatever import dependencies we can, but imports which
        //    can not be resolved are left to wait for another module addition. If an import that should be satisfied by
        //    this module isn't, we can fail.

        // TODO: failing is harder since we already modified 'self'. We will circle back to this later.

        for module in &mut self.modules {
            for fn_store_idx in &mut module.functions {
                let func = &self.functions[*fn_store_idx];
                if let FuncInst::Imported(import) = func {
                    let resolved_idx =
                        self.lookup_function(&import.module_name, &import.function_name);

                    if resolved_idx.is_none() && import.module_name == name {
                        // TODO: Failed resolution... BAD!
                    } else {
                        *fn_store_idx = resolved_idx.unwrap();
                    }
                }
            }
        }

        Ok(())
    }

    pub fn lookup_function(&self, target_module: &str, target_function: &str) -> Option<usize> {
        let mut module_name: &str = target_module;
        let mut function_name: &str = target_function;
        let mut import_path: Vec<(String, String)> = vec![];

        for _ in 0..100 {
            import_path.push((module_name.to_string(), function_name.to_string()));
            let module_idx = self.module_names.get(module_name)?;
            let module = &self.modules[*module_idx];

            let mut same_name_exports = module.exports.iter().filter_map(|export| {
                if export.name == function_name {
                    Some(&export.desc)
                } else {
                    None
                }
            });

            // TODO: what if there are two exports with the same name -- error out?
            if same_name_exports.clone().count() != 1 {
                return None;
            }

            let target_export = same_name_exports.next()?;

            match target_export {
                ExportDesc::FuncIdx(local_idx) => {
                    // Note: if we go ahead with the offset proposal, we can do
                    // store_idx = module.functions_offset + *local_idx
                    let store_idx = module.functions[*local_idx];

                    match &self.functions[store_idx] {
                        FuncInst::Local(_local_func_inst) => {
                            return Some(store_idx);
                        }
                        FuncInst::Imported(import) => {
                            if import_path.contains(&(
                                import.module_name.clone(),
                                import.function_name.clone(),
                            )) {
                                // TODO: find a way around this reference to clone thing. Rust is uppsety spaghetti for
                                // understandable but dumb reasons.

                                // TODO: cycle detected :(
                                return None;
                            }

                            module_name = &import.module_name;
                            function_name = &import.function_name;
                        }
                    }
                }
                _ => return None,
            }
        }

        // At this point, we are 100-imports deep. This isn't okay, and could be a sign of an infinte loop. We don't
        // want our plane's CPU to keep searching for imports so we just assume we haven't found any.
        None
    }
}

impl<'b> ValidationInfo<'b> {
    pub fn instantiate_functions(&self) -> Result<Vec<FuncInst>> {
        let mut wasm_reader = WasmReader::new(self.wasm);

        let functions = self.functions.iter();
        let func_blocks = self.func_blocks.iter();

        let local_function_inst = functions.zip(func_blocks).map(|(ty, (func, sidetable))| {
            wasm_reader
                .move_start_to(*func)
                .expect("function index to be in the bounds of the WASM binary");

            let (locals, bytes_read) = wasm_reader
                .measure_num_read_bytes(crate::code::read_declared_locals)
                .unwrap_validated();

            let code_expr = wasm_reader
                .make_span(func.len() - bytes_read)
                .expect("TODO remove this expect");

            FuncInst::Local(LocalFuncInst {
                ty: *ty,
                locals,
                code_expr,
                // TODO figure out where we want our sidetables
                sidetable: sidetable.clone(),
            })
        });

        let imported_function_inst = self.imports.iter().filter_map(|import| match &import.desc {
            ImportDesc::Func(type_idx) => Some(FuncInst::Imported(ImportedFuncInst {
                ty: *type_idx,
                module_name: import.module_name.clone(),
                function_name: import.name.clone(),
            })),
            _ => None,
        });

        Ok(imported_function_inst.chain(local_function_inst).collect())
    }

    pub fn instantiate_tables(&self) -> Result<Vec<TableInst>> {
        Ok(self.tables.iter().map(|ty| TableInst::new(*ty)).collect())
    }

    pub fn instantiate_elements(
        &self,
        tables: &mut [TableInst],
    ) -> Result<(Vec<ElemInst>, Vec<usize>)> {
        let mut passive_elem_indexes: Vec<usize> = vec![];
        // https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
        let elements: Vec<ElemInst> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(i, elem)| {
                trace!("Instantiating element {:#?}", elem);

                let offsets = match &elem.init {
                    ElemItems::Exprs(_ref_type, init_exprs) => init_exprs
                        .iter()
                        .map(|expr| {
                            get_address_offset(
                                run_const_span(self.wasm, expr, ()).unwrap_validated(),
                            )
                        })
                        .collect::<Vec<Option<u32>>>(),
                    ElemItems::RefFuncs(indicies) => {
                        // This branch gets taken when the elements are direct function references (i32 values), so we just return the indices
                        indicies
                            .iter()
                            .map(|el| Some(*el))
                            .collect::<Vec<Option<u32>>>()
                    }
                };

                let references: Vec<Ref> = offsets
                    .iter()
                    .map(|offset| {
                        let offset = offset.as_ref().map(|offset| *offset as usize);
                        match elem.ty() {
                            RefType::FuncRef => Ref::Func(FuncAddr::new(offset)),
                            RefType::ExternRef => Ref::Extern(ExternAddr::new(offset)),
                        }
                    })
                    .collect();

                let instance = ElemInst {
                    ty: elem.ty(),
                    references,
                };

                match &elem.mode {
                    // As per https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
                    // A declarative element segment is not available at runtime but merely serves to forward-declare
                    //  references that are formed in code with instructions like `ref.func`

                    // Also, the answer given by Andreas Rossberg (the editor of the WASM Spec - Release 2.0)
                    // Per https://stackoverflow.com/questions/78672934/what-is-the-purpose-of-a-wasm-declarative-element-segment
                    // "[...] The reason Wasm requires this (admittedly ugly) forward declaration is to support streaming compilation [...]"
                    ElemMode::Declarative => None,
                    ElemMode::Passive => {
                        passive_elem_indexes.push(i);
                        Some(instance)
                    }
                    ElemMode::Active(active_elem) => {
                        let table_idx = active_elem.table_idx as usize;

                        let offset = match run_const_span(self.wasm, &active_elem.init_expr, ())
                            .unwrap_validated()
                        {
                            Value::I32(offset) => offset as usize,
                            // We are already asserting that on top of the stack there is an I32 at validation time
                            _ => unreachable!(),
                        };

                        let table = &mut tables[table_idx];
                        // This can't be verified at validation-time because we don't keep track of actual values when validating expressions
                        //  we only keep track of the type of the values. As such we can't pop the exact value of an i32 from the validation stack
                        assert!(table.len() >= (offset + instance.len()));

                        table.elem[offset..offset + instance.references.len()]
                            .copy_from_slice(&instance.references);

                        Some(instance)
                    }
                }
            })
            .collect();

        Ok((elements, passive_elem_indexes))
    }

    pub fn instantiate_memories(&self) -> Result<Vec<MemInst>> {
        let memories: Vec<MemInst> = self.memories.iter().map(|ty| MemInst::new(*ty)).collect();

        let import_memory_instances_len = self
            .imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Mem(_)))
            .count();

        match memories.len().checked_add(import_memory_instances_len) {
            None => {
                return Err(Error::StoreInstantiationError(
                    StoreInstantiationError::TooManyMemories(usize::MAX),
                ))
            }
            Some(mem_instances) => {
                if mem_instances > 1 {
                    return Err(Error::UnsupportedProposal(Proposal::MultipleMemories));
                }
            }
        };

        Ok(memories)
    }

    pub fn instantiate_data(&self, memory_instances: &mut [MemInst]) -> Result<Vec<DataInst>> {
        self.data
            .iter()
            .map(|d| {
                use crate::core::reader::types::data::DataMode;
                use crate::NumType;
                if let DataMode::Active(active_data) = d.mode.clone() {
                    let mem_idx = active_data.memory_idx;
                    if mem_idx != 0 {
                        todo!("Active data has memory_idx different than 0");
                    }
                    assert!(
                        memory_instances.len() > mem_idx,
                        "Multiple memories not yet supported"
                    );

                    let boxed_value = {
                        let mut wasm = WasmReader::new(self.wasm);
                        wasm.move_start_to(active_data.offset).unwrap_validated();
                        let mut stack = Stack::new();
                        run_const(wasm, &mut stack, ());
                        stack.pop_value(ValType::NumType(NumType::I32))
                        // stack.peek_unknown_value().ok_or(MissingValueOnTheStack)?
                    };

                    // TODO: this shouldn't be a simple value, should it? I mean it can't be, but it can also be any type of ValType
                    // TODO: also, do we need to forcefully make it i32?
                    let offset: u32 = match boxed_value {
                        Value::I32(val) => val,
                        // Value::I64(val) => {
                        //     if val > u32::MAX as u64 {
                        //         return Err(I64ValueOutOfReach("data segment".to_owned()));
                        //     }
                        //     val as u32
                        // }
                        // TODO: implement all value types
                        _ => todo!(),
                    };

                    let mem_inst = memory_instances.get_mut(mem_idx).unwrap();

                    let len = mem_inst.data.len();
                    if offset as usize + d.init.len() > len {
                        return Err(Error::StoreInstantiationError(
                            StoreInstantiationError::ActiveDataWriteOutOfBounds,
                        ));
                    }
                    let data = mem_inst
                        .data
                        .get_mut(offset as usize..offset as usize + d.init.len())
                        .unwrap();
                    data.copy_from_slice(&d.init);
                }
                Ok(DataInst {
                    data: d.init.clone(),
                })
            })
            .collect::<Result<Vec<DataInst>>>()
    }

    pub fn instantiate_globals(&self) -> Result<Vec<GlobalInst>> {
        Ok(self
            .globals
            .iter()
            .map({
                let mut stack = Stack::new();
                move |global| {
                    let mut wasm = WasmReader::new(self.wasm);
                    // The place we are moving the start to should, by all means, be inside the wasm bytecode.
                    wasm.move_start_to(global.init_expr).unwrap_validated();
                    // We shouldn't need to clear the stack. If validation is correct, it will remain empty after execution.

                    // TODO: imported globals
                    run_const(wasm, &mut stack, ());
                    let value = stack.pop_value(global.ty.ty);

                    GlobalInst {
                        global: *global,
                        value,
                    }
                }
            })
            .collect())
    }
}

#[derive(Debug)]
pub enum FuncInst {
    Local(LocalFuncInst),
    Imported(ImportedFuncInst),
}

#[derive(Debug)]
pub struct LocalFuncInst {
    pub ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
    pub sidetable: Sidetable,
}

#[derive(Debug)]
pub struct ImportedFuncInst {
    pub ty: TypeIdx,
    pub module_name: String,
    pub function_name: String,
}

impl FuncInst {
    pub fn ty(&self) -> TypeIdx {
        match self {
            FuncInst::Local(f) => f.ty,
            FuncInst::Imported(f) => f.ty,
        }
    }

    pub fn try_into_local(&self) -> Option<&LocalFuncInst> {
        match self {
            FuncInst::Local(f) => Some(f),
            FuncInst::Imported(_) => None,
        }
    }

    pub fn try_into_imported(&self) -> Option<&ImportedFuncInst> {
        match self {
            FuncInst::Local(_) => None,
            FuncInst::Imported(f) => Some(f),
        }
    }
}

#[derive(Clone, Debug)]
/// <https://webassembly.github.io/spec/core/exec/runtime.html#element-instances>
pub struct ElemInst {
    pub ty: RefType,
    pub references: Vec<Ref>,
}

impl ElemInst {
    pub fn len(&self) -> usize {
        self.references.len()
    }
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }
}

#[derive(Debug)]
pub struct TableInst {
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

impl TableInst {
    pub fn len(&self) -> usize {
        self.elem.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elem.is_empty()
    }

    pub fn new(ty: TableType) -> Self {
        Self {
            ty,
            elem: vec![Ref::default_from_ref_type(ty.et); ty.lim.min as usize],
        }
    }
}

#[derive(Debug)]
pub struct MemInst {
    #[allow(warnings)]
    pub ty: MemType,
    pub data: Vec<u8>,
}

impl MemInst {
    pub fn new(ty: MemType) -> Self {
        let initial_size = (crate::Limits::MEM_PAGE_SIZE as usize) * ty.limits.min as usize;

        Self {
            ty,
            data: vec![0u8; initial_size],
        }
    }

    pub fn grow(&mut self, delta_pages: usize) {
        self.data
            .extend(iter::repeat(0).take(delta_pages * (crate::Limits::MEM_PAGE_SIZE as usize)))
    }

    /// Can never be bigger than 65,356 pages
    pub fn size(&self) -> usize {
        self.data.len() / (crate::Limits::MEM_PAGE_SIZE as usize)
    }
}

#[derive(Debug)]
pub struct GlobalInst {
    pub global: Global,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}

#[derive(Debug)]
pub struct DataInst {
    pub data: Vec<u8>,
}
