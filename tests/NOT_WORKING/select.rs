use wasm::{validate, RuntimeInstance};

const SELECT_TEST: &str = r#"
(module
  (func $select_test (param $num i32) (result i32)
    (if (result i32)
      (i32.le_s
        (local.get $num)
        (i32.const 1)
      )
      (then
        (select {{TYPE_1}}
          (i32.const 8)
          (i32.const 4)
          (local.get $num)
        )
      )
      (else
        (i32.wrap_i64
          (select {{TYPE_2}}
            (i64.const 16)
            (i64.const 15)
            (i32.sub (local.get $num) (i32.const 2))
          )
        )
      )
    )
  )

  (export "select_test" (func $select_test))
)"#;

#[test_log::test]
fn polymorphic_select_test() {
    let wat = String::from(SELECT_TEST)
        .replace("{{TYPE_1}}", "")
        .replace("{{TYPE_2}}", "");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    validate(&wasm_bytes).expect("validation failed");

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let select_test_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(4, instance.invoke(&select_test_fn, 0).unwrap());
    assert_eq!(8, instance.invoke(&select_test_fn, 1).unwrap());
    assert_eq!(15, instance.invoke(&select_test_fn, 2).unwrap());
    assert_eq!(16, instance.invoke(&select_test_fn, 3).unwrap());
}

#[test_log::test]
fn typed_select_test() {
    let wat = String::from(SELECT_TEST)
        .replace("{{TYPE_1}}", "(result i32)")
        .replace("{{TYPE_2}}", "(result i64)");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    validate(&wasm_bytes).expect("validation failed");

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let select_test_fn = instance.get_function_by_index(0, 0).unwrap();

    assert_eq!(4, instance.invoke(&select_test_fn, 0).unwrap());
    assert_eq!(8, instance.invoke(&select_test_fn, 1).unwrap());
    assert_eq!(15, instance.invoke(&select_test_fn, 2).unwrap());
    assert_eq!(16, instance.invoke(&select_test_fn, 3).unwrap());
}
