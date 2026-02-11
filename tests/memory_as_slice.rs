use std::io::Write;

use wasm::{Limits, MemType, Store};

#[test_log::test]
fn simple_byte_writes() {
    let mut store = Store::new(());
    let mem = store.mem_alloc(MemType {
        limits: Limits { min: 1, max: None },
    });

    store
        .mem_access_mut_slice(mem, |mem_as_slice| {
            for (n, x) in mem_as_slice.iter_mut().enumerate() {
                *x = u8::try_from(n % 256).expect("this to never be larger than 255");
            }
        })
        .unwrap();
}

#[test_log::test]
fn interpret_as_str() {
    let mut store = Store::new(());
    let mem = store.mem_alloc(MemType {
        limits: Limits { min: 1, max: None },
    });

    const STR_TO_WRITE: &str = "Hello World!";

    // Write a string into the memory
    store
        .mem_access_mut_slice(mem, |mut mem_as_slice| {
            let bytes_written = mem_as_slice.write(STR_TO_WRITE.as_bytes()).unwrap();
            assert_eq!(bytes_written, 12);
        })
        .unwrap();

    // Read the string again and check if it is equal to the original one
    store
        .mem_access_mut_slice(mem, |mem_as_slice| {
            let bytes = &mem_as_slice[0..STR_TO_WRITE.len()];
            let as_str = std::str::from_utf8(bytes).unwrap();
            assert_eq!(as_str, STR_TO_WRITE);
        })
        .unwrap();
}
