//! Embedder API
//!
//! Most functions from the embedder API are supported and defined on the [`Store`]. This example
//! lists a random selection of them without doing anything in particular.
//!
//! See: [WebAssembly Specification 2.0 - A.1 Embedding](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#a1-embedding)

use std::error::Error;

use dlr_wasm_interpreter::{
    GlobalAddr, GlobalType, Limits, MemAddr, MemType, NumType, Ref, RefType, Store, TableAddr,
    TableType, ValType, Value, F64,
};

fn main() -> Result<(), Box<dyn Error>> {
    // This is the same as `store_init`. In Rust we prefer `new`.
    let mut store: Store<()> = Store::new(());

    // We can allocate and interact with memories
    let my_memory: MemAddr = store.mem_alloc(MemType {
        limits: Limits {
            min: 1,    // minimum size is one page
            max: None, // no upper limit
        },
    });
    // SAFETY: The memory address just came from the same store.
    unsafe {
        store.mem_write(my_memory, 128, 0x50)?;
        store.mem_write(my_memory, 129, 0x51)?;
        store.mem_write(my_memory, 130, 0x52)?;
        store.mem_write(my_memory, 131, 0x53)?;

        let x: u8 = store.mem_read(my_memory, 129)?;
        assert_eq!(x, 0x51);

        // Note: Multi-byte memory accesses are work-in-progress. For now, use `mem_data_mut` and
        // cast it to your type:
        let mem_data: &mut [u8] = store.mem_data_mut(my_memory);
        let four_bytes: [u8; 4] = mem_data[128..=131].try_into().expect("4 bytes");
        let as_integer: u32 = u32::from_le_bytes(four_bytes);
        assert_eq!(as_integer, u32::from_le_bytes([0x50, 0x51, 0x52, 0x53]));
    }

    // The same works for tables & globals.
    //
    // SAFETY: No addresses are passed as parameters.
    let _my_table: TableAddr = unsafe {
        store.table_alloc(
            TableType {
                et: RefType::FuncRef,
                lim: Limits { min: 1, max: None },
            },
            Ref::Null(RefType::FuncRef),
        )
    }?;
    // SAFETY: No addresses are passed as parameters.
    let _my_global: GlobalAddr = unsafe {
        store.global_alloc(
            GlobalType {
                ty: ValType::NumType(NumType::F64),
                is_mut: true,
            },
            Value::F64(F64(123.456)),
        )
    }?;

    Ok(())
}
