use log::LevelFilter;

use wasm::{validate, RuntimeInstance};

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let wat = r#"
    (module
        (memory 1)
        (func (export "store_num") (param $x i32)
            i32.const 0
            local.get $x
            i32.store)
        (func (export "load_num") (result i32)
            i32.const 0
            i32.load)
    )
    "#;
    let wasm_bytes = wat::parse_str(&wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let () = instance.invoke_func(0, 42);
    assert_eq!(42, instance.invoke_func(1, ()));
}
