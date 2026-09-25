use dlr_wasm_interpreter::{decode_and_validate, BytecodeProvider, RunState, Store, Value};
use dlr_wasm_interpreter_interop::StoreTypedInvocationExt;

struct SingleBytecodeRef<'a>(&'a [u8]);
impl BytecodeProvider for SingleBytecodeRef<'_> {
    fn get_bytecode(&self, _id: usize) -> &[u8] {
        self.0
    }
}

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

    let module = decode_and_validate(&wasm_bytes, &mut ()).unwrap();
    let mut store = Store::new(());

    let bytecode_provider = SingleBytecodeRef(&wasm_bytes);

    // SAFETY: There are no extern values and therefore none can be invalid in
    // this store.
    let module =
        unsafe { store.module_instantiate(&module, &bytecode_provider, 0, Vec::new(), None) }
            .unwrap()
            .module_addr;

    // SAFETY: This module address just came from the same store.
    let add_two = unsafe { store.instance_export(module, &bytecode_provider, "add_two") }
        .unwrap()
        .as_func()
        .unwrap();

    // SAFETY: The function address just came from the same store and no address
    // type values are used.
    let five_plus_two =
        unsafe { store.invoke_simple_typed::<i32, i32, _>(&bytecode_provider, add_two, 5) }
            .unwrap();

    assert_eq!(five_plus_two, 7);
}

#[test_log::test]
fn host_function() {
    let mut store = Store::new(());

    let consume_i32 = store.func_alloc_typed::<i32, ()>(123);

    let bytecode_provider = SingleBytecodeRef(&[]);
    // SAFETY: The function address just came from the same store and no address
    // type values are used.
    let run_state =
        unsafe { store.invoke(consume_i32, vec![Value::I32(20)], None, &bytecode_provider) }
            .unwrap();

    match run_state {
        RunState::HostCalled { host_call, .. } => {
            assert_eq!(host_call.hostcode, 123);
            assert_eq!(&*host_call.params, &[Value::I32(20)]);
        }
        _ => panic!("expected RunState::HostCalled, but got other run state"),
    }
}
