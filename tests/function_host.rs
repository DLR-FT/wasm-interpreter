use log::info;

use interop::{host_function_wrapper, StoreTypedInvocationExt};
use wasm::{
    validate,
    value::{F32, F64},
    ExternVal, HaltExecutionError, RuntimeError, Store, Value,
};

fn hello(_: &mut (), _values: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
    info!("Host function says hello from wasm!");
    Ok(Vec::new())
}

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

    fn hello(_: &mut (), _values: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        info!("Host function says hello from wasm!");
        Ok(Vec::new())
    }

    let mut store = Store::new(());
    // SAFETY: The host function does not have any parameter and return types.
    // Therefore it cannot use invalid addresses.
    let hello = unsafe { store.func_alloc_typed::<(), ()>(hello) };
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let importing_mod =
        unsafe { store.module_instantiate(&validation_info, vec![ExternVal::Func(hello)], None) }
            .unwrap()
            .module_addr;
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let function_ref = unsafe { store.instance_export(importing_mod, "hello_caller") }
        .unwrap()
        .as_func()
        .unwrap();
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let result = unsafe { store.invoke_typed_without_fuel::<i32, i32>(function_ref, 2) }
        .expect("wasm function invocation failed");
    assert_eq!(4, result);
}

#[test_log::test]
pub fn host_func_call_as_first_func() {
    let mut store = Store::new(());
    // SAFETY: The host function does not have any parameter and return types.
    // Therefore it cannot use invalid addresses.
    let hello = unsafe { store.func_alloc_typed::<(), ()>(hello) };
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let result = unsafe { store.invoke_typed_without_fuel::<(), ()>(hello, ()) };
    assert_eq!(Ok(()), result);
}

#[test_log::test]
pub fn host_func_call_as_start_func() {
    let wat = r#"(module
    (import "hello_mod" "hello" (func $hello (param) (result)))
    (start $hello)
)"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();

    let mut store = Store::new(());
    // SAFETY: The host function does not have any parameter and return types.
    // Therefore it cannot use invalid addresses.
    let hello = unsafe { store.func_alloc_typed::<(), ()>(hello) };
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let _module_addr =
        unsafe { store.module_instantiate(&validation_info, vec![ExternVal::Func(hello)], None) }
            .expect("instantiation to be successful");
}

#[test_log::test]
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
    // SAFETY: The host function does not have any parameter and return types.
    // Therefore it cannot use invalid addresses.
    let hello = unsafe { store.func_alloc_typed::<(), ()>(hello) };
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let _module_addr =
        unsafe { store.module_instantiate(&validation_info, vec![ExternVal::Func(hello)], None) }
            .expect("instantiation to be successful");
}

fn fancy_add_mult(_: &mut (), values: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
    let x: u32 = values[0].try_into().unwrap();
    let y: f64 = values[1].try_into().unwrap();

    info!("multiplying, adding, casting, swapping as host function");

    Ok(Vec::from([
        Value::F64(F64((x as f64) * y)),
        Value::I32(x + (y as u32)),
    ]))
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
    // SAFETY: The host function does not have any address parameter or return
    // types. Therefore it cannot use invalid addresses.
    let fancy_add_mult =
        unsafe { store.func_alloc_typed::<(i32, f64), (f64, i32)>(fancy_add_mult) };
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let importing_mod = unsafe {
        store.module_instantiate(
            &validation_info,
            vec![ExternVal::Func(fancy_add_mult)],
            None,
        )
    }
    .unwrap()
    .module_addr;

    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let function_ref = unsafe { store.instance_export(importing_mod, "fancy_add_mult_caller") }
        .unwrap()
        .as_func()
        .unwrap();
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let result =
        unsafe { store.invoke_typed_without_fuel::<(), (f64, i32, i64)>(function_ref, ()) }
            .expect("wasm function invocation failed");
    assert_eq!((8.0, 6, 5), result);
}

