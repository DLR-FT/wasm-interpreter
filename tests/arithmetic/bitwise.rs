const BASE_WAT: &'static str = r#"
    (module
      (func (export "template") (param $x i32) (param $y i32) (result i32)
          local.get $x
          local.get $y
          i32.{{0}})
    )
"#;
/// A simple function to test the i32.and bitwise operation
#[test_log::test]
pub fn i32_bitwise_and() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "and");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(1, instance.invoke_func(0, (33, 11)).unwrap());
    assert_eq!(5, instance.invoke_func(0, (77, 23)).unwrap());
    assert_eq!(180244, instance.invoke_func(0, (192534, 1231412)).unwrap());
    assert_eq!(0, instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap());
}

/// A simple function to test the i32.or bitwise operation
#[test_log::test]
pub fn i32_bitwise_or() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "or");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(43, instance.invoke_func(0, (33, 11)).unwrap());
    assert_eq!(95, instance.invoke_func(0, (77, 23)).unwrap());
    assert_eq!(1243702, instance.invoke_func(0, (192534, 1231412)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap());
}

/// A simple function to test the i32.xor bitwise operation
#[test_log::test]
pub fn i32_bitwise_xor() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "xor");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(42, instance.invoke_func(0, (33, 11)).unwrap());
    assert_eq!(90, instance.invoke_func(0, (77, 23)).unwrap());
    assert_eq!(1063458, instance.invoke_func(0, (192534, 1231412)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap());
}

/// A simple function to test the i32.shl bitwise operation
#[test_log::test]
pub fn i32_bitwise_shl() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "shl");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(67584, instance.invoke_func(0, (33, 11)).unwrap());
    assert_eq!(645922816, instance.invoke_func(0, (77, 23)).unwrap());
    assert_eq!(
        23068672,
        instance.invoke_func(0, (192534, 1231412)).unwrap()
    );
    assert_eq!(0, instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap());
}

/// A simple function to test the i32.shr_s bitwise operation
#[test_log::test]
pub fn i32_bitwise_shr_s() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "shr_s");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(8881445, instance.invoke_func(0, (142_103_123, 4)).unwrap());
    assert_eq!(23879, instance.invoke_func(0, (391_248_921, 14)).unwrap());
    assert_eq!(
        601955006,
        instance.invoke_func(0, (1_203_910_012, 33)).unwrap()
    );
    assert_eq!(
        1056594615,
        instance.invoke_func(0, (2_113_189_231, 33)).unwrap()
    );
    assert_eq!(-1, instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap());

    // Basic positive number
    assert_eq!(4, instance.invoke_func(0, (8, 1)).unwrap());

    // Shifting by 0 (no shift)
    assert_eq!(-1, instance.invoke_func(0, (-1, 0)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 0)).unwrap());

    // Shifting negative numbers
    assert_eq!(-4, instance.invoke_func(0, (-8, 1)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (-1, 1)).unwrap());

    // Shifting by 31 (maximum shift for 32-bit int)
    assert_eq!(-1, instance.invoke_func(0, (-1, 31)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (i32::MIN, 31)).unwrap());
    assert_eq!(0, instance.invoke_func(0, (i32::MAX, 31)).unwrap());

    // Shifting by more than 31
    assert_eq!(-1, instance.invoke_func(0, (-1, 32)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 32)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (-1, 100)).unwrap());

    // Minimum and maximum 32-bit integers
    assert_eq!(
        i32::MIN / 2,
        instance.invoke_func(0, (i32::MIN, 1)).unwrap()
    );
    assert_eq!(
        i32::MAX / 2,
        instance.invoke_func(0, (i32::MAX, 1)).unwrap()
    );

    // Shifting out all bits except sign
    assert_eq!(-2, instance.invoke_func(0, (i32::MIN, 30)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (i32::MAX, 30)).unwrap());
}

