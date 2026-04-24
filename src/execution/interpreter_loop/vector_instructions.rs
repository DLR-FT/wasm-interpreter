use core::{
    array,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::{
    assert_validated::UnwrapValidatedExt,
    core::reader::types::opcode,
    execution::interpreter_loop::{define_instruction, from_lanes, to_lanes, Args},
    value::{F32, F64},
    Value,
};

// v128.const
define_instruction!(
    fd_fuel_check,
    v128_const,
    opcode::fd_extensions::V128_CONST,
    |Args {
         wasm, resumable, ..
     }| {
        let mut data = [0; 16];
        for byte_ref in &mut data {
            *byte_ref = wasm.read_u8().unwrap_validated();
        }

        resumable.stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);

// v128.vvunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvunop>
define_instruction!(
    fd_fuel_check,
    v128_not,
    opcode::fd_extensions::V128_NOT,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        resumable
            .stack
            .push_value::<T>(Value::V128(data.map(|byte| !byte)))?;
        Ok(None)
    }
);

// v128.vvbinop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvbinop>
define_instruction!(
    fd_fuel_check,
    v128_and,
    opcode::fd_extensions::V128_AND,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] & data2[i]);
        resumable.stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_andnot,
    opcode::fd_extensions::V128_ANDNOT,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] & !data2[i]);
        resumable.stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_or,
    opcode::fd_extensions::V128_OR,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] | data2[i]);
        resumable.stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    v128_xor,
    opcode::fd_extensions::V128_XOR,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] ^ data2[i]);
        resumable.stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

// v128.vvternop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvternop>
define_instruction!(
    fd_fuel_check,
    v128_bitselect,
    opcode::fd_extensions::V128_BITSELECT,
    |Args { resumable, .. }| {
        let data3: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| (data1[i] & data3[i]) | (data2[i] & !data3[i]));
        resumable.stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

// v128.vvtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvtestop>
define_instruction!(
    fd_fuel_check,
    v128_any_true,
    opcode::fd_extensions::V128_ANY_TRUE,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let any_true = data.into_iter().any(|byte| byte > 0);
        resumable
            .stack
            .push_value::<T>(Value::I32(any_true as u32))?;
        Ok(None)
    }
);

// i8x16.swizzle
define_instruction!(
    fd_fuel_check,
    i8x16_swizzle,
    opcode::fd_extensions::I8X16_SWIZZLE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| *data1.get(usize::from(data2[i])).unwrap_or(&0));
        resumable.stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

// i8x16.shuffle
define_instruction!(
    fd_fuel_check,
    i8x16_shuffle,
    opcode::fd_extensions::I8X16_SHUFFLE,
    |Args {
         wasm, resumable, ..
     }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();

        let lane_selector_indices: [u8; 16] = array::from_fn(|_| wasm.read_u8().unwrap_validated());

        let result = lane_selector_indices.map(|i| {
            *data1
                .get(usize::from(i))
                .or_else(|| data2.get(usize::from(i) - 16))
                .unwrap_validated()
        });

        resumable.stack.push_value::<T>(Value::V128(result))?;
        Ok(None)
    }
);

// shape.splat
define_instruction!(
    fd_fuel_check,
    i8x16_splat,
    opcode::fd_extensions::I8X16_SPLAT,
    |Args { resumable, .. }| {
        let value: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let lane = value as u8;
        let data = from_lanes([lane; 16]);
        resumable.stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_splat,
    opcode::fd_extensions::I16X8_SPLAT,
    |Args { resumable, .. }| {
        let value: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let lane = value as u16;
        let data = from_lanes([lane; 8]);
        resumable.stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_splat,
    opcode::fd_extensions::I32X4_SPLAT,
    |Args { resumable, .. }| {
        let lane: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 4]);
        resumable.stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_splat,
    opcode::fd_extensions::I64X2_SPLAT,
    |Args { resumable, .. }| {
        let lane: u64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 2]);
        resumable.stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_splat,
    opcode::fd_extensions::F32X4_SPLAT,
    |Args { resumable, .. }| {
        let lane: F32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 4]);
        resumable.stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_splat,
    opcode::fd_extensions::F64X2_SPLAT,
    |Args { resumable, .. }| {
        let lane: F64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 2]);
        resumable.stack.push_value::<T>(Value::V128(data))?;
        Ok(None)
    }
);

