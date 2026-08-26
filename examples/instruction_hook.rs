//! A hook executed on every instruction
//!
//! An instruction hook is a function that is called prior to interpretation of every instruction.
//! It can be used to do various things, like measuring coverage, tracing, logging or gathering
//! statistics about executed code.
//!
//! In this example, we setup an instruction hook that counts how many times every instruction is
//! executed. In the end a full summary is printed as well.
//!
//! Note, that this type of instruction hook may come with a severe performance penalty.
use std::{collections::HashMap, error::Error, fmt};

use dlr_wasm_interpreter::{
    decode_and_validate, Config, FuncAddr, Module, ModuleAddr, Store, Value,
};

// This is the same Wasm code as in the fuel example
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

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_bytecode: Vec<u8> = wat::parse_str(WAT_CODE)?;
    let module: Module = decode_and_validate(&wasm_bytecode, &mut ())?;

    let mut store: Store<MyHookConfig> = Store::new(MyHookConfig::default());

    // SAFETY: There are no extern values.
    let module_addr: ModuleAddr =
        unsafe { store.module_instantiate(&module, vec![], None) }?.module_addr;

    // SAFETY: The module address was just returned from module instantiation in the same store.
    let fibonacci: FuncAddr = unsafe { store.instance_export(module_addr, "fibonacci") }?
        .as_func()
        .ok_or("fibonacci is not a function")?;

    // SAFETY: The function address was just returned from the same store. Also, no addresses are
    // passed as parameters.
    let return_values: Vec<Value> = unsafe { store.invoke_simple(fibonacci, vec![Value::I32(3)]) }?;

    let [Value::I32(nth_fibonacci)] = *return_values else {
        return Err("expected one i32 as a return value".into());
    };

    println!();
    println!("fib(3)={nth_fibonacci}");
    println!();

    // We can still access our user data
    println!("{}", store.user_data);

    Ok(())
}

#[derive(Default)]
struct MyHookConfig {
    count_per_instruction: HashMap<u8, usize>,
}

impl fmt::Display for MyHookConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Instruction: Execution Count")?;
        writeln!(f, "----------------------------")?;

        for (instruction, count) in &self.count_per_instruction {
            writeln!(f, "{instruction:#X}: {count}")?;
        }

        Ok(())
    }
}

impl Config for MyHookConfig {
    fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {
        let instruction_byte: u8 = bytecode[pc];

        println!("{:#X} @ {}", instruction_byte, pc,);

        self.count_per_instruction
            .entry(instruction_byte)
            .and_modify(|count: &mut usize| *count += 1)
            .or_insert(1);
    }
}
