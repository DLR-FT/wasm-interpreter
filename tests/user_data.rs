use std::sync::mpsc::Sender;

use interop::StoreTypedInvocationExt;
use wasm::{config::Config, HaltExecutionError, Store, Value};

#[test_log::test]
fn counter() {
    #[derive(Debug, PartialEq)]
    struct MyCounter(pub u32);
    impl Config for MyCounter {}

    fn add_one(
        user_data: &mut MyCounter,
        _params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        user_data.0 += 1;

        Ok(Vec::new())
    }

    let mut store = Store::new(MyCounter(0));
    // SAFETY: The host function does not have any parameter or return types.
    // Therefore it cannot use invalid addresses.
    let add_one = unsafe { store.func_alloc_typed_unchecked::<(), ()>(add_one) };

    for _ in 0..5 {
        // SAFETY: Only one store exists in this test. Therefore, it is always
        // the correct store.
        unsafe { store.invoke_typed_without_fuel_unchecked::<(), ()>(add_one, ()) }.unwrap();
    }

    assert_eq!(store.user_data, MyCounter(5));
}

#[test_log::test]
fn channels() {
    struct MySender(pub Sender<String>);
    impl Config for MySender {}

    let (tx, rx) = std::sync::mpsc::channel::<String>();

    std::thread::spawn(|| {
        fn send_message(
            user_data: &mut MySender,
            _params: Vec<Value>,
        ) -> Result<Vec<Value>, HaltExecutionError> {
            user_data
                .0
                .send("Hello from host function!".to_owned())
                .unwrap();

            Ok(Vec::new())
        }

        let mut store = Store::new(MySender(tx));
        // SAFETY: The host function does not have any parameter or return
        // types. Therefore it cannot use invalid addresses.
        let send_message = unsafe { store.func_alloc_typed_unchecked::<(), ()>(send_message) };

        // SAFETY: Only one store exists in this test. Therefore, it is always
        // the correct store.
        unsafe { store.invoke_typed_without_fuel_unchecked::<(), ()>(send_message, ()) }.unwrap();
    });

    assert_eq!(rx.recv(), Ok("Hello from host function!".to_owned()));
}
