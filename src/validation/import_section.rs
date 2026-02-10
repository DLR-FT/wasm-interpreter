use alloc::vec::Vec;

use crate::{
    core::{
        indices::TypeIdx,
        reader::{types::import::Import, WasmReader},
    },
    GlobalType, MemType, TableType,
};

struct ImportSection<'wasm> {
    functions: Vec<TypeIdx>,
    globals: Vec<GlobalType>,
    memories: Vec<MemType>,
    tables: Vec<TableType>,

    iter_imports: Iter<'wasm>,
}

#[derive(Clone)]
struct Iter<'wasm> {
    reader: WasmReader<'wasm>,
    num_elements_left: usize,
}

impl<'wasm> Iterator for Iter<'wasm> {
    type Item = Import;

    fn next(&mut self) -> Option<Self::Item> {
        self.num_elements_left = self.num_elements_left.checked_sub(1)?;
        let import = unsafe { Import::read_unchecked(&mut self.reader) };
        Some(import)
    }
}

fn read_and_validate(
    wasm: &mut WasmReader,
    c_types: &IdxVec<TypeIdx, FuncType>,
) -> Result<ImportSection, ValidationError> {
    let imports = wasm.read_vec(|wasm| Import::read_and_validate(wasm, &types));
}
