use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::core::error::{Proposal, Result as CustomResult, StoreInstantiationError};
use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::data::DataSegment;
use crate::core::reader::types::element::{ElemItems, ElemMode};
use crate::core::reader::types::export::ExportDesc;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::{Import, ImportDesc};
use crate::core::reader::types::{check_limits, ExternType, FuncType, MemType, TableType, ValType};
use crate::core::reader::WasmReader;
use crate::execution::value::{Ref, Value};
use crate::execution::{get_address_offset, run_const, run_const_span, Stack};
use crate::value::{ExternAddr, FuncAddr};
use crate::{unreachable_validated, Error, RefType, RuntimeError, ValidationInfo};

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
    pub modules: Vec<ModuleInst<'b>>,
    pub module_names: BTreeMap<String, usize>,
}

impl<'b> Store<'b> {
    /// instantiates a validated module with `validation_info` as validation evidence with name `name`
    /// with the steps in <https://webassembly.github.io/spec/core/exec/modules.html#instantiation>
    /// this method roughly matches the suggested embedder function`module_instantiate`
    /// https://webassembly.github.io/spec/core/appendix/embedding.html#modules
    /// except external values for module instantiation are retrieved from `self`.
    pub fn add_module(
        &mut self,
        name: String,
        validation_info: &ValidationInfo<'b>,
    ) -> CustomResult<()> {
        // instantiation: step 1 and 2 are skipped since validation_info acts as validation evidence.

        // instantation: the loop below for gathering extern_vals implicitly satisfy step 3 and 4.

        let mut extern_vals = Vec::new();

        for Import {
            module_name: exporting_module_name,
            name: import_name,
            desc: import_desc,
        } in &validation_info.imports
        {
            let import_extern_type = import_desc.extern_type(validation_info);
            let exporting_module = self
                .modules
                .get(
                    *self
                        .module_names
                        .get(exporting_module_name)
                        .ok_or(Error::TodoErrorVariantError)?,
                )
                .ok_or(Error::TodoErrorVariantError)?;
            extern_vals.push(
                *exporting_module
                    .exports
                    .iter()
                    .find_map(
                        |ExportInst {
                             name: export_name,
                             value: export_extern_val,
                         }| {
                            // TODO: this should be a subtyping relation import_extern_type => export_extern_type
                            (import_name == export_name
                                && import_extern_type == export_extern_val.extern_type(self))
                            .then_some(export_extern_val)
                        },
                    )
                    .ok_or(Error::TodoErrorVariantError)?,
            );
        }

        // instantiation: step 5
        // module_inst_init is unfortunately circularly defined from parts of module_inst that would be defined in step 11, which uses module_inst_init again implicitly.
        // therefore I am mimicking the reference interpreter code here, I will allocate functions in the store in this step instead of step 11.
        // https://github.com/WebAssembly/spec/blob/8d6792e3d6709e8d3e90828f9c8468253287f7ed/interpreter/exec/eval.ml#L789
        let mut module_inst = ModuleInst {
            types: validation_info.types,
            func_addrs: ExternVal::funcs(&extern_vals),
            table_addrs: Vec::new(),
            mem_addrs: Vec::new(),
            global_addrs: ExternVal::globals(&extern_vals),
            elem_addrs: Vec::new(),
            data_addrs: Vec::new(),
            exports: Vec::new(),
            wasm: validation_info.wasm,
        };

        // TODO possibly rewrite to reflect Function struct from validation better
        // <https://webassembly.github.io/spec/core/exec/modules.html#functions>
        module_inst.func_addrs.extend(
            validation_info
                .functions
                .iter()
                .zip(validation_info.func_blocks_stps.iter())
                .map(|(ty_idx, (span, stp))| {
                    self.alloc_func((*ty_idx, (*span, *stp)), module_inst)
                }),
        );

        // instantiation: this roughly matches step 6,7,8
        // validation guarantees these will evaluate without errors.
        let global_init_vals: Vec<Value> = validation_info
            .globals
            .iter()
            .map(|global| {
                run_const_span(validation_info.wasm, &global.init_expr, &module_inst, self)
                    .unwrap_validated()
            })
            .collect();

        // instantiation: this roughly matches step 9,10
        fn cast_val_to_ref_validated(val: Value) -> Ref {
            match val {
                Value::Ref(reff) => reff,
                _ => unreachable_validated!(),
            }
        }

        let element_init_ref_lists: Vec<Vec<Ref>> = validation_info
            .elements
            .iter()
            .map(|elem| {
                match elem.init {
                    // shortcut of evaluation of "ref.func <func_idx>; end;"
                    // validation guarantees corresponding func_idx's existence
                    ElemItems::RefFuncs(ref_funcs) => ref_funcs
                        .iter()
                        .map(|func_idx| {
                            Ref::Func(FuncAddr {
                                addr: Some(module_inst.func_addrs[*func_idx as usize]),
                            })
                        })
                        .collect(),
                    ElemItems::Exprs(ref_typ, exprs) => exprs
                        .iter()
                        .map(|expr| {
                            cast_val_to_ref_validated(
                                run_const_span(validation_info.wasm, expr, &module_inst, self)
                                    .unwrap_validated(),
                            )
                        })
                        .collect(),
                }
            })
            .collect();

        // instantiation: step 11 - module allocation (except function allocation - which was made in step 5)
        // https://webassembly.github.io/spec/core/exec/modules.html#alloc-module
        {
            // allocation: step 1
            let module = validation_info;

            let extern_vals = extern_vals;
            let vals = global_init_vals;
            let ref_lists = element_init_ref_lists;

            // allocation: skip step 2 as it was done in instantiation step 5
            let mut module_inst = module_inst;

            // allocation: step 3-13
            let table_addrs = module
                .tables
                .iter()
                .map(|table_type| self.alloc_table(*table_type, Ref::Func(FuncAddr { addr: None })))
                .collect();
            let mem_addrs = module
                .memories
                .iter()
                .map(|mem_type| self.alloc_mem(*mem_type))
                .collect();
            let global_addrs = module
                .globals
                .iter()
                .zip(vals)
                .map(
                    |(
                        Global {
                            ty: global_type, ..
                        },
                        val,
                    )| self.alloc_global(global_type, val),
                )
                .collect();
            let elem_addrs = module
                .elements
                .iter()
                .zip(ref_lists)
                .map(|(elem, refs)| self.alloc_elem(elem.ty(), refs))
                .collect();
            let data_addrs = module
                .data
                .iter()
                .map(|DataSegment { init: bytes, .. }| self.alloc_data(*bytes))
                .collect();

            // allocation: skip step 14 as it was done in instantiation step 5

            // allocation: step 15,16
            // let mut table_addrs_mod = ExternVal::tables(&extern_vals);
            // table_addrs_mod.extend(table_addrs);

            // let mut mem

            // // skipping step 17 as it was partially done in instantiation step

            // // allocation: step 18
            // let export_insts = module.exports.iter().map(|Export{name, export_desc}| {
            //     match export_desc {

            //     }
            // });
        }

        Ok(())
    }

