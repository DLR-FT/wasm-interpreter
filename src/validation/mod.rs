use alloc::{
    boxed::Box,
    collections::btree_set::{self, BTreeSet},
    vec::Vec,
};
use core::iter::Map;

use crate::{
    core::{
        decoding::{
            decoder::{span::Span, WasmDecoder},
            modules::sections::{decode_section_if_ty_matches, SectionTy},
        },
        fixed_capacity_vec::FixedCapacityVec,
        sidetable::Sidetable,
        structure::{
            modules::{
                data_segments::DataSegment,
                element_segments::ElemType,
                exports::{Export, ExportDesc},
                globals::Global,
                imports::{Import, ImportDesc},
                indices::{
                    DataIdx, ElemIdx, ExtendedIdxVec, FuncIdx, GlobalIdx, IdxVec,
                    IdxVecOverflowError, MemIdx, TableIdx, TypeIdx,
                },
            },
            types::{ExternType, FuncType, MemType, TableType},
        },
        trusted_exact_size_iterator::{
            AssumeTrustedExactSize, TrustedExactSizeIterator, TryCollectIntoBoxedSliceExt,
        },
        utils::ToUsizeExt,
    },
    validation::{config::ValidationConfig, modules::functions::decode_and_validate_code_section},
    CustomSection, DecodingError, ValidationError,
};

pub mod error;
pub mod instructions;
pub mod modules;
pub mod types;
pub mod validation_stack;

pub mod config;

