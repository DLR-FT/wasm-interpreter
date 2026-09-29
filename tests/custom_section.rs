use dlr_wasm_interpreter_checked::decode_and_validate;

const EMPTY_MODULE_WITH_CUSTOM_SECTION: &[u8] = b"\x00\x61\x73\x6d\x01\x00\x00\x00\
                                                    \x00\x22\x10a custom sectionhello how are you";

#[test_log::test]
fn custom_section() {
    let module = decode_and_validate(EMPTY_MODULE_WITH_CUSTOM_SECTION, &mut ()).unwrap();

    let (name, custom_section) = module.custom_sections().next().unwrap();

    assert_eq!("a custom section", name);
    assert_eq!(
        "hello how are you",
        core::str::from_utf8(custom_section).unwrap()
    );
}
