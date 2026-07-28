use core::{num::NonZeroU64, ops::ControlFlow};

use crate::{
    core::{
        structure::{
            instructions,
            modules::indices::{ElemIdx, TableIdx},
        },
        utils::ToUsizeExt,
    },
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction, elem_drop, table_init, InterpreterLoopOutcome, State},
    },
    trace, Config, Ref, RuntimeError, TrapError, Value,
};

define_instruction!(
    super::table_get,
    table_get_mod,
    fuel_check = flat(TABLE_GET)
);
#[inline(always)]
pub unsafe fn table_get(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid table index
    // next.
    let table_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the table index to be valid in
    // the current module.
    let table_addr = *unsafe { module.table_addrs.get(table_idx) };
    // SAFETY: This table address was just read from the current
    // store. Therefore, it is valid in the current store.
    let tab = unsafe { state.store_inner.tables.get(table_addr) };

    let i: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let val = tab
        .elem
        .get(i.cast_unsigned().into_usize())
        .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

    state.resumable.stack.push_value((*val).into())?;
    trace!(
        "Instruction: table.get '{}' [{}] -> [{}]",
        table_idx,
        i,
        val
    );
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::table_set,
    table_set_mod,
    fuel_check = flat(TABLE_SET)
);
#[inline(always)]
pub unsafe fn table_set(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be valid table index
    // next.
    let table_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };
    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the table index to be valid in
    // the current module.
    let table_addr = *unsafe { module.table_addrs.get(table_idx) };
    // SAFETY: This table address was just read from the current
    // store. Therefore, it is valid in the current store.
    let tab = unsafe { state.store_inner.tables.get_mut(table_addr) };

    let val: Ref = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let i: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

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
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::table_size,
    table_size_mod,
    fuel_check = flat_fc(TABLE_SIZE)
);
#[inline(always)]
pub unsafe fn table_size(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be valid table
    // index next.
    let table_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the table index to be
    // valid in the current module.
    let table_addr = *unsafe { module.table_addrs.get(table_idx) };
    // SAFETY: This table address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let tab = unsafe { state.store_inner.tables.get_mut(table_addr) };

    let sz = tab.elem.len() as u32;

    state.resumable.stack.push_value(Value::I32(sz))?;

    trace!("Instruction: table.size '{}' [] -> [{}]", table_idx, sz);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::table_grow::<T>, table_grow_mod, fuel_check = omit);
#[inline(always)]
pub unsafe fn table_grow<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid
    // table index next.
    let table_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the table index to be
    // valid in the current module.
    let table_addr = *unsafe { module.table_addrs.get(table_idx) };
    // SAFETY: This table address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let tab = unsafe { state.store_inner.tables.get_mut(table_addr) };

    let sz = tab.elem.len() as u32;

    let n: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::TABLE_GROW)
        + u64::from(n)
            * T::get_fc_extension_cost_per_element(instructions::fc_extensions::TABLE_GROW);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(n))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    let val: Ref = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    // TODO this instruction is non-deterministic w.r.t. spec, and can fail if the embedder wills it.
    // for now we execute it always according to the following match expr.
    // if the grow operation fails, err := Value::I32(2^32-1) is pushed to the state.resumable.stack per spec
    let pushed_value = match tab.grow(n, val) {
        Ok(_) => sz,
        Err(RuntimeError::TableGrowOverflowed | RuntimeError::TableGrowExceededLimit) => u32::MAX,
        Err(_) => unreachable!("table grow operation cannot produce any other errors"),
    };
    state.resumable.stack.push_value(Value::I32(pushed_value))?;

    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::table_fill::<T>, table_fill_mod, fuel_check = omit);