// shape.extract_lane
define_instruction!(
    fd_fuel_check,
    i8x16_extract_lane_s,
    opcode::fd_extensions::I8X16_EXTRACT_LANE_S,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_extract_lane_u,
    opcode::fd_extensions::I8X16_EXTRACT_LANE_U,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extract_lane_s,
    opcode::fd_extensions::I16X8_EXTRACT_LANE_S,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extract_lane_u,
    opcode::fd_extensions::I16X8_EXTRACT_LANE_U,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::I32(lane as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extract_lane,
    opcode::fd_extensions::I32X4_EXTRACT_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::I32(lane))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extract_lane,
    opcode::fd_extensions::I64X2_EXTRACT_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::I64(lane))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_extract_lane,
    opcode::fd_extensions::F32X4_EXTRACT_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::F32(lane))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_extract_lane,
    opcode::fd_extensions::F64X2_EXTRACT_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        resumable.stack.push_value::<T>(Value::F64(lane))?;
        Ok(None)
    }
);

// shape.replace_lane
define_instruction!(
    fd_fuel_check,
    i8x16_replace_lane,
    opcode::fd_extensions::I8X16_REPLACE_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let value: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let new_lane = value as u8;
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u8; 16] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_replace_lane,
    opcode::fd_extensions::I16X8_REPLACE_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let value: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let new_lane = value as u16;
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u16; 8] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_replace_lane,
    opcode::fd_extensions::I32X4_REPLACE_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_replace_lane,
    opcode::fd_extensions::I64X2_REPLACE_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: u64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_replace_lane,
    opcode::fd_extensions::F32X4_REPLACE_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: F32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [F32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_replace_lane,
    opcode::fd_extensions::F64X2_REPLACE_LANE,
    |Args {
         wasm, resumable, ..
     }| {
        let lane_idx = usize::from(wasm.read_u8().unwrap_validated());
        let new_lane: F64 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [F64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(lanes)))?;
        Ok(None)
    }
);

// shape.vunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vunop>
define_instruction!(
    fd_fuel_check,
    i8x16_abs,
    opcode::fd_extensions::I8X16_ABS,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(i8::wrapping_abs);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_abs,
    opcode::fd_extensions::I16X8_ABS,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(i16::wrapping_abs);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_abs,
    opcode::fd_extensions::I32X4_ABS,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(i32::wrapping_abs);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_abs,
    opcode::fd_extensions::I64X2_ABS,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(i64::wrapping_abs);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_neg,
    opcode::fd_extensions::I8X16_NEG,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(i8::wrapping_neg);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_neg,
    opcode::fd_extensions::I16X8_NEG,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(i16::wrapping_neg);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_neg,
    opcode::fd_extensions::I32X4_NEG,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(i32::wrapping_neg);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_neg,
    opcode::fd_extensions::I64X2_NEG,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(i64::wrapping_neg);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_abs,
    opcode::fd_extensions::F32X4_ABS,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.abs());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_abs,
    opcode::fd_extensions::F64X2_ABS,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.abs());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_neg,
    opcode::fd_extensions::F32X4_NEG,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_neg,
    opcode::fd_extensions::F64X2_NEG,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_sqrt,
    opcode::fd_extensions::F32X4_SQRT,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.sqrt());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_sqrt,
    opcode::fd_extensions::F64X2_SQRT,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.sqrt());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_ceil,
    opcode::fd_extensions::F32X4_CEIL,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.ceil());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_ceil,
    opcode::fd_extensions::F64X2_CEIL,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.ceil());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_floor,
    opcode::fd_extensions::F32X4_FLOOR,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.floor());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_floor,
    opcode::fd_extensions::F64X2_FLOOR,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.floor());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_trunc,
    opcode::fd_extensions::F32X4_TRUNC,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.trunc());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_trunc,
    opcode::fd_extensions::F64X2_TRUNC,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.trunc());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_nearest,
    opcode::fd_extensions::F32X4_NEAREST,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.nearest());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_nearest,
    opcode::fd_extensions::F64X2_NEAREST,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.nearest());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_popcnt,
    opcode::fd_extensions::I8X16_POPCNT,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.count_ones() as u8);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// shape.vbinop  <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vbinop>
