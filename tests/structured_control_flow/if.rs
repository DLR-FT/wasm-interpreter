use wasm::{validate, RuntimeInstance};

#[test_log::test]
fn odd_with_if_else() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func (export "odd") (param $n i32) (result i32)
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
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let odd = instance
        .store
        .instance_export(module, "odd")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(1, instance.invoke_typed(odd, -5).unwrap());
    assert_eq!(0, instance.invoke_typed(odd, 0).unwrap());
    assert_eq!(1, instance.invoke_typed(odd, 3).unwrap());
    assert_eq!(0, instance.invoke_typed(odd, 4).unwrap());
}

#[test_log::test]
fn odd_with_if() {
    let wasm_bytes = wat::parse_str(
        r#"(module
    (func (export "odd") (param $n i32) (result i32)
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
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let odd = instance
        .store
        .instance_export(module, "odd")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(1, instance.invoke_typed(odd, -5).unwrap());
    assert_eq!(0, instance.invoke_typed(odd, 0).unwrap());
    assert_eq!(1, instance.invoke_typed(odd, 3).unwrap());
    assert_eq!(0, instance.invoke_typed(odd, 4).unwrap());
}

#[test_log::test]
fn odd_with_if_else_recursive() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func $odd (export "odd") (param $n i32) (result i32)
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
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let even_odd_fn = instance
        .store
        .instance_export(module, "odd")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(1, instance.invoke_typed(even_odd_fn, 1).unwrap());
    assert_eq!(0, instance.invoke_typed(even_odd_fn, 0).unwrap());
    assert_eq!(1, instance.invoke_typed(even_odd_fn, 3).unwrap());
    assert_eq!(0, instance.invoke_typed(even_odd_fn, 4).unwrap());
}

#[test_log::test]
fn recursive_fibonacci_if_else() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
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
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let fibonacci_fn = instance
        .store
        .instance_export(module, "fibonacci")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(1, instance.invoke_typed(fibonacci_fn, -5).unwrap());
    assert_eq!(1, instance.invoke_typed(fibonacci_fn, 0).unwrap());
    assert_eq!(1, instance.invoke_typed(fibonacci_fn, 1).unwrap());
    assert_eq!(2, instance.invoke_typed(fibonacci_fn, 2).unwrap());
    assert_eq!(3, instance.invoke_typed(fibonacci_fn, 3).unwrap());
    assert_eq!(5, instance.invoke_typed(fibonacci_fn, 4).unwrap());
    assert_eq!(8, instance.invoke_typed(fibonacci_fn, 5).unwrap());
}

#[test_log::test]
fn if_without_else_type_check1() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func (export "empty") (param $cond i32)
        (if (local.get $cond) (then))
    )
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let empty_fn = instance
        .store
        .instance_export(module, "empty")
        .unwrap()
        .as_func()
        .unwrap();

    instance.invoke_typed::<i32, ()>(empty_fn, 1).unwrap();
    instance.invoke_typed::<i32, ()>(empty_fn, 0).unwrap();
}

#[test_log::test]
fn if_without_else_type_check2() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func $empty (param $cond i32)
        (i32.const 1)
        (if (param i32) (local.get $cond) (then drop))
    )

    (export "empty" (func $empty))
)"#,
    )
    .unwrap();
    assert_eq!(
        validate(&wasm_bytes).err().unwrap(),
        wasm::ValidationError::IfWithoutMatchingElse
    );
}

#[test_log::test]
fn if_without_else_type_check3() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func (export "add_one_if_true") (param $cond i32) (result i32)
        (i32.const 5)
        (if (param i32) (result i32) (local.get $cond) (then (i32.const 2) (i32.add)))
    )
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let add_one_if_true_fn = instance
        .store
        .instance_export(module, "add_one_if_true")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(7, instance.invoke_typed(add_one_if_true_fn, 1).unwrap());
    assert_eq!(5, instance.invoke_typed(add_one_if_true_fn, 0).unwrap());
}

#[test_log::test]
fn if_without_else_type_check4() {
    let wasm_bytes = wat::parse_str(
        r#"
(module
    (func (export "do_stuff_if_true") (param $cond i32) (result i32) (result i64)
        (i32.const 5)
        (i64.const 20)
        (if (param i32) (param i64) (result i32) (result i64) (local.get $cond) (then drop (i32.const 2) (i32.add) (i64.const 42)))
    )
)"#,
    )
    .unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let add_one_if_true_fn = instance
        .store
        .instance_export(module, "do_stuff_if_true")
        .unwrap()
        .as_func()
        .unwrap();
    assert_eq!(
        (7, 42),
        instance
            .invoke_typed::<i32, (i32, i64)>(add_one_if_true_fn, 1)
            .unwrap()
    );
    assert_eq!(
        (5, 20),
        instance
            .invoke_typed::<i32, (i32, i64)>(add_one_if_true_fn, 0)
            .unwrap()
    );
}
