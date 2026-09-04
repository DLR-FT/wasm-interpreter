//! Allocate host functions into the store and invoke them
//!
//! Unlike other runtimes, which adopt a callback like mechanism for host functions, where the
//! interpreter executes this callback on its own through function pointers or closures, our
//! interpreter halts execution, returning the control flow to the host, allowing for more
//! flexibility, especially in terms of:
//!
//! - alternative execution schemes (e.g. exiting the interpreter is much easier compared to doing it
//!   within a callback)
//!
//! - resource management (e.g. the interpreter holds minimal information of host functions - just
//!   hostcodes instead of heavy closure like objects)
//!
//! - static correctness (e.g. borrow checking issues are much simpler since the context of the host
//!   function is statically more concrete)
//!
//! In this example, we register two host functions to the store, `read_num` and `print_num`, which
//! read an i32 from stdin and print an i32 to stdout respectively. We then import these host
//! functions within our Wasm module where we do simple arithmetic (with ill-conditions ignored for
//! the simplicity of the example).

use std::{error::Error, io};

use dlr_wasm_interpreter::{
    decode_and_validate, ExternVal, FuncAddr, FuncType, HostCall, InstantiationOutcome, Module,
    ModuleAddr, NumType, ResultType, RunState, Store, ValType, Value,
};

/// A Wasm module that sums up two i32s read from stdin, printing the result on stdout
const WAT_CODE: &str = r#"
(module $wasm_src.wasm
    (import "host" "read_num" (func $read_num (param) (result i32)))
    (import "host" "print_num" (func $print_num (param i32) (result)))
    (func $add_two_nums (export "add_two_nums") (param) (result)
        call $read_num
        call $read_num
        i32.add ;; just add two numbers up
        call $print_num
    )
)
"#;

/// Our custom pick for hostcode value of `read_num`. Hostcodes are arbitrary usize values we pick
/// for host functions to identify them later. Any choice for `READ_NUM_HOSTCODE` is fine as long as
/// it does not clash with other hostcodes.
const READ_NUM_HOSTCODE: usize = 0;

/// A simple function that reads an i32 from stdin
fn host_read_num_inner() -> Result<i32, Box<dyn Error>> {
    let mut line = String::new();
    println!("Type an i32:");
    io::stdin().read_line(&mut line)?;

    Ok(line.trim().parse()?)
}

/// Our custom pick for hostcode value of `print_num`.
const PRINT_NUM_HOSTCODE: usize = 1;

