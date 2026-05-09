use std::fmt::Write;

fn main() {
    let src = include_bytes!("./fibonacci.wasm");

    let mut out = String::with_capacity(1_000_000);

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

    for _ in 0..100 {
        let result = unsafe { store.invoke_simple(fibonacci, vec![wasm::Value::I32(1_000_000)]) };
        writeln!(out, "{result:?}").unwrap();
    }

    print!("{out}");
}
