//! Configuration of fuel consumption per instruction
//!
//! This example shows how to configure how much fuel is consumed by each instruction. This is done
//! by defining a new struct [`MyCustomFuelConfig`], implementing [`Config`] for it and then passing
//! it into [`Store::new`].

use std::error::Error;

use wasm::{Config, Store};

fn main() -> Result<(), Box<dyn Error>> {
    let _store: Store<MyCustomFuelConfig> = Store::new(MyCustomFuelConfig);

    // See `fuel.rs` for how to invoke a function with fuel enabled.

    Ok(())
}

pub struct MyCustomFuelConfig;

impl Config for MyCustomFuelConfig {
    /// Returns a flat amount of fuel consumed for any given instruction.
    ///
    /// We recommend explicitly inlining these functions. Because of the way we call it internally,
    /// it is very likely for it to be completely optimized away, when it is fairly simple.
    #[inline(always)]
    fn get_flat_cost(instr: u8) -> u64 {
        match instr {
            // Let's say we want additions to be very expensive...
            wasm::instructions::I32_ADD
            | wasm::instructions::I64_ADD
            | wasm::instructions::F32_ADD
            | wasm::instructions::F64_ADD => 20,

            // ...and subtractions to be very cheap...
            wasm::instructions::I32_SUB
            | wasm::instructions::I64_SUB
            | wasm::instructions::F32_SUB
            | wasm::instructions::F64_SUB => 1,

            // ... with everything else in the middle.
            _ => 10,
        }
    }

    #[inline(always)]
    fn get_fc_extension_flat_cost(_instr: u32) -> u64 {
        // Let's use 10 fuel for FC and FD extensions as well.
        10
    }

    #[inline(always)]
    fn get_fd_extension_flat_cost(_instr: u32) -> u64 {
        10
    }

    /// Some instructions operate on N elements. For example, memory.copy copies N bytes. For these
    /// it is possible to specify how much fuel is needed per element in addition to the flat amount.
    #[inline(always)]
    fn get_cost_per_element(_instr: u8) -> u64 {
        0
    }

    /// This works exactly like [`Self::get_cost_per_element`] but for FC extension instructions
    /// (including table.copy, memory.fill, table.init).
    #[inline(always)]
    fn get_fc_extension_cost_per_element(_instr: u32) -> u64 {
        0
    }
}
