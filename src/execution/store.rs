use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::core::error::{Proposal, Result as CustomResult, StoreInstantiationError};
use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::element::{ElemItems, ElemMode};
use crate::core::reader::types::export::ExportDesc;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::{Import, ImportDesc};
use crate::core::reader::types::{check_limits, FuncType, MemType, TableType, ValType};
use crate::core::reader::WasmReader;
use crate::execution::value::{Ref, Value};
use crate::execution::{get_address_offset, run_const, run_const_span, Stack};
use crate::value::{ExternAddr, FuncAddr};
use crate::{Error, RefType, RuntimeError, ValidationInfo};

use super::execution_info::ExecutionInfo;
use super::hooks::EmptyHookSet;
use super::locals::Locals;
use super::value::InteropValueList;
use super::{run, UnwrapValidatedExt};

use crate::linear_memory::LinearMemory;

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
    pub modules: Vec<ExecutionInfo<'b>>,
    pub module_names: BTreeMap<String, usize>,
}

#[derive(Default)]
pub struct CleanupStore {
    pub added_functions: usize,
    pub added_memories: usize,
    pub added_globals: usize,
    pub added_data: usize,
    pub added_tables: usize,
    pub added_elements: usize,
    // EXAMPLE: testsuite -> linking.wast lines 252-261 (assert_unlinkable) and line 262 (assert_trap)
    //          see the TODO's right below for explanation of the problem
    //            if we execute the assert_unlinkable we can still have a broken state of the store as we've tried to instantiate elements (and maybe data in other cases)
    //            that is because we only check for added functoins, memories, globals, data, tables AND elements BUT NOT FOR MODIFIED TABLES AND MEMORY!

    // TODO: add instantiated elements before the module
    //          so basically, we can error after instantiating elements (and possible data as well)
    // TODO: maybe add for instantiated data, as well
    // for now, for both the first TODO: dirty fix: moved instantiate_elements at the bottom
    // another solution would be to do the checks for both elements and data before actually instantiating
    // for now im done
}

