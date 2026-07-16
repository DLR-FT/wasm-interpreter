//! An explanation for how to instantiate a simple Wasm module and then invoke an exported function.
//!
//! # A Note on Safety
//!
//! Many operations on the [`Store`] are marked as unsafe. Unlike to other runtime we do not perform
//! a check on every operation to see if all passed addresses are correct. However, a wrapper that
//! performs automatic address checking can easily be implemented and is provided by the [`checked`]
//! utility crate. The crate is not used here as safety comments are usually very easy to write for
//! application that only ever create a single [`Store`] instance.

use std::error::Error;

use wasm::{ExternVal, FuncAddr, InstantiationOutcome, Module, Store, Value};

/// The Wasm module is defined in the WebAssembly Text Format (WAT). We convert it to the Wasm
/// bytecode format at runtime using the [`wat`] crate. You can also do this conversion at
/// compile-time by running a tool such as wat2wasm from [WABT](https://github.com/WebAssembly/wabt)
/// manually.
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
    // Convert WAT to Wasm bytecode
    let wasm_bytecode: Vec<u8> = wat::parse_str(WAT_CODE)?;

    // Decoding and validation is always done in one step. It produces a module we can use later.
    // User data is used for configuration of the validation process. You can ignore it for now.
    let module: Module = wasm::decode_and_validate(&wasm_bytecode, &mut ())?;

    // A store contains all of the objects needed during execution. It can also be configured via
    // some user data.
    let mut store: Store<()> = Store::new(());

    // Now we can instantiate our module in the store. The instantiation process creates a new
    // module instance. The module instance is stored in the store and a module address that
    // uniquely identifies it within this store is returned.
    //
    // Instantiation also requires us to pass in a vector of extern values, i.e. values imported by
    // the module. In our case the module does not specify any imports.
    //
    // Also, a fuel amount can be set to limit how much Wasm bytecode can be executed if a start
    // function exists.
    //
    // SAFETY: There are no extern values.
    let instantiation_outcome: InstantiationOutcome =
        unsafe { store.module_instantiate(&module, vec![], None) }?;

    let InstantiationOutcome {
        // The address identifying the newly created module instance
        module_addr,
        // Instantiation returns the amount of remaining fuel, which we disabled before.
        maybe_remaining_fuel: _,
    } = instantiation_outcome;

    // `Store::instance_export` is part of the Embedder API (see also: embedder_api.rs)
    // It allows to lookup extern values exported by a module.
    //
    // SAFETY: The module address was just returned from module instantiation in the same store.
    let add_one: ExternVal = unsafe { store.instance_export(module_addr, "add_one") }?;
    // Wasm modules can not only export functions, but also globals, memories and tables. We know
    // add_one is a function.
    let add_one: FuncAddr = add_one.as_func().ok_or("add_one is not a function")?;

    // Collect all parameters for invocation of add_one in a vec. Note: Reducing the number of
    // allocations made is work-in-progress.
    let parameters: Vec<Value> = vec![Value::I32(16)];
    // Invoke the add_one function by its unique function address with the parameters.
    //
    // We use the `Store::invoke_simple` method here because it offers a simplified interface
    // compared to its more powerful, but complex sibling `Store::invoke` (see host_functions.rs or
    // fuel.rs for how to use it).
    //
    // SAFETY: The function address was just returned from the same store. Also no addresses are
    // passed as parameters.
    let return_values: Vec<Value> = unsafe { store.invoke_simple(add_one, parameters) }?;

    // Now simply destructure the returned values
    let [Value::I32(n)] = *return_values else {
        return Err("expected one i32 as a return value".into());
    };

    assert_eq!(n, 16 + 1);

    Ok(())
}
