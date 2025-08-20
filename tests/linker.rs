use wasm::{validate, ExternVal, Store};

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
    let mut store = Store::new(());
    // let mut linker: Linker = Default::default();

    let wasm_bytes_addon = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info_addon = validate(&wasm_bytes_addon).expect("validation failed");

    let res = store.add_module("addon", &validation_info_addon);
    if res.is_err() {
        // println!("{:#?}", res.unwrap_err());
        panic!("{}", res.unwrap_err());
    }

    let wasm_bytes_base = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info_base = validate(&wasm_bytes_base).expect("validation failed");

    store.add_module("base", &validation_info_base).unwrap();
    // let mut instance_base = linker
    //     .instantiate(&mut store, &validation_info_base)
    //     .unwrap();
    // let mut instance =
    //     RuntimeInstance::new_named((), "base", &validation_info_base).expect("instantiation failed");

    let &ExternVal::Func(func_addr) = store
        .registry
        .lookup("base".into(), "get_three".into())
        .unwrap()
    else {
        panic!("this entity is not a function")
    };

    println!("{:#?}", store.invoke(func_addr, Vec::new()).unwrap());

    // let mut instance_addon = linker
    //     .instantiate(&mut store, &validation_info_addon)
    //     .unwrap();

    // instance
    //     .add_module("addon", &validation_info)
    // .expect("Successful instantiation");
}
