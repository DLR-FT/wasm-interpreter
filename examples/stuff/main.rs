use std::env;
use std::process::ExitCode;
use std::str::FromStr;

use log::{error, LevelFilter};

use wasm::{validate, Store};

fn main() -> ExitCode {
    let level = LevelFilter::from_str(&env::var("RUST_LOG").unwrap_or("TRACE".to_owned())).unwrap();
    env_logger::builder().filter_level(level).init();

    let wat = r#"
    (module
        (memory 1)
        (func $add_one (export "add_one") (param $x i32) (result i32) (local $ununsed_local i32)
            local.get $x
            i32.const 1
            i32.add)

        (func $add (export "add") (param $x i32) (param $y i32) (result i32)
            local.get $y
            local.get $x
            i32.add)

        (func (export "store_num") (param $x i32)
            i32.const 0
            local.get $x
            i32.store)
        (func (export "load_num") (result i32)
            i32.const 0
            i32.load)

        (export "add_one" (func $add_one))
        (export "add" (func $add))
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

    let mut store = Store::new(());

    let module = match store.module_instantiate_unchecked(&validation_info, Vec::new(), None) {
        Ok(outcome) => outcome.module_addr,
        Err(err) => {
            error!("Instantiation failed: {err:?} [{err}]");
            return ExitCode::FAILURE;
        }
    };

    let add_one = store
        .instance_export_unchecked(module, "add_one")
        .unwrap()
        .as_func()
        .unwrap();

    let add = store
        .instance_export_unchecked(module, "add")
        .unwrap()
        .as_func()
        .unwrap();

    let store_num = store
        .instance_export_unchecked(module, "store_num")
        .unwrap()
        .as_func()
        .unwrap();

    let load_num = store
        .instance_export_unchecked(module, "load_num")
        .unwrap()
        .as_func()
        .unwrap();

    let twelve: i32 = store
        .invoke_typed_without_fuel_unchecked(add, (5, 7))
        .unwrap();
    assert_eq!(twelve, 12);

    let twelve_plus_one: i32 = store
        .invoke_typed_without_fuel_unchecked(add_one, twelve)
        .unwrap();
    assert_eq!(twelve_plus_one, 13);

    store
        .invoke_typed_without_fuel_unchecked::<_, ()>(store_num, 42_i32)
        .unwrap();

    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(), i32>(load_num, ())
            .unwrap(),
        42_i32
    );

    ExitCode::SUCCESS
}
