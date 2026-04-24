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
use std::process::{ExitCode, Termination};

#[macro_use]
extern crate log_wrapper;

use clap::Parser;
use lib_wasm_coverage::{probes::{CovListTraceToVec, FullTraceToVec}, reporter::{DwarfAddr2LineLookup, SourceCodeLocation, parse_dwarf}};
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
        Ok(wasm_bytes) => wasm_bytes,
        Err(e) => {
            let wasm_file_path = &args.wasm_file_path;
            error!("Failed to read {wasm_file_path:?}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let inputs = match std::fs::read_to_string(&args.input_file_path) {
        Ok(inputs) => {
            let mut result = Vec::new();
            for s in inputs.split_whitespace() {
                let v= match s.parse::<i32>() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to parse input file, input is i32s seperated by whitespace: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                result.push(Value::I32(v.cast_unsigned()));
            }
            result
        }
        Err(e) => {
            let input_file_path = &args.input_file_path;
            error!("Failed to read {input_file_path:?}: {e}");
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

    let mut dwarf_table = parse_dwarf(&wasm_bytes);

    match &args.probe_type {
        ProbeType::FullTrace => continue_with_fulltrace_probe(&wasm_bytes, &validation_info, inputs, &mut dwarf_table),
        ProbeType::CovList => continue_with_covlisttrace_probe(&wasm_bytes, &validation_info, inputs, &mut dwarf_table)
    }
    
}

fn continue_with_fulltrace_probe(wasm_bytes: &[u8], validation_info: &ValidationInfo, inputs: Vec<Value>, dwarf_table: &mut DwarfAddr2LineLookup) -> ExitCode {
    let user_data = FullTraceToVec::default();
    // intialize a coverage enabled store
    let mut store = Store::new(user_data);

    // instantiate the module
    let module = match unsafe { store.module_instantiate(validation_info, Vec::new(), None) } {
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
        store.invoke_without_fuel(entry_function, inputs)
    } {
        Ok(x) => eprintln!("execution finished with return value(s) {x:?}"),
        Err(e) => eprintln!("execution aborted due to {e:?}"),
    }

    for pc in &store.user_data.trace {
        if
        //already_seen_pc.insert(pc)
        //&&
        let Some((file_idx, line, col, hit)) = dwarf_table.pc_to_source_file_cache.get_mut(pc) {
            let scl = SourceCodeLocation {
                path: &dwarf_table.source_file_list[*file_idx],
                line: *line,
                col: *col
            };
            eprintln!("pc = {pc:#x?} <- {scl}");
            *hit = true;
        }
    }

    ExitCode::SUCCESS
}

fn continue_with_covlisttrace_probe(wasm_bytes: &[u8], validation_info: &ValidationInfo, inputs: Vec<Value>, dwarf_table: &mut DwarfAddr2LineLookup) -> ExitCode {
    let user_data = CovListTraceToVec::default();
    // intialize a coverage enabled store
    let mut store = Store::new(user_data);

    // instantiate the module
    let module = match unsafe { store.module_instantiate(validation_info, Vec::new(), None) } {
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
        store.invoke_without_fuel(entry_function, inputs)
    } {
        Ok(x) => eprintln!("execution finished with return value(s) {x:?}"),
        Err(e) => eprintln!("execution aborted due to {e:?}"),
    }

    for range in &store.user_data.trace {
        eprintln!("{:?}",range)
    }

    for (pc, (file_idx, line, col, hit)) in dwarf_table.pc_to_source_file_cache.iter_mut() {
        if store.user_data.trace.contains(*pc) {
            let scl = SourceCodeLocation {
                path: &dwarf_table.source_file_list[*file_idx],
                line: *line,
                col: *col
            };
            eprintln!("pc = {pc:#x?} <- {scl}");
            *hit = true;
        }
    }

    ExitCode::SUCCESS
}
