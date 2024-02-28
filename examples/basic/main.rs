use std::env;
use std::process::ExitCode;
use std::str::FromStr;

use log::{error, LevelFilter};

use wasm::validate;

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

    let _table = match validate(&wasm_bytes) {
        Ok(table) => table,
        Err(err) => {
            error!("Validation failed: {err:?}");
            return ExitCode::FAILURE;
        }
    };

    ExitCode::SUCCESS
}
