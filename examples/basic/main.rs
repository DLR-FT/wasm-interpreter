use std::env;
use std::process::ExitCode;
use std::str::FromStr;

use log::{error, LevelFilter};

use wasm::value::Value;
use wasm::{instantiate, invocate_fn, validate};

fn main() -> ExitCode {
    let level = LevelFilter::from_str(&env::var("RUST_LOG").unwrap_or("TRACE".to_owned())).unwrap();
    env_logger::builder().filter_level(level).init();

    let wat = r#"
    (module
        (func $add_one (param $x i32) (result i32) (local $ununsed_local f32)
            local.get $x
            i32.const 1
            i32.add)
        (export "add_one" (func $add_one))
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

    let mut instance = match instantiate(&wasm_bytes, validation_info) {
        Ok(instance) => instance,
        Err(err) => {
            error!("Instantiation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    let param = Value::I32(u32::from_le_bytes(5_i32.to_le_bytes()));
    let ret = invocate_fn(&mut instance, 0, param);
    dbg!(ret);

    ExitCode::SUCCESS
}
