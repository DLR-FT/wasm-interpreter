/// A simple function to add 2 to an i32 using a recusive call to "add_one" and return the result
#[test_log::test]
fn recursion_valid() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func $add_one (export "add_one") (param $x i32) (result i32)
            local.get $x
            i32.const 1
            i32.add
        )
        (func (export "add_two") (param $x i32) (result i32)
            local.get $x
            call $add_one
            call $add_one
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(12, instance.invoke_named("add_two", 10).unwrap());
    assert_eq!(2, instance.invoke_named("add_two", 0).unwrap());
    assert_eq!(-4, instance.invoke_named("add_two", -6).unwrap());
}

#[test_log::test]
fn recursion_busted_stack() {
    use wasm::{validate, Error};

    let wat = r#"
    (module
        (func $add_one (export "add_one") (param $x i32) (result i32 i32)
            local.get $x
            i32.const 1
            i32.add
            local.get $x
            i32.const 1
            i32.add
        )
        (func (export "add_two") (param $x i32) (result i32)
            local.get $x
            call $add_one
            call $add_one
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    assert!(
        matches!(validate(&wasm_bytes), Err(Error::EndInvalidValueStack)),
        "validation incorrectly passed"
    );
}