/// A simple function to test the i32.shr_u bitwise operation
#[test_log::test]
pub fn i32_bitwise_shr_u() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "shr_u");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(8881445, instance.invoke_func(0, (142_103_123, 4)).unwrap());
    assert_eq!(23879, instance.invoke_func(0, (391_248_921, 14)).unwrap());
    assert_eq!(
        601955006,
        instance.invoke_func(0, (1_203_910_012, 33)).unwrap()
    );
    assert_eq!(
        1056594615,
        instance.invoke_func(0, (2_113_189_231, 33)).unwrap()
    );
    assert_eq!(1, instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap());

    // Basic positive number
    assert_eq!(4, instance.invoke_func(0, (8, 1)).unwrap());

    // Shifting by 0 (no shift)
    assert_eq!(-1, instance.invoke_func(0, (-1, 0)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 0)).unwrap());

    // Shifting negative numbers
    assert_eq!(i32::MAX - 3, instance.invoke_func(0, (-8, 1)).unwrap());
    assert_eq!(i32::MAX, instance.invoke_func(0, (-1, 1)).unwrap());

    // Shifting by 31 (maximum shift for 32-bit int)
    assert_eq!(1, instance.invoke_func(0, (-1, 31)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (i32::MIN, 31)).unwrap());
    assert_eq!(0, instance.invoke_func(0, (i32::MAX, 31)).unwrap());

    // Shifting by more than 31
    assert_eq!(-1, instance.invoke_func(0, (-1, 32)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 32)).unwrap());
    assert_eq!(268435455, instance.invoke_func(0, (-1, 100)).unwrap());

    // Minimum and maximum 32-bit integers
    assert_eq!(
        (i32::MIN / 2) * (-1),
        instance.invoke_func(0, (i32::MIN, 1)).unwrap()
    );
    assert_eq!(
        i32::MAX / 2,
        instance.invoke_func(0, (i32::MAX, 1)).unwrap()
    );

    // Shifting out all bits except sign
    assert_eq!(2, instance.invoke_func(0, (i32::MIN, 30)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (i32::MAX, 30)).unwrap());
}

/// A simple function to test the i32.rotl bitwise operation
#[test_log::test]
pub fn i32_bitwise_rotl() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "rotl");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -2021317328,
        instance.invoke_func(0, (142_103_123, 4)).unwrap()
    );
    assert_eq!(
        2131117524,
        instance.invoke_func(0, (391_248_921, 14)).unwrap()
    );
    assert_eq!(
        -1887147272,
        instance.invoke_func(0, (1_203_910_012, 33)).unwrap()
    );
    assert_eq!(
        -68588834,
        instance.invoke_func(0, (2_113_189_231, 33)).unwrap()
    );
    assert_eq!(
        1073741824,
        instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap()
    );

    // Basic positive number
    assert_eq!(16, instance.invoke_func(0, (8, 1)).unwrap());

    // Rotating by 0 (no shift)
    assert_eq!(-1, instance.invoke_func(0, (-1, 0)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 0)).unwrap());

    // Shifting negative numbers
    assert_eq!(-15, instance.invoke_func(0, (-8, 1)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (-1, 1)).unwrap());

    // Rotating by 31
    assert_eq!(-1, instance.invoke_func(0, (-1, 31)).unwrap());
    assert_eq!(
        i32::MAX / 2 + 1,
        instance.invoke_func(0, (i32::MIN, 31)).unwrap()
    );
    assert_eq!(
        i32::MIN / 2 - 1,
        instance.invoke_func(0, (i32::MAX, 31)).unwrap()
    );

    // Rotating by more than 31
    assert_eq!(-1, instance.invoke_func(0, (-1, 32)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 32)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (-1, 100)).unwrap());

    // Minimum and maximum 32-bit integers
    assert_eq!(1, instance.invoke_func(0, (i32::MIN, 1)).unwrap());
    assert_eq!(-2, instance.invoke_func(0, (i32::MAX, 1)).unwrap());

    // Shifting out all bits except sign
    assert_eq!(
        i32::MAX / 4 + 1,
        instance.invoke_func(0, (i32::MIN, 30)).unwrap()
    );
    assert_eq!(
        i32::MIN / 4 - 1,
        instance.invoke_func(0, (i32::MAX, 30)).unwrap()
    );
}

