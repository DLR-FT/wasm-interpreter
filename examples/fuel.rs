//! Function invocation with fuel and progress reporting
//!
//! In this example we use a Wasm module exporting one memory and a function "fibonacci", which
//! calculates the N-th fibonacci number. For every calculation step it also writes the remaining
//! number of iterations into the first byte in memory.
//!
//! After the initial setup, this function is called with [`FUEL_PER_CYCLE`] fuel. When the fuel is
//! used up and execution returns, we read the remaining number of iterations from the memory and
//! print it to stdout. Then we refill the [`WasmResumable`](wasm::WasmResumable) object with
//! [`FUEL_PER_CYCLE`] fuel and continue execution. This is repeated until the [`N`]-th fibonacci
//! number is calculated and returned.
//!
//! Additionally, a custom configuration struct [`SlowExecutionConfig`] is defined and passed into
//! the store upon its creation. This defines an instruction hook, which sleeps a set duration
//! [`SLEEP_DURATION_PER_INSTRUCTION`] before every instruction.

use std::{error::Error, time::Duration};

use dlr_wasm_interpreter::{
    decode_and_validate, Config, FuncAddr, InstantiationOutcome, MemAddr, Module, RunState, Store,
    Value,
};

const WAT_CODE: &str = r#"
(module
    (memory (export "memory") 1)
    (func $fibonacci (export "fibonacci") (param $n i32) (result i32) (local $tmp i32) (local $tmp2 i32)
        i32.const 1
        i32.const 0

        (loop (param i32) (param i32) (result i32)
            (block (param i32) (param i32) (result i32)
                ;; store the current $n at the start of the memory
                i32.const 0
                local.get $n
                i32.store8

                local.get $n
                local.tee $tmp
                i32.eqz
                br_if 0
                local.get $tmp
                i32.const 1
                i32.sub
                local.set $n

                local.set $tmp
                local.tee $tmp2
                local.get $tmp
                i32.add
                local.get $tmp2

                br 1
            )
        )
    )
)
"#;

/// We compute the N-th fibonacci number
pub const N: u8 = 10;
/// The amount of fuel we reload the resumable with every time fuel runs out.
pub const FUEL_PER_CYCLE: u64 = 5;
/// This is just used to sleep after every instruction so that execution is not instantaneous. It
/// does not have an effect on the compuatation itself.
pub const SLEEP_DURATION_PER_INSTRUCTION: Duration = Duration::from_millis(30);

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_bytecode: Vec<u8> = wat::parse_str(WAT_CODE)?;
    let module: Module = decode_and_validate(&wasm_bytecode, &mut ())?;

    let mut store: Store<SlowExecutionConfig> = Store::new(SlowExecutionConfig);

    // SAFETY: There are no extern values.
    let instantiation_outcome: InstantiationOutcome =
        unsafe { store.module_instantiate(&module, vec![], None) }?;

    let InstantiationOutcome {
        module_addr,
        maybe_remaining_fuel: _,
    } = instantiation_outcome;

    // SAFETY: The module address was just returned from module instantiation in the same store.
    let fibonacci: FuncAddr = unsafe { store.instance_export(module_addr, "fibonacci") }?
        .as_func()
        .ok_or("fibonacci is not a function")?;

    // SAFETY: The module address was just returned from module instantiation in the same store.
    let memory: MemAddr = unsafe { store.instance_export(module_addr, "memory") }?
        .as_mem()
        .ok_or("memory is not a memory")?;

    let parameters: Vec<Value> = vec![Value::I32(u32::from(N))];

    // After the initial setup, we can now invoke the fibonacci function. To be able to use fuel we
    // have to use `Store::invoke` which supports fuel and host functions (in contrast to the
    // simpler variant `Store::invoke_simple`).
    //
    // Its parameters are the same, except that we can now pass fuel into it. Also it returns a
    // `RunState` which depends on why execution stopped. In our case, execution can either finish
    // (`RunState::Finished`) or be stopped when fuel is empty (`RunState::Resumable`).
    //
    // SAFETY: The function address was just returned from the same store. Also, no addresses are
    // passed as parameters.
    let mut run_state = unsafe { store.invoke(fibonacci, parameters, Some(FUEL_PER_CYCLE)) }?;

    let return_values: Vec<Value> = loop {
        match run_state {
            RunState::Finished {
                values,
                maybe_remaining_fuel,
            } => {
                println!(
                    "Execution finished with {} fuel remaining",
                    maybe_remaining_fuel.expect("fuel to be enabled")
                );
                break values;
            }
            RunState::Resumable {
                mut resumable,
                required_fuel,
            } => {
                // Read the progress from the memory at index 0 as a single byte and print it
                //
                // SAFETY: The memory address was just returned from the same store.
                let current_n_from_memory = unsafe { store.mem_read(memory, 0) }?;
                println!(
                    "Fuel ran out. At least {required_fuel:?} is required for next instruction. Current n stored in memory is {current_n_from_memory}.",
                );

                // Refill fuel directly through the resumable object
                let fuel = resumable.fuel_mut().as_mut().expect("fuel is enabled");
                *fuel += FUEL_PER_CYCLE;

                // Continue execution, save its resulting run state and restart the loop.
                //
                // SAFETY: The resumable was just returned from the same store.
                run_state = unsafe { store.resume_wasm(resumable) }?;
            }
            RunState::HostCalled { .. } => unreachable!("no host functions exist"),
        }
    };

    let [Value::I32(nth_fibonacci)] = *return_values else {
        return Err("expected one i32 as a return value".into());
    };

    println!("fib({N})={nth_fibonacci}");

    Ok(())
}

struct SlowExecutionConfig;

impl Config for SlowExecutionConfig {
    fn instruction_hook(&mut self, _bytecode: &[u8], _pc: usize) {
        std::thread::sleep(SLEEP_DURATION_PER_INSTRUCTION);
    }
}
