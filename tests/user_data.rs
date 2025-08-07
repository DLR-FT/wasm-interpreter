use std::sync::mpsc::Sender;

use wasm::{RuntimeInstance, Value};

#[test_log::test]
fn counter() {
    fn add_one(user_data: &mut u32, _params: Vec<Value>) -> Vec<Value> {
        *user_data += 1;

        Vec::new()
    }

    let mut instance = RuntimeInstance::new(0);
    instance
        .add_host_function_typed::<(), ()>("host", "add_one", add_one)
        .unwrap();

    let add_one_func_ref = instance.get_function_by_name("host", "add_one").unwrap();

    for _ in 0..5 {
        instance
            .invoke_typed::<(), ()>(&add_one_func_ref, ())
            .unwrap();
    }

    assert_eq!(*instance.user_data(), 5);
}

#[test_log::test]
fn channels() {
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    std::thread::spawn(|| {
        fn send_message(user_data: &mut Sender<String>, _params: Vec<Value>) -> Vec<Value> {
            user_data
                .send("Hello from host function!".to_owned())
                .unwrap();

            Vec::new()
        }

        let mut instance = RuntimeInstance::new(tx);
        instance
            .add_host_function_typed::<(), ()>("host", "send_message", send_message)
            .unwrap();

        let send_message_func_ref = instance
            .get_function_by_name("host", "send_message")
            .unwrap();
        instance
            .invoke_typed::<(), ()>(&send_message_func_ref, ())
            .unwrap();
    });

    assert_eq!(rx.recv(), Ok("Hello from host function!".to_owned()));
}
