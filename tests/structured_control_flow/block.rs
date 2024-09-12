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

    assert_eq!(
        (),
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

#[test_log::test]
fn branch() {
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "with_branch") (result i32)
                (block $outer_block (result i32)
                    (block $my_block (result i32)
                        i32.const 5
                        br $my_block
                        i32.const 3
                    )
                    i32.const 3
                    i32.add
                )
            )
        )
        "#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        8,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
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

    assert_eq!(
        7,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 6)
            .unwrap()
    );
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

    assert_eq!(
        3,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
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

    let abs_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(6, instance.invoke(&abs_fn, 6).unwrap());
    assert_eq!(123, instance.invoke(&abs_fn, -123).unwrap());
    assert_eq!(0, instance.invoke(&abs_fn, 0).unwrap());
}

#[test_log::test]
fn recursive_fibonacci() {
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "fibonacci") (param $x i32) (result i32)
                (call $fib_internal
                    (i32.const 0)
                    (i32.const 1)
                    (local.get $x)
              	)
            )

            (func $fib_internal (param $x0 i32) (param $x1 i32) (param $n_left i32) (result i32)
                (block $zero_check
                    ;; if n_left reached 0, we return
                    local.get $n_left
                    br_if $zero_check
                    local.get $x0
                    return  
                )
              
                ;; otherwise decrement n_left
                local.get $n_left
                i32.const -1
                i32.add
                local.set $n_left
              
                ;; store x1 temporarily
                local.get $x1

                ;; calculate new x1
                local.get $x0
                local.get $x1
                i32.add
                local.set $x1

                ;; set x0 to the previous x1
                local.set $x0

              
                (call $fib_internal
                  (local.get $x0)
                  (local.get $x1)
                  (local.get $n_left)
              	)
            )
        )
        "#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let fib_fn = instance.get_function_by_index(0, 0).unwrap();

    let first_ten = (0..10).map(|n| instance.invoke(&fib_fn, n).unwrap()).collect::<Vec<i32>>();
    assert_eq!(&first_ten, &[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
}