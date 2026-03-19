use log::info;

use checked::{Store, StoredExternVal, StoredValue};
use registry::Registry;
use wasm::{
    validate,
    value::{F32, F64},
    FuncType, NumType, ResultType, RuntimeError, ValType,
};

#[test_log::test]
pub fn host_func_call_within_module() {
    let wat = r#"(module
    (import "hello_mod" "hello" (func $hello (param) (result)))
    (func (export "hello_caller") (param i32) (result i32)
        local.get 0
        i32.const 2
        call $hello
        i32.add
    )
)"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut store = Store::new(());
    let mut registry = Registry::default();
    let hello = registry.alloc_host_function_typed(&mut store, |(), ()| {
        info!("Host function says hello from wasm!");
    });
    let importing_mod = store
        .module_instantiate(&validation_info, vec![StoredExternVal::Func(hello)], None)
        .unwrap()
        .module_addr;
    let function_ref = store
        .instance_export(importing_mod, "hello_caller")
        .unwrap()
        .as_func()
        .unwrap();
    let result = registry
        .invoke_without_fuel_typed::<_, i32, i32>(&mut (), &mut store, function_ref, 2)
        .expect("wasm function invocation failed");
    assert_eq!(4, result);
}

#[test_log::test]
pub fn host_func_call_as_first_func() {
    let mut store = Store::new(());
    let mut registry = Registry::default();

    let hello = registry.alloc_host_function_typed(&mut store, |(), ()| {
        info!("Host function says hello from wasm!");
    });

    let result = registry.invoke_without_fuel_typed(&mut (), &mut store, hello, ());
    assert_eq!(Ok(()), result);
}

#[test_log::test]
#[ignore = "host functions calls are not yet supported from the start function"]
pub fn host_func_call_as_start_func() {
    let wat = r#"(module
    (import "hello_mod" "hello" (func $hello (param) (result)))
    (start $hello)
)"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();

    let mut store = Store::new(());
    let mut registry = Registry::default();
    let hello = registry.alloc_host_function_typed(&mut store, |(), ()| {
        info!("Host function says hello from wasm!");
    });

    let _module_addr = store
        .module_instantiate(&validation_info, vec![StoredExternVal::Func(hello)], None)
        .expect("instantiation to be successful");
}

#[test_log::test]
#[ignore = "host functions calls are not yet supported from the start function"]
pub fn host_func_call_within_start_func() {
    let wat = r#"(module
    (import "hello_mod" "hello" (func $hello (param) (result)))
    (func $hello_caller (export "hello_caller") (param) (result)
        i32.const 2
        i32.const 2
        call $hello
        i32.add
        drop
    )
    (start $hello_caller)
)"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut store = Store::new(());
    let mut registry = Registry::default();
    let hello = registry.alloc_host_function_typed(&mut store, |(), ()| {
        info!("Host function says hello from wasm!");
    });
    let _module_addr = store
        .module_instantiate(&validation_info, vec![StoredExternVal::Func(hello)], None)
        .expect("instantiation to be successful");
}

fn fancy_add_mult(_: &mut (), (x, y): (u32, f64)) -> (f64, u32) {
    info!("multiplying, adding, casting, swapping as host function");
    ((x as f64) * y, x + (y as u32))
}

const SIMPLE_MULTIVARIATE_MODULE_EXAMPLE: &str = r#"(module
    (import "hello_mod" "fancy_add_mult" (func $fancy_add_mult (param i32 f64) (result f64 i32)))
    (func $fancy_add_mult_caller (export "fancy_add_mult_caller") (param) (result f64 i32 i64)
        i32.const 2
        f64.const 4.0
        call $fancy_add_mult
        i64.const 5
    )
)"#;

#[test_log::test]
pub fn simple_multivariate_host_func_within_module() {
    let wasm_bytes = wat::parse_str(SIMPLE_MULTIVARIATE_MODULE_EXAMPLE).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();

    let mut store = Store::new(());
    let mut registry = Registry::default();
    let fancy_add_mult = registry.alloc_host_function_typed(&mut store, fancy_add_mult);

    let importing_mod = store
        .module_instantiate(
            &validation_info,
            vec![StoredExternVal::Func(fancy_add_mult)],
            None,
        )
        .unwrap()
        .module_addr;

    let function_ref = store
        .instance_export(importing_mod, "fancy_add_mult_caller")
        .unwrap()
        .as_func()
        .unwrap();
    let result = registry
        .invoke_without_fuel_typed::<_, (), (f64, i32, i64)>(&mut (), &mut store, function_ref, ())
        .expect("wasm function invocation failed");
    assert_eq!((8.0, 6, 5), result);
}

