//! Host functions using the registry utility
//!
//! This is a copy of host_functions.rs except that we use the [`dlr_wasm_interpreter_registry`]
//! utility crate. As of right now, the registry is still work-in-progress and requires us to also
//! use the [`dlr_wasm_interpreter_checked`] crate. Also, the registry can take advantage of the
//! [`dlr_wasm_interpreter_interop`] crate to provide statically-typed function, which we will use
//! in this example.

use std::{error::Error, io};

use dlr_wasm_interpreter::{decode_and_validate, FuncAddr, Module, ModuleAddr};
use dlr_wasm_interpreter_checked::{
    Store, Stored, StoredExternVal, StoredInstantiationOutcome, StoredRunState,
};
use dlr_wasm_interpreter_registry::Registry;

/// A Wasm module that converts uppercase ASCII characters into lowercase ones with simple arithmetic
const WAT_CODE: &str = r#"
(module $wasm_src.wasm
    (import "host" "host_getc" (func $getc (param) (result i32)))
    (import "host" "host_putc" (func $putc (param i32) (result)))
    (func $lowercase (export "lowercase") (param) (result)
        call $getc
        i32.const 32 ;; difference between uppercase and lowercase letters in ASCII
        i32.add
        call $putc
    )
)
"#;

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_bytecode: Vec<u8> = wat::parse_str(WAT_CODE)?;
    let module: Module = decode_and_validate(&wasm_bytecode, &mut ())?;
    let mut store = Store::new(());

    let mut registry: Registry<()> = Registry::default();

    // This registry implementation allows us to define host functions as closures.
    let host_getc_addr: Stored<FuncAddr> =
        registry.alloc_host_function_typed(&mut store, |&mut (), ()| {
            let mut line = String::new();
            println!("Type a character:");
            let bytes_read = io::stdin()
                .read_line(&mut line)
                .expect("could not read from stdin");
            if bytes_read <= 1 {
                panic!("No char entered")
            }
            line.as_bytes()[0] as u32
        });

    let host_putc_addr: Stored<FuncAddr> =
        registry.alloc_host_function_typed(&mut store, |&mut (), c: u32| {
            let c = c as u8 as char;
            println!("Printing character: {c}");
        });

    let instantiation_outcome: StoredInstantiationOutcome = store.module_instantiate(
        &module,
        vec![
            StoredExternVal::Func(host_getc_addr),
            StoredExternVal::Func(host_putc_addr),
        ],
        None,
    )?;

    let module_addr: Stored<ModuleAddr> = instantiation_outcome.module_addr;

    let lowercase_addr: Stored<FuncAddr> = store
        .instance_export(module_addr, "lowercase")?
        .as_func()
        .ok_or("lowercase is not a function")?;

    let mut run_state: StoredRunState = store.invoke(lowercase_addr, vec![], None)?;

    loop {
        match run_state {
            StoredRunState::HostCalled {
                host_call,
                resumable,
            } => {
                run_state = registry.perform_host_call(&mut (), &mut store, host_call, resumable)?
            }
            StoredRunState::Resumable {
                resumable,
                required_fuel: _,
            } => run_state = store.resume_wasm(resumable)?,
            StoredRunState::Finished { .. } => break,
        }
    }
    Ok(())
}