/// Information collected from validating a module.
///
/// This can be used to instantiate a new module instance in some
/// [`Store`](crate::Store) thorugh
/// [`Store::module_instantiate`](crate::Store::module_instantiate)
#[derive(Clone, Debug)]
pub struct Module<'bytecode> {
    pub(crate) wasm: &'bytecode [u8],
    pub(crate) types: IdxVec<TypeIdx, FuncType>,
    // TODO: Do not allocate for imports. They can be read on demand during instantiation or when
    // Module::imports is called. Especially, because only the import descriptions are required in
    // instantiation.
    pub(crate) imports: Box<[Import<'bytecode>]>,
    pub(crate) functions: ExtendedIdxVec<FuncIdx, TypeIdx>,
    pub(crate) tables: ExtendedIdxVec<TableIdx, TableType>,
    pub(crate) memories: ExtendedIdxVec<MemIdx, MemType>,
    pub(crate) globals: ExtendedIdxVec<GlobalIdx, Global>,
    pub(crate) exports: Box<[Export<'bytecode>]>,
    pub(crate) elements: IdxVec<ElemIdx, ElemType>,
    pub(crate) data: IdxVec<DataIdx, DataSegment>,
    /// Each block contains the validated code section and the stp corresponding to
    /// the beginning of that code section
    pub(crate) func_blocks_stps: Box<[(Span, usize)]>,
    pub(crate) sidetable: Sidetable,
    /// The start function which is automatically executed during instantiation
    pub(crate) start: Option<FuncIdx>,
    pub(crate) custom_sections: Box<[CustomSection<'bytecode>]>,
    // pub(crate) exports_length: Exported,
}

fn validate_no_duplicate_exports(module: &Module) -> Result<(), ValidationError> {
    let mut found_export_names: btree_set::BTreeSet<&str> = btree_set::BTreeSet::new();
    for export in &module.exports {
        if found_export_names.contains(export.name) {
            return Err(ValidationError::DuplicateExportName);
        }
        found_export_names.insert(export.name);
    }
    Ok(())
}

pub fn decode_and_validate<'wasm, T: ValidationConfig>(
    wasm: &'wasm [u8],
    user_data: &mut T,
) -> Result<Module<'wasm>, ValidationError> {
    let mut wasm = WasmDecoder::new(wasm);

    // represents C.refs in https://webassembly.github.io/spec/core/valid/conventions.html#context
    // A func.ref instruction is onlv valid if it has an immediate that is a member of C.refs.
    // this list holds all the func_idx's occurring in the module, except in its functions or start function.
    // I make an exception here by not including func_idx's occuring within data segments in C.refs as well, so that single pass validation is possible.
    // If there is a func_idx within the data segment, this would ultimately mean that data segment cannot be validated,
    // therefore this hack is acceptable.
    // https://webassembly.github.io/spec/core/valid/modules.html#data-segments
    // https://webassembly.github.io/spec/core/valid/modules.html#valid-module

    let mut validation_context_refs: BTreeSet<FuncIdx> = BTreeSet::new();

    let [0x00, 0x61, 0x73, 0x6d] = wasm.strip_bytes::<4>()? else {
        return Err(DecodingError::InvalidMagic.into());
    };

    let [0x01, 0x00, 0x00, 0x00] = wasm.strip_bytes::<4>()? else {
        return Err(DecodingError::InvalidBinaryFormatVersion.into());
    };

    let mut custom_sections = Vec::new();
    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let types = decode_section_if_ty_matches::<_, DecodingError>(&mut wasm, SectionTy::Type, |wasm, _| {
        let func_types= wasm.decode_vec()?.decode_elements_with(|wasm, _| FuncType::decode(wasm));
        let idx_vec = IdxVec::try_from_iter(func_types).expect("index space creation only fails if there are more than u32::MAX elements, which cannot happen because vector lengths are encoded as a 32-bit integer")?;
        Ok(idx_vec)
    })?.unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let mut num_imported_functions: u32 = 0;
    let mut num_imported_tables: u32 = 0;
    let mut num_imported_memories: u32 = 0;
    let mut num_imported_globals: u32 = 0;
    let imports = decode_section_if_ty_matches(&mut wasm, SectionTy::Import, |wasm, _| {
        wasm.decode_vec()?
            .decode_elements_with(|wasm, _| {
                // Decode and validate the import itself
                let import = Import::decode_and_validate(wasm, &types)?;

                // Additional bookkeeping to count functions/tables/mems/globals
                let num_imported = match import.desc {
                    ImportDesc::Func(_) => &mut num_imported_functions,
                    ImportDesc::Table(_) => &mut num_imported_tables,
                    ImportDesc::Mem(_) => &mut num_imported_memories,
                    ImportDesc::Global(_) => &mut num_imported_globals,
                };
                *num_imported = num_imported
                    .checked_add(1)
                    .expect("the total number of import can never exceed u32::MAX");

                Ok::<Import, ValidationError>(import)
            })
            .try_collect_into_boxed_slice()
    })?
    .unwrap_or_default();

    // From the results of decoding the import section, we can now create iterators with exact
    // sizes. These help to determine the exact alloation size required when allocating index spaces
    // for functions, tables, memories and globals later.
    let imported_functions = imports.iter().filter_map(|import| {
        if let ImportDesc::Func(type_idx) = import.desc {
            Some(type_idx)
        } else {
            None
        }
    });
    let imported_tables = imports.iter().filter_map(|import| {
        if let ImportDesc::Table(table_type) = import.desc {
            Some(table_type)
        } else {
            None
        }
    });
    let imported_memories = imports.iter().filter_map(|import| {
        if let ImportDesc::Mem(mem_type) = import.desc {
            Some(mem_type)
        } else {
            None
        }
    });
    let imported_globals = imports.iter().filter_map(|import| {
        if let ImportDesc::Global(global_type) = import.desc {
            Some(global_type)
        } else {
            None
        }
    });

    let imported_functions = unsafe {
        imported_functions.assume_trusted_exact_size(num_imported_functions.into_usize())
    };
    let imported_tables =
        unsafe { imported_tables.assume_trusted_exact_size(num_imported_tables.into_usize()) };
    let imported_memories =
        unsafe { imported_memories.assume_trusted_exact_size(num_imported_memories.into_usize()) };
    let imported_globals =
        unsafe { imported_globals.assume_trusted_exact_size(num_imported_globals.into_usize()) };

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    // The `Function` section only covers module-level (or "local") functions.
    // Imported functions have their types known in the `import` section. Both
    // local and imported functions share the same index space.
    //
    // Imported functions are given priority and have the first indicies, and
    // only after that do the local functions get assigned their indices.
    let functions = decode_section_if_ty_matches(&mut wasm, SectionTy::Function, |wasm, _| {
        let local_functions = wasm
            .decode_vec()?
            .decode_elements_with(|wasm, _| TypeIdx::decode_and_validate(wasm, &types));
        ExtendedIdxVec::try_from_iter(imported_functions.map(Ok), local_functions)
            .map_err(|IdxVecOverflowError| ValidationError::TooManyFunctions)
    })?
    .transpose()?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let tables = decode_section_if_ty_matches(&mut wasm, SectionTy::Table, |wasm, _| {
        let local_tables = wasm
            .decode_vec()?
            .decode_elements_with(|wasm, _| TableType::decode_and_validate(wasm));
        ExtendedIdxVec::try_from_iter(imported_tables.map(Ok), local_tables)
            .map_err(|IdxVecOverflowError| ValidationError::TooManyTables)
    })?
    .transpose()?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let memories = decode_section_if_ty_matches(&mut wasm, SectionTy::Memory, |wasm, _| {
        let local_memories = wasm
            .decode_vec()?
            .decode_elements_with(|wasm, _| MemType::decode_and_validate(wasm));
        ExtendedIdxVec::try_from_iter(imported_memories.map(Ok), local_memories)
            .map_err(|IdxVecOverflowError| ValidationError::TooManyMemories)
    })?
    .transpose()?
    .unwrap_or_default();

    // Additional validation check
    if memories.inner().len() > 1 {
        return Err(ValidationError::UnsupportedMultipleMemoriesProposal);
    }

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let globals = decode_section_if_ty_matches(&mut wasm, SectionTy::Global, |wasm, _| {
        // Decoding and validating the globals section is tricky to do without any excess
        // allocations. For other Wasm objects we can simply decode and validate each object by
        // itself and then combine the imported and local objects at the end in a single allocation.
        //
        // However, the validation of local globals can depend on imported global types, because
        // local globals' initializer expressions may reference imported globals.
        //
        // In order to use only a single allocation for the final global index space, the following steps are done:
        // 1. Read number of local globals from the vec's header.
        // 2. Allocate a FixedCapacityVec with final capacity but only push the imported globals
        //    into it.
        // 3. Decode and validate the local globals one by one and then push them into the vector.
        // 4. Convert the vector into a `Box<[Global]>`.

        let imported_globals = imported_globals.map(|ty| Global {
            // TODO using a default MAX value for spans that are never executed is not really safe.
            // Maybe opt for an Option instead.
            init_expr: Span::new(usize::MAX, 0),
            ty,
        });

        // Step 1
        let (num_local_globals, wasm_vec_elements) =
            wasm.decode_vec()?.into_inner_len_and_decoder();

        // Step 2
        let num_globals = num_imported_globals
            .checked_add(num_local_globals)
            .ok_or(ValidationError::TooManyGlobals)?;
        let mut globals = FixedCapacityVec::with_capacity(num_globals.into_usize());

        debug_assert!(globals.spare_capacity() >= imported_globals.len_trusted());
        // SAFETY: The globals vector is empty enough capacity to fit exactly all imported and local
        // globals.
        unsafe { globals.extend_unchecked(imported_globals) };

        for _ in 0..num_local_globals {
            let imported_globals = globals.get(0..num_imported_globals.into_usize()).expect(
                "number of imported globals is not larger than it plus the number of local globals",
            );
            let next_local_global = Global::decode_and_validate(
                wasm_vec_elements,
                &imported_globals,
                &mut validation_context_refs,
                functions.inner(),
            )?;

            globals
                .push(next_local_global)
                .expect("globals vector has enough capacity for all imported and local globals");
        }

        let globals = unsafe { globals.into_boxed_slice() };

        unsafe { ExtendedIdxVec::new(globals, num_imported_globals) }
            .map_err(|IdxVecOverflowError| ValidationError::TooManyGlobals)
    })?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let exports: Box<[Export]> =
        decode_section_if_ty_matches(&mut wasm, SectionTy::Export, |wasm, _| {
            wasm.decode_vec()?
                .decode_elements_with(|wasm, _| {
                    Export::decode_and_validate(
                        wasm,
                        functions.inner(),
                        tables.inner(),
                        memories.inner(),
                        globals.inner(),
                    )
                })
                .try_collect_into_boxed_slice()
        })?
        .unwrap_or_default();
    validation_context_refs.extend(exports.iter().filter_map(
        |Export { name: _, desc }| match desc {
            ExportDesc::Func(func_idx) => Some(func_idx),
            _ => None,
        },
    ));

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let start = decode_section_if_ty_matches(&mut wasm, SectionTy::Start, |wasm, _| {
        let func_idx = FuncIdx::decode_and_validate(wasm, functions.inner())?;

        // start function signature must be [] -> []
        // https://webassembly.github.io/spec/core/valid/modules.html#start-function
        // SAFETY: We just validated this function index using the same
        // `IdxVec`.
        let type_idx = unsafe { functions.inner().get(func_idx) };

        // SAFETY: There exists only one `IdxVec<TypeIdx, FuncType>` in the
        // current function. Therefore, this has to be the same one used to
        // create and validate this `TypeIdx`.
        let func_type = unsafe { types.get(*type_idx) };
        if !func_type.is_empty() {
            Err(ValidationError::InvalidStartFunctionSignature)
        } else {
            Ok(func_idx)
        }
    })?;

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let elements = decode_section_if_ty_matches::<_, ValidationError>(&mut wasm, SectionTy::Element, |wasm, _| {
        let collected = wasm.decode_vec()?.decode_elements_with(|wasm, _| {
            ElemType::decode_and_validate(
            wasm,
            functions.inner(),
            &mut validation_context_refs,
            tables.inner(),
            globals.imported_definitions(),
        )});
        let idx_vec = IdxVec::try_from_iter(collected).expect("that index space creation never fails because the length of the elements vector is encoded as a 32-bit integer in the bytecode")?;
        Ok(idx_vec)
    })?
    .unwrap_or_default();

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    // As per the official documentation:
    //
    // The data count section is used to simplify single-pass validation. Since the data section occurs after the code section, the `memory.init` and `data.drop` and instructions would not be able to check whether the data segment index is valid until the data section is read. The data count section occurs before the code section, so a single-pass validator can use this count instead of deferring validation.
    let data_count: Option<u32> =
        decode_section_if_ty_matches(&mut wasm, SectionTy::DataCount, |wasm, _| {
            wasm.decode_var_u32()
        })?;

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let mut sidetable = Sidetable::new();
    let func_blocks_stps = decode_section_if_ty_matches(&mut wasm, SectionTy::Code, |wasm, _| {
        // SAFETY: It is required that all passed index values are valid in all
        // passed `IdxVec`s. The current function does not take any index types
        // as arguments and every `IdxVec<..., ...>` is unique because they use
        // different generics. Therefore, all index types must be valid in their
        // relevant `IdxVec`s.
        unsafe {
            decode_and_validate_code_section(
                wasm,
                &types,
                &functions,
                globals.inner(),
                memories.inner(),
                data_count,
                tables.inner(),
                &elements,
                &validation_context_refs,
                &mut sidetable,
                user_data,
            )
        }
    })?
    .unwrap_or_default();

    if func_blocks_stps.len() != functions.len_local_definitions().into_usize() {
        return Err(ValidationError::FunctionAndCodeSectionsHaveDifferentLengths);
    }

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    let data_section = decode_section_if_ty_matches::<_, ValidationError>(&mut wasm, SectionTy::Data, |wasm, _| {
        let collected = wasm.decode_vec()?.decode_elements_with(|wasm, _| {
            DataSegment::decode_and_validate(wasm, globals.imported_definitions(), functions.inner(), memories.inner())
        }).try_collect_into_boxed_slice()?;
        let idx_vec = IdxVec::new(collected).expect("that index space creation never fails because the length of the data segments vector is encoded as a 32-bit integer in the bytecode");
        Ok(idx_vec)
    })?
    .unwrap_or_default();

    // https://webassembly.github.io/spec/core/binary/modules.html#data-count-section
    if let Some(data_count) = data_count {
        if data_count != data_section.len() {
            return Err(ValidationError::DataCountAndDataSectionsLengthAreDifferent);
        }
    }

    read_all_custom_sections(&mut wasm, &mut custom_sections)?;

    // All sections should have been handled
    if !wasm.remaining_bytes().is_empty() {
        let remaining_section_ty = SectionTy::decode(&mut wasm).expect(
            "that the section type is not malformed, because it must have been peeked before",
        );
        return Err(DecodingError::SectionOutOfOrder(remaining_section_ty).into());
    }

    let module = Module {
        wasm: wasm.into_inner(),
        types,
        imports,
        functions,
        tables,
        memories,
        globals,
        exports,
        func_blocks_stps,
        sidetable,
        data: data_section,
        start,
        elements,
        custom_sections: custom_sections.into_boxed_slice(),
    };
    validate_no_duplicate_exports(&module)?;

    Ok(module)
}

