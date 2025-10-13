use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::data::{DataModeActive, DataSegment};
use crate::core::reader::types::element::{ActiveElem, ElemItems, ElemMode, ElemType};
use crate::core::reader::types::export::{Export, ExportDesc};
use crate::core::reader::types::global::{Global, GlobalType};
use crate::core::reader::types::import::Import;
use crate::core::reader::types::{
    ExternType, FuncType, ImportSubTypeRelation, MemType, TableType, ValType,
};
use crate::core::reader::WasmReader;
use crate::core::sidetable::Sidetable;
use crate::execution::interpreter_loop::{self, memory_init, table_init};
use crate::execution::value::{Ref, Value};
use crate::execution::{run_const_span, Stack};
use crate::registry::Registry;
use crate::resumable::{Dormitory, Resumable, RunState};
use crate::value::FuncAddr;
use crate::{Limits, RefType, RuntimeError, TrapError, ValidationInfo};
use alloc::borrow::ToOwned;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::hooks::EmptyHookSet;
use super::interpreter_loop::{data_drop, elem_drop};
use super::UnwrapValidatedExt;

use crate::linear_memory::LinearMemory;

/// The store represents all global state that can be manipulated by WebAssembly programs. It
/// consists of the runtime representation of all instances of functions, tables, memories, and
/// globals, element segments, and data segments that have been allocated during the life time of
/// the abstract machine.
/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
pub struct Store<'b, T> {
    pub functions: Vec<FuncInst<T>>,
    pub memories: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub data: Vec<DataInst>,
    pub tables: Vec<TableInst>,
    pub elements: Vec<ElemInst>,
    pub modules: Vec<ModuleInst<'b>>,

    // fields outside of the spec but are convenient are below

    // all visible exports and entities added by hand or module instantiation by the interpreter
    // currently, all of the exports of an instantiated module is made visible (this is outside of spec)
    pub registry: Registry,
    pub user_data: T,

    // data structure holding all resumable objects that belong to this store
    pub dormitory: Dormitory,
}

impl<'b, T> Store<'b, T> {
    /// Creates a new empty store with some user data
    pub fn new(user_data: T) -> Self {
        Self {
            functions: Vec::default(),
            memories: Vec::default(),
            globals: Vec::default(),
            data: Vec::default(),
            tables: Vec::default(),
            elements: Vec::default(),
            modules: Vec::default(),
            registry: Registry::default(),
            dormitory: Dormitory::default(),
            user_data,
        }
    }

