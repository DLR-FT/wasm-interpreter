use core::panic;
use log::info;
use wasm::{resumable::RunState, validate, RuntimeInstance};

#[test_log::test]

fn out_of_fuel() {
    let wat = r#"
            (module
            (func (export "loop_forever") (loop br 0)
            ))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = &validate(&wasm_bytes).expect("validation failed");
    let mut runtime_instance = RuntimeInstance::new_named((), "module", validation_info).unwrap();
    let func_ref = runtime_instance
        .get_function_by_name("module", "loop_forever")
        .unwrap();
    let resumable_ref = runtime_instance
        .create_resumable(&func_ref, Vec::new(), 40)
        .unwrap();
    assert!(matches!(
        runtime_instance.resume(resumable_ref).unwrap(),
        RunState::Resumable { .. }
    ));
}
#[test_log::test]
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
                nop
                nop
                i32.mul
                global.set $global_0
                br 0
            )
        )

        ;; add 3 to global_1 forever
        (func (export "add_global_1")
            (loop
                global.get $global_1
                i32.const 3
                i32.add
                global.set $global_1
                br 0
            )
        )
    )"#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut runtime_instance = RuntimeInstance::new_named((), "module", &validation_info).unwrap();
    let mult_global_0 = runtime_instance
        .get_function_by_name("module", "mult_global_0")
        .unwrap();
    let add_global_1 = runtime_instance
        .get_function_by_name("module", "add_global_1")
        .unwrap();

    let resumable_ref_mult = runtime_instance
        .create_resumable(&mult_global_0, vec![], 0)
        .unwrap();
    let resumable_ref_add = runtime_instance
        .create_resumable(&add_global_1, vec![], 0)
        .unwrap();

    let mut run_state_mult = runtime_instance.resume(resumable_ref_mult).unwrap();
    let mut run_state_add = runtime_instance.resume(resumable_ref_add).unwrap();

    let increment = |maybe_fuel: &mut Option<u32>| *maybe_fuel = maybe_fuel.map(|fuel| fuel + 2);

    for _ in 0..20 {
        run_state_mult = match run_state_mult {
            RunState::Finished(_) => panic!("should not terminate"),
            RunState::Resumable {
                mut resumable_ref, ..
            } => {
                runtime_instance
                    .access_fuel_mut(&mut resumable_ref, increment)
                    .unwrap();
                runtime_instance.resume(resumable_ref).unwrap()
            }
        };
        info!("Global values are {:?}", &runtime_instance.store.globals);
        run_state_add = match run_state_add {
            RunState::Finished(_) => panic!("should not terminate"),
            RunState::Resumable {
                mut resumable_ref, ..
            } => {
                runtime_instance
                    .access_fuel_mut(&mut resumable_ref, increment)
                    .unwrap();
                runtime_instance.resume(resumable_ref).unwrap()
            }
        };
        info!("Global values are {:?}", &runtime_instance.store.globals)
    }
}

#[test_log::test]
fn resumable_internal_state() {
    let wat = r#"(module
        (global $global_0 (mut i32)
            i32.const 0
        )
        ;; add 1 to global_0 to track internal state
        (func (export "add_global_0")
          global.get $global_0
          i32.const 1
          i32.add
          global.set $global_0
          global.get $global_0
          i32.const 10
          i32.add
          global.set $global_0
          global.get $global_0
          i32.const 100
          i32.add
          global.set $global_0
          global.get $global_0
          i32.const 1000
          i32.add
          global.set $global_0
        )
    )"#;
    let expected = [0, 1, 11, 111, 1111];
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut runtime_instance = RuntimeInstance::new_named((), "module", &validation_info).unwrap();
    let add_global_0 = runtime_instance
        .get_function_by_name("module", "add_global_0")
        .unwrap();
    let resumable_ref_add = runtime_instance
        .create_resumable(&add_global_0, vec![], 4)
        .unwrap();
    assert_eq!(
        runtime_instance.store.globals[0].value,
        wasm::Value::I32(expected[0])
    );
    let mut run_state_add = runtime_instance.resume(resumable_ref_add).unwrap();
    let increment = |maybe_fuel: &mut Option<u32>| *maybe_fuel = maybe_fuel.map(|fuel| fuel + 4);
    for i in 1..4 {
        run_state_add = match run_state_add {
            RunState::Finished(_) => {
                assert_eq!(
                    runtime_instance.store.globals[0].value,
                    wasm::Value::I32(expected[i])
                );
                return;
            }
            RunState::Resumable {
                mut resumable_ref, ..
            } => {
                assert_eq!(
                    runtime_instance.store.globals[0].value,
                    wasm::Value::I32(expected[i])
                );
                runtime_instance
                    .access_fuel_mut(&mut resumable_ref, increment)
                    .unwrap();
                runtime_instance.resume(resumable_ref).unwrap()
            }
        }
    }
}

#[test_log::test]
fn resumable_drop() {
    let wat = r#"
            (module
            (func (export "loop_forever") (loop br 0)
            ))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = &validate(&wasm_bytes).expect("validation failed");
    let mut runtime_instance = RuntimeInstance::new_named((), "module", validation_info).unwrap();
    let func_ref = runtime_instance
        .get_function_by_name("module", "loop_forever")
        .unwrap();
    let resumable_ref = runtime_instance
        .create_resumable(&func_ref, Vec::new(), 40)
        .unwrap();
    {
        let resumable_ref = runtime_instance
            .create_resumable(&func_ref, Vec::new(), 40)
            .unwrap();
        assert!(matches!(
            runtime_instance.resume(resumable_ref).unwrap(),
            RunState::Resumable { .. }
        ));
        // now drop it, the other resumable should still be able to access the dormitory in store
    }
    assert!(matches!(
        runtime_instance.resume(resumable_ref).unwrap(),
        RunState::Resumable { .. }
    ));
}
