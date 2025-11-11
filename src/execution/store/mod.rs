use core::mem;

use crate::addrs::{
    AddrVec, DataAddr, ElemAddr, FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr,
};
use crate::config::Config;
use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::data::{DataModeActive, DataSegment};
use crate::core::reader::types::element::{ActiveElem, ElemItems, ElemMode, ElemType};
use crate::core::reader::types::export::{Export, ExportDesc};
use crate::core::reader::types::global::{Global, GlobalType};
use crate::core::reader::types::import::Import;
use crate::core::reader::types::{ExternType, FuncType, ImportSubTypeRelation, MemType, TableType};
use crate::core::reader::WasmReader;
use crate::execution::interpreter_loop::{self, memory_init, table_init};
use crate::execution::value::{Ref, Value};
use crate::execution::{run_const_span, Stack};
use crate::registry::Registry;
use crate::resumable::{
    Dormitory, FreshResumableRef, InvokedResumableRef, Resumable, ResumableRef, RunState,
};
use crate::{RefType, RuntimeError, ValidationInfo};
use alloc::borrow::ToOwned;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use instances::{
    DataInst, ElemInst, FuncInst, GlobalInst, HostFuncInst, MemInst, ModuleInst, TableInst,
    WasmFuncInst,
};
use linear_memory::LinearMemory;

use super::interpreter_loop::{data_drop, elem_drop};
use super::UnwrapValidatedExt;

pub mod addrs;
pub(crate) mod instances;
pub(crate) mod linear_memory;

/// The store represents all global state that can be manipulated by WebAssembly programs. It
/// consists of the runtime representation of all instances of functions, tables, memories, and
/// globals, element segments, and data segments that have been allocated during the life time of
/// the abstract machine.
/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
pub struct Store<'b, T: Config> {
    pub(crate) functions: AddrVec<FuncAddr, FuncInst<T>>,
    pub(crate) tables: AddrVec<TableAddr, TableInst>,
    pub(crate) memories: AddrVec<MemAddr, MemInst>,
    pub(crate) globals: AddrVec<GlobalAddr, GlobalInst>,
    pub(crate) elements: AddrVec<ElemAddr, ElemInst>,
    pub(crate) data: AddrVec<DataAddr, DataInst>,

    // fields outside of the spec but are convenient are below
    /// An address space of modules instantiated within the context of this [`Store`].
    ///
    /// Although the WebAssembly Specification 2.0 does not specify module instances
    /// to be part of the [`Store`], in reality they can be managed very similar to
    /// other instance types. Therefore, we extend the [`Store`] by a module address
    /// space along with a `ModuleAddr` index type.
    pub(crate) modules: AddrVec<ModuleAddr, ModuleInst<'b>>,

    // all visible exports and entities added by hand or module instantiation by the interpreter
    // currently, all of the exports of an instantiated module is made visible (this is outside of spec)
    pub registry: Registry,
    pub user_data: T,

    // data structure holding all resumable objects that belong to this store
    pub(crate) dormitory: Dormitory,
}