    /// instantiates a validated module with `validation_info` as validation evidence with name `name`
    /// with the steps in <https://webassembly.github.io/spec/core/exec/modules.html#instantiation>
    /// this method roughly matches the suggested embedder function`module_instantiate`
    /// <https://webassembly.github.io/spec/core/appendix/embedding.html#modules>
    /// except external values for module instantiation are retrieved from `self`.
    pub fn add_module(
        &mut self,
        name: &str,
        validation_info: &ValidationInfo<'b>,
        maybe_fuel: Option<u32>,
    ) -> Result<(), RuntimeError> {
        // instantiation step -1: collect extern_vals, this section basically acts as a linker between modules
        // best attempt at trying to match the spec implementation in terms of errors
        debug!("adding module with name {:?}", name);
        let mut extern_vals = Vec::new();

        for Import {
            module_name: exporting_module_name,
            name: import_name,
            desc: import_desc,
        } in &validation_info.imports
        {
            trace!(
                "trying to import from exporting module instance named {:?}, the entity with name {:?} with desc: {:?}",
                exporting_module_name,
                import_name,
                import_desc
            );
            let import_extern_type = import_desc.extern_type(validation_info);
            let export_extern_val_candidate = *self.registry.lookup(
                exporting_module_name.clone().into(),
                import_name.clone().into(),
            )?;
            trace!("export candidate found: {:?}", export_extern_val_candidate);
            if !export_extern_val_candidate
                .extern_type(self)
                .is_subtype_of(&import_extern_type)
            {
                return Err(RuntimeError::InvalidImportType);
            }
            trace!("import and export matches. Adding to externvals");
            extern_vals.push(export_extern_val_candidate)
        }

        // instantiation: step 5
        // module_inst_init is unfortunately circularly defined from parts of module_inst that would be defined in step 11, which uses module_inst_init again implicitly.
        // therefore I am mimicking the reference interpreter code here, I will allocate functions in the store in this step instead of step 11.
        // https://github.com/WebAssembly/spec/blob/8d6792e3d6709e8d3e90828f9c8468253287f7ed/interpreter/exec/eval.ml#L789
        let mut module_inst = ModuleInst {
            types: validation_info.types.clone(),
            func_addrs: extern_vals.iter().funcs().collect(),
            table_addrs: Vec::new(),
            mem_addrs: Vec::new(),
            global_addrs: extern_vals.iter().globals().collect(),
            elem_addrs: Vec::new(),
            data_addrs: Vec::new(),
            exports: BTreeMap::new(),
            wasm_bytecode: validation_info.wasm,
            sidetable: validation_info.sidetable.clone(),
        };

        // TODO rewrite this part
        // <https://webassembly.github.io/spec/core/exec/modules.html#functions>
        let func_addrs: Vec<usize> = validation_info
            .functions
            .iter()
            .zip(validation_info.func_blocks_stps.iter())
            .map(|(ty_idx, (span, stp))| {
                self.alloc_func((*ty_idx, (*span, *stp)), &module_inst, self.modules.len())
            })
            .collect();

        module_inst.func_addrs.extend(func_addrs);

        // instantiation: this roughly matches step 6,7,8
        // validation guarantees these will evaluate without errors.
        let maybe_global_init_vals: Result<Vec<Value>, _> = validation_info
            .globals
            .iter()
            .map(|global| {
                run_const_span(validation_info.wasm, &global.init_expr, &module_inst, self)
                    .transpose()
                    .unwrap_validated()
            })
            .collect();
        let global_init_vals = maybe_global_init_vals?;

        // instantiation: this roughly matches step 9,10

        let mut element_init_ref_lists: Vec<Vec<Ref>> =
            Vec::with_capacity(validation_info.elements.len());

        for elem in &validation_info.elements {
            let mut new_list = Vec::new();
            match &elem.init {
                // shortcut of evaluation of "ref.func <func_idx>; end;"
                // validation guarantees corresponding func_idx's existence
                ElemItems::RefFuncs(ref_funcs) => {
                    for func_idx in ref_funcs {
                        let func_addr = *module_inst
                            .func_addrs
                            .get(*func_idx as usize)
                            .unwrap_validated();

                        new_list.push(Ref::Func(FuncAddr(func_addr)));
                    }
                }
                ElemItems::Exprs(_, exprs) => {
                    for expr in exprs {
                        new_list.push(
                            run_const_span(validation_info.wasm, expr, &module_inst, self)?
                                .unwrap_validated() // there is a return value
                                .try_into()
                                .unwrap_validated(), // return value has the correct type
                        )
                    }
                }
            }
            element_init_ref_lists.push(new_list);
        }

        // instantiation: step 11 - module allocation (except function allocation - which was made in step 5)
        // https://webassembly.github.io/spec/core/exec/modules.html#alloc-module

        // allocation: begin

        // allocation: step 1
        let module = validation_info;

        let extern_vals = extern_vals;
        let vals = global_init_vals;
        let ref_lists = element_init_ref_lists;

        // allocation: skip step 2 as it was done in instantiation step 5

        // allocation: step 3-13
        let table_addrs: Vec<usize> = module
            .tables
            .iter()
            .map(|table_type| self.alloc_table(*table_type, Ref::Null(table_type.et)))
            .collect();
        let mem_addrs: Vec<usize> = module
            .memories
            .iter()
            .map(|mem_type| self.alloc_mem(*mem_type))
            .collect();
        let global_addrs: Vec<usize> = module
            .globals
            .iter()
            .zip(vals)
            .map(
                |(
                    Global {
                        ty: global_type, ..
                    },
                    val,
                )| self.alloc_global(*global_type, val),
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
            .map(|DataSegment { init: bytes, .. }| self.alloc_data(bytes))
            .collect();

        // allocation: skip step 14 as it was done in instantiation step 5

        // allocation: step 15,16
        let mut table_addrs_mod: Vec<usize> = extern_vals.iter().tables().collect();
        table_addrs_mod.extend(table_addrs);

        let mut mem_addrs_mod: Vec<usize> = extern_vals.iter().mems().collect();
        mem_addrs_mod.extend(mem_addrs);

        // skipping step 17 partially as it was partially done in instantiation step
        module_inst.global_addrs.extend(global_addrs);

        // allocation: step 18,19
        let export_insts: BTreeMap<String, ExternVal> = module
            .exports
            .iter()
            .map(|Export { name, desc }| {
                let value = match desc {
                    ExportDesc::FuncIdx(func_idx) => {
                        ExternVal::Func(module_inst.func_addrs[*func_idx])
                    }
                    ExportDesc::TableIdx(table_idx) => {
                        ExternVal::Table(table_addrs_mod[*table_idx])
                    }
                    ExportDesc::MemIdx(mem_idx) => ExternVal::Mem(mem_addrs_mod[*mem_idx]),
                    ExportDesc::GlobalIdx(global_idx) => {
                        ExternVal::Global(module_inst.global_addrs[*global_idx])
                    }
                };
                (String::from(name), value)
            })
            .collect();

        // allocation: step 20,21 initialize module (except functions and globals due to instantiation step 5, allocation step 14,17)
        module_inst.table_addrs = table_addrs_mod;
        module_inst.mem_addrs = mem_addrs_mod;
        module_inst.elem_addrs = elem_addrs;
        module_inst.data_addrs = data_addrs;
        module_inst.exports = export_insts;

        // allocation: end

        // register module exports, this is outside of the spec
        self.registry
            .register_module(name.to_owned().into(), &module_inst)?;

        // instantiation step 11 end: module_inst properly allocated after this point.
        // TODO: it is too hard with our codebase to do the following steps without adding the module to the store
        let current_module_idx = self.modules.len();
        self.modules.push(module_inst);

        // instantiation: step 12-15
        // TODO have to stray away from the spec a bit since our codebase does not lend itself well to freely executing instructions by themselves
        for (
            i,
            ElemType {
                init: elem_items,
                mode,
            },
        ) in validation_info.elements.iter().enumerate()
        {
            match mode {
                ElemMode::Active(ActiveElem {
                    table_idx: table_idx_i,
                    init_expr: einstr_i,
                }) => {
                    let n = elem_items.len() as i32;
                    // equivalent to init.len() in spec
                    // instantiation step 14:
                    // TODO (for now, we are doing hopefully what is equivalent to it)
                    // execute:
                    //   einstr_i
                    //   i32.const 0
                    //   i32.const n
                    //   table.init table_idx_i i
                    //   elem.drop i
                    let d: i32 = run_const_span(
                        validation_info.wasm,
                        einstr_i,
                        &self.modules[current_module_idx],
                        self,
                    )?
                    .unwrap_validated() // there is a return value
                    .try_into()
                    .unwrap_validated(); // return value has correct type

                    let s = 0;
                    table_init(
                        &self.modules,
                        &mut self.tables,
                        &self.elements,
                        current_module_idx,
                        i,
                        *table_idx_i as usize,
                        n,
                        s,
                        d,
                    )?;
                    elem_drop(&self.modules, &mut self.elements, current_module_idx, i)?;
                }
                ElemMode::Declarative => {
                    // instantiation step 15:
                    // TODO (for now, we are doing hopefully what is equivalent to it)
                    // execute:
                    //   elem.drop i
                    elem_drop(&self.modules, &mut self.elements, current_module_idx, i)?;
                }
                ElemMode::Passive => (),
            }
        }

        // instantiation: step 16
        // TODO have to stray away from the spec a bit since our codebase does not lend itself well to freely executing instructions by themselves
        for (i, DataSegment { init, mode }) in validation_info.data.iter().enumerate() {
            match mode {
                crate::core::reader::types::data::DataMode::Active(DataModeActive {
                    memory_idx,
                    offset: dinstr_i,
                }) => {
                    let n = init.len() as i32;
                    // assert: mem_idx is 0
                    if *memory_idx != 0 {
                        // TODO fix error
                        return Err(RuntimeError::MoreThanOneMemory);
                    }

                    // TODO (for now, we are doing hopefully what is equivalent to it)
                    // execute:
                    //   dinstr_i
                    //   i32.const 0
                    //   i32.const n
                    //   memory.init i
                    //   data.drop i
                    let d: i32 = run_const_span(
                        validation_info.wasm,
                        dinstr_i,
                        &self.modules[current_module_idx],
                        self,
                    )?
                    .unwrap_validated() // there is a return value
                    .try_into()
                    .unwrap_validated(); // return value has the correct type

                    let s = 0;
                    memory_init(
                        &self.modules,
                        &mut self.memories,
                        &self.data,
                        current_module_idx,
                        i,
                        0,
                        n,
                        s,
                        d,
                    )?;
                    data_drop(&self.modules, &mut self.data, current_module_idx, i)?;
                }
                crate::core::reader::types::data::DataMode::Passive => (),
            }
        }

        // instantiation: step 17
        if let Some(func_idx) = validation_info.start {
            // TODO (for now, we are doing hopefully what is equivalent to it)
            // execute
            //   call func_ifx
            let func_addr = self.modules[current_module_idx].func_addrs[func_idx];
            self.invoke(func_addr, Vec::new(), maybe_fuel)?;
        };

        Ok(())
    }

    /// roughly matches <https://webassembly.github.io/spec/core/exec/modules.html#functions> with the addition of sidetable pointer to the input signature
    // TODO refactor the type of func
    // TODO module_addr
    fn alloc_func(
        &mut self,
        func: (TypeIdx, (Span, usize)),
        module_inst: &ModuleInst,
        module_addr: usize,
    ) -> usize {
        let (ty, (span, stp)) = func;

        // TODO rewrite this huge chunk of parsing after generic way to re-parse(?) structs lands
        let mut wasm_reader = WasmReader::new(module_inst.wasm_bytecode);
        wasm_reader.move_start_to(span).unwrap_validated();

        let (locals, bytes_read) = wasm_reader
            .measure_num_read_bytes(crate::code::read_declared_locals)
            .unwrap_validated();

        let code_expr = wasm_reader
            .make_span(span.len() - bytes_read)
            .unwrap_validated();

        // core of the method below

        // validation guarantees func_ty_idx exists within module_inst.types
        // TODO fix clone
        let func_inst = FuncInst::WasmFunc(WasmFuncInst {
            function_type: module_inst.types[ty].clone(),
            ty,
            locals,
            code_expr,
            stp,
            module_addr,
        });

        let addr = self.functions.len();
        self.functions.push(func_inst);
        addr
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#host-functions>
    pub(super) fn alloc_host_func(
        &mut self,
        func_type: FuncType,
        host_func: fn(&mut T, Vec<Value>) -> Vec<Value>,
    ) -> usize {
        let func_inst = FuncInst::HostFunc(HostFuncInst {
            function_type: func_type,
            hostcode: host_func,
        });
        let addr = self.functions.len();
        self.functions.push(func_inst);
        addr
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#tables>
    fn alloc_table(&mut self, table_type: TableType, reff: Ref) -> usize {
        let table_inst = TableInst {
            ty: table_type,
            elem: vec![reff; table_type.lim.min as usize],
        };

        let addr = self.tables.len();
        self.tables.push(table_inst);
        addr
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#memories>
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

    /// <https://webassembly.github.io/spec/core/exec/modules.html#globals>
    fn alloc_global(&mut self, global_type: GlobalType, val: Value) -> usize {
        let global_inst = GlobalInst {
            ty: global_type,
            value: val,
        };

        let addr = self.globals.len();
        self.globals.push(global_inst);
        addr
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#element-segments>
    fn alloc_elem(&mut self, ref_type: RefType, refs: Vec<Ref>) -> usize {
        let elem_inst = ElemInst {
            ty: ref_type,
            references: refs,
        };

        let addr = self.elements.len();
        self.elements.push(elem_inst);
        addr
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#data-segments>
    fn alloc_data(&mut self, bytes: &[u8]) -> usize {
        let data_inst = DataInst {
            data: Vec::from(bytes),
        };

        let addr = self.data.len();
        self.data.push(data_inst);
        addr
    }

    pub fn invoke(
        &mut self,
        func_addr: usize,
        params: Vec<Value>,
        maybe_fuel: Option<u32>,
    ) -> Result<RunState, RuntimeError> {
        let func_inst = self
            .functions
            .get(func_addr)
            .ok_or(RuntimeError::FunctionNotFound)?;

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

        match &func_inst {
            FuncInst::HostFunc(host_func_inst) => {
                let returns = (host_func_inst.hostcode)(&mut self.user_data, params);
                debug!("Successfully invoked function");

                // Verify that the return parameters match the host function parameters
                // since we have no validation guarantees for host functions

                let return_types = returns.iter().map(|v| v.to_ty()).collect::<Vec<_>>();
                if func_ty.returns.valtypes != return_types {
                    trace!(
                        "Func param types len: {}; Given args len: {}",
                        func_ty.params.valtypes.len(),
                        param_types.len()
                    );
                    return Err(RuntimeError::HostFunctionSignatureMismatch);
                }

                Ok(RunState::Finished(returns))
            }
            FuncInst::WasmFunc(wasm_func_inst) => {
                // Prepare a new stack with the locals for the entry function
                let mut stack = Stack::new_with_values(params);

                stack.push_call_frame(
                    usize::MAX,
                    &func_ty,
                    &wasm_func_inst.locals,
                    usize::MAX,
                    usize::MAX,
                )?;

                let mut resumable = Resumable {
                    current_func_addr: func_addr,
                    stack,
                    pc: wasm_func_inst.code_expr.from,
                    stp: wasm_func_inst.stp + 1, // skip dummy sidetable entry at the beginning of the func
                    maybe_fuel,
                };

                // check if there is enough fuel
                if let Some(fuel) = maybe_fuel {
                    let required_fuel = self.modules[wasm_func_inst.module_addr].sidetable
                        [wasm_func_inst.stp]
                        .delta_fuel;
                    if required_fuel > fuel {
                        return Ok(RunState::Resumable {
                            resumable_ref: self.dormitory.insert(resumable),
                            required_fuel,
                        });
                    }
                }

                // Run the interpreter
                let result = interpreter_loop::run(&mut resumable, self, EmptyHookSet);

                match result {
                    Ok(()) => {
                        debug!("Successfully invoked function");
                        Ok(RunState::Finished(resumable.stack.into_values()))
                    }
                    Err(RuntimeError::OutOfFuel { required_fuel }) => {
                        debug!("Successfully invoked function, but ran out of fuel");
                        Ok(RunState::Resumable {
                            resumable_ref: self.dormitory.insert(resumable),
                            required_fuel,
                        })
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
}

#[derive(Debug)]
// TODO does not match the spec FuncInst
pub enum FuncInst<T> {
    WasmFunc(WasmFuncInst),
    HostFunc(HostFuncInst<T>),
}

#[derive(Debug)]
pub struct WasmFuncInst {
    pub function_type: FuncType,
    pub ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
    ///index of the sidetable corresponding to the beginning of this functions code
    pub stp: usize,

    // implicit back ref required for function invocation and is in the spec
    // TODO module_addr or module ref?
    pub module_addr: usize,
}

#[derive(Debug)]
pub struct HostFuncInst<T> {
    pub function_type: FuncType,
    pub hostcode: fn(&mut T, Vec<Value>) -> Vec<Value>,
}

impl<T> FuncInst<T> {
    pub fn ty(&self) -> FuncType {
        match self {
            FuncInst::WasmFunc(wasm_func_inst) => wasm_func_inst.function_type.clone(),
            FuncInst::HostFunc(host_func_inst) => host_func_inst.function_type.clone(),
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
            elem: vec![Ref::Null(ty.et); ty.lim.min as usize],
        }
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#growing-tables>
    pub fn grow(&mut self, n: u32, reff: Ref) -> Result<(), RuntimeError> {
        // TODO refactor error, the spec Table.grow raises Table.{SizeOverflow, SizeLimit, OutOfMemory}
        let len = n
            .checked_add(self.elem.len() as u32)
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

        // roughly matches step 4,5,6
        // checks limits_prime.valid() for limits_prime := { min: len, max: self.ty.lim.max }
        // https://webassembly.github.io/spec/core/valid/types.html#limits
        if self.ty.lim.max.map(|max| len > max).unwrap_or(false) {
            return Err(TrapError::TableOrElementAccessOutOfBounds.into());
        }
        let limits_prime = Limits {
            min: len,
            max: self.ty.lim.max,
        };

        self.elem.extend(vec![reff; n as usize]);

        self.ty.lim = limits_prime;
        Ok(())
    }
}

pub struct MemInst {
    #[allow(warnings)]
    pub ty: MemType,
    pub mem: LinearMemory,
}
impl core::fmt::Debug for MemInst {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MemInst")
            .field("ty", &self.ty)
            .finish_non_exhaustive()
    }
}

impl MemInst {
    pub fn new(ty: MemType) -> Self {
        Self {
            ty,
            mem: LinearMemory::new_with_initial_pages(ty.limits.min.try_into().unwrap()),
        }
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#growing-memories>
    pub fn grow(&mut self, n: u32) -> Result<(), RuntimeError> {
        // TODO refactor error, the spec Table.grow raises Memory.{SizeOverflow, SizeLimit, OutOfMemory}
        let len = n + self.mem.pages() as u32;
        if len > Limits::MAX_MEM_PAGES {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // roughly matches step 4,5,6
        // checks limits_prime.valid() for limits_prime := { min: len, max: self.ty.lim.max }
        // https://webassembly.github.io/spec/core/valid/types.html#limits
        if self.ty.limits.max.map(|max| len > max).unwrap_or(false) {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }
        let limits_prime = Limits {
            min: len,
            max: self.ty.limits.max,
        };

        self.mem.grow(n.try_into().unwrap());

        self.ty.limits = limits_prime;
        Ok(())
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

impl core::fmt::Debug for DataInst {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DataInst").finish_non_exhaustive()
    }
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
    ///
    /// Note: This method may panic if self does not come from the given [`Store`].
    ///<https://webassembly.github.io/spec/core/valid/modules.html#imports>
    pub fn extern_type<T>(&self, store: &Store<T>) -> ExternType {
        match self {
            // TODO: fix ugly clone in function types
            ExternVal::Func(func_addr) => ExternType::Func(
                store
                    .functions
                    .get(*func_addr)
                    .expect("the correct store to be used")
                    .ty(),
            ),
            ExternVal::Table(table_addr) => ExternType::Table(
                store
                    .tables
                    .get(*table_addr)
                    .expect("the correct store to be used")
                    .ty,
            ),
            ExternVal::Mem(mem_addr) => ExternType::Mem(
                store
                    .memories
                    .get(*mem_addr)
                    .expect("the correct store to be used")
                    .ty,
            ),
            ExternVal::Global(global_addr) => ExternType::Global(
                store
                    .globals
                    .get(*global_addr)
                    .expect("the correct store to be used")
                    .ty,
            ),
        }
    }
}

/// common convention functions defined for lists of ExternVals, ExternTypes, Exports
/// <https://webassembly.github.io/spec/core/exec/runtime.html#conventions>
/// <https://webassembly.github.io/spec/core/syntax/types.html#id3>
/// <https://webassembly.github.io/spec/core/syntax/modules.html?highlight=convention#id1>
// TODO implement this trait for ExternType lists Export lists
pub trait ExternFilterable<T> {
    fn funcs(self) -> impl Iterator<Item = T>;
    fn tables(self) -> impl Iterator<Item = T>;
    fn mems(self) -> impl Iterator<Item = T>;
    fn globals(self) -> impl Iterator<Item = T>;
}

impl<'a, I> ExternFilterable<usize> for I
where
    I: Iterator<Item = &'a ExternVal>,
{
    fn funcs(self) -> impl Iterator<Item = usize> {
        self.filter_map(|extern_val| {
            if let ExternVal::Func(func_addr) = extern_val {
                Some(*func_addr)
            } else {
                None
            }
        })
    }

    fn tables(self) -> impl Iterator<Item = usize> {
        self.filter_map(|extern_val| {
            if let ExternVal::Table(table_addr) = extern_val {
                Some(*table_addr)
            } else {
                None
            }
        })
    }

    fn mems(self) -> impl Iterator<Item = usize> {
        self.filter_map(|extern_val| {
            if let ExternVal::Mem(mem_addr) = extern_val {
                Some(*mem_addr)
            } else {
                None
            }
        })
    }

    fn globals(self) -> impl Iterator<Item = usize> {
        self.filter_map(|extern_val| {
            if let ExternVal::Global(global_addr) = extern_val {
                Some(*global_addr)
            } else {
                None
            }
        })
    }
}

///<https://webassembly.github.io/spec/core/exec/runtime.html#module-instances>
#[derive(Debug)]
pub struct ModuleInst<'b> {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<usize>,
    pub table_addrs: Vec<usize>,
    pub mem_addrs: Vec<usize>,
    pub global_addrs: Vec<usize>,
    pub elem_addrs: Vec<usize>,
    pub data_addrs: Vec<usize>,
    ///<https://webassembly.github.io/spec/core/exec/runtime.html#export-instances>
    /// matches the list of ExportInst structs in the spec, however the spec never uses the name attribute
    /// except during linking, which is up to the embedder to implement.
    /// therefore this is a map data structure instead.
    pub exports: BTreeMap<String, ExternVal>,

    // TODO the bytecode is not in the spec, but required for re-parsing
    pub wasm_bytecode: &'b [u8],

    // sidetable is not in the spec, but required for control flow
    pub sidetable: Sidetable,
}
