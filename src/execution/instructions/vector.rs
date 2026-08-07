#![expect(
    clippy::missing_safety_doc,
    reason = "see `instructions::State` for more information"
)]

use core::{
    array,
    ops::{Add, ControlFlow, Div, Mul, Neg, Sub},
};

use crate::{
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{from_lanes, to_lanes, InterpreterLoopOutcome, State},
    },
    RuntimeError, Value, F32, F64,
};

// v128.const

#[inline(always)]
pub unsafe fn v128_const(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let mut data = [0; 16];
    for byte_ref in &mut data {
        *byte_ref = state.wasm.decode_u8().unwrap_validated();
    }

    state.resumable.stack.push_value(Value::V128(data))?;
    Ok(ControlFlow::Continue(()))
}

// v128.vvunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvunop>

#[inline(always)]
pub unsafe fn v128_not(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    state
        .resumable
        .stack
        .push_value(Value::V128(data.map(|byte| !byte)))?;
    Ok(ControlFlow::Continue(()))
}

// v128.vvbinop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvbinop>

#[inline(always)]
pub unsafe fn v128_and(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let result = array::from_fn(|i| data1[i] & data2[i]);
    state.resumable.stack.push_value(Value::V128(result))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn v128_andnot(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let result = array::from_fn(|i| data1[i] & !data2[i]);
    state.resumable.stack.push_value(Value::V128(result))?;
    Ok(ControlFlow::Continue(()))
}
#[inline(always)]
pub unsafe fn v128_or(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let result = array::from_fn(|i| data1[i] | data2[i]);
    state.resumable.stack.push_value(Value::V128(result))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn v128_xor(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let result = array::from_fn(|i| data1[i] ^ data2[i]);
    state.resumable.stack.push_value(Value::V128(result))?;
    Ok(ControlFlow::Continue(()))
}

// v128.vvternop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvternop>

#[inline(always)]
pub unsafe fn v128_bitselect(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data3: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let result = array::from_fn(|i| (data1[i] & data3[i]) | (data2[i] & !data3[i]));
    state.resumable.stack.push_value(Value::V128(result))?;
    Ok(ControlFlow::Continue(()))
}

// v128.vvtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvtestop>

#[inline(always)]
pub unsafe fn v128_any_true(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let any_true = data.into_iter().any(|byte| byte > 0);
    state
        .resumable
        .stack
        .push_value(Value::I32(any_true as u32))?;
    Ok(ControlFlow::Continue(()))
}

// i8x16.swizzle

#[inline(always)]
pub unsafe fn i8x16_swizzle(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let result = array::from_fn(|i| *data1.get(usize::from(data2[i])).unwrap_or(&0));
    state.resumable.stack.push_value(Value::V128(result))?;
    Ok(ControlFlow::Continue(()))
}

// i8x16.shuffle

#[inline(always)]
pub unsafe fn i8x16_shuffle(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();

    let lane_selector_indices: [u8; 16] =
        array::from_fn(|_| state.wasm.decode_u8().unwrap_validated());

    let result = lane_selector_indices.map(|i| {
        *data1
            .get(usize::from(i))
            .or_else(|| data2.get(usize::from(i) - 16))
            .unwrap_validated()
    });

    state.resumable.stack.push_value(Value::V128(result))?;
    Ok(ControlFlow::Continue(()))
}

// shape.splat

#[inline(always)]
pub unsafe fn i8x16_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let value: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lane = value as u8;
    let data = from_lanes([lane; 16]);
    state.resumable.stack.push_value(Value::V128(data))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let value: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lane = value as u16;
    let data = from_lanes([lane; 8]);
    state.resumable.stack.push_value(Value::V128(data))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let lane: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let data = from_lanes([lane; 4]);
    state.resumable.stack.push_value(Value::V128(data))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let lane: u64 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let data = from_lanes([lane; 2]);
    state.resumable.stack.push_value(Value::V128(data))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let lane: F32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let data = from_lanes([lane; 4]);
    state.resumable.stack.push_value(Value::V128(data))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_splat(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let lane: F64 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let data = from_lanes([lane; 2]);
    state.resumable.stack.push_value(Value::V128(data))?;
    Ok(ControlFlow::Continue(()))
}

// shape.extract_lane

#[inline(always)]
pub unsafe fn i8x16_extract_lane_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::I32(lane as u32))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_extract_lane_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::I32(lane as u32))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extract_lane_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::I32(lane as u32))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extract_lane_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u16; 8] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::I32(lane as u32))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extract_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::I32(lane))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extract_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u64; 2] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::I64(lane))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_extract_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::F32(lane))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_extract_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let lane = *lanes.get(lane_idx).unwrap_validated();
    state.resumable.stack.push_value(Value::F64(lane))?;
    Ok(ControlFlow::Continue(()))
}