#[inline(always)]
pub unsafe fn table_fill<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid
    // table index next.
    let table_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the table index to be
    // valid in the current module.
    let table_addr = *unsafe { module.table_addrs.get(table_idx) };
    // SAFETY: This table address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let tab = unsafe { state.store_inner.tables.get_mut(table_addr) };

    let len: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::TABLE_FILL)
        + u64::from(len)
            * T::get_fc_extension_cost_per_element(instructions::fc_extensions::TABLE_FILL);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(len))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    let val: Ref = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let dst: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

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
    Ok(ControlFlow::Continue(()))
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-copy-x-y
define_instruction!(super::table_copy::<T>, table_copy_mod, fuel_check = omit);
#[inline(always)]
pub unsafe fn table_copy<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid
    // table index next.
    let table_x_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };
    // SAFETY: Validation guarantees there to be a valid
    // table index next.
    let table_y_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };

    // SAFETY: The current module address must come from the current
    // store, because it is the only parameter to this function that
    // can contain module addresses. All stores guarantee all
    // addresses in them to be valid within themselves.
    let module = unsafe { state.modules.get(*state.current_module) };

    // SAFETY: Validation guarantees the table index to be
    // valid in the current module.
    let table_addr_x = *unsafe { module.table_addrs.get(table_x_idx) };
    // SAFETY: Validation guarantees the table index to be
    // valid in the current module.
    let table_addr_y = *unsafe { module.table_addrs.get(table_y_idx) };

    // SAFETY: This table address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let tab_x_elem_len = unsafe { state.store_inner.tables.get(table_addr_x) }
        .elem
        .len();
    // SAFETY: This table address was just read from the
    // current store. Therefore, it is valid in the current
    // store.
    let tab_y_elem_len = unsafe { state.store_inner.tables.get(table_addr_y) }
        .elem
        .len();

    let n: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated(); // size
    let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::TABLE_COPY)
        + u64::from(n)
            * T::get_fc_extension_cost_per_element(instructions::fc_extensions::TABLE_COPY);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(n))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    let s: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated(); // source
    let d: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated(); // destination

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
        let table = unsafe { state.store_inner.tables.get_mut(table_addr_x) };

        table.elem.copy_within(s as usize..src_res, d as usize);
    } else {
        let dst_addr = table_addr_x;
        let src_addr = table_addr_y;

        // SAFETY: These table addresses were just read from
        // the current store. Therefore, they are valid in
        // the current store.
        let (src_table, dst_table) =
            unsafe { state.store_inner.tables.get_two_mut(src_addr, dst_addr) }
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
    Ok(ControlFlow::Continue(()))
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-table-mathsf-table-init-x-y
// https://webassembly.github.io/spec/core/binary/instructions.html#table-instructions
// in binary format it seems that elemidx is first ???????
// this is ONLY for passive elements
define_instruction!(
    super::table_init_fn::<T>,
    table_init_fn_mod,
    fuel_check = omit
);
#[inline(always)]
pub unsafe fn table_init_fn<T: Config>(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there to be a valid
    // element index next.
    let elem_idx = unsafe { ElemIdx::decode_unchecked(state.wasm) };
    // SAFETY: Validation guarantees there to be a valid
    // table index next.
    let table_idx = unsafe { TableIdx::decode_unchecked(state.wasm) };

    let n: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated(); // size
    let cost = T::get_fc_extension_flat_cost(instructions::fc_extensions::TABLE_INIT)
        + u64::from(n)
            * T::get_fc_extension_cost_per_element(instructions::fc_extensions::TABLE_INIT);
    if let Some(fuel) = &mut state.resumable.maybe_fuel {
        if *fuel >= cost {
            *fuel -= cost;
        } else {
            state
                .resumable
                .stack
                .push_value(Value::I32(n))
                .unwrap_validated(); // we are pushing back what was just popped, this can't panic.
            return Ok(ControlFlow::Break(InterpreterLoopOutcome::OutOfFuel {
                required_fuel: NonZeroU64::new(cost - *fuel)
                    .expect("the last check guarantees that the current fuel is smaller than cost"),
            }));
        }
    }

    let s: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated(); // offset
    let d: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated(); // dst

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
            state.modules,
            &mut state.store_inner.tables,
            &state.store_inner.elements,
            *state.current_module,
            elem_idx,
            table_idx,
            n,
            s,
            d,
        )?
    };
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::elem_drop_fn,
    elem_drop_fn_mod,
    fuel_check = flat_fc(ELEM_DROP)
);
#[inline(always)]
pub unsafe fn elem_drop_fn(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees there a valid element
    // index next.
    let elem_idx = unsafe { ElemIdx::decode_unchecked(state.wasm) };

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
            state.modules,
            &mut state.store_inner.elements,
            *state.current_module,
            elem_idx,
        );
    }
    Ok(ControlFlow::Continue(()))
}