#[test_log::test]
pub fn simple_multivariate_host_func_with_host_func_wrapper() {
    let wasm_bytes = wat::parse_str(SIMPLE_MULTIVARIATE_MODULE_EXAMPLE).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();

    fn wrapped_add_mult(_: &mut (), (x, y): (i32, f64)) -> (f64, i32) {
        (y + (x as f64), x * (y as i32))
    }

    let mut store = Store::new(());
    let mut registry = Registry::default();
    let wrapped_add_mult = registry.alloc_host_function_typed(&mut store, wrapped_add_mult);
    let importing_mod = store
        .module_instantiate(
            &validation_info,
            vec![StoredExternVal::Func(wrapped_add_mult)],
            None,
        )
        .unwrap()
        .module_addr;

    let function_ref = store
        .instance_export(importing_mod, "fancy_add_mult_caller")
        .unwrap()
        .as_func()
        .unwrap();
    let result = registry
        .invoke_without_fuel_typed::<_, (), (f64, i32, i64)>(&mut (), &mut store, function_ref, ())
        .expect("wasm function invocation failed");
    assert_eq!((6.0, 8, 5), result);
}

#[test_log::test]
pub fn simple_multivariate_host_func_as_first_func() {
    let mut store = Store::new(());
    let mut registry = Registry::default();
    let fancy_add_mult = registry.alloc_host_function_typed(&mut store, fancy_add_mult);

    let result = registry
        .invoke_without_fuel_typed::<_, (i32, f64), (f64, i32)>(
            &mut (),
            &mut store,
            fancy_add_mult,
            (3, 5.0),
        )
        .expect("wasm function invocation failed");
    assert_eq!((15.0, 8), result);
}

#[test_log::test]
pub fn weird_multi_typed_host_func() {
    let wat = r#"(module
    (import "hello_mod" "weird_mult" (func $weird_mult (param i32) (result f64)))
    (import "hello_mod" "weird_add" (func $weird_add (param f32) (result i64)))
    (func $weird_add_mult_caller (export "weird_add_mult_caller") (param) (result f64 i64)
        i32.const 2
        call $weird_mult
        f32.const 3.0
        call $weird_add
))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();

    fn weird_add_mult(_: &mut (), values: Vec<StoredValue>) -> Vec<StoredValue> {
        Vec::from([match values[0] {
            StoredValue::I32(val) => {
                info!("host function saw I32");
                StoredValue::F64(F64((val * 5) as f64))
            }
            StoredValue::F32(F32(val)) => {
                info!("host function saw F32");
                StoredValue::I64((val + 3.0) as u64)
            }
            _ => panic!("no other types admitted"),
        }])
    }

    let mut store = Store::new(());
    let mut registry = Registry::default();

    let weird_mult = registry.alloc_host_function(
        &mut store,
        FuncType {
            params: ResultType {
                valtypes: vec![ValType::NumType(NumType::I32)],
            },
            returns: ResultType {
                valtypes: vec![ValType::NumType(NumType::F64)],
            },
        },
        weird_add_mult,
    );

    let weird_add = registry.alloc_host_function(
        &mut store,
        FuncType {
            params: ResultType {
                valtypes: vec![ValType::NumType(NumType::F32)],
            },
            returns: ResultType {
                valtypes: vec![ValType::NumType(NumType::I64)],
            },
        },
        weird_add_mult,
    );

    let importing_mod = store
        .module_instantiate(
            &validation_info,
            vec![
                StoredExternVal::Func(weird_mult),
                StoredExternVal::Func(weird_add),
            ],
            None,
        )
        .unwrap()
        .module_addr;

    let function_ref = store
        .instance_export(importing_mod, "weird_add_mult_caller")
        .unwrap()
        .as_func()
        .unwrap();

    let result = registry
        .invoke_without_fuel_typed::<_, (), (f64, i64)>(&mut (), &mut store, function_ref, ())
        .expect("wasm function invocation failed");
    assert_eq!((10.0, 6), result);
}

#[test_log::test]
pub fn host_func_runtime_error() {
    let wat = r#"(module
    (import "hello_mod" "mult3" (func $mult3 (param i32) (result i32)))
    (func $mult3_caller (export "mult3_caller") (param) (result)
        i32.const 2
        call $mult3
      	drop
    )
)"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");

    fn mult3(_: &mut (), values: Vec<StoredValue>) -> Vec<StoredValue> {
        let val: i32 = values[0].try_into().unwrap();
        info!("careless host function making type errors...");
        vec![StoredValue::I64((val * 3) as u64)]
    }

    let mut store = Store::new(());
    let mut registry = Registry::default();
    let mult3 = registry.alloc_host_function(
        &mut store,
        FuncType {
            params: ResultType {
                valtypes: vec![ValType::NumType(NumType::I32)],
            },
            returns: ResultType {
                valtypes: vec![ValType::NumType(NumType::I32)],
            },
        },
        mult3,
    );
    let importing_mod = store
        .module_instantiate(&validation_info, vec![StoredExternVal::Func(mult3)], None)
        .unwrap()
        .module_addr;
    let function_ref = store
        .instance_export(importing_mod, "mult3_caller")
        .unwrap()
        .as_func()
        .unwrap();
    let result = registry.invoke_without_fuel_typed::<_, (), (f64, i64)>(
        &mut (),
        &mut store,
        function_ref,
        (),
    );
    assert_eq!(Err(RuntimeError::HostFunctionSignatureMismatch), result);
}