#[test_log::test]
pub fn simple_multivariate_host_func_with_host_func_wrapper() {
    let wasm_bytes = wat::parse_str(SIMPLE_MULTIVARIATE_MODULE_EXAMPLE).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();

    fn wrapped_add_mult(_: &mut (), params: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper(
            params,
            |(x, y): (i32, f64)| -> Result<(f64, i32), HaltExecutionError> {
                Ok((y + (x as f64), x * (y as i32)))
            },
        )
    }

    let mut store = Store::new(());

    // SAFETY: The host function does not have any address parameter or return
    // types. Therefore it cannot use invalid addresses.
    let wrapped_add_mult =
        unsafe { store.func_alloc_typed::<(i32, f64), (f64, i32)>(wrapped_add_mult) };
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let importing_mod = unsafe {
        store.module_instantiate(
            &validation_info,
            vec![ExternVal::Func(wrapped_add_mult)],
            None,
        )
    }
    .unwrap()
    .module_addr;

    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let function_ref = unsafe { store.instance_export(importing_mod, "fancy_add_mult_caller") }
        .unwrap()
        .as_func()
        .unwrap();
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let result =
        unsafe { store.invoke_typed_without_fuel::<(), (f64, i32, i64)>(function_ref, ()) }
            .expect("wasm function invocation failed");
    assert_eq!((6.0, 8, 5), result);
}

#[test_log::test]
pub fn simple_multivariate_host_func_as_first_func() {
    let mut store = Store::new(());
    // SAFETY: The host function does not have any address parameter or return
    // types. Therefore it cannot use invalid addresses.
    let fancy_add_mult =
        unsafe { store.func_alloc_typed::<(i32, f64), (f64, i32)>(fancy_add_mult) };

    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let result = unsafe {
        store.invoke_typed_without_fuel::<(i32, f64), (f64, i32)>(fancy_add_mult, (3, 5.0))
    }
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

    fn weird_add_mult(_: &mut (), values: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        Ok(Vec::from([match values[0] {
            Value::I32(val) => {
                info!("host function saw I32");
                Value::F64(F64((val * 5) as f64))
            }
            Value::F32(F32(val)) => {
                info!("host function saw F32");
                Value::I64((val + 3.0) as u64)
            }
            _ => panic!("no other types admitted"),
        }]))
    }

    let mut store = Store::new(());

    // SAFETY: The host function does not have any address parameter or return
    // types. Therefore it cannot use invalid addresses.
    let weird_mult = unsafe { store.func_alloc_typed::<i32, f64>(weird_add_mult) };
    // SAFETY: The host function does not have any address parameter or return
    // types. Therefore it cannot use invalid addresses.
    let weird_add = unsafe { store.func_alloc_typed::<f32, i64>(weird_add_mult) };

    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let importing_mod = unsafe {
        store.module_instantiate(
            &validation_info,
            vec![ExternVal::Func(weird_mult), ExternVal::Func(weird_add)],
            None,
        )
    }
    .unwrap()
    .module_addr;

    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let function_ref = unsafe { store.instance_export(importing_mod, "weird_add_mult_caller") }
        .unwrap()
        .as_func()
        .unwrap();

    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let result = unsafe { store.invoke_typed_without_fuel::<(), (f64, i64)>(function_ref, ()) }
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

    fn mult3(_: &mut (), values: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        let val: i32 = values[0].try_into().unwrap();
        info!("careless host function making type errors...");
        Ok(Vec::from([Value::I64((val * 3) as u64)]))
    }

    let mut store = Store::new(());
    // SAFETY: The host function does not have any address parameter or return
    // types. Therefore it cannot use invalid addresses.
    let mult3 = unsafe { store.func_alloc_typed::<i32, i32>(mult3) };
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let importing_mod =
        unsafe { store.module_instantiate(&validation_info, vec![ExternVal::Func(mult3)], None) }
            .unwrap()
            .module_addr;
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let function_ref = unsafe { store.instance_export(importing_mod, "mult3_caller") }
        .unwrap()
        .as_func()
        .unwrap();
    // SAFETY: Only one store exists in this test. Therefore, it is always the
    // correct store.
    let result = unsafe { store.invoke_typed_without_fuel::<(), (f64, i64)>(function_ref, ()) };
    assert_eq!(Err(RuntimeError::HostFunctionSignatureMismatch), result);
}
