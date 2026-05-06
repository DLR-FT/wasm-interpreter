use core::num::NonZeroU64;

use crate::{
    assert_validated::UnwrapValidatedExt,
    core::{
        indices::{ElemIdx, TableIdx},
        reader::types::opcode,
        utils::ToUsizeExt,
    },
    execution::interpreter_loop::{
        define_instruction, elem_drop, table_init, Args, InterpreterLoopOutcome,
    },
    value::Ref,
    TrapError, Value,
};

define_instruction!(
    table_get,
    opcode::TABLE_GET,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid table index
        // next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be valid in
        // the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the current
        // store. Therefore, it is valid in the current store.
        let tab = unsafe { store_inner.tables.get(table_addr) };

        let i: i32 = resumable.stack.pop_value().try_into().unwrap_validated();

        let val = tab
            .elem
            .get(i.cast_unsigned().into_usize())
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

        resumable.stack.push_value::<T>((*val).into())?;
        trace!(
            "Instruction: table.get '{}' [{}] -> [{}]",
            table_idx,
            i,
            val
        );
        Ok(None)
    }
);

define_instruction!(
    table_set,
    opcode::TABLE_SET,
    |Args {
         store_inner,
         modules,
         resumable,
         wasm,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be valid table index
        // next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };
        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be valid in
        // the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the current
        // store. Therefore, it is valid in the current store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let val: Ref = resumable.stack.pop_value().try_into().unwrap_validated();
        let i: i32 = resumable.stack.pop_value().try_into().unwrap_validated();

        tab.elem
            .get_mut(i.cast_unsigned().into_usize())
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)
            .map(|r| *r = val)?;
        trace!(
            "Instruction: table.set '{}' [{} {}] -> []",
            table_idx,
            i,
            val
        );
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    table_size,
    opcode::fc_extensions::TABLE_SIZE,
    |Args {
         wasm,
         resumable,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be valid table
        // index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let sz = tab.elem.len() as u32;

        resumable.stack.push_value::<T>(Value::I32(sz))?;

        trace!("Instruction: table.size '{}' [] -> [{}]", table_idx, sz);
        Ok(None)
    }
);

define_instruction!(
    no_fuel_check,
    table_grow,
    opcode::fc_extensions::TABLE_GROW,
    |Args {
         resumable,
         wasm,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let sz = tab.elem.len() as u32;

        let n: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_GROW)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_GROW);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(n))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let val: Ref = resumable.stack.pop_value().try_into().unwrap_validated();

        // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
        // for now we execute it always according to the following match expr.
        // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the resumable.stack per spec
        match tab.grow(n, val) {
            Ok(_) => {
                resumable.stack.push_value::<T>(Value::I32(sz))?;
            }
            Err(_) => {
                resumable.stack.push_value::<T>(Value::I32(u32::MAX))?;
            }
        }
        Ok(None)
    }
);

