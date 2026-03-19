use std::sync::mpsc::Sender;

use checked::Store;
use registry::Registry;

#[test_log::test]
fn counter() {
    #[derive(Debug, PartialEq)]
    struct MyCounter(pub u32);

    fn add_one(user_data: &mut MyCounter, _params: ()) {
        user_data.0 += 1;
    }

    let mut my_counter = MyCounter(0);
    let mut store = Store::new(());
    let mut registry = Registry::default();
    let add_one = registry.alloc_host_function_typed(&mut store, add_one);

    for _ in 0..5 {
        registry
            .invoke_without_fuel_typed::<_, (), ()>(&mut my_counter, &mut store, add_one, ())
            .unwrap();
    }

    assert_eq!(my_counter, MyCounter(5));
}

#[test_log::test]
fn channels() {
    struct MySender(pub Sender<String>);

    let (tx, rx) = std::sync::mpsc::channel::<String>();

    std::thread::spawn(|| {
        fn send_message(user_data: &mut MySender, _params: ()) {
            user_data
                .0
                .send("Hello from host function!".to_owned())
                .unwrap();
        }

        let mut my_sender = MySender(tx);
        let mut store = Store::new(());
        let mut registry = Registry::default();
        let send_message = registry.alloc_host_function_typed(&mut store, send_message);

        registry
            .invoke_without_fuel_typed::<_, (), ()>(&mut my_sender, &mut store, send_message, ())
            .unwrap();
    });

    assert_eq!(rx.recv(), Ok("Hello from host function!".to_owned()));
}
