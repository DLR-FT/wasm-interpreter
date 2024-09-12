use wasm::{validate, RuntimeInstance};

/// Runs a function that does nothing and contains only a single empty block
#[test_log::test]
fn empty() {
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "do_nothing") (block)
            )
        )
        "#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!((), instance.invoke_named("do_nothing", ()).unwrap());
}

#[test_log::test]
fn branch() {
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "with_branch")
                (block $my_block
                    br $my_block
                )
            )
        )
        "#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!((), instance.invoke_named("with_branch", ()).unwrap());
}

#[test_log::test]
fn param_and_result() {
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "add_one") (param $x i32) (result)
                local.get $x
                (block $my_block (param i32) (result i32)
                    i32.const 1
                    i32.add
                    br $my_block
                )
            )
        )
        "#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(7, instance.invoke_named("add_one", 6).unwrap());
}

#[test_log::test]
fn return_out_of_block() {
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "get_three") (result i32)
                (block
                    i32.const 5
                    i32.const 3
                    return
                )
                unreachable
            )
        )
        "#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(3, instance.invoke_named("get_three", ()).unwrap());
}

#[test_log::test]
fn branch_if() {
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "abs") (param $x i32) (result i32)
                (block $my_block
                    local.get $x
                    i32.const 0
                    i32.ge_s
                    br_if $my_block
                    local.get $x
                    i32.const -1
                    i32.mul
                    return
                )
                local.get $x
            )
        )
        "#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(6, instance.invoke_named("abs", 6).unwrap());
    assert_eq!(123, instance.invoke_named("abs", -123).unwrap());
    assert_eq!(0, instance.invoke_named("abs", 0).unwrap());
}
