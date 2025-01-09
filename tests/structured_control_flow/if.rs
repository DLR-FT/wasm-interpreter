use wasm::{validate, RuntimeInstance};

#[test_log::test]
fn odd_with_if_else() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func $odd (param $n i32) (result i32)
        local.get $n
        i32.const 2
        i32.rem_s
        (if (result i32)
            (then 
                i32.const 1
            )
            (else 
                i32.const 0
            )
        )
    )

    (export "odd" (func $odd))
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let odd_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(1, instance.invoke(&odd_fn, -5).unwrap());
    assert_eq!(0, instance.invoke(&odd_fn, 0).unwrap());
    assert_eq!(1, instance.invoke(&odd_fn, 3).unwrap());
    assert_eq!(0, instance.invoke(&odd_fn, 4).unwrap());
}

#[test_log::test]
fn odd_with_if() {
    let wasm_bytes = wat::parse_str(
        r#"(module
    (func $odd (param $n i32) (result i32)
        local.get $n
        i32.const 2
        i32.rem_s
        (if
            (then 
                i32.const 1
                return
            )
        )
        i32.const 0
    )

    (export "odd" (func $odd))
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let odd_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(1, instance.invoke(&odd_fn, -5).unwrap());
    assert_eq!(0, instance.invoke(&odd_fn, 0).unwrap());
    assert_eq!(1, instance.invoke(&odd_fn, 3).unwrap());
    assert_eq!(0, instance.invoke(&odd_fn, 4).unwrap());
}

#[test_log::test]
fn odd_with_if_else_recursive() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func $odd (param $n i32) (result i32)
        local.get $n
        (if (result i32)
            (then 
                local.get $n
                i32.const 1
                i32.sub
                call $even
                return
            )
            (else 
                i32.const 0
                return
            )
        )
    )

    (func $even (param $n i32) (result i32)
        local.get $n
        (if (result i32)
            (then
                local.get $n
                i32.const 1
                i32.sub
                call $odd
                return
            )
            (else 
                i32.const 1
                return
            )
        )
    )

    (export "odd" (func $odd))
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let even_odd_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(1, instance.invoke(&even_odd_fn, 1).unwrap());
    assert_eq!(0, instance.invoke(&even_odd_fn, 0).unwrap());
    assert_eq!(1, instance.invoke(&even_odd_fn, 3).unwrap());
    assert_eq!(0, instance.invoke(&even_odd_fn, 4).unwrap());
}

#[test_log::test]
fn recursive_fibonacci_if_else() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func $fibonacci (param $n i32) (result i32)
        local.get $n
        i32.const 1
        i32.le_s
        (if (result i32)
            (then 
                i32.const 1
                return
            )
            (else
                local.get $n
                i32.const 1
                i32.sub
                call $fibonacci
                local.get $n
                i32.const 2
                i32.sub
                call $fibonacci
                i32.add
                return
            )
        )
    )

    (export "fibonacci" (func $fibonacci))
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let fibonacci_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(1, instance.invoke(&fibonacci_fn, -5).unwrap());
    assert_eq!(1, instance.invoke(&fibonacci_fn, 0).unwrap());
    assert_eq!(1, instance.invoke(&fibonacci_fn, 1).unwrap());
    assert_eq!(2, instance.invoke(&fibonacci_fn, 2).unwrap());
    assert_eq!(3, instance.invoke(&fibonacci_fn, 3).unwrap());
    assert_eq!(5, instance.invoke(&fibonacci_fn, 4).unwrap());
    assert_eq!(8, instance.invoke(&fibonacci_fn, 5).unwrap());
}