    /// returns the module instance within the store

    /// roughly matches <https://webassembly.github.io/spec/core/exec/modules.html#functions> with the addition of sidetable pointer to the input signature
    // TODO refactor the type of func
    fn alloc_func(&mut self, func: (TypeIdx, (Span, usize)), module_inst: ModuleInst) -> usize {
        let (ty, (span, stp)) = func;

        // TODO rewrite this huge chunk of parsing after generic way to re-parse(?) structs lands
        let mut wasm_reader = WasmReader::new(module_inst.wasm);
        wasm_reader.move_start_to(span).unwrap_validated();

        let (locals, bytes_read) = wasm_reader
            .measure_num_read_bytes(crate::code::read_declared_locals)
            .unwrap_validated();

        let code_expr = wasm_reader
            .make_span(span.len() - bytes_read)
            .unwrap_validated();

        // core of the method below

        let func_inst = FuncInst {
            ty,
            locals,
            code_expr,
            stp,
            // validation guarantees func_ty_idx exists within module_inst.types
            function_type: module_inst.types[ty],
        };

        let addr = self.functions.len();
        self.functions.push(func_inst);
        addr
    }

    /// https://webassembly.github.io/spec/core/exec/modules.html#tables
    fn alloc_table(&mut self, table_type: TableType, reff: Ref) -> usize {
        let table_inst = TableInst {
            ty: table_type,
            elem: vec![reff; table_type.lim.min as usize],
        };

        let addr = self.tables.len();
        self.tables.push(table_inst);
        addr
    }

    /// https://webassembly.github.io/spec/core/exec/modules.html#memories
    fn alloc_mem(&mut self, mem_type: MemType) -> usize {
        let mem_inst = MemInst {
            ty: mem_type,
            mem: LinearMemory::new_with_initial_pages(
                mem_type.limits.min.try_into().unwrap_validated(),
            ),
        };

        let addr = self.memories.len();
        self.memories.push(mem_inst);
        addr
    }

