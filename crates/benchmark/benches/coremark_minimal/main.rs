use dlr_wasm_interpreter::{
    decode_and_validate, Config, DispatchMechanism, ExternVal, FuncType, NumType, ResultType,
    RunState, Store, ValType, Value,
};
use envconfig::Envconfig;
use std::{str::FromStr, time::UNIX_EPOCH};

const COREMARK_MINIMAL_BYTECODE: &[u8] = include_bytes!("coremark-minimal.wasm");

#[derive(Debug, Clone, Default)]
enum OutputFormat {
    #[default]
    HumanReadable,
    /// Bencher Metric Format: https://bencher.dev/docs/reference/bencher-metric-format/
    Bmf,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bmf" => Ok(Self::Bmf),
            "human" => Ok(Self::HumanReadable),
            other => Err(format!("invalid output format: {other}")),
        }
    }
}

#[derive(Envconfig, Debug, Default)]
struct EnvConfig {
    /// Output format
    #[envconfig(from = "BENCH_OUTPUT_FORMAT")]
    format: Option<OutputFormat>,
}

fn main() {
    let env_config = EnvConfig::init_from_env().unwrap();

    let score = run(InterpreterConfig);

    let format = env_config.format.unwrap_or_default();
    match format {
        OutputFormat::HumanReadable => {
            println!("Score: {score}");
        }
        OutputFormat::Bmf => {
            println!(r#"{{ "coremark_minimal": {{ "score": {{ "value": {score} }} }} }}"#)
        }
    }
}

pub fn run<T: Config>(interpreter_config: T) -> f32 {
    let module = decode_and_validate(COREMARK_MINIMAL_BYTECODE, &mut ()).unwrap();

    let mut store = Store::new(interpreter_config);
    let env_clock_ms_function = store.func_alloc(
        FuncType {
            params: ResultType::default(),
            returns: ResultType {
                valtypes: Box::from([ValType::NumType(NumType::I64)]),
            },
        },
        0, // Use arbitrary host code, as there is only one host function
    );

    // SAFETY: This function address was just returned from a function allocation in the same store.
    let module = unsafe {
        store.module_instantiate(&module, vec![ExternVal::Func(env_clock_ms_function)], None)
    }
    .unwrap()
    .module_addr;

    // SAFETY: This module address was just returned from module instantiation in the same store.
    let run_function = unsafe { store.instance_export(module, "run") }
        .unwrap()
        .as_func()
        .unwrap();

    // SAFETY: This function address was just returned from the same store.
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
                // SAFETY: This resumable was just returned by a function invocation in the same
                // store.
                run_state = unsafe { store.resume_wasm(resumable) }.unwrap();
            }
            RunState::HostCalled { resumable, .. } => {
                let clock_ms = clock_ms();
                // SAFETY: This resumable was just returned by a function invocation in the same
                // store. Also no address values are passed as host call return values.
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

struct InterpreterConfig;

impl Config for InterpreterConfig {
    const DISPATCH_MECHANISM: DispatchMechanism = DispatchMechanism::LoopMatch;
}
