use checked::Store;
use wasm::{decode_and_validate, RuntimeError};

#[test_log::test]
fn use_incorrect_number_of_extern_vals() {
    const MODULE_WITH_IMPORTS: &str = r#"
        (module
            (import "host" "foo" (func $foo))
            (import "host" "bar" (global $bar (mut i32)))
        )
    "#;

    let wasm_bytes = wat::parse_str(MODULE_WITH_IMPORTS).unwrap();
    let module = decode_and_validate(&wasm_bytes, &mut ()).unwrap();

    let mut store = Store::new(());

    assert_eq!(
        store.module_instantiate(&module, Vec::new(), None).err(),
        Some(RuntimeError::ExternValsLenMismatch)
    );
}
