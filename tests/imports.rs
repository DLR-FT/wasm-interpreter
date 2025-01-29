use log::trace;
use wasm::{validate, RuntimeError, RuntimeInstance, DEFAULT_MODULE};

const UNMET_IMPORTS: &str = r#"
(module
    (import "env" "dummy1" (func (param i32)))
    (import "env" "dummy2" (func (param i32)))
    (func (export "get_three") (param) (result i32)
        i32.const 1
        i32.const 2
        i32.add
    )
)"#;

const SIMPLE_IMPORT_BASE: &str = r#"
(module
    (import "env" "get_one" (func $get_one (param) (result i32)))
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

const CYCLICAL_IMPORT: &str = r#"
(module
    (import "base" "get_three" (func $get_three (param) (result i32)))
    (export "get_three" (func $get_three))
)"#;

const CALL_INDIRECT_BASE: &str = r#"
(module
    (import "env" "get_one" (func $get_one (param) (result i32)))
    (func $get_three (param) (result i32)
        call $get_one
        i32.const 2
        i32.add
    )

    (table 2 funcref)
    (elem (i32.const 0) $get_one $get_three)
    (type $void_to_i32 (func (param) (result i32)))

    (func (export "run") (result i32 i32)
        i32.const 0
        call_indirect (type $void_to_i32)
        
        i32.const 1
        call_indirect (type $void_to_i32)
    )
)"#;

#[test_log::test]
pub fn unmet_imports() {
    let wasm_bytes = wat::parse_str(UNMET_IMPORTS).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let get_three = instance
        .get_function_by_name(DEFAULT_MODULE, "get_three")
        .unwrap();

    assert_eq!(
        RuntimeError::UnmetImport,
        instance
            .invoke::<(), i32>(&get_three, ())
            .expect_err("Expected invoke to fail due to unmet imports")
    );
}

#[test_log::test]
pub fn compile_simple_import() {
    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance =
        RuntimeInstance::new_named("base", &validation_info).expect("instantiation failed");

    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    instance
        .add_module("addon", &validation_info)
        .expect("Successful instantiation");

    // assert_eq!((), instance.invoke_named("print_three", ()).unwrap());
    // Function 0 should be the imported function
    // assert_eq!((), instance.invoke_func(1, ()).unwrap());
}

#[test_log::test]
pub fn run_simple_import() {
    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance =
        RuntimeInstance::new_named("base", &validation_info).expect("instantiation failed");

    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    instance
        .add_module("env", &validation_info)
        .expect("Successful instantiation");

    let get_three = instance.get_function_by_name("base", "get_three").unwrap();
    assert_eq!(3, instance.invoke(&get_three, ()).unwrap());

    // Function 0 should be the imported function
    let get_three = instance.get_function_by_index(0, 1).unwrap();
    assert_eq!(3, instance.invoke(&get_three, ()).unwrap());
}

#[test_log::test]
pub fn run_call_indirect() {
    let wasm_bytes = wat::parse_str(CALL_INDIRECT_BASE).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance =
        RuntimeInstance::new_named("base", &validation_info).expect("instantiation failed");

    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    instance
        .add_module("env", &validation_info)
        .expect("Successful instantiation");

    let run = instance.get_function_by_name("base", "run").unwrap();
    assert_eq!((1, 3), instance.invoke(&run, ()).unwrap());
}

#[test_log::test]
pub fn run_cyclical() {
    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance =
        RuntimeInstance::new_named("base", &validation_info).expect("instantiation failed");

    let wasm_bytes = wat::parse_str(CYCLICAL_IMPORT).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    instance
        .add_module("env", &validation_info)
        .expect("Successful instantiation");

    let run = instance.get_function_by_name("base", "get_three").unwrap();
    // Unmet import since we can't have cyclical imports
    // Currently, this passes since we don't allow chained imports.
    assert!(instance.invoke::<(), i32>(&run, ()).unwrap_err() == wasm::RuntimeError::UnmetImport);
}

const EXPORT_MEMORY: &str = r#"
(module
    ;; Initialize with 1 page, maximum of 2 pages.
    (memory (export "shared_memory") 1 2)
)
"#;

const USE_MEMORY: &str = r#"
(module
    (import "env" "shared_memory" (memory 1))
    
    (func (export "store_i32") (param $offset i32) (param $value i32)
        local.get $offset
        local.get $value
        i32.store
    )
  
    (func (export "load_i32") (param $offset i32) (result i32)
        local.get $offset
        i32.load
    )

    (func (export "memory_copy") 
        (param $source i32) (param $dest i32) (param $size i32)
        local.get $dest    ;; destination offset
        local.get $source  ;; source offset
        local.get $size    ;; size in bytes
        memory.copy
    )
)
"#;

#[test_log::test]
pub fn run_memory() {
    let wasm_bytes = wat::parse_str(USE_MEMORY).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance =
        RuntimeInstance::new_named("base", &validation_info).expect("instantiation failed");

    let wasm_bytes = wat::parse_str(EXPORT_MEMORY).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    instance
        .add_module("env", &validation_info)
        .expect("Successful instantiation");

    let store_i32 = instance.get_function_by_name("base", "store_i32").unwrap();
    let load_i32 = instance.get_function_by_name("base", "load_i32").unwrap();
    let memory_copy = instance
        .get_function_by_name("base", "memory_copy")
        .unwrap();

    // TODO: debug access for memory contents
    // trace!(
    //     "{:?}",
    //     &instance.modules[1].store.mems[0]
    //         .try_into_local()
    //         .unwrap()
    //         .data[0..16]
    // );

    let _: () = instance.invoke(&store_i32, (0, 123)).unwrap();
    let res: i32 = instance.invoke(&load_i32, 0).unwrap();

    // trace!(
    //     "{:?}",
    //     &instance.modules[1].store.mems[0]
    //         .try_into_local()
    //         .unwrap()
    //         .data[0..16]
    // );
    assert_eq!(res, 123);

    let _: () = instance.invoke(&memory_copy, (0, 4, 4)).unwrap();
    let res: i32 = instance.invoke(&load_i32, 0).unwrap();
    let res2: i32 = instance.invoke(&load_i32, 4).unwrap();

    // trace!(
    //     "{:?}",
    //     &instance.modules[1].store.mems[0]
    //         .try_into_local()
    //         .unwrap()
    //         .data[0..16]
    // );
    assert_eq!(res, 123);
    assert_eq!(res2, 123);
}
