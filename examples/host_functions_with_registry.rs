//! Host functions using the registry utility
//!
//! In this example we run a Wasm program that imports `getc` an `putc` functions. With these
//! functions it could freely implement many things, however here it only converts a single
//! character to lowercase.
//!
//! The design is quite very similar to the `host_functions.rs` example. However, here the
//! [`dlr_wasm_interpreter_registry`] utility crate is used to manage host functions and their
//! handlers. As of right now, the registry is still work-in-progress and requires us to also use
//! the [`dlr_wasm_interpreter_checked`] crate. Furthermore, the registry can take advantage of the
//! [`dlr_wasm_interpreter_interop`] crate to provide statically-typed function, which we will also
//! make use of in this example.

use std::{
    error::Error,
    io::{self, Write},
};

use dlr_wasm_interpreter::{FuncAddr, ModuleAddr};
use dlr_wasm_interpreter_checked::{
    Module, Store, Stored, StoredExternVal, StoredInstantiationOutcome, StoredRunState,
    decode_and_validate,
};
use dlr_wasm_interpreter_registry::Registry;

/// A Wasm module that converts uppercase ASCII characters into lowercase ones with simple arithmetic
const WAT_CODE: &str = r#"
(module $wasm_src.wasm
    (import "host" "host_getc" (func $getc (param) (result i32)))
    (import "host" "host_putc" (func $putc (param i32) (result)))
    (func $run (export "run") (local $char i32)
        call $getc
        local.tee $char

        ;; Is char uppercase?
        ;; Is it >= 'A'?
        i32.const 65
        i32.ge_u
        ;; and is it <= 'Z'?
        local.get $char
        i32.const 90
        i32.le_u
        i32.and
        ;; Make char lowercase
        (if (result i32)
            (then
                local.get $char
                i32.const 32 ;; difference between uppercase and lowercase letters in ASCII
                i32.add
            )
            (else local.get $char)
        )

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
            print!("Wasm requested character: ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
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
            println!("Wasm prints: {c}");
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

    let run_addr: Stored<FuncAddr> = store
        .instance_export(module_addr, "run")?
        .as_func()
        .ok_or("run is not a function")?;

    let mut run_state: StoredRunState = store.invoke(run_addr, vec![], None)?;

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
