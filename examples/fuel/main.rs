use std::env;
use std::process::ExitCode;
use std::str::FromStr;

use log::{error, LevelFilter};

use wasm::{validate, RuntimeInstance};

fn main() -> ExitCode {
    let level = LevelFilter::from_str(&env::var("RUST_LOG").unwrap_or("TRACE".to_owned())).unwrap();
    env_logger::builder().filter_level(level).init();

    let wat = r#"
        (module
            (func $fac (export "fac") (param f64) (result f64)
                local.get 0
                f64.const 1
                f64.lt
                if (result f64)
                    f64.const 1
                else
                    nop
                    local.get 0
                    local.get 0
                    f64.const 1
                    f64.sub
                    call $fac
                    f64.mul
                end
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = match validate(&wasm_bytes) {
        Ok(table) => table,
        Err(err) => {
            error!("Validation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    let mut instance = match RuntimeInstance::new(&validation_info) {
        Ok(instance) => instance,
        Err(err) => {
            error!("Instantiation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    let mut state = instance
        .invoke_resumable(&instance.get_function_by_index(0, 0).unwrap(), 1.0f64)
        .unwrap();

    let mut res: Option<f64> = None;
    loop {
        match state {
            wasm::InvocationState::Finished(ret) => {
                res.replace(ret);
                break;
            }
            wasm::InvocationState::OutOfFuel(res) => {
                state = res.resume().unwrap();
            }
            wasm::InvocationState::Canceled => {
                break;
            }
        };
    }

    assert_eq!(res.unwrap(), 1.0f64);

    ExitCode::SUCCESS
}
