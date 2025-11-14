use wasm::{linker::Linker, resumable::RunState, validate, RuntimeError, Store, Value};

const SIMPLE_IMPORT_BASE: &str = r#"
(module
    (import "addon" "get_one" (func $get_one (param) (result i32)))
    (func (export "get_three") (param) (result i32)
        call $get_one
        i32.const 2
        i32.add
    )
)"#;

const SIMPLE_IMPORT_ADDON: &str = r#"
(module
    (func (export "get_one") (param) (result i32)
        i32.const 1
    )
)"#;

#[test_log::test]
pub fn compile_simple_import() {
    let wasm_bytes_addon = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info_addon = validate(&wasm_bytes_addon).expect("validation failed");

    let wasm_bytes_base = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info_base = validate(&wasm_bytes_base).expect("validation failed");

    let mut store = Store::new(());
    let mut linker = Linker::new();

    // First instantiate the addon module
    let addon = linker
        .module_instantiate(&mut store, &validation_info_addon, None)
        .unwrap()
        .module_addr;
    // We also want to define all of its exports, to makes them discoverable for
    // linking of the base module.
    linker
        .define_module_instance(&store, "addon".to_owned(), addon)
        .unwrap();

    // Now we link and instantiate the base module. We can also perform linking
    // and instantiating them separately instead of going through
    // `Linker::module_instantiate`.  This lets us inspect the linked extern
    // values in between.

    // 1. Perform linking
    let linked_base_imports = linker.instantiate_pre(&validation_info_base).unwrap();

    // 1.5 Freely inspect the linked extern values
    assert_eq!(
        &linked_base_imports,
        &[store.instance_export(addon, "get_one").unwrap()]
    );

    // 2. Perform the actual instantiation directly on the `Store`
    let base = store
        .module_instantiate(&validation_info_base, linked_base_imports, None)
        .unwrap()
        .module_addr;

    let get_three = store
        .instance_export(base, "get_three")
        .unwrap()
        .as_func()
        .unwrap();

    // Perform a call to see if everything works as expected
    let get_three_result = store
        .invoke(get_three, Vec::new(), None)
        .map(|rs| match rs {
            RunState::Finished { values, .. } => values,
            _ => unreachable!("fuel is disabled"),
        });
    assert_eq!(get_three_result.unwrap(), &[Value::I32(3)],);
}

#[test_log::test]
fn define_duplicate_extern_value() {
    const MODULE_WITH_EMPTY_FUNCTION: &str = r#"(module (func (export "foo") nop))"#;
    let wasm_bytes = wat::parse_str(MODULE_WITH_EMPTY_FUNCTION).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();

    let mut store = Store::new(());

    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let foo_function = store.instance_export(module, "foo").unwrap();

    {
        let mut linker = Linker::new();

        linker
            .define_module_instance(&store, "bar".to_owned(), module)
            .unwrap();
        assert_eq!(
            linker.define_module_instance(&store, "bar".to_owned(), module),
            Err(RuntimeError::DuplicateExternDefinition)
        );
    }
    {
        let mut linker = Linker::new();
        linker
            .define_module_instance(&store, "bar".to_owned(), module)
            .unwrap();
        assert_eq!(
            linker.define("bar".to_owned(), "foo".to_owned(), foo_function),
            Err(RuntimeError::DuplicateExternDefinition)
        );
    }
    {
        let mut linker = Linker::new();
        linker
            .define("bar".to_owned(), "foo".to_owned(), foo_function)
            .unwrap();
        assert_eq!(
            linker.define("bar".to_owned(), "foo".to_owned(), foo_function),
            Err(RuntimeError::DuplicateExternDefinition)
        );
    }
    {
        let mut linker = Linker::new();
        linker
            .define("bar".to_owned(), "foo".to_owned(), foo_function)
            .unwrap();
        assert_eq!(
            linker.define_module_instance(&store, "bar".to_owned(), module),
            Err(RuntimeError::DuplicateExternDefinition)
        );
    }
}