impl<'b> Store<'b> {
    pub fn add_module(&mut self, name: String, module: ValidationInfo<'b>) -> CustomResult<()> {
        let functions_imports_indexes = {
            let mut function_imports_indexes = Vec::new();
            for import in &module.imports {
                if let ImportDesc::Func(func) = import.desc {
                    let global_function_idx = self
                        .get_global_function_idx_by_name(
                            self.get_module_idx_from_name(&import.module_name)?,
                            &import.name,
                        )
                        .ok_or(Error::UnknownFunction)?;

                    let ty = &module.types[func];

                    let global_func_ty = self.functions[global_function_idx].ty();

                    if global_func_ty != *ty {
                        return Err(Error::InvalidImportType);
                    }

                    trace!(
                        "Imported function! Global function idx: {}",
                        global_function_idx
                    );
                    function_imports_indexes.push(global_function_idx);
                }
            }
            function_imports_indexes
        };

        // let function_type_inst = module.instantiate_function_types()?;
        let table_imports_indexes = {
            let mut table_imports_indexes = Vec::new();
            for import in &module.imports {
                if let ImportDesc::Table(table) = import.desc {
                    let global_table_idx = self
                        .get_global_table_idx(
                            self.get_module_idx_from_name(&import.module_name)?,
                            &import.name,
                        )
                        .ok_or(Error::UnknownTable)?;

                    if table.et != self.tables[global_table_idx].ty.et
                        || !check_limits(
                            // the table could've grown, don't take initial min size
                            self.tables[global_table_idx].len() as u32,
                            self.tables[global_table_idx].ty.lim.max,
                            table.lim.min,
                            table.lim.max,
                        )
                    {
                        return Err(Error::InvalidImportType);
                    }
                    table_imports_indexes.push(global_table_idx);
                }
            }
            table_imports_indexes
        };

        let globals_imports = {
            let mut globals_imports = Vec::new();

            for import in &module.imports {
                if let ImportDesc::Global(..) = import.desc {
                    globals_imports.push(import.clone());
                }
            }

            globals_imports
        };

        let imported_globals = {
            let mut imported_globals = Vec::new();
            for global_import in &globals_imports {
                match global_import.desc {
                    ImportDesc::Global(global_type_import) => {
                        let value = self
                            .get_global_global_idx(
                                self.get_module_idx_from_name(&global_import.module_name)?,
                                &global_import.name,
                            )
                            .ok_or(Error::UnknownGlobal)?;

                        if global_type_import != self.globals[value].global.ty {
                            return Err(Error::InvalidImportType);
                        }
                        // let global = self.globals[value].value;

                        imported_globals.push(GlobalInst {
                            global: Global {
                                init_expr: Span::new(usize::MAX, 0),
                                ty: global_type_import,
                            },
                            value: self.globals[value].value,
                        })
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }

            imported_globals
        };

        let memory_imports_indexes = {
            let mut memory_imports_indexes = Vec::new();
            for import in &module.imports {
                if let ImportDesc::Mem(mem) = import.desc {
                    let global_memory_idx = self
                        .get_global_memory_idx(
                            self.get_module_idx_from_name(&import.module_name)?,
                            &import.name,
                        )
                        .ok_or(Error::UnknownMemory)?;

                    if !check_limits(
                        // the memory could've grown, don't take initial min size
                        self.memories[global_memory_idx].size() as u32,
                        self.memories[global_memory_idx].ty.limits.max,
                        mem.limits.min,
                        mem.limits.max,
                    ) {
                        return Err(Error::InvalidImportType);
                    };
                    memory_imports_indexes.push(global_memory_idx);
                }
            }
            memory_imports_indexes
        };
        let local_memories = module.instantiate_local_memories()?;
        let memories_offset = self.memories.len();
        let exec_memories = self.get_memories_indexes(&memory_imports_indexes, &local_memories)?;
        self.memories.extend(local_memories);

        let local_inst_funcs = module.instantiate_functions()?;

        let functions_offset = self.functions.len();
        let exec_functions =
            self.get_functions_indexes(&functions_imports_indexes, &local_inst_funcs)?;
        self.functions.extend(local_inst_funcs);

        let imported_globals_len = imported_globals.len();
        let mut globals = module.instantiate_globals(imported_globals)?;

        let data =
            module.instantiate_data(self, &exec_memories, &globals[0..imported_globals_len])?;

        let mut local_tables = module.instantiate_local_tables()?;
        let (element_inst, passive_idxs) = module.instantiate_elements(
            self,
            &exec_functions,
            &mut local_tables,
            &table_imports_indexes,
            &globals,
        )?;

        let tables_offset = self.tables.len();
        let exec_tables = self.get_tables_indexes(&table_imports_indexes, &local_tables)?;
        self.tables.extend(local_tables);

        let imported_functions = functions_imports_indexes.len();
        let imported_memories = memory_imports_indexes.len();
        let imported_globals = imported_globals_len;
        let imported_tables = table_imports_indexes.len(); // TODO: not yet supported

        let globals_offset = self.globals.len();
        let exec_globals =
            self.get_globals_indexes(&globals_imports, &globals[globals_imports.len()..])?;
        // let exec_globals = (globals_offset..(globals_offset + globals.len())).collect();
        globals.drain(0..globals_imports.len());
        self.globals.extend(globals);

        let data_offset = self.data.len();
        let exec_data = (data_offset..(data_offset + data.len())).collect();
        self.data.extend(data);

        let elements_offset = self.elements.len();
        let exec_elements = (elements_offset..(elements_offset + element_inst.len())).collect();
        self.elements.extend(element_inst);

        let execution_info = ExecutionInfo {
            name: name.clone(),
            wasm_bytecode: module.wasm,
            //TODO make this a ref
            sidetable: module.sidetable.clone(),

            functions: exec_functions,
            functions_offset,
            imported_functions_len: imported_functions,

            function_types: module.types,

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

        self.module_names.insert(name.clone(), self.modules.len());
        self.modules.push(execution_info);

        Ok(())
    }

    pub fn lookup_local_function(
        &self,
        target_module: &str,
        target_function: &str,
    ) -> Option<usize> {
        for module in &self.modules {
            if module.name == target_module {
                for export in &module.exports {
                    if export.name == target_function {
                        return export.desc.get_function_idx();
                    }
                }
            }
        }
        None
    }

    pub fn lookup_function(&self, target_module: &str, target_function: &str) -> Option<usize> {
        let module_name: &str = target_module;
        let function_name: &str = target_function;
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
                ExportDesc::FuncIdx(local_idx) => return Some(module.functions[*local_idx]),
                _ => return None,
            }
        }

        // At this point, we are 100-imports deep. This isn't okay, and could be a sign of an infinte loop. We don't
        // want our plane's CPU to keep searching for imports so we just assume we haven't found any.
        None
    }

    // TODO: why do get_module and get_function functions return both Result and Option? Settle on something
    pub fn get_module_idx_from_name(&self, module_name: &str) -> Result<usize, RuntimeError> {
        Ok(*(self
            .module_names
            .get(module_name)
            .ok_or(RuntimeError::ModuleNotFound)?))
    }

    pub(crate) fn get_module_idx_from_func_idx(
        &self,
        func_idx: usize,
    ) -> Result<usize, RuntimeError> {
        for module_idx in 0..self.modules.len() {
            let module = &self.modules[module_idx];
            let start = module.imported_functions_len;
            for i in start..module.functions.len() {
                if module.functions[i] == func_idx {
                    return Ok(module_idx);
                }
            }
        }

        Err(RuntimeError::ModuleNotFound)
    }

    pub fn get_local_function_idx_by_global_function_idx(
        &self,
        module_idx: usize,
        global_function_idx: usize,
    ) -> Option<usize> {
        let functions = &self.modules[module_idx].functions;
        for (i, item) in functions.iter().enumerate() {
            if *item == global_function_idx {
                return Some(i);
            }
        }
        None
    }

    pub fn get_local_function_idx_by_function_name(
        &self,
        module_idx: usize,
        function_name: &str,
    ) -> Option<usize> {
        for export in &self.modules[module_idx].exports {
            if export.name == function_name {
                return export.desc.get_function_idx();
            }
        }

        None
    }

    pub fn get_global_function_idx_by_name(
        &self,
        // module_name: &str,
        module_idx: usize,
        function_name: &str,
    ) -> Option<usize> {
        for export in &self.modules[module_idx].exports {
            if export.name == function_name {
                if let Some(local_func_idx) = export.desc.get_function_idx() {
                    return Some(self.modules[module_idx].functions[local_func_idx]);
                };
                return None;
            }
        }

        None
    }

    pub fn get_global_global_idx(&self, module_idx: usize, global_name: &str) -> Option<usize> {
        for export in &self.modules[module_idx].exports {
            if export.name == global_name {
                return export
                    .desc
                    .get_global_idx()
                    .map(|idx| self.modules[module_idx].globals[idx]);
            }
        }
        None
    }

    pub fn get_global_memory_idx(&self, module_idx: usize, memory_name: &str) -> Option<usize> {
        for export in &self.modules[module_idx].exports {
            if export.name == memory_name {
                return export
                    .desc
                    .get_memory_idx()
                    .map(|idx| self.modules[module_idx].memories[idx]);
            }
        }
        None
    }

    pub fn get_global_table_idx(&self, module_idx: usize, table_name: &str) -> Option<usize> {
        for export in &self.modules[module_idx].exports {
            if export.name == table_name {
                return export
                    .desc
                    .get_table_idx()
                    .map(|idx| self.modules[module_idx].tables[idx]);
            }
        }
        None
    }

    fn get_globals_indexes(
        &self,
        globals_imports: &Vec<Import>,
        local_globals: &[GlobalInst],
    ) -> CustomResult<Vec<usize>> {
        let mut indexes: Vec<usize> = Vec::new();

        for import in globals_imports {
            indexes.push(
                self.get_global_global_idx(
                    self.get_module_idx_from_name(&import.module_name)?,
                    &import.name,
                )
                .ok_or(Error::UnknownGlobal)?,
            );
        }

        let globals_offset = self.globals.len();
        for global_idx in globals_offset..local_globals.len() + globals_offset {
            indexes.push(global_idx)
        }

        Ok(indexes)
    }

    fn get_memories_indexes(
        &self,
        memories_imports_indexes: &[usize],
        local_memories: &[MemInst],
    ) -> CustomResult<Vec<usize>> {
        let mut indexes: Vec<usize> = Vec::new();
        indexes.extend_from_slice(memories_imports_indexes);
        let memories_offset = self.memories.len();
        for memory_idx in memories_offset..local_memories.len() + memories_offset {
            indexes.push(memory_idx);
        }

        Ok(indexes)
    }

    fn get_functions_indexes(
        &self,
        functions_imports_indexes: &[usize],
        local_functions: &[FuncInst],
    ) -> CustomResult<Vec<usize>> {
        let mut indexes: Vec<usize> = Vec::new();
        indexes.extend_from_slice(functions_imports_indexes);
        let functions_offset = self.functions.len();
        for function_idx in functions_offset..local_functions.len() + functions_offset {
            indexes.push(function_idx);
        }

        Ok(indexes)
    }

    fn get_tables_indexes(
        &self,
        tables_imports_indexes: &[usize],
        local_tables: &[TableInst],
    ) -> CustomResult<Vec<usize>> {
        let mut indexes: Vec<usize> = Vec::new();
        indexes.extend_from_slice(tables_imports_indexes);
        let tables_offset = self.tables.len();
        for table_idx in tables_offset..local_tables.len() + tables_offset {
            indexes.push(table_idx);
        }

        Ok(indexes)
    }

    pub fn invoke<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        func_idx: usize,
        params: Param,
    ) -> Result<Returns, RuntimeError> {
        let func_inst = self
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        if func_ty.params.valtypes != Param::TYS {
            panic!("Invalid `Param` generics");
        }
        if func_ty.returns.valtypes != Returns::TYS {
            panic!("Invalid `Returns` generics");
        }

        let mut stack = Stack::new();
        let locals = Locals::new(
            params.into_values().into_iter(),
            func_inst.locals.iter().cloned(),
        );

        let module_idx = self.get_module_idx_from_func_idx(func_idx)?;
        let local_func_idx = self
            .get_local_function_idx_by_global_function_idx(module_idx, func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;
        // setting `usize::MAX` as return address for the outermost function ensures that we
        // observably fail upon errornoeusly continuing execution after that function returns.
        stack.push_stackframe(
            module_idx,
            local_func_idx,
            &func_ty,
            locals,
            usize::MAX,
            usize::MAX,
        );

        let mut current_module_idx = module_idx;
        // Run the interpreter
        run(
            // &mut self.modules,
            &mut current_module_idx,
            // self.lut.as_ref().ok_or(RuntimeError::UnmetImport)?,
            &mut stack,
            EmptyHookSet,
            self,
        )?;

        // Pop return values from stack
        let return_values = Returns::TYS
            .iter()
            .rev()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret: Returns = Returns::from_values(reversed_values);
        debug!("Successfully invoked function");
        Ok(ret)
    }

    pub fn invoke_dynamic(
        &mut self,
        func_idx: usize,
        params: Vec<Value>,
        ret_types: &[ValType],
    ) -> Result<Vec<Value>, RuntimeError> {
        let func_inst = self
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Verify that the given parameters match the function parameters
        let param_types = params.iter().map(|v| v.to_ty()).collect::<Vec<_>>();

        if func_ty.params.valtypes != param_types {
            // format!()
            trace!(
                "Func param types len: {}; Given args len: {}",
                func_ty.params.valtypes.len(),
                param_types.len()
            );
            panic!("Invalid parameters for function");
        }

        // Verify that the given return types match the function return types
        if func_ty.returns.valtypes != ret_types {
            panic!("Invalid return types for function");
        }

        // Prepare a new stack with the locals for the entry function
        let mut stack = Stack::new();
        let locals = Locals::new(params.into_iter(), func_inst.locals.iter().cloned());
        let module_idx = self.get_module_idx_from_func_idx(func_idx)?;
        let local_func_idx = self
            .get_local_function_idx_by_global_function_idx(module_idx, func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        stack.push_stackframe(
            module_idx,
            local_func_idx,
            &func_ty,
            locals,
            usize::MAX,
            usize::MAX,
        );

        let mut currrent_module_idx = module_idx;
        // Run the interpreter
        run(
            // &mut self.modules,
            &mut currrent_module_idx,
            // self.lut.as_ref().ok_or(RuntimeError::UnmetImport)?,
            &mut stack,
            EmptyHookSet,
            self,
        )?;

        let func_inst = self
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Pop return values from stack
        let return_values = func_ty
            .returns
            .valtypes
            .iter()
            .rev()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret = reversed_values.collect();
        debug!("Successfully invoked function");
        Ok(ret)
    }

    pub fn invoke_dynamic_unchecked_return_ty(
        &mut self,
        func_idx: usize,
        params: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        let func_inst = self
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let module_idx = self.get_module_idx_from_func_idx(func_idx)?;

        let func_ty = func_inst.ty();

        // Verify that the given parameters match the function parameters
        let param_types = params.iter().map(|v| v.to_ty()).collect::<Vec<_>>();

        if func_ty.params.valtypes != param_types {
            trace!(
                "Func param types len: {}; Given args len: {}",
                func_ty.params.valtypes.len(),
                param_types.len()
            );
            panic!("Invalid parameters for function");
        }

        // Prepare a new stack with the locals for the entry function
        let mut stack = Stack::new();
        let locals = Locals::new(params.into_iter(), func_inst.locals.iter().cloned());
        let local_func_idx = self
            .get_local_function_idx_by_global_function_idx(module_idx, func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;
        stack.push_stackframe(module_idx, local_func_idx, &func_ty, locals, 0, 0);

        let mut currrent_module_idx = module_idx;
        // Run the interpreter
        run(
            // &mut self.modules,
            &mut currrent_module_idx,
            // self.lut.as_ref().ok_or(RuntimeError::UnmetImport)?,
            &mut stack,
            EmptyHookSet,
            self,
        )?;

        // Pop return values from stack
        let return_values = func_ty
            .returns
            .valtypes
            .iter()
            .rev()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret = reversed_values.collect();
        debug!("Successfully invoked function");
        Ok(ret)
    }

    pub fn register_alias(&mut self, alias_name: String, module_idx: usize) {
        self.module_names.insert(alias_name, module_idx);
    }
}

impl ValidationInfo<'_> {
    pub fn instantiate_functions(&self) -> CustomResult<Vec<FuncInst>> {
        let mut wasm_reader = WasmReader::new(self.wasm);

        let functions = self.functions.iter();
        let func_blocks_stps = self.func_blocks_stps.iter();

        Ok(functions
            .zip(func_blocks_stps)
            .map(|(ty, (func, stp))| {
                wasm_reader
                    .move_start_to(*func)
                    .expect("function index to be in the bounds of the WASM binary");

                let (locals, bytes_read) = wasm_reader
                    .measure_num_read_bytes(crate::code::read_declared_locals)
                    .unwrap_validated();

                let code_expr = wasm_reader
                    .make_span(func.len() - bytes_read)
                    .expect("TODO remove this expect");

                FuncInst {
                    ty: *ty,
                    locals,
                    code_expr,
                    stp: *stp,
                    function_type: self.types[*ty].clone(),
                }
            })
            .collect())
    }

    pub fn instantiate_local_tables(&self) -> CustomResult<Vec<TableInst>> {
        Ok(self.tables.iter().map(|ty| TableInst::new(*ty)).collect())
    }

    // TODO: funcref tables should contain the GLOBAL index of the functions they reference

    pub fn instantiate_elements(
        &self,
        store: &mut Store,
        imported_and_local_functions_indexes: &[usize],
        tables: &mut [TableInst],
        imported_tables_indexes: &[usize],
        globals: &[GlobalInst],
    ) -> CustomResult<(Vec<ElemInst>, Vec<usize>)> {
        let mut passive_elem_indexes: Vec<usize> = vec![];
        // https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
        let mut elements: Vec<ElemInst> = Vec::new();

        // let elements: Vec<ElemInst> = self
        for (i, elem) in self.elements.iter().enumerate() {
            trace!("Instantiating element {:#?}", elem);

            let offsets = match &elem.init {
                ElemItems::Exprs(_ref_type, init_exprs) => init_exprs
                    .iter()
                    .map(|expr| {
                        get_address_offset(
                            run_const_span(self.wasm, expr, globals).unwrap_validated(),
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

            // validate

            for offset in &offsets {
                match offset {
                    None => {
                        // what should we do here?
                        // should we just crash?
                        // for now just don't do anything
                    }
                    Some(offset) => {
                        match elem.ty() {
                            RefType::ExternRef => {
                                // we don't care about extern refs for now
                            }
                            RefType::FuncRef => {
                                if *offset as usize >= imported_and_local_functions_indexes.len() {
                                    return Err(Error::FunctionIsNotDefined(*offset as usize));
                                }
                            }
                        }
                    }
                }
            }

            let references: Vec<Ref> = {
                let mut temp: Vec<Ref> = Vec::new();

                for offset in offsets {
                    let offset = offset.as_ref().map(|offset| *offset as usize);
                    temp.push(match elem.ty() {
                        RefType::FuncRef => Ref::Func(FuncAddr::new({
                            match offset {
                                None => None,
                                Some(offset) => {
                                    trace!(
                                        "Table element with global function idx: {}",
                                        imported_and_local_functions_indexes[offset]
                                    );
                                    Some(imported_and_local_functions_indexes[offset])
                                }
                            }
                        })),

                        // TODO: ExternRefs - when implemented
                        RefType::ExternRef => Ref::Extern(ExternAddr::new(offset)),
                    });
                }

                temp
            };

            // let references: Vec<Ref> = offsets
            //     .iter()
            //     .map(|offset| {
            //         let offset = offset.as_ref().map(|offset| *offset as usize);
            //         match elem.ty() {
            //             RefType::FuncRef => Ref::Func(FuncAddr::new(offset)),

            //             // TODO: ExternRef's
            //             RefType::ExternRef => Ref::Extern(ExternAddr::new(offset)),
            //         }
            //     })
            //     .collect();

            let instance = ElemInst {
                ty: elem.ty(),
                references,
            };

            let elem_inst_opt = match &elem.mode {
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

                    let offset = match run_const_span(self.wasm, &active_elem.init_expr, globals)
                        .unwrap_validated()
                    {
                        Value::I32(offset) => offset as usize,
                        // We are already asserting that on top of the stack there is an I32 at validation time
                        _ => unreachable!(),
                    };

                    let table = if table_idx >= imported_tables_indexes.len() {
                        let actual_offset = table_idx - imported_tables_indexes.len();
                        &mut tables[actual_offset]
                    } else {
                        // self.tables[imported_tables_indexes[table_idx]]
                        &mut store.tables[imported_tables_indexes[table_idx]]
                    };
                    // This can't be verified at validation-time because we don't keep track of actual values when validating expressions
                    //  we only keep track of the type of the values. As such we can't pop the exact value of an i32 from the validation stack
                    assert!(table.len() >= (offset + instance.len()));

                    // trace!("")
                    table.elem[offset..offset + instance.references.len()]
                        .copy_from_slice(&instance.references);

                    Some(instance)
                }
            };

            match elem_inst_opt {
                None => {
                    // todo idk
                }
                Some(elem_inst) => elements.push(elem_inst),
            };
        }
        // .filter_map(|(i, elem)| {
        //     })
        // .collect();

        Ok((elements, passive_elem_indexes))
    }

    pub fn instantiate_local_memories(&self) -> CustomResult<Vec<MemInst>> {
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

    pub fn instantiate_data(
        &self,
        store: &mut Store,
        // memory_instances: &mut [MemInst],
        memories_indexes: &[usize],
        imported_globals: &[GlobalInst],
    ) -> CustomResult<Vec<DataInst>> {
        self.data
            .iter()
            .map(|d| {
                use crate::core::reader::types::data::DataMode;
                use crate::NumType;
                if let DataMode::Active(active_data) = d.mode.clone() {
                    trace!("Instantiating active data (DataMode::Active)...");
                    let mem_idx = active_data.memory_idx;
                    if mem_idx != 0 {
                        todo!("Active data has memory_idx different than 0");
                    }

                    assert!(
                        memories_indexes.len() <= 1,
                        "Multiple memories not yet supported"
                    );

                    let boxed_value = {
                        let mut wasm = WasmReader::new(self.wasm);
                        wasm.move_start_to(active_data.offset).unwrap_validated();
                        let mut stack = Stack::new();
                        run_const(&mut wasm, &mut stack, imported_globals);
                        let boxed_value = stack.pop_value(ValType::NumType(NumType::I32));

                        // we should have NO value on the stack whatsoever, otherwise it's wrong
                        if stack.peek_unknown_value().is_some() {
                            return Err(Error::EndInvalidValueStack);
                        }
                        // stack.peek_unknown_value().ok_or(MissingValueOnTheStack)?

                        boxed_value
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

                    let index = memories_indexes.get(mem_idx).ok_or(Error::UnknownMemory)?;
                    let mem_inst = store.memories.get_mut(*index).ok_or(Error::UnknownMemory)?;
                    // let mem_inst = memory_instances.get_mut(mem_idx).unwrap();

                    let len = mem_inst.mem.len();
                    if offset as usize + d.init.len() > len {
                        return Err(Error::StoreInstantiationError(
                            StoreInstantiationError::ActiveDataWriteOutOfBounds,
                        ));
                    }

                    mem_inst
                        .mem
                        .init(offset as usize, &d.init, 0, d.init.len())?;
                }
                Ok(DataInst {
                    data: d.init.clone(),
                })
            })
            .collect::<CustomResult<Vec<DataInst>>>()
    }

    pub fn instantiate_globals(
        &self,
        imported_globals: Vec<GlobalInst>,
    ) -> CustomResult<Vec<GlobalInst>> {
        // let mut globals = imported_globals;
        let mut local_globals = Vec::new();

        for global in &self.globals {
            let mut stack = Stack::new();
            let mut wasm = WasmReader::new(self.wasm);
            // The place we are moving the start to should, by all means, be inside the wasm bytecode.
            wasm.move_start_to(global.init_expr).unwrap_validated();
            // We shouldn't need to clear the stack. If validation is correct, it will remain empty after execution.

            // TODO: imported globals
            run_const(&mut wasm, &mut stack, &imported_globals);
            let value = stack.pop_value(global.ty.ty);

            local_globals.push(GlobalInst {
                global: *global,
                value,
            })
        }
        let mut all_globals = vec![];
        all_globals.extend(imported_globals);
        all_globals.extend(local_globals);
        Ok(all_globals)
    }
}

#[derive(Debug)]
pub struct FuncInst {
    pub ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
    ///index of the sidetable corresponding to the beginning of this functions code
    pub stp: usize,
    pub function_type: FuncType,
}

impl FuncInst {
    pub fn ty_idx(&self) -> TypeIdx {
        self.ty
    }

    pub fn ty(&self) -> FuncType {
        self.function_type.clone()
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

// TODO: The tables have to be both imported and exported (an enum instead of a struct)
//       That is because when we import tables we can give a different size to the imported table
//        thus having a wrapper over the initial table
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

pub struct MemInst {
    #[allow(warnings)]
    pub ty: MemType,
    pub mem: LinearMemory,
}

impl MemInst {
    pub fn new(ty: MemType) -> Self {
        Self {
            ty,
            mem: LinearMemory::new_with_initial_pages(ty.limits.min.try_into().unwrap()),
        }
    }

    pub fn grow(&mut self, delta_pages: usize) {
        self.mem.grow(delta_pages.try_into().unwrap())
    }

    /// Can never be bigger than 65,356 pages
    pub fn size(&self) -> usize {
        self.mem.len() / (crate::Limits::MEM_PAGE_SIZE as usize)
    }
}

// pub struct GlobalInstV2 {
//     Local(LocalGlobalInst),
//     Imported(ImportedGlobalInst)
// }

#[derive(Debug)]
pub struct GlobalInst {
    pub global: Global,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}

pub struct DataInst {
    pub data: Vec<u8>,
}
