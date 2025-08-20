use itertools::repeat_n;
use wasm::{validate, RunState, RuntimeInstance, Store};

#[test_log::test]

fn out_of_fuel() {
    let wat = r#"
            (module
            (func (export "loop_forever") (loop br 0)
            ))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = &validate(&wasm_bytes).expect("validation failed");
    let mut runtime_instance = RuntimeInstance::new_named((), "module", &validation_info).unwrap();
    let func_ref = runtime_instance
        .get_function_by_name("module", "loop_forever")
        .unwrap();
    assert!(matches!(
        runtime_instance
            .invoke_resumable(&func_ref, vec![], 40)
            .unwrap(),
        RunState::Resumable(_)
    ));
}

fn resumable() {
    let wat = r#"
    (module
        (global $global_0 (mut i32)
            i32.const 4
        )
        (global $global_1 (mut i32)
            i32.const 8
        )

        ;; multiply global 0 forever
        (func (export "mult_global_0")
            (loop
                global.get $global_0
                i32.const 2
                i32.mul
                global.set $global_0
            )
        )

        ;; add 3 to global_1 forever
        (func (export "add_global_1")
            (loop
                global.get $global_1
                i32.const 3
                i32.add
                global.set $global_1
            )
        )
    )"#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut runtime_instance = RuntimeInstance::new_named((), "module", &validation_info).unwrap();
    let mult_global_0 = runtime_instance
        .get_function_by_name("module", "mult_global_0")
        .unwrap();
    // let add_global_1 = runtime_instance.get_function_by_name("module", "add_global_1").unwrap();

    let mut run_state_mult = runtime_instance
        .invoke_resumable(&mult_global_0, vec![], 0)
        .unwrap();
    // multiple resumables cause problems with borrow checker
    // let add_state_mult = runtime_instance.invoke_resumable(&mult_global_0, vec![], 0).unwrap();

    for _ in 0..10 {
        match run_state_mult {
            RunState::Finished(_) => unreachable!("infinite loop can't terminate"),
            RunState::Resumable(mut resumable) => {
                run_state_mult = resumable.resume(5).unwrap();
            }
        }
    }
}