define_instruction!(
    fd_fuel_check,
    i8x16_add,
    opcode::fd_extensions::I8X16_ADD,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_add,
    opcode::fd_extensions::I16X8_ADD,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_add,
    opcode::fd_extensions::I32X4_ADD,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_add,
    opcode::fd_extensions::I64X2_ADD,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_sub,
    opcode::fd_extensions::I8X16_SUB,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_sub,
    opcode::fd_extensions::I16X8_SUB,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_sub,
    opcode::fd_extensions::I32X4_SUB,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_sub,
    opcode::fd_extensions::I64X2_SUB,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_add,
    opcode::fd_extensions::F32X4_ADD,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_add,
    opcode::fd_extensions::F64X2_ADD,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_sub,
    opcode::fd_extensions::F32X4_SUB,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_sub,
    opcode::fd_extensions::F64X2_SUB,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_mul,
    opcode::fd_extensions::F32X4_MUL,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_mul,
    opcode::fd_extensions::F64X2_MUL,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_div,
    opcode::fd_extensions::F32X4_DIV,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_div,
    opcode::fd_extensions::F64X2_DIV,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_min,
    opcode::fd_extensions::F32X4_MIN,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_min,
    opcode::fd_extensions::F64X2_MIN,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_max,
    opcode::fd_extensions::F32X4_MAX,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_max,
    opcode::fd_extensions::F64X2_MAX,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_pmin,
    opcode::fd_extensions::F32X4_PMIN,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_pmin,
    opcode::fd_extensions::F64X2_PMIN,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_pmax,
    opcode::fd_extensions::F32X4_PMAX,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_pmax,
    opcode::fd_extensions::F64X2_PMAX,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_min_s,
    opcode::fd_extensions::I8X16_MIN_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_min_s,
    opcode::fd_extensions::I16X8_MIN_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_min_s,
    opcode::fd_extensions::I32X4_MIN_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_min_u,
    opcode::fd_extensions::I8X16_MIN_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_min_u,
    opcode::fd_extensions::I16X8_MIN_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_min_u,
    opcode::fd_extensions::I32X4_MIN_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_max_s,
    opcode::fd_extensions::I8X16_MAX_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_max_s,
    opcode::fd_extensions::I16X8_MAX_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_max_s,
    opcode::fd_extensions::I32X4_MAX_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_max_u,
    opcode::fd_extensions::I8X16_MAX_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_max_u,
    opcode::fd_extensions::I16X8_MAX_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_max_u,
    opcode::fd_extensions::I32X4_MAX_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    i8x16_add_sat_s,
    opcode::fd_extensions::I8X16_ADD_SAT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_add_sat_s,
    opcode::fd_extensions::I16X8_ADD_SAT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_add_sat_u,
    opcode::fd_extensions::I8X16_ADD_SAT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_add_sat_u,
    opcode::fd_extensions::I16X8_ADD_SAT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_sub_sat_s,
    opcode::fd_extensions::I8X16_SUB_SAT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_sub_sat_s,
    opcode::fd_extensions::I16X8_SUB_SAT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_sub_sat_u,
    opcode::fd_extensions::I8X16_SUB_SAT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_sub_sat_u,
    opcode::fd_extensions::I16X8_SUB_SAT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_mul,
    opcode::fd_extensions::I16X8_MUL,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_mul,
    opcode::fd_extensions::I32X4_MUL,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_mul,
    opcode::fd_extensions::I64X2_MUL,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_avgr_u,
    opcode::fd_extensions::I8X16_AVGR_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] =
            array::from_fn(|i| (lanes1[i] as u16 + lanes2[i] as u16).div_ceil(2) as u8);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_avgr_u,
    opcode::fd_extensions::I16X8_AVGR_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] =
            array::from_fn(|i| (lanes1[i] as u32 + lanes2[i] as u32).div_ceil(2) as u16);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_q15mulrsat_s,
    opcode::fd_extensions::I16X8_Q15MULRSAT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| {
            (((lanes1[i] as i64).mul(lanes2[i] as i64) + 2i64.pow(14)) >> 15i64)
                .clamp(i16::MIN as i64, i16::MAX as i64) as i16
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// txN.vrelop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vrelop>
define_instruction!(
    fd_fuel_check,
    i8x16_eq,
    opcode::fd_extensions::I8X16_EQ,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_eq,
    opcode::fd_extensions::I16X8_EQ,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_eq,
    opcode::fd_extensions::I32X4_EQ,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_eq,
    opcode::fd_extensions::I64X2_EQ,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_ne,
    opcode::fd_extensions::I8X16_NE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_ne,
    opcode::fd_extensions::I16X8_NE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_ne,
    opcode::fd_extensions::I32X4_NE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_ne,
    opcode::fd_extensions::I64X2_NE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_lt_s,
    opcode::fd_extensions::I8X16_LT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_lt_s,
    opcode::fd_extensions::I16X8_LT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_lt_s,
    opcode::fd_extensions::I32X4_LT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_lt_s,
    opcode::fd_extensions::I64X2_LT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_lt_u,
    opcode::fd_extensions::I8X16_LT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_lt_u,
    opcode::fd_extensions::I16X8_LT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_lt_u,
    opcode::fd_extensions::I32X4_LT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_gt_s,
    opcode::fd_extensions::I8X16_GT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_gt_s,
    opcode::fd_extensions::I16X8_GT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_gt_s,
    opcode::fd_extensions::I32X4_GT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_gt_s,
    opcode::fd_extensions::I64X2_GT_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_gt_u,
    opcode::fd_extensions::I8X16_GT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_gt_u,
    opcode::fd_extensions::I16X8_GT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_gt_u,
    opcode::fd_extensions::I32X4_GT_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_le_s,
    opcode::fd_extensions::I8X16_LE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_le_s,
    opcode::fd_extensions::I16X8_LE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_le_s,
    opcode::fd_extensions::I32X4_LE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_le_s,
    opcode::fd_extensions::I64X2_LE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_le_u,
    opcode::fd_extensions::I8X16_LE_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_le_u,
    opcode::fd_extensions::I16X8_LE_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_le_u,
    opcode::fd_extensions::I32X4_LE_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

define_instruction!(
    fd_fuel_check,
    i8x16_ge_s,
    opcode::fd_extensions::I8X16_GE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_ge_s,
    opcode::fd_extensions::I16X8_GE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_ge_s,
    opcode::fd_extensions::I32X4_GE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_ge_s,
    opcode::fd_extensions::I64X2_GE_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_ge_u,
    opcode::fd_extensions::I8X16_GE_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_ge_u,
    opcode::fd_extensions::I16X8_GE_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_ge_u,
    opcode::fd_extensions::I32X4_GE_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
// vfrelop
define_instruction!(
    fd_fuel_check,
    f32x4_eq,
    opcode::fd_extensions::F32X4_EQ,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_eq,
    opcode::fd_extensions::F64X2_EQ,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_ne,
    opcode::fd_extensions::F32X4_NE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_ne,
    opcode::fd_extensions::F64X2_NE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_lt,
    opcode::fd_extensions::F32X4_LT,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_lt,
    opcode::fd_extensions::F64X2_LT,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_gt,
    opcode::fd_extensions::F32X4_GT,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_gt,
    opcode::fd_extensions::F64X2_GT,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_le,
    opcode::fd_extensions::F32X4_LE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_le,
    opcode::fd_extensions::F64X2_LE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_ge,
    opcode::fd_extensions::F32X4_GE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_ge,
    opcode::fd_extensions::F64X2_GE,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// txN.vishiftop
define_instruction!(
    fd_fuel_check,
    i8x16_shl,
    opcode::fd_extensions::I8X16_SHL,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shl(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_shl,
    opcode::fd_extensions::I16X8_SHL,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shl(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_shl,
    opcode::fd_extensions::I32X4_SHL,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shl(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_shl,
    opcode::fd_extensions::I64X2_SHL,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shl(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_shr_s,
    opcode::fd_extensions::I8X16_SHR_S,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_shr_u,
    opcode::fd_extensions::I8X16_SHR_U,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_shr_s,
    opcode::fd_extensions::I16X8_SHR_S,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_shr_u,
    opcode::fd_extensions::I16X8_SHR_U,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_shr_s,
    opcode::fd_extensions::I32X4_SHR_S,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_shr_u,
    opcode::fd_extensions::I32X4_SHR_U,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_shr_s,
    opcode::fd_extensions::I64X2_SHR_S,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_shr_u,
    opcode::fd_extensions::I64X2_SHR_U,
    |Args { resumable, .. }| {
        let shift: u32 = resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// shape.vtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vtestop>
define_instruction!(
    fd_fuel_check,
    i8x16_all_true,
    opcode::fd_extensions::I8X16_ALL_TRUE,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        resumable
            .stack
            .push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_all_true,
    opcode::fd_extensions::I16X8_ALL_TRUE,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        resumable
            .stack
            .push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_all_true,
    opcode::fd_extensions::I32X4_ALL_TRUE,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        resumable
            .stack
            .push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_all_true,
    opcode::fd_extensions::I64X2_ALL_TRUE,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        resumable
            .stack
            .push_value::<T>(Value::I32(all_true as u32))?;
        Ok(None)
    }
);

// ishape.bitmask
define_instruction!(
    fd_fuel_check,
    i8x16_bitmask,
    opcode::fd_extensions::I8X16_BITMASK,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        resumable.stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_bitmask,
    opcode::fd_extensions::I16X8_BITMASK,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        resumable.stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_bitmask,
    opcode::fd_extensions::I32X4_BITMASK,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        resumable.stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_bitmask,
    opcode::fd_extensions::I64X2_BITMASK,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        resumable.stack.push_value::<T>(Value::I32(bitmask))?;
        Ok(None)
    }
);

// ishape.narrow_ishape_sx
define_instruction!(
    fd_fuel_check,
    i8x16_narrow_i16x8_s,
    opcode::fd_extensions::I8X16_NARROW_I16X8_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(i8::MIN as i16, i8::MAX as i16) as i8);
        let result: [i8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i8x16_narrow_i16x8_u,
    opcode::fd_extensions::I8X16_NARROW_I16X8_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(u8::MIN as i16, u8::MAX as i16) as u8);
        let result: [u8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_narrow_i32x4_s,
    opcode::fd_extensions::I16X8_NARROW_I32X4_S,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(i16::MIN as i32, i16::MAX as i32) as i16);
        let result: [i16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_narrow_i32x4_u,
    opcode::fd_extensions::I16X8_NARROW_I32X4_U,
    |Args { resumable, .. }| {
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(u16::MIN as i32, u16::MAX as i32) as u16);
        let result: [u16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// t_2xN.vcvtop_t_1xM_sx
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f32x4_s,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F32X4_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f32x4_u,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F32X4_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_convert_i32x4_s,
    opcode::fd_extensions::F32X4_CONVERT_I32X4_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_convert_i32x4_u,
    opcode::fd_extensions::F32X4_CONVERT_I32X4_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// t_2xN.vcvtop_half_t_1xM_sx? <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vcvtop>
define_instruction!(
    fd_fuel_check,
    i16x8_extend_high_i8x16_s,
    opcode::fd_extensions::I16X8_EXTEND_HIGH_I8X16_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let high_lanes: [i8; 8] = lanes[8..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i16);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extend_high_i8x16_u,
    opcode::fd_extensions::I16X8_EXTEND_HIGH_I8X16_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let high_lanes: [u8; 8] = lanes[8..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u16);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extend_low_i8x16_s,
    opcode::fd_extensions::I16X8_EXTEND_LOW_I8X16_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let low_lanes: [i8; 8] = lanes[..8].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i16);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extend_low_i8x16_u,
    opcode::fd_extensions::I16X8_EXTEND_LOW_I8X16_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let low_lanes: [u8; 8] = lanes[..8].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u16);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_high_i16x8_s,
    opcode::fd_extensions::I32X4_EXTEND_HIGH_I16X8_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let high_lanes: [i16; 4] = lanes[4..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i32);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_high_i16x8_u,
    opcode::fd_extensions::I32X4_EXTEND_HIGH_I16X8_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let high_lanes: [u16; 4] = lanes[4..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u32);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_low_i16x8_s,
    opcode::fd_extensions::I32X4_EXTEND_LOW_I16X8_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let low_lanes: [i16; 4] = lanes[..4].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i32);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extend_low_i16x8_u,
    opcode::fd_extensions::I32X4_EXTEND_LOW_I16X8_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let low_lanes: [u16; 4] = lanes[..4].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u32);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_high_i32x4_s,
    opcode::fd_extensions::I64X2_EXTEND_HIGH_I32X4_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let high_lanes: [i32; 2] = lanes[2..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i64);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_high_i32x4_u,
    opcode::fd_extensions::I64X2_EXTEND_HIGH_I32X4_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let high_lanes: [u32; 2] = lanes[2..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u64);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_low_i32x4_s,
    opcode::fd_extensions::I64X2_EXTEND_LOW_I32X4_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i64);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extend_low_i32x4_u,
    opcode::fd_extensions::I64X2_EXTEND_LOW_I32X4_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u64);
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_convert_low_i32x4_s,
    opcode::fd_extensions::F64X2_CONVERT_LOW_I32X4_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| F64(lane as f64));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_convert_low_i32x4_u,
    opcode::fd_extensions::F64X2_CONVERT_LOW_I32X4_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| F64(lane as f64));
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f64x2_promote_low_f32x4,
    opcode::fd_extensions::F64X2_PROMOTE_LOW_F32X4,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let half_lanes: [F32; 2] = lanes[..2].try_into().unwrap();
        let result = half_lanes.map(|lane| lane.as_f64());
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// t_2xN.vcvtop_t_1xM_sx?_zero
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f64x2_s_zero,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F64X2_S_ZERO,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_trunc_sat_f64x2_u_zero,
    opcode::fd_extensions::I32X4_TRUNC_SAT_F64X2_U_ZERO,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    f32x4_demote_f64x2_zero,
    opcode::fd_extensions::F32X4_DEMOTE_F64X2_ZERO,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes = to_lanes::<8, 2, F64>(data);
        let half_lanes = lanes.map(|lane| lane.as_f32());
        let result = [half_lanes[0], half_lanes[1], F32(0.0), F32(0.0)];
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(result)))?;
        Ok(None)
    }
);