impl<'b, T: Config> Store<'b, T> {
    /// Creates a new empty store with some user data
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.4 - store_init
    pub fn new(user_data: T) -> Self {
        // 1. Return the empty store.
        // For us the store is empty except for the user data, which we do not have control over.
        Self {
            functions: AddrVec::default(),
            tables: AddrVec::default(),
            memories: AddrVec::default(),
            globals: AddrVec::default(),
            elements: AddrVec::default(),
            data: AddrVec::default(),
            modules: AddrVec::default(),
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
    /// Returns the module addr of the new module instance
    pub fn add_module(
        &mut self,
        name: &str,
        validation_info: &ValidationInfo<'b>,
        maybe_fuel: Option<u32>,
    ) -> Result<ModuleAddr, RuntimeError> {
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
        let module_inst = ModuleInst {
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
        let module_addr = self.modules.insert(module_inst);

        // TODO rewrite this part
        // <https://webassembly.github.io/spec/core/exec/modules.html#functions>
        let func_addrs: Vec<FuncAddr> = validation_info
            .functions
            .iter()
            .zip(validation_info.func_blocks_stps.iter())
            .map(|(ty_idx, (span, stp))| self.alloc_func((*ty_idx, (*span, *stp)), module_addr))
            .collect();

        self.modules
            .get_mut(module_addr)
            .func_addrs
            .extend(func_addrs);

        // instantiation: this roughly matches step 6,7,8
        // validation guarantees these will evaluate without errors.
        let maybe_global_init_vals: Result<Vec<Value>, _> = validation_info
            .globals
            .iter()
            .map(|global| {
                run_const_span(validation_info.wasm, &global.init_expr, module_addr, self)
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
                        let func_addr = *self
                            .modules
                            .get(module_addr)
                            .func_addrs
                            .get(*func_idx as usize)
                            .unwrap_validated();

                        new_list.push(Ref::Func(func_addr));
                    }
                }
                ElemItems::Exprs(_, exprs) => {
                    for expr in exprs {
                        new_list.push(
                            run_const_span(validation_info.wasm, expr, module_addr, self)?
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
        let table_addrs: Vec<TableAddr> = module
            .tables
            .iter()
            .map(|table_type| self.alloc_table(*table_type, Ref::Null(table_type.et)))
            .collect();
        let mem_addrs: Vec<MemAddr> = module
            .memories
            .iter()
            .map(|mem_type| self.alloc_mem(*mem_type))
            .collect();
        let global_addrs: Vec<GlobalAddr> = module
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
        let mut table_addrs_mod: Vec<TableAddr> = extern_vals.iter().tables().collect();
        table_addrs_mod.extend(table_addrs);

        let mut mem_addrs_mod: Vec<MemAddr> = extern_vals.iter().mems().collect();
        mem_addrs_mod.extend(mem_addrs);

        // skipping step 17 partially as it was partially done in instantiation step
        self.modules
            .get_mut(module_addr)
            .global_addrs
            .extend(global_addrs);

        // allocation: step 18,19
        let export_insts: BTreeMap<String, ExternVal> = module
            .exports
            .iter()
            .map(|Export { name, desc }| {
                let module_inst = self.modules.get(module_addr);
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
        let module_inst = self.modules.get_mut(module_addr);
        module_inst.table_addrs = table_addrs_mod;
        module_inst.mem_addrs = mem_addrs_mod;
        module_inst.elem_addrs = elem_addrs;
        module_inst.data_addrs = data_addrs;
        module_inst.exports = export_insts;

        // allocation: end

        // register module exports, this is outside of the spec
        self.registry
            .register_module(name.to_owned().into(), module_inst, module_addr)?;

        // instantiation step 11 end: module_inst properly allocated after this point.

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
                    let d: i32 = run_const_span(validation_info.wasm, einstr_i, module_addr, self)?
                        .unwrap_validated() // there is a return value
                        .try_into()
                        .unwrap_validated(); // return value has correct type

                    let s = 0;
                    table_init(
                        &self.modules,
                        &mut self.tables,
                        &self.elements,
                        module_addr,
                        i,
                        *table_idx_i as usize,
                        n,
                        s,
                        d,
                    )?;
                    elem_drop(&self.modules, &mut self.elements, module_addr, i)?;
                }
                ElemMode::Declarative => {
                    // instantiation step 15:
                    // TODO (for now, we are doing hopefully what is equivalent to it)
                    // execute:
                    //   elem.drop i
                    elem_drop(&self.modules, &mut self.elements, module_addr, i)?;
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
                    let d: i32 = run_const_span(validation_info.wasm, dinstr_i, module_addr, self)?
                        .unwrap_validated() // there is a return value
                        .try_into()
                        .unwrap_validated(); // return value has the correct type

                    let s = 0;
                    memory_init(
                        &self.modules,
                        &mut self.memories,
                        &self.data,
                        module_addr,
                        i,
                        0,
                        n,
                        s,
                        d,
                    )?;
                    data_drop(&self.modules, &mut self.data, module_addr, i)?;
                }
                crate::core::reader::types::data::DataMode::Passive => (),
            }
        }

        // instantiation: step 17
        if let Some(func_idx) = validation_info.start {
            // TODO (for now, we are doing hopefully what is equivalent to it)
            // execute
            //   call func_ifx
            let func_addr = self.modules.get(module_addr).func_addrs[func_idx];
            self.invoke(func_addr, Vec::new(), maybe_fuel)?;
        };

        Ok(module_addr)
    }

    /// Gets an export of a specific module instance by its name
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.6 - instance_export
    pub fn instance_export(
        &self,
        module_addr: ModuleAddr,
        name: &str,
    ) -> Result<ExternVal, RuntimeError> {
        // Fetch the module instance because we store them in the [`Store`]
        let module_inst = self.modules.get(module_addr);

        // 1. Assert: due to validity of the module instance `moduleinst`, all its export names are different

        // 2. If there exists an `exportinst_i` in `moduleinst.exports` such that name `exportinst_i.name` equals `name`, then:
        //   a. Return the external value `exportinst_i.value`.
        // 3. Else return `error`.
        module_inst
            .exports
            .get(name)
            .copied()
            .ok_or(RuntimeError::UnknownExport)
    }

    /// Allocates a new function with some host code.
    ///
    /// This type of function is also called a host function.
    ///
    /// # Panics & Unexpected Behavior
    /// The specification states that:
    ///
    /// > This operation must make sure that the provided host function satisfies the pre-
    /// > and post-conditions required for a function instance with type `functype`.
    ///
    /// Therefore, all "invalid" host functions (e.g. those which return incorrect return values)
    /// can cause the interpreter to panic or behave unexpectedly.
    ///
    /// See: <https://webassembly.github.io/spec/core/exec/modules.html#host-functions>
    /// See: WebAssembly Specification 2.0 - 7.1.7 - func_alloc
    pub fn func_alloc(
        &mut self,
        func_type: FuncType,
        host_func: fn(&mut T, Vec<Value>) -> Result<Vec<Value>, HaltExecutionError>,
    ) -> FuncAddr {
        // 1. Pre-condition: `functype` is valid.

        // 2. Let `funcaddr` be the result of allocating a host function in `store` with
        //    function type `functype` and host function code `hostfunc`.
        // 3. Return the new store paired with `funcaddr`.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        self.functions.insert(FuncInst::HostFunc(HostFuncInst {
            function_type: func_type,
            hostcode: host_func,
        }))
    }

    /// Gets the type of a function by its addr.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.7 - func_type
    pub fn func_type(&self, func_addr: FuncAddr) -> FuncType {
        // 1. Return `S.funcs[a].type`.
        self.functions.get(func_addr).ty()

        // 2. Post-condition: the returned function type is valid.
    }

    /// See: WebAssembly Specification 2.0 - 7.1.7 - func_invoke
    pub fn invoke(
        &mut self,
        func_addr: FuncAddr,
        params: Vec<Value>,
        maybe_fuel: Option<u32>,
    ) -> Result<RunState, RuntimeError> {
        self.resume(self.create_resumable(func_addr, params, maybe_fuel)?)
    }

    /// Allocates a new table with some table type and an initialization value `ref` and returns its table address.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.8 - table_alloc
    pub fn table_alloc(
        &mut self,
        table_type: TableType,
        r#ref: Ref,
    ) -> Result<TableAddr, RuntimeError> {
        // Check pre-condition: ref has correct type
        if table_type.et != r#ref.ty() {
            return Err(RuntimeError::TableTypeMismatch);
        }

        // 1. Pre-condition: `tabletype` is valid

        // 2. Let `tableaddr` be the result of allocating a table in `store` with table type `tabletype`
        //    and initialization value `ref`.
        let table_addr = self.alloc_table(table_type, r#ref);

        // 3. Return the new store paired with `tableaddr`.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        Ok(table_addr)
    }

    /// Gets the type of some table by its addr.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.8 - table_type
    pub fn table_type(&self, table_addr: TableAddr) -> TableType {
        // 1. Return `S.tables[a].type`.
        self.tables.get(table_addr).ty

        // 2. Post-condition: the returned table type is valid.
    }

    /// Reads a single reference from a table by its table address and an index into the table.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.8 - table_read
    pub fn table_read(&self, table_addr: TableAddr, i: u32) -> Result<Ref, RuntimeError> {
        // Convert `i` to usize for indexing
        let i = usize::try_from(i).expect("the architecture to be at least 32-bit");

        // 1. Let `ti` be the table instance `store.tables[tableaddr]`
        let ti = self.tables.get(table_addr);

        // 2. If `i` is larger than or equal to the length of `ti.elem`, then return `error`.
        // 3. Else, return the reference value `ti.elem[i]`.
        ti.elem
            .get(i)
            .copied()
            .ok_or(RuntimeError::TableAccessOutOfBounds)
    }

    /// Writes a single reference into a table by its table address and an index into the table.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.8 - table_write
    pub fn table_write(
        &mut self,
        table_addr: TableAddr,
        i: u32,
        r#ref: Ref,
    ) -> Result<(), RuntimeError> {
        // Convert `i` to usize for indexing
        let i = usize::try_from(i).expect("the architecture to be at least 32-bit");

        // 1. Let `ti` be the table instance `store.tables[tableaddr]`.
        let ti = self.tables.get_mut(table_addr);

        // Check pre-condition: ref has correct type
        if ti.ty.et != r#ref.ty() {
            return Err(RuntimeError::TableTypeMismatch);
        }

        // 2. If `i` is larger than or equal to the length of `ti.elem`, then return `error`.
        // 3. Replace `ti.elem[i]` with the reference value `ref`
        *ti.elem
            .get_mut(i)
            .ok_or(RuntimeError::TableAccessOutOfBounds)? = r#ref;

        // 4. Return the updated store.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        Ok(())
    }

    /// Gets the current size of a table by its table address.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.8 - table_size
    pub fn table_size(&self, table_addr: TableAddr) -> u32 {
        // 1. Return the length of `store.tables[tableaddr].elem`.
        let len = self.tables.get(table_addr).elem.len();

        // In addition we have to convert the length back to a `u32`
        u32::try_from(len).expect(
            "the maximum table length to be u32::MAX because thats what the specification allows for indexing",
        )
    }

    /// Grows a table referenced by its table address by `n` elements.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.8 - table_grow
    pub fn table_grow(
        &mut self,
        table_addr: TableAddr,
        n: u32,
        r#ref: Ref,
    ) -> Result<(), RuntimeError> {
        // 1. Try growing the table instance `store.tables[tableaddr] by `n` elements with initialization value `ref`:
        //   a. If it succeeds, return the updated store.
        //   b. Else, return `error`.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        self.tables.get_mut(table_addr).grow(n, r#ref)
    }

    /// Allocates a new linear memory and returns its memory address.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.9 - mem_alloc
    pub fn mem_alloc(&mut self, mem_type: MemType) -> MemAddr {
        // 1. Pre-condition: `memtype` is valid.

        // 2. Let `memaddr` be the result of allocating a memory in `store` with memory type `memtype`.
        // 3. Return the new store paired with `memaddr`.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        self.alloc_mem(mem_type)
    }

    /// Gets the memory type of some memory by its memory address
    ///
    /// See: WebAssemblySpecification 2.0 - 7.1.9 - mem_type
    pub fn mem_type(&self, mem_addr: MemAddr) -> MemType {
        // 1. Return `S.mems[a].type`.
        self.memories.get(mem_addr).ty

        // 2. Post-condition: the returned memory type is valid.
    }

    /// Reads a byte from some memory by its memory address and an index into the memory
    ///
    /// See: WebAssemblySpecification 2.0 - 7.1.9 - mem_read
    pub fn mem_read(&self, mem_addr: MemAddr, i: u32) -> Result<u8, RuntimeError> {
        // Convert the index type
        let i = usize::try_from(i).expect("the architecture to be at least 32-bit");

        // 1. Let `mi` be the memory instance `store.mems[memaddr]`.
        let mi = self.memories.get(mem_addr);

        // 2. If `i` is larger than or equal to the length of `mi.data`, then return `error`.
        // 3. Else, return the byte `mi.data[i]`.
        mi.mem.load(i)
    }

    /// Writes a byte into some memory by its memory address and an index into the memory
    ///
    /// See: WebAssemblySpecification 2.0 - 7.1.9 - mem_write
    pub fn mem_write(&self, mem_addr: MemAddr, i: u32, byte: u8) -> Result<(), RuntimeError> {
        // Convert the index type
        let i = usize::try_from(i).expect("the architecture to be at least 32-bit");

        // 1. Let `mi` be the memory instance `store.mems[memaddr]`.
        let mi = self.memories.get(mem_addr);

        mi.mem.store(i, byte)
    }

    /// Gets the size of some memory by its memory address in pages.
    ///
    /// See: WebAssemblySpecification 2.0 - 7.1.9 - mem_size
    pub fn mem_size(&self, mem_addr: MemAddr) -> u32 {
        // 1. Return the length of `store.mems[memaddr].data` divided by the page size.
        let length = self.memories.get(mem_addr).size();

        // In addition we have to convert the length back to a `u32`
        length.try_into().expect(
            "the maximum memory length to be smaller than u32::MAX because thats what the specification allows for indexing into the memory. Also the memory size is measured in pages, not bytes.")
    }

    /// Grows some memory by its memory address by `n` pages.
    ///
    /// See: WebAssemblySpecification 2.0 - 7.1.9 - mem_grow
    pub fn mem_grow(&mut self, mem_addr: MemAddr, n: u32) -> Result<(), RuntimeError> {
        // 1. Try growing the memory instance `store.mems[memaddr]` by `n` pages:
        //   a. If it succeeds, then return the updated store.
        //   b. Else, return `error`.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        self.memories.get_mut(mem_addr).grow(n)
    }

    /// Allocates a new global and returns its global address.
    ///
    /// See: WebAssemblySpecification 2.0 - 7.1.10 - global_alloc
    pub fn global_alloc(
        &mut self,
        global_type: GlobalType,
        val: Value,
    ) -> Result<GlobalAddr, RuntimeError> {
        // Check pre-condition: val has correct type
        if global_type.ty != val.to_ty() {
            return Err(RuntimeError::GlobalTypeMismatch);
        }

        // 1. Pre-condition: `globaltype` is valid.

        // 2. Let `globaladdr` be the result of allocating a global with global type `globaltype` and initialization value `val`.
        let global_addr = self.alloc_global(global_type, val);

        // 3. Return the new store paired with `globaladdr`.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        Ok(global_addr)
    }

    /// Returns the global type of some global instance by its addr.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.10 - global_type
    pub fn global_type(&self, global_addr: GlobalAddr) -> GlobalType {
        // 1. Return `S.globals[a].type`.
        self.globals.get(global_addr).ty
        // 2. Post-condition: the returned global type is valid
    }

    /// Returns the current value of some global instance by its addr.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.10 - global_read
    pub fn global_read(&self, global_addr: GlobalAddr) -> Value {
        // 1. Let `gi` be the global instance `store.globals[globaladdr].
        let gi = self.globals.get(global_addr);

        // 2. Return the value `gi.value`.
        gi.value
    }

    /// Sets a new value of some global instance by its addr.
    ///
    /// # Errors
    /// - [` RuntimeError::WriteOnImmutableGlobal`]
    /// - [` RuntimeError::GlobalTypeMismatch`]
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.10 - global_write
    pub fn global_write(
        &mut self,
        global_addr: GlobalAddr,
        val: Value,
    ) -> Result<(), RuntimeError> {
        // 1. Let `gi` be the global instance `store.globals[globaladdr]`.
        let gi = self.globals.get_mut(global_addr);

        // 2. Let `mut t` be the structure of the global type `gi.type`.
        let r#mut = gi.ty.is_mut;
        let t = gi.ty.ty;

        // 3. If `mut` is not `var`, then return error.
        if !r#mut {
            return Err(RuntimeError::WriteOnImmutableGlobal);
        }

        // Check invariant:
        //   It is an invariant of the semantics that the value has a type equal to the value type of `globaltype`.
        // See: WebAssembly Specification 2.0 - 4.2.9
        if t != val.to_ty() {
            return Err(RuntimeError::GlobalTypeMismatch);
        }

        // 4. Replace `gi.value` with the value `val`.
        gi.value = val;

        // 5. Return the updated store.
        // This is a noop for us, as our store `self` is mutable.

        Ok(())
    }

