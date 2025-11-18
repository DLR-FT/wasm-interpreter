use std::sync::mpsc::Sender;

use wasm::{config::Config, HaltExecutionError, RuntimeInstance, Value};

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

    let mut instance = RuntimeInstance::new(MyCounter(0));
    let add_one = instance.store.func_alloc_typed::<(), ()>(add_one);

    for _ in 0..5 {
        instance.invoke_typed::<(), ()>(add_one, ()).unwrap();
    }

    assert_eq!(instance.store.user_data, MyCounter(5));
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

        let mut instance = RuntimeInstance::new(MySender(tx));
        let send_message = instance.store.func_alloc_typed::<(), ()>(send_message);

        instance.invoke_typed::<(), ()>(send_message, ()).unwrap();
    });

    assert_eq!(rx.recv(), Ok("Hello from host function!".to_owned()));
}
