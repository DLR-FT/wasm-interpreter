use checked::{Linker, Store, StoredExternVal, StoredRunState, StoredValue};

use crate::registry::Registry;

mod registry;

fn main() {
    let mut store = Store::new(());

    // Create a registry and then allocate some host functions in our store
    let mut registry = Registry::default();
    let add = registry.alloc_host_function_typed(&mut store, |(a, b): (u32, u32)| a + b);
    let mul = registry.alloc_host_function_typed(&mut store, |(a, b): (u32, u32)| a * b);

    // Add the host functions with names to a linker.
    let mut linker = Linker::new();
    linker
        .define(
            "env".to_owned(),
            "add".to_owned(),
            StoredExternVal::Func(add),
        )
        .unwrap();
    linker
        .define(
            "env".to_owned(),
            "mul".to_owned(),
            StoredExternVal::Func(mul),
        )
        .unwrap();

    // Now validate our module and instantiate it in the previously created linker context.
    let validation_info = wasm::validate(include_bytes!("./module.wasm")).unwrap();
    let module = linker
        .module_instantiate(&mut store, &validation_info, None)
        .unwrap()
        .unwrap()
        .module_addr;

    // Find the exported function of our module
    let mul_then_add = store
        .instance_export(module, "mul_then_add")
        .unwrap()
        .as_func()
        .unwrap();

    // Create a resumable
    let resumable = store
        .create_resumable(
            mul_then_add,
            vec![
                StoredValue::I32(3),
                StoredValue::I32(4),
                StoredValue::I32(5),
            ],
            Some(1),
        )
        .unwrap();

    // Run until finished, keep re-fueling when fuel runs out and forward host
    // calls to the registry.
    let mut run_state = store.resume(resumable).unwrap();
    let values = loop {
        match run_state {
            StoredRunState::Finished { values, .. } => break values,
            StoredRunState::Resumable { mut resumable, .. } => {
                if let Some(fuel) = resumable.fuel_mut() {
                    *fuel += 2;
                }
                run_state = store.resume_wasm(resumable).unwrap();
            }
            StoredRunState::HostCalled {
                host_call,
                resumable: host_resumable,
            } => run_state = registry.perform_host_call(&mut store, host_call, host_resumable),
        }
    };

    assert_eq!(&values, &[StoredValue::I32(3 * 4 + 5)]);
}
