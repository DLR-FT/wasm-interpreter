use interop::StoreTypedInvocationExt;
use wasm::{resumable::RunState, validate, Store, Value};

#[test_log::test]
fn invoke_typed() {
    let wat = r#"
    (module
        (func (export "add_two") (param i32) (result i32)
            local.get 0
            i32.const 2
            i32.add
        )
    )"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).unwrap();
    let mut store = Store::new(());

    // SAFETY: There are no extern values and therefore none can be invalid in
    // this store.
    let module = unsafe { store.module_instantiate(&validation_info, Vec::new(), None) }
        .unwrap()
        .module_addr;

    // SAFETY: This module address just came from the same store.
    let add_two = unsafe { store.instance_export(module, "add_two") }
        .unwrap()
        .as_func()
        .unwrap();

    // SAFETY: The function address just came from the same store and no address
    // type values are used.
    let five_plus_two = unsafe { store.invoke_simple_typed::<i32, i32>(add_two, 5) }.unwrap();

    assert_eq!(five_plus_two, 7);
}

#[test_log::test]
fn host_function() {
    let mut store = Store::new(());

    let consume_i32 = store.func_alloc_typed::<i32, ()>(123);

    // SAFETY: The function address just came from the same store and no address
    // type values are used.
    let run_state = unsafe { store.invoke(consume_i32, vec![Value::I32(20)], None) }.unwrap();

    match run_state {
        RunState::HostCalled { host_call, .. } => {
            assert_eq!(host_call.hostcode, 123);
            assert_eq!(&*host_call.params, &[Value::I32(20)]);
        }
        _ => panic!("expected RunState::HostCalled, but got other run state"),
    }
}
