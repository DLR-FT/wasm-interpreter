const COREMARK_MINIMAL_BYTECODE: &[u8] = include_bytes!("coremark-minimal.wasm");

type RunnerFn = fn() -> f32;
const RUNNERS: &[(&str, RunnerFn)] = &[
    ("our", our),
    #[cfg(feature = "wasmtime")]
    ("wasmtime", wasmtime),
    #[cfg(feature = "wasmi")]
    ("wasmi", wasmi),
    // TODO(wasm3-support)
    // #[cfg(feature = "wasm3")]
    // ("wasm3", wasm3),
];

/// Runs the coremark-minimal benchmark for all runners, which can be enabled through features.
/// Their scores are returned in an iterator.
pub fn run() -> impl Iterator<Item = (&'static str, f32)> {
    RUNNERS
        .iter()
        .cloned()
        .map(|(name, runner)| (name, runner()))
}

fn clock_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis(),
    )
    .unwrap()
}

pub fn our() -> f32 {
    use wasm::{
        decode_and_validate, ExternVal, FuncType, ResultType, RunState, Store, ValType, Value,
    };

    let module = decode_and_validate(COREMARK_MINIMAL_BYTECODE).unwrap();
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

#[cfg(feature = "wasmi")]
pub fn wasmi() -> f32 {
    use wasmi::{core::ValType, Engine, Linker, Module, Store, Val};

    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    linker.func_wrap("env", "clock_ms", clock_ms).unwrap();

    let module = Module::new(&engine, COREMARK_MINIMAL_BYTECODE).unwrap();

    let module_instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();

    let run_function = module_instance
        .get_export(&mut store, "run")
        .unwrap()
        .into_func()
        .unwrap();

    let mut outputs = [Val::default(ValType::F32)];

    run_function.call(&mut store, &[], &mut outputs).unwrap();

    outputs[0].f32().unwrap().to_float()
}

#[cfg(feature = "wasmtime")]
pub fn wasmtime() -> f32 {
    use wasmtime::{FuncType, Linker, Module, Store, Val, ValType};

    let mut store: Store<()> = Store::default();
    let mut linker = Linker::new(store.engine());
    linker
        .func_new(
            "env",
            "clock_ms",
            FuncType::new(store.engine(), [], [ValType::I64]),
            |_, _, res| {
                res[0] = Val::I64(clock_ms().cast_signed());
                Ok(())
            },
        )
        .unwrap();

    let module = Module::new(store.engine(), COREMARK_MINIMAL_BYTECODE).unwrap();
    let module_instance = linker.instantiate(&mut store, &module).unwrap();

    let run_function = module_instance.get_func(&mut store, "run").unwrap();

    let mut res = [Val::default_for_ty(&ValType::F32).unwrap()];
    run_function.call(&mut store, &[], &mut res).unwrap();

    let [Val::F32(res)] = res else { panic!() };
    f32::from_bits(res)
}

// TODO(wasm3-support): See Cargo.toml for why wasm3 is commented out
// #[cfg(feature = "wasm3")]
// fn wasm3() -> f32 {
//     use wasm3::{Environment, Module};

//     let env = Environment::new().expect("Unable to create environment");
//     let rt = env.create_runtime(2048).expect("Unable to create runtime");
//     let mut module = rt
//         .load_module(
//             Module::parse(&env, COREMARK_MINIMAL_BYTECODE).expect("Unable to parse module"),
//         )
//         .expect("Unable to load module");

//     module
//         .link_function::<(), u64>("env", "clock_ms", clock_ms_wrap)
//         .expect("Unable to link function");

//     module
//         .find_function::<(), f32>("run")
//         .expect("Unable to find function")
//         .call()
//         .expect("Calling coremark failed in wasm3")
// }

// #[cfg(feature = "wasm3")]
// wasm3::make_func_wrapper!(clock_ms_wrap: clock_ms() -> u64);
