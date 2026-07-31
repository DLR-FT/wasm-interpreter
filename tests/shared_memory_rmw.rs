use std::{iter, sync::Arc};

use dlr_wasm_interpreter::{Limits, MemType, Ordering, SharedLinearMemory};
use dlr_wasm_interpreter_checked::Store;

#[test]
fn multithreaded_counting() {
    let mut store = Store::new(());

    let mem_addr = store
        .mem_alloc(MemType {
            limits: Limits {
                min: 1,
                max: Some(10),
                shared: true,
            },
        })
        .unwrap();

    let memory = store
        .mem_get_as_shared(mem_addr)
        .expect("this memory is shared");

    let memory: Arc<SharedLinearMemory> = Arc::clone(memory);

    let spawn_counter_thread = || {
        let cloned_memory = Arc::clone(&memory);
        std::thread::spawn(|| {
            let memory = cloned_memory;

            for _ in 0..100 {
                unsafe { memory.rmw_data_u32_add(0, 1) };
            }
        })
    };

    let join_handles = iter::repeat_with(spawn_counter_thread)
        .take(8)
        .collect::<Vec<_>>();

    // Wait for thread completion, then check counter
    for join_handle in join_handles {
        join_handle.join().unwrap();
    }
    assert_eq!(memory.load::<_, u32>(0, Ordering::SeqCst).unwrap(), 8 * 100);
}
