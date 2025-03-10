use std::{thread, time::Duration};

use log::info;
use wasm::rw_spinlock::*;

#[test_log::test]
fn rw_spin_lock_basic() {
    let rw_lock = RwSpinLock::new(0i128);

    // one reader
    {
        let reader = rw_lock.read();
        assert_eq!(*reader, 0);
    }

    // one writer
    {
        let mut writer = rw_lock.write();
        *writer = 13;
    }

    // one writer used for reading
    {
        let writer = rw_lock.write();
        assert_eq!(*writer, 13);
    }

    // 10_000 readers
    {
        let mut vec = Vec::new();
        for _ in 0..10_000 {
            vec.push(rw_lock.read());
        }
    }

    // a final writer
    assert_eq!(*rw_lock.write(), 13);
}

#[test_log::test]
fn write_dominates_read() {
    let rw_lock = RwSpinLock::new(0u128);

    thread::scope(|s| {
        // this thread ensures that there first is a read lock
        s.spawn(|| {
            info!("t1: acquiring read lock");
            let read_guard = rw_lock.read();

            info!("t1: waiting");
            thread::sleep(Duration::from_millis(500));

            info!("t1: reading once");
            assert_eq!(*read_guard, 0u128);
            info!("t1: terminating");
        });

        for _ in 0..64 {
            s.spawn(|| {
                thread::sleep(Duration::from_millis(250));

                loop {
                    info!("tx: acquiring readlock");
                    let read_guard = rw_lock.read();
                    thread::sleep(Duration::from_millis(500));
                    if *read_guard != 0 {
                        return;
                    }
                }
            });
        }

        s.spawn(|| {
            thread::sleep(Duration::from_millis(750));

            info!("t2: acquiring write lock");
            let mut write_guard = rw_lock.write();

            *write_guard = 1u128 << 74;
        });
    });

    assert_eq!(*rw_lock.read(), 1u128 << 74);
}
