use wasm::{validate, ExternType, FuncType, GlobalType, NumType, ResultType, ValType};

#[test_log::test]
fn empty_module() {
    const EMPTY_MODULE: &str = r#"
        (module)
    "#;
    let wasm_bytes = wat::parse_str(EMPTY_MODULE).unwrap();

    let validation_info = validate(&wasm_bytes).unwrap();

    assert_eq!(validation_info.imports().len(), 0);
    assert_eq!(validation_info.exports().len(), 0);
}

#[test_log::test]
fn imports() {
    const MODULE_WITH_IMPORTS: &str = r#"
        (module
            (import "foo" "baz" (func))
            (import "bar" "bat" (global (mut i64)))
        )
    "#;

    let wasm_bytes = wat::parse_str(MODULE_WITH_IMPORTS).unwrap();

    let validation_info = validate(&wasm_bytes).unwrap();

    let imports: Vec<(&str, &str, ExternType)> = validation_info.imports().collect();

    assert_eq!(
        &imports,
        &[
            (
                "foo",
                "baz",
                ExternType::Func(FuncType {
                    params: ResultType {
                        valtypes: Vec::new()
                    },
                    returns: ResultType {
                        valtypes: Vec::new()
                    },
                })
            ),
            (
                "bar",
                "bat",
                ExternType::Global(GlobalType {
                    ty: ValType::NumType(NumType::I64),
                    is_mut: true,
                }),
            )
        ]
    );

    assert_eq!(validation_info.exports().len(), 0);
}

#[test_log::test]
fn exports() {
    const MODULE_WITH_EXPORTED_DEFINITIONS: &str = r#"
        (module
            (func $my_func (export "foo") (param i32) (result i64)
                local.get 0
                i64.extend_i32_u
            )
            (global $my_global (export "bar") (mut i32)
                i32.const 123
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(MODULE_WITH_EXPORTED_DEFINITIONS).unwrap();

    let validation_info = validate(&wasm_bytes).unwrap();

    let exports: Vec<(&str, ExternType)> = validation_info.exports().collect();

    assert_eq!(
        &exports,
        &[
            (
                "foo",
                ExternType::Func(FuncType {
                    params: ResultType {
                        valtypes: vec![ValType::NumType(NumType::I32)]
                    },
                    returns: ResultType {
                        valtypes: vec![ValType::NumType(NumType::I64)],
                    }
                }),
            ),
            (
                "bar",
                ExternType::Global(GlobalType {
                    ty: ValType::NumType(NumType::I32),
                    is_mut: true
                })
            )
        ]
    )
}
