use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use const_interpreter_loop::{run_const, run_const_span};
use function_ref::FunctionRef;
use interpreter_loop::run;
use locals::Locals;
use store::{DataInst, ElemInst, ImportedFuncInst, LocalFuncInst, TableInst};
use value::{ExternAddr, FuncAddr, Ref};
use value_stack::Stack;

use crate::core::error::StoreInstantiationError;
use crate::core::reader::types::element::{ElemItems, ElemMode};
use crate::core::reader::types::export::{Export, ExportDesc};
use crate::core::reader::types::import::ImportDesc;
use crate::core::reader::types::FuncType;
use crate::core::reader::WasmReader;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::{EmptyHookSet, HookSet};
use crate::execution::store::{FuncInst, GlobalInst, MemInst, Store};
use crate::execution::value::Value;
use crate::validation::code::read_declared_locals;
use crate::value::InteropValueList;
use crate::{RefType, Result as CustomResult, RuntimeError, ValType, ValidationInfo};

// TODO
pub(crate) mod assert_validated;
pub mod const_interpreter_loop;
pub mod function_ref;
pub mod hooks;
mod interpreter_loop;
pub(crate) mod locals;
pub(crate) mod lut;
pub(crate) mod store;
pub mod value;
pub mod value_stack;

/// The default module name if a [RuntimeInstance] was created using [RuntimeInstance::new].
pub const DEFAULT_MODULE: &str = "__interpreter_default__";

pub struct RuntimeInstance<'b, H = EmptyHookSet>
where
    H: HookSet,
{
    pub wasm_bytecode: &'b [u8],
    types: Vec<FuncType>,
    exports: Vec<Export>,
    pub store: Store,
    pub hook_set: H,
}

impl<'b> RuntimeInstance<'b, EmptyHookSet> {
    pub fn new(validation_info: &'_ ValidationInfo<'b>) -> CustomResult<Self> {
        Self::new_with_hooks(DEFAULT_MODULE, validation_info, EmptyHookSet)
    }