/// A simple function to test the i32.rotr bitwise operation
#[test_log::test]
pub fn i32_bitwise_rotr() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_WAT).replace("{{0}}", "rotr");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        814187813,
        instance.invoke_func(0, (142_103_123, 4)).unwrap()
    );
    assert_eq!(
        -261857977,
        instance.invoke_func(0, (391_248_921, 14)).unwrap()
    );
    assert_eq!(
        601955006,
        instance.invoke_func(0, (1_203_910_012, 33)).unwrap()
    );
    assert_eq!(
        -1090889033,
        instance.invoke_func(0, (2_113_189_231, 33)).unwrap()
    );
    assert_eq!(1, instance.invoke_func(0, (i32::MIN, i32::MAX)).unwrap());

    // Basic positive number
    assert_eq!(4, instance.invoke_func(0, (8, 1)).unwrap());

    // Rotating by 0 (no shift)
    assert_eq!(-1, instance.invoke_func(0, (-1, 0)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 0)).unwrap());

    // Shifting negative numbers
    assert_eq!(i32::MAX - 3, instance.invoke_func(0, (-8, 1)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (-1, 1)).unwrap());

    // Rotating by 31
    assert_eq!(-1, instance.invoke_func(0, (-1, 31)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (i32::MIN, 31)).unwrap());
    assert_eq!(-2, instance.invoke_func(0, (i32::MAX, 31)).unwrap());

    // Rotating by more than 31
    assert_eq!(-1, instance.invoke_func(0, (-1, 32)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (1, 32)).unwrap());
    assert_eq!(-1, instance.invoke_func(0, (-1, 100)).unwrap());

    // Minimum and maximum 32-bit integers
    assert_eq!(
        i32::MAX / 2 + 1,
        instance.invoke_func(0, (i32::MIN, 1)).unwrap()
    );
    assert_eq!(
        i32::MIN / 2 - 1,
        instance.invoke_func(0, (i32::MAX, 1)).unwrap()
    );

    // Shifting out all bits except sign
    assert_eq!(2, instance.invoke_func(0, (i32::MIN, 30)).unwrap());
    assert_eq!(-3, instance.invoke_func(0, (i32::MAX, 30)).unwrap());
}

const BASE_COUNT_WAT: &'static str = r#"
    (module
      (func (export "template") (param $x i32) (result i32)
          local.get $x
          i32.{{0}})
    )
"#;

/// A simple function to test the i32.clz bitwise operation
#[test_log::test]
pub fn i32_bitwise_clz() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_COUNT_WAT).replace("{{0}}", "clz");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(26, instance.invoke_func(0, 33).unwrap());
    assert_eq!(25, instance.invoke_func(0, 77).unwrap());
    assert_eq!(14, instance.invoke_func(0, 192534).unwrap());
    assert_eq!(0, instance.invoke_func(0, i32::MIN).unwrap());
    assert_eq!(1, instance.invoke_func(0, i32::MAX).unwrap());
    assert_eq!(32, instance.invoke_func(0, 0).unwrap());
}

/// A simple function to test the i32.ctz bitwise operation
#[test_log::test]
pub fn i32_bitwise_ctz() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_COUNT_WAT).replace("{{0}}", "ctz");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(0, instance.invoke_func(0, 33).unwrap());
    assert_eq!(0, instance.invoke_func(0, 77).unwrap());
    assert_eq!(1, instance.invoke_func(0, 192534).unwrap());
    assert_eq!(31, instance.invoke_func(0, i32::MIN).unwrap());
    assert_eq!(0, instance.invoke_func(0, i32::MAX).unwrap());
    assert_eq!(32, instance.invoke_func(0, 0).unwrap());
}

/// A simple function to test the i32.popcnt bitwise operation
#[test_log::test]
pub fn i32_bitwise_popcnt() {
    use wasm::{validate, RuntimeInstance};

    let wat = String::from(BASE_COUNT_WAT).replace("{{0}}", "popcnt");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(2, instance.invoke_func(0, 33).unwrap());
    assert_eq!(4, instance.invoke_func(0, 77).unwrap());
    assert_eq!(8, instance.invoke_func(0, 192534).unwrap());
    assert_eq!(1, instance.invoke_func(0, i32::MIN).unwrap());
    assert_eq!(31, instance.invoke_func(0, i32::MAX).unwrap());
    assert_eq!(0, instance.invoke_func(0, 0).unwrap());
}
