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

const BRANCH23_WAT: &str = r#"
(module
    (func (export "with_branch") (result i32)
        (block $outer_outer_block (result i32)
            i64.const 3
            (block $outer_block (param i64) (result i32) (result i32)
                drop
                i32.const 14
                (block $my_block (result i32)
                    i32.const 11
                    i32.const 8
                    i32.const 5
                    br {{LABEL}}
                    i32.const 3
                )
                i32.const 3
                i32.add
            )
            drop
            i32.const 5
            i32.add
        )
    )
)
"#;

#[test_log::test]
fn branch2() {
    let wat = String::from(BRANCH23_WAT).replace("{{LABEL}}", "$outer_block");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        13,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

#[test_log::test]
fn branch3() {
    let wat = String::from(BRANCH23_WAT).replace("{{LABEL}}", "$outer_outer_block");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        5,
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
            (func (export "add_one") (param $x i32) (result i32)
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

const RETURN_OUT_OF_BLOCK: &str = r#"
(module
    (func (export "get_three") (result i32)
        (block
            i32.const 5
            i32.const 3
            {{RETURN}}
        )
        unreachable
    )
)
"#;

const RETURN_OUT_OF_BLOCK2: &str = r#"
(module
    (func (export "get_three") (result i32)
        (block
            i32.const 5
            {{RETURN}}
            drop
            drop
            drop
        )
        unreachable
    )
)
"#;

#[test_log::test]
fn return_out_of_block() {
    let wat = String::from(RETURN_OUT_OF_BLOCK).replace("{{RETURN}}", "return");
    let wasm_bytes = wat::parse_str(wat).unwrap();

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
fn br_return_out_of_block() {
    let wat = String::from(RETURN_OUT_OF_BLOCK).replace("{{RETURN}}", "br 1");
    let wasm_bytes = wat::parse_str(wat).unwrap();

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
fn return_out_of_block2() {
    let wat = String::from(RETURN_OUT_OF_BLOCK2).replace("{{RETURN}}", "return");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        5,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

#[test_log::test]
fn br_return_out_of_block2() {
    let wat = String::from(RETURN_OUT_OF_BLOCK2).replace("{{RETURN}}", "br 1");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        5,
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

    let switch_case_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(6, instance.invoke(&switch_case_fn, 6).unwrap());
    assert_eq!(123, instance.invoke(&switch_case_fn, -123).unwrap());
    assert_eq!(0, instance.invoke(&switch_case_fn, 0).unwrap());
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

    let first_ten = (0..10)
        .map(|n| instance.invoke(&fib_fn, n).unwrap())
        .collect::<Vec<i32>>();
    assert_eq!(&first_ten, &[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
}

#[test_log::test]
fn switch_case() {
    let wasm_bytes = wat::parse_str(
        r#"
    (module
        (func $switch_case (param $value i32) (result i32)
        (block $default
            (block $case4
                (block $case3
                    (block $case2
                        (block $case1
                            local.get $value
                            (br_table $case1 $case2 $case3 $case4 $default)
                        )
                        i32.const 1
                        return
                    )
                    i32.const 3
                    return
                )
                i32.const 5
                return
            )
            i32.const 7
            return
        )
        i32.const 9
        return
    )
    (export "switch_case" (func $switch_case))
    )"#,
    )
    .unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let switch_case_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(9, instance.invoke(&switch_case_fn, -5).unwrap());
    assert_eq!(9, instance.invoke(&switch_case_fn, -1).unwrap());
    assert_eq!(1, instance.invoke(&switch_case_fn, 0).unwrap());
    assert_eq!(3, instance.invoke(&switch_case_fn, 1).unwrap());
    assert_eq!(5, instance.invoke(&switch_case_fn, 2).unwrap());
    assert_eq!(7, instance.invoke(&switch_case_fn, 3).unwrap());
    assert_eq!(9, instance.invoke(&switch_case_fn, 4).unwrap());
    assert_eq!(9, instance.invoke(&switch_case_fn, 7).unwrap());
}
