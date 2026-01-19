use std::{cell::{RefCell, UnsafeCell}, mem};

use wasi_common::{WasiCtx, snapshots::preview_1::wasi_snapshot_preview1, sync::WasiCtxBuilder};
use wasm::{HaltExecutionError, Limits, MemType, Store, Value, addrs::MemAddr, checked::{Stored, StoredInstantiationOutcome}, config::Config, host_function_wrapper, linker::Linker, validate};
use wiggle::{GuestMemory, borrow::BorrowChecker, run_in_dummy_executor, wasmtime_crate::Val};

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


struct MyWasiCtx(WasiCtx,Option<Stored<MemAddr>>);
impl Config for MyWasiCtx {}

fn main() {
    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdout();
    let mut store = Store::new(MyWasiCtx(builder.build(), None));

    fn wrapped_fd_write(store: &mut Store<MyWasiCtx>, params: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
     host_function_wrapper(params, |(arg0, arg1, arg2, arg3): (i32,i32,i32,i32)| {
         let mem_addr = store.user_data.1.unwrap();
         let mut wasi_ctx = store.user_data.0.clone();
         let guest_memory = WasiGuestMemory{store, mem_addr, bc: BorrowChecker::new()};
            let result = wasi_common::snapshots::preview_1::wasi_snapshot_preview1::fd_write(
                    &mut wasi_ctx,
                    &guest_memory,
                    arg0, arg1, arg2, arg3
                );
         Ok(run_in_dummy_executor(result).unwrap().unwrap())
     })
    }

    let func_addr = store.func_alloc_typed_unchecked::<(i32, i32, i32, i32), i32>(wrapped_fd_write);

    let mut linker = Linker::new();
    linker.define_unchecked("wasi_snapshot_preview1".to_string(), "fd_write".to_string(), wasm::ExternVal::Func(func_addr)).unwrap();

    let wat = wat::parse_file("hello_world.wat").unwrap();
    let validation_info = validate(&wat).unwrap();
    let stored_instantiation_outcome = linker.module_instantiate(&mut store, &validation_info, None).unwrap();
    let mem_addr = store.instance_export(stored_instantiation_outcome.module_addr, "memory").unwrap().as_mem();
    store.user_data.1 = mem_addr;

    let start = store.instance_export(stored_instantiation_outcome.module_addr, "_start").unwrap();
    store.invoke(start.as_func().unwrap(), Vec::new(), None).unwrap();
}