/// A simple function that prints an i32
fn host_print_num_inner(num: i32) {
    println!("Printing number: {num}");
}

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_bytecode: Vec<u8> = wat::parse_str(WAT_CODE)?;
    let module: Module = decode_and_validate(&wasm_bytecode, &mut ())?;
    let mut store = Store::new(());

    // Wasm type of `read_num` is () -> (i32).
    // It is the developers responsibility to adhere to the output type signature specified in the
    // `returns` field.
    let host_read_num_type = FuncType {
        params: ResultType::default(),
        returns: ResultType {
            valtypes: Box::from([ValType::NumType(NumType::I32)]),
        },
    };

    // Wasm type of `print_num` is (i32) -> ().
    // It is the developers responsibility to adhere to the output type signature specified in the
    // `returns` field.
    let host_print_num_type = FuncType {
        params: ResultType {
            valtypes: Box::from([ValType::NumType(NumType::I32)]),
        },
        returns: ResultType::default(),
    };

    // Allocate host functions in the store with their associated hostcode.
    let host_read_num_addr: FuncAddr = store.func_alloc(host_read_num_type, READ_NUM_HOSTCODE);
    let host_print_num_addr: FuncAddr = store.func_alloc(host_print_num_type, PRINT_NUM_HOSTCODE);

    // Instantiate the module with the externvals it requires as imports. In this case, these
    // externals are the host functions we have just allocated. The order of the externvals matter.
    // Since `read_num` was declared first as an import, it precedes `print_num`.

    // SAFETY: The externvals come from this store, as witnessed by The `Store::func_alloc` calls
    // above.
    let instantiation_outcome: InstantiationOutcome = unsafe {
        store.module_instantiate(
            &module,
            vec![
                ExternVal::Func(host_read_num_addr),
                ExternVal::Func(host_print_num_addr),
            ],
            None,
        )
    }?;

    let module_addr: ModuleAddr = instantiation_outcome.module_addr;

    // Retrieve the address of the exported native Wasm function, `add_two_nums`.

    // SAFETY: `module_addr`` originates from this store, as witnessed by
    // `Store::module_instantiate` call above.
    let add_two_nums_addr: FuncAddr =
        unsafe { store.instance_export(module_addr, "add_two_nums") }?
            .as_func()
            .ok_or("add_two_nums is not a function")?;

    // Invoke `add_two_nums`.
    // SAFETY: `add_two_nums_addr` originates from this store, as witnessed by
    // `Store::instance_export` call above.
    let mut run_state: RunState = unsafe { store.invoke(add_two_nums_addr, Vec::new(), None) }?;

    // Handling host functions are very similar to handling resumables that are out of fuel (see
    // `examples/fuel.rs`). The only difference here is the `run_state` returned by the invocation
    // possibly contains a hostcode that was associated to the host function that was just invoked.
    //
    // Specifically, there are 3 possibilities, corresponding to the `RunState` enum variants:
    loop {
        match run_state {
            // 3. RunState::HostCalled{host_call: HostCall { params, hostcode }, resumable}:
            // represents an ongoing function execution made with `Store::invoke`, that is about to
            // execute a host function. The information which host function should be executed is
            // relayed to the developer through `hostcode` field, which holds the hostcode value of
            // the particular host function that should be executed. The interpreter also exposes
            // the input values of the host function in the params field, ensuring the input type
            // signature is adhered to. After the execution of the host function is handled, the
            // return values of the host functions are to be supplied through
            // `Store::finish_host_call` with the associated `resumable`. Here, unlike input values,
            // the type correctness of the output is the responsibility of the developer.
            RunState::HostCalled {
                host_call: HostCall { params, hostcode },
                resumable,
            } => {
                run_state = match hostcode {
                    // The code block for this match case can be seen as the implementation of
                    // `read_num`.
                    READ_NUM_HOSTCODE => {
                        // function prologue: retrieve arguments.
                        // In this case there aren't any.

                        // function body
                        let num: i32 = host_read_num_inner()?;

                        // function epilogue: return arguments. In this case, we must ensure that a
                        // value of Wasm type i32 are returned.

                        // SAFETY: the `run_state` variable, which contains `resumable`, is only
                        // assigned to the values produced by this `store`. Therefore the store is
                        // always invoked with a resumable it owns. Additionally, the host function
                        // adheres to its result type signature in Wasm, which is i32.
                        unsafe { store.finish_host_call(resumable, vec![Value::I32(num as u32)]) }?
                    }

                    // The code block for this match case can be seen as the implementation of
                    // `host_print_num`.
                    PRINT_NUM_HOSTCODE => {
                        // function prologue: retrieve arguments.
                        // In this case it is a single Wasm type i32.
                        let top_wasm: Value =
                            *params.first().ok_or("no value at the top of the stack")?;

                        //function body
                        let Ok(top): Result<i32, _> = top_wasm.try_into() else {
                            return Err("Top value should have been a wasm i32".into());
                        };
                        host_print_num_inner(top);

                        // function epilogue: return arguments. In this case, we must ensure that no
                        // value is returned.

                        // SAFETY: the `run_state` variable, which contains `resumable`, is only
                        // assigned to the values produced by this `store`. Therefore the store is
                        // always invoked with a resumable it owns. Additionally, the host function
                        // adheres to its result type signature in wasm, which is ().
                        unsafe { store.finish_host_call(resumable, Vec::new()) }?
                    }
                    _ => unreachable!("there are no other host functions"),
                };
            }

            // 2. RunState::Resumable{resumable, required_fuel}: represents an ongoing function
            // execution made with `Store::invoke`, ready to continue natively with
            // `Store::resume_wasm`. This enum variant might appear when we run out of fuel or we
            // have just executed code on the host side and ready to continue natively in the Wasm
            // bytecode.
            RunState::Resumable {
                resumable,
                required_fuel: _,
            } => {
                // SAFETY: the `run_state` variable, which contains `resumable`, is only assigned to
                // the values produced by this `store`. Therefore the store is always invoked with a
                // resumable it owns.
                run_state = unsafe { store.resume_wasm(resumable) }?;
            }

            // 3. RunState::Finished{values, maybe_remaining_fuel}: represents a completed execution
            // of a function invocation made with `Store::invoke`, comprising computed return values
            // of the function and the remaining fuel if fuel metering is enabled.
            RunState::Finished { .. } => break,
        }
    }
    Ok(())
}