// shape.replace_lane

#[inline(always)]
pub unsafe fn i8x16_replace_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let value: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let new_lane = value as u8;
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let mut lanes: [u8; 16] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_replace_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let value: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let new_lane = value as u16;
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let mut lanes: [u16; 8] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_replace_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let new_lane: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let mut lanes: [u32; 4] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_replace_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let new_lane: u64 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let mut lanes: [u64; 2] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_replace_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let new_lane: F32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let mut lanes: [F32; 4] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_replace_lane(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let lane_idx = usize::from(state.wasm.decode_u8().unwrap_validated());
    // SAFETY: Validation guarantees that there is a value on the stack.
    let new_lane: F64 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let mut lanes: [F64; 2] = to_lanes(data);
    *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(lanes)))?;
    Ok(ControlFlow::Continue(()))
}

// shape.vunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vunop>

#[inline(always)]
pub unsafe fn i8x16_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let result: [i8; 16] = lanes.map(i8::wrapping_abs);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let result: [i16; 8] = lanes.map(i16::wrapping_abs);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let result: [i32; 4] = lanes.map(i32::wrapping_abs);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i64; 2] = to_lanes(data);
    let result: [i64; 2] = lanes.map(i64::wrapping_abs);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let result: [i8; 16] = lanes.map(i8::wrapping_neg);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let result: [i16; 8] = lanes.map(i16::wrapping_neg);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let result: [i32; 4] = lanes.map(i32::wrapping_neg);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i64; 2] = to_lanes(data);
    let result: [i64; 2] = lanes.map(i64::wrapping_neg);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| lane.abs());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result: [F64; 2] = lanes.map(|lane| lane.abs());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| lane.neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result: [F64; 2] = lanes.map(|lane| lane.neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_sqrt(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| lane.sqrt());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_sqrt(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result: [F64; 2] = lanes.map(|lane| lane.sqrt());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_ceil(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| lane.ceil());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_ceil(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result: [F64; 2] = lanes.map(|lane| lane.ceil());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_floor(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| lane.floor());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_floor(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result: [F64; 2] = lanes.map(|lane| lane.floor());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_trunc(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| lane.trunc());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_trunc(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result: [F64; 2] = lanes.map(|lane| lane.trunc());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_nearest(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| lane.nearest());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_nearest(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result: [F64; 2] = lanes.map(|lane| lane.nearest());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_popcnt(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let result: [u8; 16] = lanes.map(|lane| lane.count_ones() as u8);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// shape.vbinop  <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vbinop>

#[inline(always)]
pub unsafe fn i8x16_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u64; 2] = to_lanes(data2);
    let lanes1: [u64; 2] = to_lanes(data1);
    let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u64; 2] = to_lanes(data2);
    let lanes1: [u64; 2] = to_lanes(data1);
    let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_div(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_div(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_min(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_min(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_max(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_max(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_pmin(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| {
        let v1 = lanes1[i];
        let v2 = lanes2[i];
        if v2 < v1 {
            v2
        } else {
            v1
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_pmin(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| {
        let v1 = lanes1[i];
        let v2 = lanes2[i];
        if v2 < v1 {
            v2
        } else {
            v1
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_pmax(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [F32; 4] = array::from_fn(|i| {
        let v1 = lanes1[i];
        let v2 = lanes2[i];
        if v1 < v2 {
            v2
        } else {
            v1
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_pmax(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [F64; 2] = array::from_fn(|i| {
        let v1 = lanes1[i];
        let v2 = lanes2[i];
        if v1 < v2 {
            v2
        } else {
            v1
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_min_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_min_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_min_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_min_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [u8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_min_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_min_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [u32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_max_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_max_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_max_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_max_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [u8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_max_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_max_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [u32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_add_sat_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_add_sat_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_add_sat_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_add_sat_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_sub_sat_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_sub_sat_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_sub_sat_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_sub_sat_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u64; 2] = to_lanes(data2);
    let lanes1: [u64; 2] = to_lanes(data1);
    let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_avgr_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [u8; 16] =
        array::from_fn(|i| (lanes1[i] as u16 + lanes2[i] as u16).div_ceil(2) as u8);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_avgr_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [u16; 8] =
        array::from_fn(|i| (lanes1[i] as u32 + lanes2[i] as u32).div_ceil(2) as u16);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_q15mulrsat_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| {
        (((lanes1[i] as i64).mul(lanes2[i] as i64) + 2i64.pow(14)) >> 15i64)
            .clamp(i16::MIN as i64, i16::MAX as i64) as i16
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// txN.vrelop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vrelop>

#[inline(always)]
pub unsafe fn i8x16_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u64; 2] = to_lanes(data2);
    let lanes1: [u64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u64; 2] = to_lanes(data2);
    let lanes1: [u64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_lt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_lt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_lt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_lt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i64; 2] = to_lanes(data2);
    let lanes1: [i64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_lt_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_lt_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_lt_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_gt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_gt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_gt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_gt_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i64; 2] = to_lanes(data2);
    let lanes1: [i64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_gt_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_gt_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_gt_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_le_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_le_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_le_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_le_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i64; 2] = to_lanes(data2);
    let lanes1: [i64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_le_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_le_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_le_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_ge_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i8; 16] = to_lanes(data2);
    let lanes1: [i8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_ge_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_ge_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_ge_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i64; 2] = to_lanes(data2);
    let lanes1: [i64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_ge_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u8; 16] = to_lanes(data2);
    let lanes1: [u8; 16] = to_lanes(data1);
    let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_ge_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u16; 8] = to_lanes(data2);
    let lanes1: [u16; 8] = to_lanes(data1);
    let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_ge_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [u32; 4] = to_lanes(data2);
    let lanes1: [u32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}
// vfrelop

#[inline(always)]
pub unsafe fn f32x4_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_lt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_lt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_gt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_gt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_le(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_le(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_ge(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F32; 4] = to_lanes(data2);
    let lanes1: [F32; 4] = to_lanes(data1);
    let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_ge(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [F64; 2] = to_lanes(data2);
    let lanes1: [F64; 2] = to_lanes(data1);
    let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// txN.vishiftop

#[inline(always)]
pub unsafe fn i8x16_shl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shl(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_shl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u16; 8] = to_lanes(data);
    let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shl(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_shl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shl(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_shl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u64; 2] = to_lanes(data);
    let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shl(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_shr_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let result: [i8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_shr_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_shr_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let result: [i16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_shr_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u16; 8] = to_lanes(data);
    let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_shr_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let result: [i32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_shr_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_shr_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i64; 2] = to_lanes(data);
    let result: [i64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_shr_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let shift: u32 = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u64; 2] = to_lanes(data);
    let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// shape.vtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vtestop>

#[inline(always)]
pub unsafe fn i8x16_all_true(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let all_true = lanes.into_iter().all(|lane| lane != 0);
    state
        .resumable
        .stack
        .push_value(Value::I32(all_true as u32))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_all_true(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u16; 8] = to_lanes(data);
    let all_true = lanes.into_iter().all(|lane| lane != 0);
    state
        .resumable
        .stack
        .push_value(Value::I32(all_true as u32))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_all_true(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let all_true = lanes.into_iter().all(|lane| lane != 0);
    state
        .resumable
        .stack
        .push_value(Value::I32(all_true as u32))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_all_true(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u64; 2] = to_lanes(data);
    let all_true = lanes.into_iter().all(|lane| lane != 0);
    state
        .resumable
        .stack
        .push_value(Value::I32(all_true as u32))?;
    Ok(ControlFlow::Continue(()))
}

// ishape.bitmask

#[inline(always)]
pub unsafe fn i8x16_bitmask(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let bits = lanes.map(|lane| lane < 0);
    let bitmask = bits
        .into_iter()
        .enumerate()
        .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
    state.resumable.stack.push_value(Value::I32(bitmask))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_bitmask(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let bits = lanes.map(|lane| lane < 0);
    let bitmask = bits
        .into_iter()
        .enumerate()
        .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
    state.resumable.stack.push_value(Value::I32(bitmask))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_bitmask(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let bits = lanes.map(|lane| lane < 0);
    let bitmask = bits
        .into_iter()
        .enumerate()
        .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
    state.resumable.stack.push_value(Value::I32(bitmask))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_bitmask(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i64; 2] = to_lanes(data);
    let bits = lanes.map(|lane| lane < 0);
    let bitmask = bits
        .into_iter()
        .enumerate()
        .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
    state.resumable.stack.push_value(Value::I32(bitmask))?;
    Ok(ControlFlow::Continue(()))
}

// ishape.narrow_ishape_sx

#[inline(always)]
pub unsafe fn i8x16_narrow_i16x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let mut concatenated_narrowed_lanes = lanes1
        .into_iter()
        .chain(lanes2)
        .map(|lane| lane.clamp(i8::MIN as i16, i8::MAX as i16) as i8);
    let result: [i8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i8x16_narrow_i16x8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i16; 8] = to_lanes(data2);
    let lanes1: [i16; 8] = to_lanes(data1);
    let mut concatenated_narrowed_lanes = lanes1
        .into_iter()
        .chain(lanes2)
        .map(|lane| lane.clamp(u8::MIN as i16, u8::MAX as i16) as u8);
    let result: [u8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_narrow_i32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let mut concatenated_narrowed_lanes = lanes1
        .into_iter()
        .chain(lanes2)
        .map(|lane| lane.clamp(i16::MIN as i32, i16::MAX as i32) as i16);
    let result: [i16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_narrow_i32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes2: [i32; 4] = to_lanes(data2);
    let lanes1: [i32; 4] = to_lanes(data1);
    let mut concatenated_narrowed_lanes = lanes1
        .into_iter()
        .chain(lanes2)
        .map(|lane| lane.clamp(u16::MIN as i32, u16::MAX as i32) as u16);
    let result: [u16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// t_2xN.vcvtop_t_1xM_sx

#[inline(always)]
pub unsafe fn i32x4_trunc_sat_f32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result = lanes.map(|lane| {
        if lane.is_nan() {
            0
        } else if lane.is_negative_infinity() {
            i32::MIN
        } else if lane.is_infinity() {
            i32::MAX
        } else {
            lane.as_i32()
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_trunc_sat_f32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let result = lanes.map(|lane| {
        if lane.is_nan() || lane.is_negative_infinity() {
            u32::MIN
        } else if lane.is_infinity() {
            u32::MAX
        } else {
            lane.as_u32()
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_convert_i32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_convert_i32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// t_2xN.vcvtop_half_t_1xM_sx? <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vcvtop>

#[inline(always)]
pub unsafe fn i16x8_extend_high_i8x16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let high_lanes: [i8; 8] = lanes[8..].try_into().unwrap();
    let result = high_lanes.map(|lane| lane as i16);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extend_high_i8x16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let high_lanes: [u8; 8] = lanes[8..].try_into().unwrap();
    let result = high_lanes.map(|lane| lane as u16);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extend_low_i8x16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let low_lanes: [i8; 8] = lanes[..8].try_into().unwrap();
    let result = low_lanes.map(|lane| lane as i16);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extend_low_i8x16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let low_lanes: [u8; 8] = lanes[..8].try_into().unwrap();
    let result = low_lanes.map(|lane| lane as u16);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extend_high_i16x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let high_lanes: [i16; 4] = lanes[4..].try_into().unwrap();
    let result = high_lanes.map(|lane| lane as i32);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extend_high_i16x8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u16; 8] = to_lanes(data);
    let high_lanes: [u16; 4] = lanes[4..].try_into().unwrap();
    let result = high_lanes.map(|lane| lane as u32);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extend_low_i16x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let low_lanes: [i16; 4] = lanes[..4].try_into().unwrap();
    let result = low_lanes.map(|lane| lane as i32);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extend_low_i16x8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u16; 8] = to_lanes(data);
    let low_lanes: [u16; 4] = lanes[..4].try_into().unwrap();
    let result = low_lanes.map(|lane| lane as u32);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extend_high_i32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let high_lanes: [i32; 2] = lanes[2..].try_into().unwrap();
    let result = high_lanes.map(|lane| lane as i64);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extend_high_i32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let high_lanes: [u32; 2] = lanes[2..].try_into().unwrap();
    let result = high_lanes.map(|lane| lane as u64);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extend_low_i32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
    let result = low_lanes.map(|lane| lane as i64);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extend_low_i32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
    let result = low_lanes.map(|lane| lane as u64);
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_convert_low_i32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i32; 4] = to_lanes(data);
    let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
    let result = low_lanes.map(|lane| F64(lane as f64));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_convert_low_i32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u32; 4] = to_lanes(data);
    let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
    let result = low_lanes.map(|lane| F64(lane as f64));
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f64x2_promote_low_f32x4(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F32; 4] = to_lanes(data);
    let half_lanes: [F32; 2] = lanes[..2].try_into().unwrap();
    let result = half_lanes.map(|lane| lane.as_f64());
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// t_2xN.vcvtop_t_1xM_sx?_zero

#[inline(always)]
pub unsafe fn i32x4_trunc_sat_f64x2_s_zero(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result = lanes.map(|lane| {
        if lane.is_nan() {
            0
        } else if lane.is_negative_infinity() {
            i32::MIN
        } else if lane.is_infinity() {
            i32::MAX
        } else {
            lane.as_i32()
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_trunc_sat_f64x2_u_zero(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [F64; 2] = to_lanes(data);
    let result = lanes.map(|lane| {
        if lane.is_nan() || lane.is_negative_infinity() {
            u32::MIN
        } else if lane.is_infinity() {
            u32::MAX
        } else {
            lane.as_u32()
        }
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn f32x4_demote_f64x2_zero(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes = to_lanes::<8, 2, F64>(data);
    let half_lanes = lanes.map(|lane| lane.as_f32());
    let result = [half_lanes[0], half_lanes[1], F32(0.0), F32(0.0)];
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(result)))?;
    Ok(ControlFlow::Continue(()))
}

// i32x4.dot_i16x8_s

#[inline(always)]
pub unsafe fn i32x4_dot_i16x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [i16; 8] = to_lanes(data1);
    let lanes2: [i16; 8] = to_lanes(data2);
    let multiplied: [i32; 8] = array::from_fn(|i| {
        let v1 = lanes1[i] as i32;
        let v2 = lanes2[i] as i32;
        v1.wrapping_mul(v2)
    });
    let added: [i32; 4] = array::from_fn(|i| {
        let v1 = multiplied[2 * i];
        let v2 = multiplied[2 * i + 1];
        v1.wrapping_add(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(added)))?;
    Ok(ControlFlow::Continue(()))
}

// ishape.extmul_half_ishape_sx

#[inline(always)]
pub unsafe fn i16x8_extmul_high_i8x16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [i8; 16] = to_lanes(data1);
    let lanes2: [i8; 16] = to_lanes(data2);
    let high_lanes1: [i8; 8] = lanes1[8..].try_into().unwrap();
    let high_lanes2: [i8; 8] = lanes2[8..].try_into().unwrap();
    let multiplied: [i16; 8] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as i16;
        let v2 = high_lanes2[i] as i16;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extmul_high_i8x16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [u8; 16] = to_lanes(data1);
    let lanes2: [u8; 16] = to_lanes(data2);
    let high_lanes1: [u8; 8] = lanes1[8..].try_into().unwrap();
    let high_lanes2: [u8; 8] = lanes2[8..].try_into().unwrap();
    let multiplied: [u16; 8] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as u16;
        let v2 = high_lanes2[i] as u16;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extmul_low_i8x16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [i8; 16] = to_lanes(data1);
    let lanes2: [i8; 16] = to_lanes(data2);
    let high_lanes1: [i8; 8] = lanes1[..8].try_into().unwrap();
    let high_lanes2: [i8; 8] = lanes2[..8].try_into().unwrap();
    let multiplied: [i16; 8] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as i16;
        let v2 = high_lanes2[i] as i16;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extmul_low_i8x16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [u8; 16] = to_lanes(data1);
    let lanes2: [u8; 16] = to_lanes(data2);
    let high_lanes1: [u8; 8] = lanes1[..8].try_into().unwrap();
    let high_lanes2: [u8; 8] = lanes2[..8].try_into().unwrap();
    let multiplied: [u16; 8] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as u16;
        let v2 = high_lanes2[i] as u16;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extmul_high_i16x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [i16; 8] = to_lanes(data1);
    let lanes2: [i16; 8] = to_lanes(data2);
    let high_lanes1: [i16; 4] = lanes1[4..].try_into().unwrap();
    let high_lanes2: [i16; 4] = lanes2[4..].try_into().unwrap();
    let multiplied: [i32; 4] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as i32;
        let v2 = high_lanes2[i] as i32;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extmul_high_i16x8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [u16; 8] = to_lanes(data1);
    let lanes2: [u16; 8] = to_lanes(data2);
    let high_lanes1: [u16; 4] = lanes1[4..].try_into().unwrap();
    let high_lanes2: [u16; 4] = lanes2[4..].try_into().unwrap();
    let multiplied: [u32; 4] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as u32;
        let v2 = high_lanes2[i] as u32;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extmul_low_i16x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [i16; 8] = to_lanes(data1);
    let lanes2: [i16; 8] = to_lanes(data2);
    let high_lanes1: [i16; 4] = lanes1[..4].try_into().unwrap();
    let high_lanes2: [i16; 4] = lanes2[..4].try_into().unwrap();
    let multiplied: [i32; 4] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as i32;
        let v2 = high_lanes2[i] as i32;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extmul_low_i16x8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [u16; 8] = to_lanes(data1);
    let lanes2: [u16; 8] = to_lanes(data2);
    let high_lanes1: [u16; 4] = lanes1[..4].try_into().unwrap();
    let high_lanes2: [u16; 4] = lanes2[..4].try_into().unwrap();
    let multiplied: [u32; 4] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as u32;
        let v2 = high_lanes2[i] as u32;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extmul_high_i32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [i32; 4] = to_lanes(data1);
    let lanes2: [i32; 4] = to_lanes(data2);
    let high_lanes1: [i32; 2] = lanes1[2..].try_into().unwrap();
    let high_lanes2: [i32; 2] = lanes2[2..].try_into().unwrap();
    let multiplied: [i64; 2] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as i64;
        let v2 = high_lanes2[i] as i64;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extmul_high_i32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [u32; 4] = to_lanes(data1);
    let lanes2: [u32; 4] = to_lanes(data2);
    let high_lanes1: [u32; 2] = lanes1[2..].try_into().unwrap();
    let high_lanes2: [u32; 2] = lanes2[2..].try_into().unwrap();
    let multiplied: [u64; 2] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as u64;
        let v2 = high_lanes2[i] as u64;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extmul_low_i32x4_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [i32; 4] = to_lanes(data1);
    let lanes2: [i32; 4] = to_lanes(data2);
    let high_lanes1: [i32; 2] = lanes1[..2].try_into().unwrap();
    let high_lanes2: [i32; 2] = lanes2[..2].try_into().unwrap();
    let multiplied: [i64; 2] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as i64;
        let v2 = high_lanes2[i] as i64;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i64x2_extmul_low_i32x4_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data1: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data2: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes1: [u32; 4] = to_lanes(data1);
    let lanes2: [u32; 4] = to_lanes(data2);
    let high_lanes1: [u32; 2] = lanes1[..2].try_into().unwrap();
    let high_lanes2: [u32; 2] = lanes2[..2].try_into().unwrap();
    let multiplied: [u64; 2] = array::from_fn(|i| {
        let v1 = high_lanes1[i] as u64;
        let v2 = high_lanes2[i] as u64;
        v1.wrapping_mul(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(multiplied)))?;
    Ok(ControlFlow::Continue(()))
}

// ishape.extadd_pairwise_ishape_sx

#[inline(always)]
pub unsafe fn i16x8_extadd_pairwise_i8x16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i8; 16] = to_lanes(data);
    let added_pairwise: [i16; 8] = array::from_fn(|i| {
        let v1 = lanes[2 * i] as i16;
        let v2 = lanes[2 * i + 1] as i16;
        v1.wrapping_add(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(added_pairwise)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i16x8_extadd_pairwise_i8x16_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u8; 16] = to_lanes(data);
    let added_pairwise: [u16; 8] = array::from_fn(|i| {
        let v1 = lanes[2 * i] as u16;
        let v2 = lanes[2 * i + 1] as u16;
        v1.wrapping_add(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(added_pairwise)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extadd_pairwise_i16x8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [i16; 8] = to_lanes(data);
    let added_pairwise: [i32; 4] = array::from_fn(|i| {
        let v1 = lanes[2 * i] as i32;
        let v2 = lanes[2 * i + 1] as i32;
        v1.wrapping_add(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(added_pairwise)))?;
    Ok(ControlFlow::Continue(()))
}

#[inline(always)]
pub unsafe fn i32x4_extadd_pairwise_i16x8_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    // SAFETY: Validation guarantees that there is a value on the stack.
    let data: [u8; 16] = unsafe { state.resumable.stack.pop_value() }
        .try_into()
        .unwrap_validated();
    let lanes: [u16; 8] = to_lanes(data);
    let added_pairwise: [u32; 4] = array::from_fn(|i| {
        let v1 = lanes[2 * i] as u32;
        let v2 = lanes[2 * i + 1] as u32;
        v1.wrapping_add(v2)
    });
    state
        .resumable
        .stack
        .push_value(Value::V128(from_lanes(added_pairwise)))?;
    Ok(ControlFlow::Continue(()))
}
