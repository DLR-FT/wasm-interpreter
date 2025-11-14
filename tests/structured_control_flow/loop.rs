use wasm::{validate, RuntimeInstance};

const FIBONACCI_WITH_LOOP_AND_BR_IF: &str = r#"
(module
  (func (export "fibonacci") (param $n i32) (result i32)
    (local $prev i32)
    (local $curr i32)
    (local $counter i32)

    i32.const 0
    local.set $prev
    i32.const 1
    local.set $curr

    local.get $n
    i32.const 1
    i32.add
    local.set $counter

    block $exit
      loop $loop
        local.get $counter
        i32.const 1
        i32.le_s
        br_if $exit

        local.get $curr
        local.get $curr
        local.get $prev
        i32.add
        local.set $curr
        local.set $prev

        local.get $counter
        i32.const 1
        i32.sub
        local.set $counter

        br $loop

        drop
        drop
        drop

      end $loop
    end $exit

    local.get $curr
  )
)"#;

#[test_log::test]
fn fibonacci_with_loop_and_br_if() {
    let wasm_bytes = wat::parse_str(FIBONACCI_WITH_LOOP_AND_BR_IF).unwrap();

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