define_instruction!(
    no_fuel_check,
    table_fill,
    opcode::fc_extensions::TABLE_FILL,
    |Args {
         resumable,
         wasm,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr = *unsafe { module.table_addrs.get(table_idx) };
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab = unsafe { store_inner.tables.get_mut(table_addr) };

        let len: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_FILL)
            + u64::from(len)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_FILL);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(len))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let val: Ref = resumable.stack.pop_value().try_into().unwrap_validated();
        let dst: u32 = resumable.stack.pop_value().try_into().unwrap_validated();

        let end = (dst.into_usize())
            .checked_add(len.into_usize())
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

        tab.elem
            .get_mut(dst.into_usize()..end)
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?
            .fill(val);

        trace!(
            "Instruction table.fill '{}' [{} {} {}] -> []",
            table_idx,
            dst,
            val,
            len
        );
        Ok(None)
    }
);

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-copy-x-y
define_instruction!(
    no_fuel_check,
    table_copy,
    opcode::fc_extensions::TABLE_COPY,
    |Args {
         resumable,
         wasm,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_x_idx = unsafe { TableIdx::read_unchecked(wasm) };
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_y_idx = unsafe { TableIdx::read_unchecked(wasm) };

        // SAFETY: The current module address must come from the current
        // store, because it is the only parameter to this function that
        // can contain module addresses. All stores guarantee all
        // addresses in them to be valid within themselves.
        let module = unsafe { modules.get(*current_module) };

        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr_x = *unsafe { module.table_addrs.get(table_x_idx) };
        // SAFETY: Validation guarantees the table index to be
        // valid in the current module.
        let table_addr_y = *unsafe { module.table_addrs.get(table_y_idx) };

        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab_x_elem_len = unsafe { store_inner.tables.get(table_addr_x) }.elem.len();
        // SAFETY: This table address was just read from the
        // current store. Therefore, it is valid in the current
        // store.
        let tab_y_elem_len = unsafe { store_inner.tables.get(table_addr_y) }.elem.len();

        let n: u32 = resumable.stack.pop_value().try_into().unwrap_validated(); // size
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_COPY)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_COPY);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(n))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: u32 = resumable.stack.pop_value().try_into().unwrap_validated(); // source
        let d: u32 = resumable.stack.pop_value().try_into().unwrap_validated(); // destination

        let src_res = match s.checked_add(n) {
            Some(res) => {
                if res > tab_y_elem_len as u32 {
                    return Err(TrapError::TableOrElementAccessOutOfBounds.into());
                } else {
                    res.into_usize()
                }
            }
            _ => return Err(TrapError::TableOrElementAccessOutOfBounds.into()),
        };

        let dst_res = match d.checked_add(n) {
            Some(res) => {
                if res > tab_x_elem_len as u32 {
                    return Err(TrapError::TableOrElementAccessOutOfBounds.into());
                } else {
                    res.into_usize()
                }
            }
            _ => return Err(TrapError::TableOrElementAccessOutOfBounds.into()),
        };

        if table_addr_x == table_addr_y {
            // SAFETY: This table address was just read from the
            // current store. Therefore, it is valid in the
            // current store.
            let table = unsafe { store_inner.tables.get_mut(table_addr_x) };

            table.elem.copy_within(s as usize..src_res, d as usize);
        } else {
            let dst_addr = table_addr_x;
            let src_addr = table_addr_y;

            // SAFETY: These table addresses were just read from
            // the current store. Therefore, they are valid in
            // the current store.
            let (src_table, dst_table) =
                unsafe { store_inner.tables.get_two_mut(src_addr, dst_addr) }
                    .expect("both addrs to never be equal");

            dst_table.elem[d.into_usize()..dst_res]
                .copy_from_slice(&src_table.elem[s.into_usize()..src_res]);
        }

        trace!(
            "Instruction: table.copy '{}' '{}' [{} {} {}] -> []",
            table_x_idx,
            table_y_idx,
            d,
            s,
            n
        );
        Ok(None)
    }
);

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-init-x-y
// https://webassembly.github.io/spec/core/binary/instructions.html#table-instructions
// in binary format it seems that elemidx is first ???????
// this is ONLY for passive elements
define_instruction!(
    no_fuel_check,
    table_init_fn,
    opcode::fc_extensions::TABLE_INIT,
    |Args {
         resumable,
         wasm,
         store_inner,
         modules,
         current_module,
         ..
     }| {
        // SAFETY: Validation guarantees there to be a valid
        // element index next.
        let elem_idx = unsafe { ElemIdx::read_unchecked(wasm) };
        // SAFETY: Validation guarantees there to be a valid
        // table index next.
        let table_idx = unsafe { TableIdx::read_unchecked(wasm) };

        let n: u32 = resumable.stack.pop_value().try_into().unwrap_validated(); // size
        let cost = T::get_fc_extension_flat_cost(opcode::fc_extensions::TABLE_INIT)
            + u64::from(n)
                * T::get_fc_extension_cost_per_element(opcode::fc_extensions::TABLE_INIT);
        if let Some(fuel) = &mut resumable.maybe_fuel {
            if *fuel >= cost {
                *fuel -= cost;
            } else {
                resumable
                    .stack
                    .push_value::<T>(Value::I32(n))
                    .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
                return Ok(Some(InterpreterLoopOutcome::OutOfFuel {
                    required_fuel: NonZeroU64::new(cost - *fuel).expect(
                        "the last check guarantees that the current fuel is smaller than cost",
                    ),
                }));
            }
        }

        let s: i32 = resumable.stack.pop_value().try_into().unwrap_validated(); // offset
        let d: i32 = resumable.stack.pop_value().try_into().unwrap_validated(); // dst

        // SAFETY: All requirements are met:
        // 1. The current module address must come from the
        //    current store, because it is the only parameter to
        //    this function that can contain module addresses. All
        //    stores guarantee all addresses in them to be valid
        //    within themselves.
        // 2. Validation guarantees the table index to be valid
        //    in the current module instance.
        // 3./5. The table/element addresses are valid for a
        //       similar reason that the module address is valid:
        //       they are stored in the current module instance,
        //       which is also part of the current store.
        // 4. Validation guarantees the element index to be
        //    valid in the current module instance.
        unsafe {
            table_init(
                modules,
                &mut store_inner.tables,
                &store_inner.elements,
                *current_module,
                elem_idx,
                table_idx,
                n,
                s,
                d,
            )?
        };
        Ok(None)
    }
);

define_instruction!(
    fc_fuel_check,
    elem_drop_fn,
    opcode::fc_extensions::ELEM_DROP,
    |Args {
         wasm,
         modules,
         current_module,
         store_inner,
         ..
     }| {
        // SAFETY: Validation guarantees there a valid element
        // index next.
        let elem_idx = unsafe { ElemIdx::read_unchecked(wasm) };

        // SAFETY: All requirements are met:
        // 1. The current module address must come from the
        //    current store, because it is the only parameter to
        //    this function that can contain module addresses. All
        //    stores guarantee all addresses in them to be valid
        //    within themselves.
        // 2. Validation guarantees the element index to be
        //    valid in the current module instance.
        // 3. The element address is valid for a similar reason
        //    that the module address is valid: it is stored in the
        //    current module instance, which is also part of the
        //    current store.
        unsafe {
            elem_drop(
                modules,
                &mut store_inner.elements,
                *current_module,
                elem_idx,
            );
        }
        Ok(None)
    }
);
