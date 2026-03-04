use core::panic;
use log::info;
use wasm::{
    checked::Store,
    checked::{StoredRunState, StoredValue},
    validate,
};

#[test_log::test]

fn out_of_fuel() {
    let wat = r#"
            (module
            (func (export "loop_forever") (loop br 0)
            ))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;
    let func_addr = store
        .instance_export(module, "loop_forever")
        .unwrap()
        .as_func()
        .unwrap();
    let resumable = store
        .create_resumable(func_addr, Vec::new(), Some(40))
        .unwrap();
    assert!(matches!(
        store.resume(resumable).unwrap(),
        StoredRunState::Resumable { .. }
    ));
}
#[test_log::test]
fn resumable() {
    let wat = r#"
    (module
        (global $global_0 (export "global_0") (mut i32)
            i32.const 4
        )
        (global $global_1 (export "global_1") (mut i32)
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let mult_global_0 = store
        .instance_export(module, "mult_global_0")
        .unwrap()
        .as_func()
        .unwrap();
    let add_global_1 = store
        .instance_export(module, "add_global_1")
        .unwrap()
        .as_func()
        .unwrap();
    let global_0 = store
        .instance_export(module, "global_0")
        .unwrap()
        .as_global()
        .unwrap();

    let global_1 = store
        .instance_export(module, "global_1")
        .unwrap()
        .as_global()
        .unwrap();

    let resumable_mult = store
        .create_resumable(mult_global_0, vec![], Some(0))
        .unwrap();
    let resumable_add = store
        .create_resumable(add_global_1, vec![], Some(0))
        .unwrap();

    let mut run_state_mult = store.resume(resumable_mult).unwrap();
    let mut run_state_add = store.resume(resumable_add).unwrap();

    for _ in 0..20 {
        run_state_mult = match run_state_mult {
            StoredRunState::Finished { .. } => panic!("should not terminate"),
            StoredRunState::Resumable { mut resumable, .. } => {
                if let Some(fuel) = resumable.fuel_mut() {
                    *fuel += 2;
                }
                store.resume(resumable).unwrap()
            }
        };

        info!(
            "Global values are global_0={:?} global_1={:?}",
            store.global_read(global_0),
            store.global_read(global_1),
        );

        run_state_add = match run_state_add {
            StoredRunState::Finished { .. } => panic!("should not terminate"),
            StoredRunState::Resumable { mut resumable, .. } => {
                if let Some(fuel) = resumable.fuel_mut() {
                    *fuel += 2;
                }
                store.resume(resumable).unwrap()
            }
        };

        info!(
            "Global values are global_0={:?} global_1={:?}",
            store.global_read(global_0),
            store.global_read(global_1),
        );
    }
}

#[test_log::test]
fn resumable_internal_state() {
    let wat = r#"(module
        (global $global_0 (export "global_0") (mut i32)
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;
    let add_global_0 = store
        .instance_export(module, "add_global_0")
        .unwrap()
        .as_func()
        .unwrap();
    let global_0 = store
        .instance_export(module, "global_0")
        .unwrap()
        .as_global()
        .unwrap();
    let resumable_add = store
        .create_resumable(add_global_0, vec![], Some(4))
        .unwrap();
    assert_eq!(
        store.global_read(global_0),
        Ok(StoredValue::I32(expected[0]))
    );
    let mut run_state_add = store.resume(resumable_add).unwrap();
    for expected in expected.into_iter().take(4).skip(1) {
        run_state_add = match run_state_add {
            StoredRunState::Finished { .. } => {
                assert_eq!(store.global_read(global_0), Ok(StoredValue::I32(expected)));
                return;
            }
            StoredRunState::Resumable { mut resumable, .. } => {
                assert_eq!(store.global_read(global_0), Ok(StoredValue::I32(expected)));
                if let Some(fuel) = resumable.fuel_mut() {
                    *fuel += 4;
                }
                store.resume(resumable).unwrap()
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
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;
    let func_addr = store
        .instance_export(module, "loop_forever")
        .unwrap()
        .as_func()
        .unwrap();
    let resumable = store
        .create_resumable(func_addr, Vec::new(), Some(40))
        .unwrap();
    {
        let resumable = store
            .create_resumable(func_addr, Vec::new(), Some(40))
            .unwrap();

        let StoredRunState::Resumable { .. } = store.resume(resumable).unwrap() else {
            panic!("expected unfinished resumable");
        };
    }

    // the outer resumable should still be able to access the dormitory in store
    assert!(matches!(
        store.resume(resumable).unwrap(),
        StoredRunState::Resumable { .. }
    ));
}

static FUELED_INITIALIZATION_WAT: &str = r#"(module
    (func $start
      (nop)
    )
    (start $start)
)"#;

#[test_log::test]
fn fueled_initialization() {
    let wasm_bytes = wat::parse_str(FUELED_INITIALIZATION_WAT).unwrap();
    let validation_info = &validate(&wasm_bytes).expect("validation falied");
    let mut store = Store::new(());
    let module = store.module_instantiate(validation_info, Vec::new(), Some(2));
    assert!(module.is_ok());
}

#[test_log::test]
fn fueled_initialization_fail() {
    let wasm_bytes = wat::parse_str(FUELED_INITIALIZATION_WAT).unwrap();
    let validation_info = &validate(&wasm_bytes).expect("validation falied");
    let mut store = Store::new(());
    let module = store.module_instantiate(validation_info, Vec::new(), Some(0));
    assert!(matches!(module, Err(wasm::RuntimeError::OutOfFuel)));
}
