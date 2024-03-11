use std::env;
use std::process::ExitCode;
use std::str::FromStr;

use log::{error, LevelFilter};

use wasm::{RuntimeInstance, validate};

fn main() -> ExitCode {
    let level = LevelFilter::from_str(&env::var("RUST_LOG").unwrap_or("TRACE".to_owned())).unwrap();
    env_logger::builder().filter_level(level).init();

    let wat = r#"
    (module
        (func $add_one (param $x i32) (result i32) (local $ununsed_local i32)
            local.get $x
            i32.const 1
            i32.add)

        (func $add (param $x i32) (param $y i32) (result i32)
            local.get $y
            local.get $x
            i32.add)
        (export "add_one" (func $add_one))
        (export "add" (func $add))
    )
    "#;
    let wasm_bytes = wat::parse_str(&wat).unwrap();

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

    let twelve: i32 = instance.invoke_func(1, (5, 7));
    assert_eq!(twelve, 12);

    let twelve_plus_one: i32 = instance.invoke_func(0, twelve);
    assert_eq!(twelve_plus_one, 13);

    ExitCode::SUCCESS
}