// i32x4.dot_i16x8_s
define_instruction!(
    fd_fuel_check,
    i32x4_dot_i16x8_s,
    opcode::fd_extensions::I32X4_DOT_I16X8_S,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
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
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(added)))?;
        Ok(None)
    }
);

// ishape.extmul_half_ishape_sx
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_high_i8x16_s,
    opcode::fd_extensions::I16X8_EXTMUL_HIGH_I8X16_S,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i8; 16] = to_lanes(data1);
        let lanes2: [i8; 16] = to_lanes(data2);
        let high_lanes1: [i8; 8] = lanes1[8..].try_into().unwrap();
        let high_lanes2: [i8; 8] = lanes2[8..].try_into().unwrap();
        let multiplied: [i16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i16;
            let v2 = high_lanes2[i] as i16;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_high_i8x16_u,
    opcode::fd_extensions::I16X8_EXTMUL_HIGH_I8X16_U,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u8; 16] = to_lanes(data1);
        let lanes2: [u8; 16] = to_lanes(data2);
        let high_lanes1: [u8; 8] = lanes1[8..].try_into().unwrap();
        let high_lanes2: [u8; 8] = lanes2[8..].try_into().unwrap();
        let multiplied: [u16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u16;
            let v2 = high_lanes2[i] as u16;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_low_i8x16_s,
    opcode::fd_extensions::I16X8_EXTMUL_LOW_I8X16_S,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i8; 16] = to_lanes(data1);
        let lanes2: [i8; 16] = to_lanes(data2);
        let high_lanes1: [i8; 8] = lanes1[..8].try_into().unwrap();
        let high_lanes2: [i8; 8] = lanes2[..8].try_into().unwrap();
        let multiplied: [i16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i16;
            let v2 = high_lanes2[i] as i16;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extmul_low_i8x16_u,
    opcode::fd_extensions::I16X8_EXTMUL_LOW_I8X16_U,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u8; 16] = to_lanes(data1);
        let lanes2: [u8; 16] = to_lanes(data2);
        let high_lanes1: [u8; 8] = lanes1[..8].try_into().unwrap();
        let high_lanes2: [u8; 8] = lanes2[..8].try_into().unwrap();
        let multiplied: [u16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u16;
            let v2 = high_lanes2[i] as u16;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_high_i16x8_s,
    opcode::fd_extensions::I32X4_EXTMUL_HIGH_I16X8_S,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i16; 8] = to_lanes(data1);
        let lanes2: [i16; 8] = to_lanes(data2);
        let high_lanes1: [i16; 4] = lanes1[4..].try_into().unwrap();
        let high_lanes2: [i16; 4] = lanes2[4..].try_into().unwrap();
        let multiplied: [i32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i32;
            let v2 = high_lanes2[i] as i32;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_high_i16x8_u,
    opcode::fd_extensions::I32X4_EXTMUL_HIGH_I16X8_U,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u16; 8] = to_lanes(data1);
        let lanes2: [u16; 8] = to_lanes(data2);
        let high_lanes1: [u16; 4] = lanes1[4..].try_into().unwrap();
        let high_lanes2: [u16; 4] = lanes2[4..].try_into().unwrap();
        let multiplied: [u32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u32;
            let v2 = high_lanes2[i] as u32;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_low_i16x8_s,
    opcode::fd_extensions::I32X4_EXTMUL_LOW_I16X8_S,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i16; 8] = to_lanes(data1);
        let lanes2: [i16; 8] = to_lanes(data2);
        let high_lanes1: [i16; 4] = lanes1[..4].try_into().unwrap();
        let high_lanes2: [i16; 4] = lanes2[..4].try_into().unwrap();
        let multiplied: [i32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i32;
            let v2 = high_lanes2[i] as i32;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extmul_low_i16x8_u,
    opcode::fd_extensions::I32X4_EXTMUL_LOW_I16X8_U,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u16; 8] = to_lanes(data1);
        let lanes2: [u16; 8] = to_lanes(data2);
        let high_lanes1: [u16; 4] = lanes1[..4].try_into().unwrap();
        let high_lanes2: [u16; 4] = lanes2[..4].try_into().unwrap();
        let multiplied: [u32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u32;
            let v2 = high_lanes2[i] as u32;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_high_i32x4_s,
    opcode::fd_extensions::I64X2_EXTMUL_HIGH_I32X4_S,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i32; 4] = to_lanes(data1);
        let lanes2: [i32; 4] = to_lanes(data2);
        let high_lanes1: [i32; 2] = lanes1[2..].try_into().unwrap();
        let high_lanes2: [i32; 2] = lanes2[2..].try_into().unwrap();
        let multiplied: [i64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i64;
            let v2 = high_lanes2[i] as i64;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_high_i32x4_u,
    opcode::fd_extensions::I64X2_EXTMUL_HIGH_I32X4_U,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u32; 4] = to_lanes(data1);
        let lanes2: [u32; 4] = to_lanes(data2);
        let high_lanes1: [u32; 2] = lanes1[2..].try_into().unwrap();
        let high_lanes2: [u32; 2] = lanes2[2..].try_into().unwrap();
        let multiplied: [u64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u64;
            let v2 = high_lanes2[i] as u64;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_low_i32x4_s,
    opcode::fd_extensions::I64X2_EXTMUL_LOW_I32X4_S,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i32; 4] = to_lanes(data1);
        let lanes2: [i32; 4] = to_lanes(data2);
        let high_lanes1: [i32; 2] = lanes1[..2].try_into().unwrap();
        let high_lanes2: [i32; 2] = lanes2[..2].try_into().unwrap();
        let multiplied: [i64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i64;
            let v2 = high_lanes2[i] as i64;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i64x2_extmul_low_i32x4_u,
    opcode::fd_extensions::I64X2_EXTMUL_LOW_I32X4_U,
    |Args { resumable, .. }| {
        let data1: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u32; 4] = to_lanes(data1);
        let lanes2: [u32; 4] = to_lanes(data2);
        let high_lanes1: [u32; 2] = lanes1[..2].try_into().unwrap();
        let high_lanes2: [u32; 2] = lanes2[..2].try_into().unwrap();
        let multiplied: [u64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u64;
            let v2 = high_lanes2[i] as u64;
            v1.wrapping_mul(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(multiplied)))?;
        Ok(None)
    }
);

