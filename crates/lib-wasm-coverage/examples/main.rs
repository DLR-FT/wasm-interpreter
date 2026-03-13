/// # Notes on this demo
///
/// ```bash
/// rustc --crate-type lib --target wasm32-unknown-unknown --codegen debuginfo=2 --codegen opt-level=z wasm_test_prog.rs
/// RUST_LOG=lib_wasm_coverage=trace cargo watch -s 'cargo run --example main -- ./wasm_test_prog.wasm' -c
///
/// wasm-tools addr2line wasm_test_prog.wasm 0x15c
/// ```
///
/// https://yurydelendik.github.io/webassembly-dwarf/#locating
use std::process::ExitCode;

#[macro_use]
extern crate log_wrapper;

use wasm::{validate, Store};

fn main() -> ExitCode {
    env_logger::init();

    let wasm_bytes;

    // TODO remove this hack
    let wasm_file_path;

    // prepare the wasm bytecode from argv1
    let mut args = std::env::args();
    if let Some(file_path) = args.nth(1) {
        wasm_file_path = file_path;
        match std::fs::read(&wasm_file_path) {
            Ok(x) => wasm_bytes = x,
            Err(e) => {
                error!("Failed to read {wasm_file_path:?}: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        error!("argv1 must name a .wasm file");
        return ExitCode::FAILURE;
    }

    // validate the bytecode
    let validation_info = match validate(&wasm_bytes) {
        Ok(table) => table,
        Err(err) => {
            error!("Validation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    // intialize a coverage enabled store
    let user_data = lib_wasm_coverage::probes::BasicBlockTraceToVec::default();
    let mut store = Store::new(user_data);

    // instantiate the module
    let module = match store.module_instantiate_unchecked(&validation_info, Vec::new(), None) {
        Ok(outcome) => outcome.module_addr,
        Err(err) => {
            error!("Instantiation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    // get funcref to the entry function
    let entry_function = store
        .instance_export_unchecked(module, "entry")
        .unwrap()
        .as_func()
        .unwrap();

    // call the entry function
    store
        .invoke_typed_without_fuel_unchecked::<i32, i32>(entry_function, 42_i32)
        .unwrap();

    // print the trace
    for val in store.user_data.trace {
        let output = std::process::Command::new("wasm-tools")
            .arg("addr2line")
            .arg(&wasm_file_path)
            .arg(format!("{val:#0x}"))
            .output()
            .expect("failed to execute wasm-tools addr2line");

        std::io::Write::write_all(&mut std::io::stdout(), &output.stdout).unwrap();
        std::io::Write::write_all(&mut std::io::stderr(), &output.stderr).unwrap();
    }

    ExitCode::SUCCESS
}