    pub fn new_named(
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> CustomResult<Self> {
        Self::new_with_hooks(module_name, validation_info, EmptyHookSet)
    }
}

impl<'b, H> RuntimeInstance<'b, H>
where
    H: HookSet,
{
    pub fn new_with_hooks(
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        hook_set: H,
    ) -> CustomResult<Self> {
        trace!("Starting instantiation of bytecode");

        let store = Self::init_store(validation_info)?;

        let mut instance = RuntimeInstance {
            wasm_bytecode: validation_info.wasm,
            types: validation_info.types.clone(),
            exports: validation_info.exports.clone(),
            store,
            hook_set,
        };

        if let Some(start) = validation_info.start {
            // "start" is not always exported, so we need create a non-API exposed function reference.
            // Note: function name is not important here, as it is not used in the verification process.
            let start_fn = FunctionRef {
                module_name: module_name.to_string(),
                function_name: "start".to_string(),
                module_index: 0,
                function_index: start,
                exported: false,
            };
            instance.invoke::<(), ()>(&start_fn, ())?;
        }

        Ok(instance)
    }

    pub fn get_function_by_name(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Result<FunctionRef, RuntimeError> {
        let (module_idx, func_idx) = self.get_indicies(module_name, function_name)?;

        Ok(FunctionRef {
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            module_index: module_idx,
            function_index: func_idx,
            exported: true,
        })
    }

    pub fn get_function_by_index(
        &self,
        module_idx: usize,
        function_idx: usize,
    ) -> Result<FunctionRef, RuntimeError> {
        // TODO: Module resolution
        let function_name = self
            .exports
            .iter()
            .find(|export| match &export.desc {
                ExportDesc::FuncIdx(idx) => *idx == function_idx,
                _ => false,
            })
            .map(|export| export.name.clone())
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok(FunctionRef {
            // TODO: get the module name from the module index
            module_name: DEFAULT_MODULE.to_string(),
            function_name,
            module_index: module_idx,
            function_index: function_idx,
            exported: true,
        })
    }

    // TODO: remove this annotation when implementing the function
    #[allow(clippy::result_unit_err)]
    pub fn add_module(
        &mut self,
        _module_name: &str,
        _validation_info: &'_ ValidationInfo<'b>,
    ) -> Result<(), ()> {
        todo!("Implement module linking");
    }

    pub fn invoke<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        function_ref: &FunctionRef,
        params: Param,
    ) -> Result<Returns, RuntimeError> {
        // First, verify that the function reference is valid
        let (_module_idx, func_idx) = self.verify_function_ref(function_ref)?;

        // -=-= Verification =-=-
        let func_inst = self
            .store
            .funcs
            .get(func_idx)
            .expect("valid FuncIdx")
            .try_into_local()
            .expect("to invoke local function");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

        // Check correct function parameters and return types
        if func_ty.params.valtypes != Param::TYS {
            panic!("Invalid `Param` generics");
        }
        if func_ty.returns.valtypes != Returns::TYS {
            panic!("Invalid `Returns` generics");
        }

        // Prepare a new stack with the locals for the entry function
        let mut stack = Stack::new();
        let locals = Locals::new(
            params.into_values().into_iter(),
            func_inst.locals.iter().cloned(),
        );

        // setting `usize::MAX` as return address for the outermost function ensures that we
        // observably fail upon errornoeusly continuing execution after that function returns.
        stack.push_stackframe(func_idx, func_ty, locals, usize::MAX, usize::MAX);

        // Run the interpreter
        run(
            self.wasm_bytecode,
            &self.types,
            &mut self.store,
            &mut stack,
            EmptyHookSet,
        )?;

        // Pop return values from stack
        let return_values = Returns::TYS
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret: Returns = Returns::from_values(reversed_values);
        debug!("Successfully invoked function");
        Ok(ret)
    }

    /// Invokes a function with the given parameters, and return types which are not known at compile time.
    pub fn invoke_dynamic(
        &mut self,
        function_ref: &FunctionRef,
        params: Vec<Value>,
        ret_types: &[ValType],
    ) -> Result<Vec<Value>, RuntimeError> {
        // First, verify that the function reference is valid
        let (_module_idx, func_idx) = self.verify_function_ref(function_ref)?;

        // -=-= Verification =-=-
        let func_inst = self
            .store
            .funcs
            .get(func_idx)
            .expect("valid FuncIdx")
            .try_into_local()
            .expect("to invoke local function");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

        // Verify that the given parameters match the function parameters
        let param_types = params.iter().map(|v| v.to_ty()).collect::<Vec<_>>();

        if func_ty.params.valtypes != param_types {
            panic!("Invalid parameters for function");
        }

        // Verify that the given return types match the function return types
        if func_ty.returns.valtypes != ret_types {
            panic!("Invalid return types for function");
        }

        // Prepare a new stack with the locals for the entry function
        let mut stack = Stack::new();
        let locals = Locals::new(params.into_iter(), func_inst.locals.iter().cloned());
        stack.push_stackframe(func_idx, func_ty, locals, 0, 0);

        // Run the interpreter
        run(
            self.wasm_bytecode,
            &self.types,
            &mut self.store,
            &mut stack,
            EmptyHookSet,
        )?;

        let func_inst = self
            .store
            .funcs
            .get(func_idx)
            .expect("valid FuncIdx")
            .try_into_local()
            .expect("to invoke local function");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

        // Pop return values from stack
        let return_values = func_ty
            .returns
            .valtypes
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret = reversed_values.collect();
        debug!("Successfully invoked function");
        Ok(ret)
    }

    // TODO: replace this with the lookup table when implmenting the linker
    fn get_indicies(
        &self,
        _module_name: &str,
        function_name: &str,
    ) -> Result<(usize, usize), RuntimeError> {
        let func_idx = self
            .exports
            .iter()
            .find_map(|export| {
                if export.name == function_name {
                    match export.desc {
                        ExportDesc::FuncIdx(func_idx) => Some(func_idx),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok((0, func_idx))
    }

    fn verify_function_ref(
        &self,
        function_ref: &FunctionRef,
    ) -> Result<(usize, usize), RuntimeError> {
        if function_ref.exported {
            let (module_idx, func_idx) =
                self.get_indicies(&function_ref.module_name, &function_ref.function_name)?;

            if module_idx != function_ref.module_index || func_idx != function_ref.function_index {
                // TODO: should we return a different error here?
                return Err(RuntimeError::FunctionNotFound);
            }

            Ok((module_idx, func_idx))
        } else {
            let (module_idx, func_idx) = (function_ref.module_index, function_ref.function_index);

            // TODO: verify module named - index mapping.

            // Sanity check that the function index is at least in the bounds of the store, though this doesn't mean
            // that it's a valid function.
            self.store
                .funcs
                .get(func_idx)
                .ok_or(RuntimeError::FunctionNotFound)?;

            Ok((module_idx, func_idx))
        }
    }

    fn init_store(validation_info: &ValidationInfo) -> CustomResult<Store> {
        use StoreInstantiationError::*;
        let function_instances: Vec<FuncInst> = {
            let mut wasm_reader = WasmReader::new(validation_info.wasm);

            let functions = validation_info.functions.iter();
            let func_blocks = validation_info.func_blocks.iter();

            let local_function_inst = functions.zip(func_blocks).map(|(ty, (func, sidetable))| {
                wasm_reader
                    .move_start_to(*func)
                    .expect("function index to be in the bounds of the WASM binary");

                let (locals, bytes_read) = wasm_reader
                    .measure_num_read_bytes(read_declared_locals)
                    .unwrap_validated();

                let code_expr = wasm_reader
                    .make_span(func.len() - bytes_read)
                    .expect("TODO remove this expect");

                FuncInst::Local(LocalFuncInst {
                    ty: *ty,
                    locals,
                    code_expr,
                    // TODO fix this ugly clone
                    sidetable: sidetable.clone(),
                })
            });

            let imported_function_inst =
                validation_info
                    .imports
                    .iter()
                    .filter_map(|import| match &import.desc {
                        ImportDesc::Func(type_idx) => Some(FuncInst::Imported(ImportedFuncInst {
                            ty: *type_idx,
                            module_name: import.module_name.clone(),
                            function_name: import.name.clone(),
                        })),
                        _ => None,
                    });

            imported_function_inst.chain(local_function_inst).collect()
        };

        // https://webassembly.github.io/spec/core/exec/modules.html#tables
        let mut tables: Vec<TableInst> = validation_info
            .tables
            .iter()
            .map(|ty| TableInst::new(*ty))
            .collect();

        let mut passive_elem_indexes: Vec<usize> = vec![];
        // https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
        let elements: Vec<ElemInst> = validation_info
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
                                run_const_span(validation_info.wasm, expr, ()).unwrap_validated(),
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

                        let offset =
                            match run_const_span(validation_info.wasm, &active_elem.init_expr, ())
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

        let mut memory_instances: Vec<MemInst> = validation_info
            .memories
            .iter()
            .map(|ty| MemInst::new(*ty))
            .collect();

        let data_sections: Vec<DataInst> = validation_info
            .data
            .iter()
            .map(|d| {
                use crate::core::reader::types::data::DataMode;
                use crate::NumType;
                if let DataMode::Active(active_data) = d.mode.clone() {
                    let mem_idx = active_data.memory_idx;
                    if mem_idx != 0 {
                        todo!("Active data has memory_idx different than 0");
                    }
                    assert!(memory_instances.len() > mem_idx);

                    let boxed_value = {
                        let mut wasm = WasmReader::new(validation_info.wasm);
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
                        return Err(ActiveDataWriteOutOfBounds);
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
            .collect::<Result<Vec<DataInst>, StoreInstantiationError>>()?;

        let global_instances: Vec<GlobalInst> = validation_info
            .globals
            .iter()
            .map({
                let mut stack = Stack::new();
                move |global| {
                    let mut wasm = WasmReader::new(validation_info.wasm);
                    // The place we are moving the start to should, by all means, be inside the wasm bytecode.
                    wasm.move_start_to(global.init_expr).unwrap_validated();
                    // We shouldn't need to clear the stack. If validation is correct, it will remain empty after execution.

                    run_const(wasm, &mut stack, ());
                    let value = stack.pop_value(global.ty.ty);

                    GlobalInst {
                        global: *global,
                        value,
                    }
                }
            })
            .collect();

        Ok(Store {
            funcs: function_instances,
            mems: memory_instances,
            globals: global_instances,
            data: data_sections,
            tables,
            elements,
            passive_elem_indexes,
        })
    }
}

/// Used for getting the offset of an address.
///
/// Related to the Active Elements
///
/// <https://webassembly.github.io/spec/core/syntax/modules.html#element-segments>
///
/// Since active elements need an offset given by a constant expression, in this case
/// they can only be an i32 (which can be understood from either a [`Value::I32`] - but
/// since we don't unbox the address of the reference, for us also a [`Value::Ref`] -
/// or from a Global)
fn get_address_offset(value: Value) -> Option<u32> {
    match value {
        Value::I32(val) => Some(val),
        Value::Ref(rref) => match rref {
            Ref::Extern(_) => todo!("Not yet implemented"),
            // TODO: fix
            Ref::Func(func_addr) => func_addr.addr.map(|addr| addr as u32),
        },
        // INFO: from wasmtime - implement only global
        _ => unreachable!(),
    }
}
