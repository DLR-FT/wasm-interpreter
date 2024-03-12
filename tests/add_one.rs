use log::{LevelFilter};
use wasm::{RuntimeInstance, validate};

fn main() {
    env_logger::builder().filter_level(LevelFilter::Trace).init();

    let wat = r#"
    (module
        (func $add_one (param $x i32) (result i32)
            local.get $x
            i32.const 1
            i32.add)
        (export "add_one" (func $add_one))
    )
    "#;
    let wasm_bytes = wat::parse_str(&wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");


    assert_eq!(12, instance.invoke_func(0, 11));
    assert_eq!(1, instance.invoke_func(0, 0));
    assert_eq!(-5, instance.invoke_func(0, -6));
}