// ishape.extadd_pairwise_ishape_sx
define_instruction!(
    fd_fuel_check,
    i16x8_extadd_pairwise_i8x16_s,
    opcode::fd_extensions::I16X8_EXTADD_PAIRWISE_I8X16_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let added_pairwise: [i16; 8] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as i16;
            let v2 = lanes[2 * i + 1] as i16;
            v1.wrapping_add(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i16x8_extadd_pairwise_i8x16_u,
    opcode::fd_extensions::I16X8_EXTADD_PAIRWISE_I8X16_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let added_pairwise: [u16; 8] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as u16;
            let v2 = lanes[2 * i + 1] as u16;
            v1.wrapping_add(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extadd_pairwise_i16x8_s,
    opcode::fd_extensions::I32X4_EXTADD_PAIRWISE_I16X8_S,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let added_pairwise: [i32; 4] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as i32;
            let v2 = lanes[2 * i + 1] as i32;
            v1.wrapping_add(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
define_instruction!(
    fd_fuel_check,
    i32x4_extadd_pairwise_i16x8_u,
    opcode::fd_extensions::I32X4_EXTADD_PAIRWISE_I16X8_U,
    |Args { resumable, .. }| {
        let data: [u8; 16] = resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let added_pairwise: [u32; 4] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as u32;
            let v2 = lanes[2 * i + 1] as u32;
            v1.wrapping_add(v2)
        });
        resumable
            .stack
            .push_value::<T>(Value::V128(from_lanes(added_pairwise)))?;
        Ok(None)
    }
);