/// Reads the next sections as long as they are custom sections and pushes them
/// into the `custom_sections` vector.
fn read_all_custom_sections<'wasm>(
    wasm: &mut WasmDecoder<'wasm>,
    custom_sections: &mut Vec<CustomSection<'wasm>>,
) -> Result<(), ValidationError> {
    while let Some(custom_section) =
        decode_section_if_ty_matches(wasm, SectionTy::Custom, CustomSection::decode)?
    {
        custom_sections.push(custom_section);
    }

    Ok(())
}

impl<'wasm> Module<'wasm> {
    /// Returns the imports of this module as an iterator. Each import consist
    /// of a module name, a name and an extern type.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.5 - module_imports
    pub fn imports<'a>(
        &'a self,
    ) -> Map<
        core::slice::Iter<'a, Import<'wasm>>,
        impl FnMut(&'a Import<'wasm>) -> (&'a str, &'a str, ExternType),
    > {
        self.imports.iter().map(|import| {
            // SAFETY: This is sound because the argument is `self` and the
            // import desc also comes from `self`.
            let extern_type = unsafe { import.desc.extern_type_owned(self) };
            (import.module_name, import.name, extern_type)
        })
    }

    /// Returns the exports of this module as an iterator. Each export consist
    /// of a name, and an extern type.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.5 - module_exports
    pub fn exports<'a>(
        &'a self,
    ) -> Map<
        core::slice::Iter<'a, Export<'wasm>>,
        impl FnMut(&'a Export<'wasm>) -> (&'a str, ExternType),
    > {
        self.exports.iter().map(|export| {
            // SAFETY: This is sound because the argument is `self` and the
            // export desc also comes from `self`.
            let extern_type = unsafe { export.desc.extern_type(self) };
            (export.name, extern_type)
        })
    }

    /// Returns a list of all custom sections in the bytecode. Every custom
    /// section consists of its name and the custom section's bytecode
    /// (excluding the name itself).
    pub fn custom_sections(&self) -> &[CustomSection<'wasm>] {
        &self.custom_sections
    }
}
