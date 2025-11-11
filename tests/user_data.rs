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
    instance
        .add_host_function_typed::<(), ()>("host", "add_one", add_one)
        .unwrap();

    let add_one_func_ref = instance.get_function_by_name("host", "add_one").unwrap();

    for _ in 0..5 {
        instance
            .invoke_typed::<(), ()>(add_one_func_ref, ())
            .unwrap();
    }

    assert_eq!(*instance.user_data(), MyCounter(5));
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
        instance
            .add_host_function_typed::<(), ()>("host", "send_message", send_message)
            .unwrap();

        let send_message_func_ref = instance
            .get_function_by_name("host", "send_message")
            .unwrap();
        instance
            .invoke_typed::<(), ()>(send_message_func_ref, ())
            .unwrap();
    });

    assert_eq!(rx.recv(), Ok("Hello from host function!".to_owned()));
}
