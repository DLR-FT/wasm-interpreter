use std::time::UNIX_EPOCH;
use wasm::{decode_and_validate, ExternVal, FuncType, ResultType, RunState, Store, ValType, Value};

const COREMARK_MINIMAL_BYTECODE: &[u8] = include_bytes!("coremark-minimal.wasm");

fn main() {
    let score = run();
    println!("{score}");
}

pub fn run() -> f32 {
    let module = decode_and_validate(COREMARK_MINIMAL_BYTECODE, &mut ()).unwrap();
    let mut store = Store::new(());
    let env_clock_ms_function = store.func_alloc(
        FuncType {
            params: ResultType {
                valtypes: Vec::new(),
            },
            returns: ResultType {
                valtypes: vec![ValType::NumType(wasm::NumType::I64)],
            },
        },
        0, // Use arbitrary host code, as there is only one host function
    );

    let module = unsafe {
        store.module_instantiate(&module, vec![ExternVal::Func(env_clock_ms_function)], None)
    }
    .unwrap()
    .module_addr;

    let run_function = unsafe { store.instance_export(module, "run") }
        .unwrap()
        .as_func()
        .unwrap();

    let mut run_state = unsafe { store.invoke(run_function, Vec::new(), None) }.unwrap();
    loop {
        match run_state {
            RunState::Finished { values, .. } => {
                let &[Value::F32(score)] = &*values else {
                    panic!()
                };
                return score.0;
            }
            RunState::Resumable { resumable, .. } => {
                run_state = unsafe { store.resume_wasm(resumable) }.unwrap();
            }
            RunState::HostCalled { resumable, .. } => {
                let clock_ms = clock_ms();
                run_state =
                    unsafe { store.finish_host_call(resumable, vec![Value::I64(clock_ms)]) }
                        .unwrap();
            }
        }
    }
}

fn clock_ms() -> u64 {
    u64::try_from(
        UNIX_EPOCH
            .elapsed()
            .expect("UNIX_EPOCH is never later than another SystemTime")
            .as_millis(),
    )
    .expect("time after UNIX_EPOCH in milliseconds never really exceeds 2^64")
}
