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

use clap::Parser;
use lib_wasm_coverage::probes::{CovListTraceToVec, ExecutionTrace, FullTraceToVec};
use wasm::{Store, ValidationInfo, Value, config::Config, validate};

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    wasm_file_path: String,
    input_file_path: String,
    #[arg(value_enum, default_value_t = ProbeType::FullTrace)]
    probe_type: ProbeType
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum ProbeType {
    FullTrace,
    CovList
}

fn main() -> ExitCode {
    env_logger::init();
    let args = Args::parse();

    let wasm_bytes= match std::fs::read(&args.wasm_file_path) {
        Ok(x) => x,
        Err(e) => {
            let wasm_file_path = &args.wasm_file_path;
            error!("Failed to read {wasm_file_path:?}: {e}");
            return ExitCode::FAILURE;
        }
    };

    // validate the bytecode
    let validation_info = match validate(&wasm_bytes) {
        Ok(table) => table,
        Err(err) => {
            error!("Validation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    match args.probe_type {
        ProbeType::FullTrace => continue_with_probe(FullTraceToVec::default(), &wasm_bytes, validation_info),
        ProbeType::CovList => continue_with_probe(CovListTraceToVec::default(), &wasm_bytes, validation_info)
    }

}

fn continue_with_probe<T>(user_data: T, wasm_bytes: &[u8], validation_info: ValidationInfo) -> ExitCode
where
    for<'a> &'a T : IntoIterator<Item = u64>,
    T: ExecutionTrace + Config {

    // intialize a coverage enabled store
    let mut store = Store::new(user_data);

    // instantiate the module
    let module = match unsafe { store.module_instantiate(&validation_info, Vec::new(), None) } {
        Ok(outcome) => outcome.module_addr,
        Err(err) => {
            error!("Instantiation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    // get funcref to the entry function
    let entry_function = unsafe { store.instance_export(module, "main") }
        .unwrap()
        .as_func()
        .unwrap();

    // call the entry function
    match unsafe {
        store.invoke_without_fuel(entry_function, vec![Value::I32(171), Value::I32(379)])
    } {
        Ok(x) => eprintln!("execution finished with return value(s) {x:?}"),
        Err(e) => eprintln!("execution abortde due to {e:?}"),
    }

    lib_wasm_coverage::reporter::report_source_lines(&wasm_bytes, store.user_data.into_iter());

    ExitCode::SUCCESS
}
