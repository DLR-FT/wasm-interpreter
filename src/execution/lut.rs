use crate::{core::reader::types::export::ExportDesc, execution::execution_info::ExecutionInfo};
use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};

pub struct Lut {
    /// function_lut\[local_module_idx\]\[function_local_idx\] = (foreign_module_idx, function_foreign_idx)
    ///
    /// - Module A imports a function "foo". Inside module A, the function has the index "function_local_idx". Module A
    ///   is assigned the index "local_module_idx".
    /// - Module B exports a function "foo". Inside module B, the function has the index "function_foreign_idx". Module
    ///   B is assigned the index "foreign_module_idx".
    function_lut: Vec<Vec<(usize, usize)>>,
}

impl Lut {
    /// Create a new linker lookup-table.
    ///
    /// # Arguments
    /// - `modules`: The modules to link together.
    /// - `module_map`: A map from module name to module index within the `modules` array.
    ///
    /// # Returns
    /// A new linker lookup-table. Can return `None` if there are import directives that cannot be resolved.
    pub fn new(modules: &[ExecutionInfo], module_map: &BTreeMap<String, usize>) -> Option<Self> {
        let mut function_lut = Vec::new();
        for module in modules {
            let module_lut = module
                .store
                .funcs
                .iter()
                .filter_map(|f| f.try_into_imported())
                .map(|import| {
                    Self::manual_lookup(
                        modules,
                        module_map,
                        &import.module_name,
                        &import.function_name,
                    )
                })
                .collect::<Option<Vec<_>>>()?;

            // TODO: what do we want to do if there is a missing import/export pair? Currently we fail the entire
            // operation. Should it be a RuntimeError if said missing pair is called?

            function_lut.push(module_lut);
        }

        Some(Self { function_lut })
    }

    /// Lookup a function by its module and function index.
    ///
    /// # Arguments
    /// - `module_idx`: The index of the module within the `modules` array passed in [Lut::new].
    /// - `function_idx`: The index of the function within the module. This index is considered in-bounds only if it is
    ///   an index of an imported function.
    ///
    /// # Returns
    /// - `None`, if the indicies are out of bound
    /// - `Some(export_module_idx, export_function_idx)`, where the new indicies are the indicies of the module which
    ///   contains the implementation of the imported function, and the implementation has the returned index within.
    pub fn lookup(&self, module_idx: usize, function_idx: usize) -> Option<(usize, usize)> {
        self.function_lut
            .get(module_idx)?
            .get(function_idx)
            .copied()
    }

    /// Manually lookup a function by its module and function name.
    ///
    /// This function is used to resolve import directives before the [Lut] is created, and can be used to resolve
    /// imports even after the [Lut] is created at the cost of speed.
    ///
    /// # Arguments
    /// - `modules`: The modules to link together.
    /// - `module_map`: A map from module name to module index within the `modules` array.
    /// - `module_name`: The name of the module which imports the function.
    /// - `function_name`: The name of the function to import.
    ///
    /// # Returns
    /// - `None`, if the module or function is not found.
    /// - `Some(export_module_idx, export_function_idx)`, where the new indicies are the indicies of the module which
    ///    contains the implementation of the imported function, and the implementation has the returned index within.
    ///    Note that this function returns the first matching function, if there are multiple functions with the same
    ///    name.
    pub fn manual_lookup(
        modules: &[ExecutionInfo],
        module_map: &BTreeMap<String, usize>,
        module_name: &str,
        function_name: &str,
    ) -> Option<(usize, usize)> {
        let module_idx = module_map.get(module_name)?;
        let module = &modules[*module_idx];

        module
            .store
            .exports
            .iter()
            .filter_map(|export| {
                if export.name == function_name {
                    Some(&export.desc)
                } else {
                    None
                }
            })
            .find_map(|desc| {
                if let ExportDesc::FuncIdx(func_idx) = desc {
                    Some((*module_idx, *func_idx))
                } else {
                    None
                }
            })
    }
}