    /// https://webassembly.github.io/spec/core/exec/modules.html#globals
    fn alloc_global(&mut self, global_type: GlobalType, val: Value) -> usize {
        let global_inst = GlobalInst {
            ty: global_type,
            value: val,
        };

        let addr = self.globals.len();
        self.globals.push(global_inst);
        addr
    }

    /// https://webassembly.github.io/spec/core/exec/modules.html#element-segments
    fn alloc_elem(&mut self, ref_type: RefType, refs: Vec<Ref>) -> usize {
        let elem_inst = ElemInst {
            ty: ref_type,
            references: refs,
        };

        let addr = self.elements.len();
        self.elements.push(elem_inst);
        addr
    }

    /// https://webassembly.github.io/spec/core/exec/modules.html#data-segments
    fn alloc_data(&mut self, bytes: Vec<u8>) -> usize {
        let data_inst = DataInst { data: bytes };

        let addr = self.data.len();
        self.data.push(data_inst);
        addr
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

#[derive(Debug)]
// TODO does not match the spec FuncInst
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
    pub ty: GlobalType,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}

pub struct DataInst {
    pub data: Vec<u8>,
}

///<https://webassembly.github.io/spec/core/exec/runtime.html#external-values>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExternVal {
    Func(usize),
    Table(usize),
    Mem(usize),
    Global(usize),
}

impl ExternVal {
    /// returns the external type of `self` according to typing relation,
    /// taking `store` as context S.
    /// typing fails if this external value does not exist within S.
    ///<https://webassembly.github.io/spec/core/valid/modules.html#imports>
    pub fn extern_type(&self, store: &Store) -> CustomResult<ExternType> {
        // TODO: implement error variants
        Ok(match self {
            // TODO: fix ugly clone in function types
            ExternVal::Func(func_addr) => ExternType::Func(
                store
                    .functions
                    .get(*func_addr)
                    .ok_or(Error::TodoErrorVariantError)?
                    .ty(),
            ),
            ExternVal::Table(table_addr) => ExternType::Table(
                store
                    .tables
                    .get(*table_addr)
                    .ok_or(Error::TodoErrorVariantError)?
                    .ty,
            ),
            ExternVal::Mem(mem_addr) => ExternType::Mem(
                store
                    .memories
                    .get(*mem_addr)
                    .ok_or(Error::TodoErrorVariantError)?
                    .ty,
            ),
            ExternVal::Global(global_addr) => ExternType::Global(
                store
                    .globals
                    .get(*global_addr)
                    .ok_or(Error::TodoErrorVariantError)?
                    .global
                    .ty,
            ),
        })
    }

    // conventional functions for extern_vals
    // https://webassembly.github.io/spec/core/exec/runtime.html#conventions
    pub fn funcs(extern_vals: &Vec<Self>) -> Vec<usize> {
        extern_vals
            .iter()
            .filter_map(|extern_val| {
                if let ExternVal::Func(func_addr) = extern_val {
                    Some(*func_addr)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn tables(extern_vals: &Vec<Self>) -> Vec<usize> {
        extern_vals
            .iter()
            .filter_map(|extern_val| {
                if let ExternVal::Table(table_addr) = extern_val {
                    Some(*table_addr)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn mems(extern_vals: &Vec<Self>) -> Vec<usize> {
        extern_vals
            .iter()
            .filter_map(|extern_val| {
                if let ExternVal::Mem(mem_addr) = extern_val {
                    Some(*mem_addr)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn globals(extern_vals: &Vec<Self>) -> Vec<usize> {
        extern_vals
            .iter()
            .filter_map(|extern_val| {
                if let ExternVal::Global(global_addr) = extern_val {
                    Some(*global_addr)
                } else {
                    None
                }
            })
            .collect()
    }
}

///<https://webassembly.github.io/spec/core/exec/runtime.html#export-instances>
pub struct ExportInst {
    pub name: String,
    pub value: ExternVal,
}

///<https://webassembly.github.io/spec/core/exec/runtime.html#module-instances>
pub struct ModuleInst<'b> {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<usize>,
    pub table_addrs: Vec<usize>,
    pub mem_addrs: Vec<usize>,
    pub global_addrs: Vec<usize>,
    pub elem_addrs: Vec<usize>,
    pub data_addrs: Vec<usize>,
    pub exports: Vec<ExportInst>,

    //TODO the bytecode is not in the spec, but required for re-parsing
    pub wasm: &'b [u8],
}
