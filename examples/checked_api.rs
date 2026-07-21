//! Using the checked utility crate for a safe API
//!
//! The majority of functions in the [`dlr_wasm_interpreter`] crate is marked as unsafe. While this
//! is useful in very specialized or resource-constrained environments, usually runtime checks to
//! ensure these safety conditions suffice. For that, we provide the utility crate
//! [`dlr_wasm_interpreter_checked`], which provides a safe and checked API.
//!
//! This example is exactly the same as `function_invocation.rs`, except that it uses the checked
//! API.

use std::error::Error;

use dlr_wasm_interpreter::{decode_and_validate, FuncAddr, Module};
use dlr_wasm_interpreter_checked::{
    Store, Stored, StoredExternVal, StoredInstantiationOutcome, StoredValue,
};

const WAT_CODE: &str = r#"
(module
    (func (export "add_one") (param $n i32) (result i32)
        local.get $n
        i32.const 1
        i32.add
    )
)
"#;

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_bytecode: Vec<u8> = wat::parse_str(WAT_CODE)?;
    let module: Module = decode_and_validate(&wasm_bytecode, &mut ())?;

    // Note: This is a `dlr_wasm_interpreter_checked::Store`, not a `dlr_wasm_interpreter::Store`
    let mut store: Store<()> = Store::new(());

    // No unsafe code here, but we get a *Stored*InstantiationOutcome back.
    let instantiation_outcome: StoredInstantiationOutcome =
        store.module_instantiate(&module, vec![], None)?;

    // `module_addr` is now of type `Stored<ModuleAddr>`. A `Stored<T>` acts like a T normally
    // would, except that it remembers which store it came from, preventing it to be used with
    // another store.
    let StoredInstantiationOutcome {
        module_addr,
        maybe_remaining_fuel: _,
    } = instantiation_outcome;

    let add_one: StoredExternVal = store.instance_export(module_addr, "add_one")?;
    let add_one: Stored<FuncAddr> = add_one.as_func().ok_or("add_one is not a function")?;

    let parameters: Vec<StoredValue> = vec![StoredValue::I32(16)];
    let return_values: Vec<StoredValue> = store.invoke_simple(add_one, parameters)?;

    let [StoredValue::I32(n)] = *return_values else {
        return Err("expected one i32 as a return value".into());
    };

    assert_eq!(n, 16 + 1);

    // Using any stored object with another store would panic:
    //
    // let mut second_store = Store::new(());
    // let _ = second_store.invoke_simple(add_one, Vec::new());

    Ok(())
}
