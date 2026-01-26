use wasi_common::{sync::WasiCtxBuilder, WasiCtx};
use wasm::{
    addrs::MemAddr, checked::Stored, config::Config, host_function_wrapper, linker::Linker,
    validate, ExternVal, HaltExecutionError, Store, Value,
};
use wiggle::{run_in_dummy_executor, GuestMemory};

struct MyWasiCtx {
    wasi_context: WasiCtx,
    /// This is set to Some after the guest is instantiated
    guest_memory: Option<Stored<MemAddr>>,
}
impl Config for MyWasiCtx {}

fn main() {
    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdout();
    let mut store = Store::new(MyWasiCtx {
        wasi_context: builder.build(),
        guest_memory: None,
    });

    // TODO after rebasing on main, there is store.func_alloc_typed which returns a `Stored<FuncAddr>`
    let func_addr = store.func_alloc_typed_unchecked::<(i32, i32, i32, i32), i32>(wrapped_fd_write);

    let mut linker = Linker::new();
    // TODO with the previous change, use `define` here
    linker
        .define_unchecked(
            "wasi_snapshot_preview1".to_string(),
            "fd_write".to_string(),
            ExternVal::Func(func_addr),
        )
        .unwrap();

    let wat = wat::parse_file("hello_world.wat").unwrap();
    let validation_info = validate(&wat).unwrap();
    let module = linker
        .module_instantiate(&mut store, &validation_info, None)
        .unwrap()
        .module_addr;
    let mem_addr = store
        .instance_export(module, "memory")
        .unwrap()
        .as_mem()
        .unwrap();

    store.user_data.guest_memory = Some(mem_addr);

    let start = store.instance_export(module, "_start").unwrap();
    store
        .invoke(start.as_func().unwrap(), Vec::new(), None)
        .unwrap();
}

fn wrapped_fd_write(
    store: &mut Store<MyWasiCtx>,
    params: Vec<Value>,
) -> Result<Vec<Value>, HaltExecutionError> {
    host_function_wrapper(params, |(arg0, arg1, arg2, arg3): (i32, i32, i32, i32)| {
        let mem_addr = store
            .user_data
            .guest_memory
            .expect("that a guest module was instantiated and that its exported memory is set");

        let mut wasi_ctx = store.user_data.wasi_context.clone();
        store
            .mem_access_mut_slice(mem_addr, |slice| {
                let mut guest_memory = GuestMemory::Unshared(slice);
                let result = wasi_common::snapshots::preview_1::wasi_snapshot_preview1::fd_write(
                    &mut wasi_ctx,
                    &mut guest_memory,
                    arg0,
                    arg1,
                    arg2,
                    arg3,
                );
                Ok(run_in_dummy_executor(result).unwrap().unwrap())
            })
            .unwrap()
    })
}
