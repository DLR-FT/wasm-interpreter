//! Linking multiple Wasm modules together
//!
//! In this example we have two modules. A "utils" module that defines an "add" utility function and
//! a main module that imports this add utility function. The main module also imports a "subtract"
//! function from the host environment while exporting an "identity" function that returns a number
//! unchanged.
//!
//! This example uses the [`dlr_wasm_interpreter_linker`] utility crate for name-based resolution
//! through its [`Linker`]. Note that this utility crate is still work-in-progress and might not be
//! properly documented/tested.

use std::error::Error;

use dlr_wasm_interpreter::{decode_and_validate, FuncAddr, Module, ModuleAddr, Store, Value};
use dlr_wasm_interpreter_linker::Linker;

const MAIN_WAT_CODE: &str = r#"
(module
    (type $binary-op (func (param i32) (param i32) (result i32)))

    (import "utils" "add" (func $add (type $binary-op)))
    (import "utils" "sub" (func $sub (type $binary-op)))

    (func (export "identity") (param $n i32) (result i32)
        local.get $n
        i32.const 123
        call $add
        i32.const 123
        call $sub
    )
)
"#;

const UTILS_WAT_CODE: &str = r#"
(module
    (func (export "add") (param $a i32) (param $b i32) (result i32)
        local.get 0
        local.get 1
        i32.add
    )

    (func (export "sub") (param $a i32) (param $b i32) (result i32)
        local.get 0
        local.get 1
        i32.sub
    )
)
"#;

fn main() -> Result<(), Box<dyn Error>> {
    // First, decode and validate both modules.
    let main_wasm_bytecode: Vec<u8> = wat::parse_str(MAIN_WAT_CODE)?;
    let utils_wasm_bytecode: Vec<u8> = wat::parse_str(UTILS_WAT_CODE)?;
    let main_module: Module = decode_and_validate(&main_wasm_bytecode, &mut ())?;
    let utils_module: Module = decode_and_validate(&utils_wasm_bytecode, &mut ())?;

    // Create an empty store and empty linker. Generally, there is a many-to-one relation between
    // linkers and stores (i.e. one linker cannot be used with multiple stores). However, most of
    // the time there will only be a single linker per store.
    let mut store: Store<()> = Store::new(());
    let mut linker: Linker = Linker::new();

    // Instead of instantiating a module via `Store::module_instantiate`, we now use
    // `Linker::module_instantiate`. Internally the linker iterates through all imports, tries to
    // match them against the symbols already known to the linker and then forwards the call to
    // `Store::module_instantiate`.
    //
    // Note that the the utils module does not have any imports. Therefore, we could also call
    // module_instantiate on the store directly.
    //
    // SAFETY: There exists only a single store in this program.
    let utils_module_addr: ModuleAddr =
        unsafe { linker.module_instantiate(&mut store, &utils_module, None) }
            .ok_or("linking failed")??
            .module_addr;

    // `module_instantiate` does not automatically define the module's export as symbols. We have to
    // do this manually after instantiation like so:
    //
    // SAFETY: There exists only a single store in this program.
    unsafe { linker.define_module_instance(&store, "utils".to_owned(), utils_module_addr) }?;

    // `define_module_instance`` is simply syntactic sugar for iterating through all exports of a
    // module and defining them as symbols in the linker. We could also define individual symbols
    // manually. In fact, this must be done when using the linker with host functions. An example in
    // pseudocode:
    //
    // linker.define("utils", "add", store.instance_export(utils_module_addr, "add"))

    // Instantiate the main module. In contrast to the last module_instantiate call, this actually
    // performs linking for the "add" and "sub" imports.
    //
    // SAFETY: There exists only a single store in this program.
    let main_module_addr: ModuleAddr =
        unsafe { linker.module_instantiate(&mut store, &main_module, None) }
            .ok_or("linking failed")??
            .module_addr;

    // Here we can either use `Store::instance_export` to get the identity function's address...
    //
    // SAFETY: There exists only a single store in this program.
    let _identity: FuncAddr = unsafe { store.instance_export(main_module_addr, "identity") }?
        .as_func()
        .ok_or("identity is not a function")?;
    // ... or define the module instance's exports in the linker context and then use `Linker::get`:
    //
    // SAFETY: There exists only a single store in this program.
    unsafe { linker.define_module_instance(&store, "main".to_owned(), main_module_addr) }?;
    let identity: FuncAddr = linker
        .get("main".to_owned(), "identity".to_owned())
        .ok_or("identity symbol does not exist")?
        .as_func()
        .ok_or("identity symbol is not a function")?;

    // Now simply invoke the function and check that it returns the same value that was passed in.
    //
    // SAFETY: There exists only a single store in this program.
    let result_values: Vec<Value> = unsafe { store.invoke_simple(identity, vec![Value::I32(42)]) }?;
    let [Value::I32(result)] = *result_values else {
        panic!("expected a single i32 as a result");
    };
    assert_eq!(result, 42);

    Ok(())
}
