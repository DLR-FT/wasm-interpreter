fn main() {
    let src = include_bytes!("./fibonacci.wasm");

    let validation_info = wasm::validate(src).unwrap();
    let mut store = wasm::Store::new(());
    let module = unsafe {
        store
            .module_instantiate(&validation_info, Vec::new(), None)
            .unwrap()
            .module_addr
    };
    let fibonacci = unsafe {
        store
            .instance_export(module, "fibonacci")
            .unwrap()
            .as_func()
            .unwrap()
    };

    let res = unsafe { store.invoke_simple(fibonacci, vec![wasm::Value::I32(1)]) };
    println!("{res:?}")
}
