use std::cell::UnsafeCell;

use wasi_common::sync::WasiCtxBuilder;
use wasm::{addrs::MemAddr, checked::Stored, config::Config, Limits, MemType, Store};
use wiggle::{borrow::BorrowChecker, run_in_dummy_executor, GuestMemory};

pub struct WasiGuestMemory<'a, 'b, T: Config> {
    pub store: &'a mut Store<'b, T>,
    pub mem_addr: Stored<MemAddr>,
    pub bc: BorrowChecker,
}

unsafe impl<'a, 'b, T: Config + Sync + Send> GuestMemory for WasiGuestMemory<'a, 'b, T> {
    fn base(&self) -> &[UnsafeCell<u8>] {
        self.store
            .mem_access_mut_slice(self.mem_addr, |slice| unsafe {
                std::slice::from_raw_parts(slice.as_ptr() as *const UnsafeCell<u8>, slice.len())
            })
            .unwrap()
    }

    fn mut_borrow(&self, r: wiggle::Region) -> Result<wiggle::BorrowHandle, wiggle::GuestError> {
        self.bc.mut_borrow(r)
    }

    fn shared_borrow(&self, r: wiggle::Region) -> Result<wiggle::BorrowHandle, wiggle::GuestError> {
        self.bc.shared_borrow(r)
    }

    fn mut_unborrow(&self, h: wiggle::BorrowHandle) {
        self.bc.mut_unborrow(h);
    }

    fn shared_unborrow(&self, h: wiggle::BorrowHandle) {
        self.bc.shared_unborrow(h);
    }

    fn can_read(&self, r: wiggle::Region) -> bool {
        self.bc.can_read(r)
    }

    fn can_write(&self, r: wiggle::Region) -> bool {
        self.bc.can_write(r)
    }
}

fn main() {
    let mut store = Store::new(());
    let guest_mem_addr = store.mem_alloc(MemType {
        limits: Limits {
            min: 1024,
            max: None,
        },
    });
    let hello_world = "Hello    World!\n".as_bytes();
    store
        .mem_access_mut_slice(guest_mem_addr, |slice| {
            slice[..hello_world.len()].copy_from_slice(hello_world);
            slice[hello_world.len()..hello_world.len() + 4].copy_from_slice(&i32::to_le_bytes(0));
            slice[hello_world.len() + 4..hello_world.len() + 8]
                .copy_from_slice(&i32::to_le_bytes(hello_world.len() as i32));
        })
        .unwrap();
    let guest_mem = WasiGuestMemory {
        store: &mut store,
        mem_addr: guest_mem_addr,
        bc: BorrowChecker::new(),
    };
    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdout();
    let mut wasi_ctx = builder.build();
    let result = wasi_common::snapshots::preview_1::wasi_snapshot_preview1::fd_write(
        &mut wasi_ctx,
        &guest_mem,
        1,
        hello_world.len() as i32,
        1,
        (hello_world.len() + 4) as i32,
    );
    println!("{v}", v = run_in_dummy_executor(result).unwrap().unwrap());
}
