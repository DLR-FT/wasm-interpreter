use std::process::ExitCode;

use log::{error, LevelFilter};

use wasm::validate;

fn main() -> ExitCode {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let wat = r#"
    (module
        (func $add_one (param $x i32) (result i32)
            local.get $x
            i32.const 2
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