    /// roughly matches <https://webassembly.github.io/spec/core/exec/modules.html#functions> with the addition of sidetable pointer to the input signature
    // TODO refactor the type of func
    // TODO module_addr
    fn alloc_func(&mut self, func: (TypeIdx, (Span, usize)), module_addr: ModuleAddr) -> FuncAddr {
        let (ty, (span, stp)) = func;

        // TODO rewrite this huge chunk of parsing after generic way to re-parse(?) structs lands
        let mut wasm_reader = WasmReader::new(self.modules.get(module_addr).wasm_bytecode);
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
            function_type: self.modules.get(module_addr).types[ty].clone(),
            _ty: ty,
            locals,
            code_expr,
            stp,
            module_addr,
        });
        self.functions.insert(func_inst)
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#tables>
    fn alloc_table(&mut self, table_type: TableType, reff: Ref) -> TableAddr {
        let table_inst = TableInst {
            ty: table_type,
            elem: vec![reff; table_type.lim.min as usize],
        };

        self.tables.insert(table_inst)
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#memories>
    fn alloc_mem(&mut self, mem_type: MemType) -> MemAddr {
        let mem_inst = MemInst {
            ty: mem_type,
            mem: LinearMemory::new_with_initial_pages(
                mem_type.limits.min.try_into().unwrap_validated(),
            ),
        };

        self.memories.insert(mem_inst)
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#globals>
    fn alloc_global(&mut self, global_type: GlobalType, val: Value) -> GlobalAddr {
        let global_inst = GlobalInst {
            ty: global_type,
            value: val,
        };

        self.globals.insert(global_inst)
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#element-segments>
    fn alloc_elem(&mut self, ref_type: RefType, refs: Vec<Ref>) -> ElemAddr {
        let elem_inst = ElemInst {
            _ty: ref_type,
            references: refs,
        };

        self.elements.insert(elem_inst)
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#data-segments>
    fn alloc_data(&mut self, bytes: &[u8]) -> DataAddr {
        let data_inst = DataInst {
            data: Vec::from(bytes),
        };

        self.data.insert(data_inst)
    }

    /// This function allows an already instantiated module to be reregistered
    /// under a different name. All previous registers of this module are not
    /// affected.
    ///
    /// Note: This method exists as a temporary solution because our suboptimal registry
    /// design. Because [`Store::add_module`] automatically registers all
    /// modules directly after instantiation, we still need to provide some way
    /// for only registering a module.
    pub fn reregister_module(
        &mut self,
        module_addr: ModuleAddr,
        name: &str,
    ) -> Result<(), RuntimeError> {
        self.registry.register_module(
            name.to_owned().into(),
            self.modules.get(module_addr),
            module_addr,
        )
    }

    pub fn create_resumable(
        &self,
        func_addr: FuncAddr,
        params: Vec<Value>,
        maybe_fuel: Option<u32>,
    ) -> Result<ResumableRef, RuntimeError> {
        let func_inst = self.functions.get(func_addr);

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

        Ok(ResumableRef::Fresh(FreshResumableRef {
            func_addr,
            params,
            maybe_fuel,
        }))
    }

    pub fn resume(&mut self, mut resumable_ref: ResumableRef) -> Result<RunState, RuntimeError> {
        match resumable_ref {
            ResumableRef::Fresh(FreshResumableRef {
                func_addr,
                params,
                maybe_fuel,
            }) => {
                let func_inst = self.functions.get(func_addr);

                match func_inst {
                    FuncInst::HostFunc(host_func_inst) => {
                        let returns = (host_func_inst.hostcode)(&mut self.user_data, params);

                        debug!("Successfully invoked function");

                        let returns = returns.map_err(|HaltExecutionError| {
                            RuntimeError::HostFunctionHaltedExecution
                        })?;

                        // Verify that the return parameters match the host function parameters
                        // since we have no validation guarantees for host functions

                        let return_types = returns.iter().map(|v| v.to_ty()).collect::<Vec<_>>();
                        if host_func_inst.function_type.returns.valtypes != return_types {
                            trace!(
                                "Func return types len: {}; returned args len: {}",
                                host_func_inst.function_type.returns.valtypes.len(),
                                return_types.len()
                            );
                            return Err(RuntimeError::HostFunctionSignatureMismatch);
                        }

                        Ok(RunState::Finished(returns))
                    }
                    FuncInst::WasmFunc(wasm_func_inst) => {
                        // Prepare a new stack with the locals for the entry function
                        let mut stack = Stack::new_with_values(params);

                        stack.push_call_frame::<T>(
                            FuncAddr::INVALID, // TODO using a default value like this is dangerous
                            &wasm_func_inst.function_type,
                            &wasm_func_inst.locals,
                            usize::MAX,
                            usize::MAX,
                        )?;

                        let mut resumable = Resumable {
                            current_func_addr: func_addr,
                            stack,
                            pc: wasm_func_inst.code_expr.from,
                            stp: wasm_func_inst.stp,
                            maybe_fuel,
                        };

                        // Run the interpreter
                        let result = interpreter_loop::run(&mut resumable, self)?;

                        match result {
                            None => {
                                debug!("Successfully invoked function");
                                Ok(RunState::Finished(resumable.stack.into_values()))
                            }
                            Some(required_fuel) => {
                                debug!("Successfully invoked function, but ran out of fuel");
                                Ok(RunState::Resumable {
                                    resumable_ref: ResumableRef::Invoked(
                                        self.dormitory.insert(resumable),
                                    ),
                                    required_fuel,
                                })
                            }
                        }
                    }
                }
            }
            ResumableRef::Invoked(InvokedResumableRef {
                dormitory: ref mut dormitory_weak,
                ref key,
            }) => {
                // Resuming requires `self`'s dormitory to still be alive
                let Some(dormitory) = dormitory_weak.upgrade() else {
                    return Err(RuntimeError::ResumableNotFound);
                };

                // Check the given `RuntimeInstance` is the same one used to create `self`
                if !Arc::ptr_eq(&dormitory, &self.dormitory.0) {
                    return Err(RuntimeError::ResumableNotFound);
                }

                // Obtain a write lock to the `Dormitory`
                let mut dormitory = dormitory.write();

                // TODO We might want to remove the `Resumable` here already and later reinsert it.
                // This would prevent holding the lock across the interpreter loop.
                let resumable = dormitory
                    .get_mut(key)
                    .expect("the key to always be valid as self was not dropped yet");

                // Resume execution
                let result = interpreter_loop::run(resumable, self)?;

                match result {
                    None => {
                        let resumable = dormitory.remove(key)
                            .expect("that the resumable could not have been removed already, because then this self could not exist");

                        // Take the `Weak` pointing to the dormitory out of `self` and replace it with a default `Weak`.
                        // This causes the `Drop` impl of `self` to directly quit preventing it from unnecessarily locking the dormitory.
                        let _dormitory = mem::take(dormitory_weak);

                        Ok(RunState::Finished(resumable.stack.into_values()))
                    }
                    Some(required_fuel) => Ok(RunState::Resumable {
                        resumable_ref,
                        required_fuel,
                    }),
                }
            }
        }
    }

    pub fn access_fuel_mut<R>(
        &mut self,
        resumable_ref: &mut ResumableRef,
        f: impl FnOnce(&mut Option<u32>) -> R,
    ) -> Result<R, RuntimeError> {
        match resumable_ref {
            ResumableRef::Fresh(FreshResumableRef { maybe_fuel, .. }) => Ok(f(maybe_fuel)),
            ResumableRef::Invoked(resumable_ref) => {
                // Resuming requires `self`'s dormitory to still be alive
                let Some(dormitory) = resumable_ref.dormitory.upgrade() else {
                    return Err(RuntimeError::ResumableNotFound);
                };

                // Check the given `RuntimeInstance` is the same one used to create `self`
                if !Arc::ptr_eq(&dormitory, &self.dormitory.0) {
                    return Err(RuntimeError::ResumableNotFound);
                }

                let mut dormitory = dormitory.write();

                let resumable = dormitory
                    .get_mut(&resumable_ref.key)
                    .expect("the key to always be valid as self was not dropped yet");

                Ok(f(&mut resumable.maybe_fuel))
            }
        }
    }
}

/// A marker error for host functions to return, in case they want execution to be halted.
pub struct HaltExecutionError;

///<https://webassembly.github.io/spec/core/exec/runtime.html#external-values>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExternVal {
    Func(FuncAddr),
    Table(TableAddr),
    Mem(MemAddr),
    Global(GlobalAddr),
}

impl ExternVal {
    /// returns the external type of `self` according to typing relation,
    /// taking `store` as context S.
    ///
    /// Note: This method may panic if self does not come from the given [`Store`].
    ///<https://webassembly.github.io/spec/core/valid/modules.html#imports>
    pub fn extern_type<T: Config>(&self, store: &Store<T>) -> ExternType {
        match self {
            // TODO: fix ugly clone in function types
            ExternVal::Func(func_addr) => ExternType::Func(store.functions.get(*func_addr).ty()),
            ExternVal::Table(table_addr) => ExternType::Table(store.tables.get(*table_addr).ty),
            ExternVal::Mem(mem_addr) => ExternType::Mem(store.memories.get(*mem_addr).ty),
            ExternVal::Global(global_addr) => {
                ExternType::Global(store.globals.get(*global_addr).ty)
            }
        }
    }
}

impl ExternVal {
    pub fn as_func(self) -> Option<FuncAddr> {
        match self {
            ExternVal::Func(func_addr) => Some(func_addr),
            _ => None,
        }
    }

    pub fn as_table(self) -> Option<TableAddr> {
        match self {
            ExternVal::Table(table_addr) => Some(table_addr),
            _ => None,
        }
    }

    pub fn as_mem(self) -> Option<MemAddr> {
        match self {
            ExternVal::Mem(mem_addr) => Some(mem_addr),
            _ => None,
        }
    }

    pub fn as_global(self) -> Option<GlobalAddr> {
        match self {
            ExternVal::Global(global_addr) => Some(global_addr),
            _ => None,
        }
    }
}

/// common convention functions defined for lists of ExternVals, ExternTypes, Exports
/// <https://webassembly.github.io/spec/core/exec/runtime.html#conventions>
/// <https://webassembly.github.io/spec/core/syntax/types.html#id3>
/// <https://webassembly.github.io/spec/core/syntax/modules.html?highlight=convention#id1>
// TODO implement this trait for ExternType lists Export lists
pub trait ExternFilterable {
    fn funcs(self) -> impl Iterator<Item = FuncAddr>;
    fn tables(self) -> impl Iterator<Item = TableAddr>;
    fn mems(self) -> impl Iterator<Item = MemAddr>;
    fn globals(self) -> impl Iterator<Item = GlobalAddr>;
}

impl<'a, I> ExternFilterable for I
where
    I: Iterator<Item = &'a ExternVal>,
{
    fn funcs(self) -> impl Iterator<Item = FuncAddr> {
        self.filter_map(|extern_val| extern_val.as_func())
    }

    fn tables(self) -> impl Iterator<Item = TableAddr> {
        self.filter_map(|extern_val| extern_val.as_table())
    }

    fn mems(self) -> impl Iterator<Item = MemAddr> {
        self.filter_map(|extern_val| extern_val.as_mem())
    }

    fn globals(self) -> impl Iterator<Item = GlobalAddr> {
        self.filter_map(|extern_val| extern_val.as_global())
    }
}
