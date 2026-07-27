use core::{
    array,
    ops::{Add, ControlFlow, Div, Mul, Neg, Sub},
};

use crate::{
    core::structure::instructions,
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction_fn, from_lanes, to_lanes, Args},
    },
    Value, F32, F64,
};

// v128.const
define_instruction_fn! {
    v128_const,
    fuel_check = flat_fc(instructions::fd_extensions::V128_CONST),
    |args: Args| {
        let mut data = [0; 16];
        for byte_ref in &mut data {
            *byte_ref = args.wasm.decode_u8().unwrap_validated();
        }

        args.resumable.stack.push_value(Value::V128(data))?;
        Ok(ControlFlow::Continue(()))
    }
}

// v128.vvunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvunop>
define_instruction_fn! {
    v128_not,
    fuel_check = flat_fc(instructions::fd_extensions::V128_NOT),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        args.resumable
            .stack
            .push_value(Value::V128(data.map(|byte| !byte)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// v128.vvbinop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvbinop>
define_instruction_fn! {
    v128_and,
    fuel_check = flat_fc(instructions::fd_extensions::V128_AND),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] & data2[i]);
        args.resumable.stack.push_value(Value::V128(result))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_andnot,
    fuel_check = flat_fc(instructions::fd_extensions::V128_ANDNOT),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] & !data2[i]);
        args.resumable.stack.push_value(Value::V128(result))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_or,
    fuel_check = flat_fc(instructions::fd_extensions::V128_OR),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] | data2[i]);
        args.resumable.stack.push_value(Value::V128(result))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    v128_xor,
    fuel_check = flat_fc(instructions::fd_extensions::V128_XOR),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| data1[i] ^ data2[i]);
        args.resumable.stack.push_value(Value::V128(result))?;
        Ok(ControlFlow::Continue(()))
    }
}

// v128.vvternop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvternop>
define_instruction_fn! {
    v128_bitselect,
    fuel_check = flat_fc(instructions::fd_extensions::V128_BITSELECT),
    |args: Args| {
        let data3: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| (data1[i] & data3[i]) | (data2[i] & !data3[i]));
        args.resumable.stack.push_value(Value::V128(result))?;
        Ok(ControlFlow::Continue(()))
    }
}

// v128.vvtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vvtestop>
define_instruction_fn! {
    v128_any_true,
    fuel_check = flat_fc(instructions::fd_extensions::V128_ANY_TRUE),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let any_true = data.into_iter().any(|byte| byte > 0);
        args.resumable.stack.push_value(Value::I32(any_true as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}

// i8x16.swizzle
define_instruction_fn! {
    i8x16_swizzle,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SWIZZLE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let result = array::from_fn(|i| *data1.get(usize::from(data2[i])).unwrap_or(&0));
        args.resumable.stack.push_value(Value::V128(result))?;
        Ok(ControlFlow::Continue(()))
    }
}

// i8x16.shuffle
define_instruction_fn! {
    i8x16_shuffle,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SHUFFLE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let lane_selector_indices: [u8; 16] = array::from_fn(|_| args.wasm.decode_u8().unwrap_validated());

        let result = lane_selector_indices.map(|i| {
            *data1
                .get(usize::from(i))
                .or_else(|| data2.get(usize::from(i) - 16))
                .unwrap_validated()
        });

        args.resumable.stack.push_value(Value::V128(result))?;
        Ok(ControlFlow::Continue(()))
    }
}

// shape.splat
define_instruction_fn! {
    i8x16_splat,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SPLAT),
    |args: Args| {
        let value: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lane = value as u8;
        let data = from_lanes([lane; 16]);
        args.resumable.stack.push_value(Value::V128(data))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_splat,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_SPLAT),
    |args: Args| {
        let value: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lane = value as u16;
        let data = from_lanes([lane; 8]);
        args.resumable.stack.push_value(Value::V128(data))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_splat,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_SPLAT),
    |args: Args| {
        let lane: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 4]);
        args.resumable.stack.push_value(Value::V128(data))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_splat,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_SPLAT),
    |args: Args| {
        let lane: u64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 2]);
        args.resumable.stack.push_value(Value::V128(data))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_splat,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_SPLAT),
    |args: Args| {
        let lane: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 4]);
        args.resumable.stack.push_value(Value::V128(data))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_splat,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_SPLAT),
    |args: Args| {
        let lane: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data = from_lanes([lane; 2]);
        args.resumable.stack.push_value(Value::V128(data))?;
        Ok(ControlFlow::Continue(()))
    }
}

// shape.extract_lane
define_instruction_fn! {
    i8x16_extract_lane_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_EXTRACT_LANE_S),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::I32(lane as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_extract_lane_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_EXTRACT_LANE_U),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::I32(lane as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extract_lane_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTRACT_LANE_S),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::I32(lane as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extract_lane_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTRACT_LANE_U),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::I32(lane as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extract_lane,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTRACT_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::I32(lane))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extract_lane,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTRACT_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::I64(lane))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_extract_lane,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_EXTRACT_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::F32(lane))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_extract_lane,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_EXTRACT_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let lane = *lanes.get(lane_idx).unwrap_validated();
        args.resumable.stack.push_value(Value::F64(lane))?;
        Ok(ControlFlow::Continue(()))
    }
}

// shape.replace_lane
define_instruction_fn! {
    i8x16_replace_lane,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_REPLACE_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let value: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let new_lane = value as u8;
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u8; 16] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        args.resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_replace_lane,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_REPLACE_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let value: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let new_lane = value as u16;
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u16; 8] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        args.resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_replace_lane,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_REPLACE_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let new_lane: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        args.resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_replace_lane,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_REPLACE_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let new_lane: u64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [u64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        args.resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_replace_lane,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_REPLACE_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let new_lane: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [F32; 4] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        args.resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_replace_lane,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_REPLACE_LANE),
    |args: Args| {
        let lane_idx = usize::from(args.wasm.decode_u8().unwrap_validated());
        let new_lane: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let mut lanes: [F64; 2] = to_lanes(data);
        *lanes.get_mut(lane_idx).unwrap_validated() = new_lane;
        args.resumable.stack.push_value(Value::V128(from_lanes(lanes)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// shape.vunop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vunop>
define_instruction_fn! {
    i8x16_abs,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_ABS),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(i8::wrapping_abs);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_abs,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_ABS),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(i16::wrapping_abs);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_abs,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_ABS),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(i32::wrapping_abs);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_abs,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_ABS),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(i64::wrapping_abs);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_neg,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_NEG),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(i8::wrapping_neg);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_neg,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_NEG),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(i16::wrapping_neg);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_neg,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_NEG),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(i32::wrapping_neg);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_neg,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_NEG),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(i64::wrapping_neg);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_abs,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_ABS),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.abs());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_abs,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_ABS),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.abs());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_neg,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_NEG),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_neg,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_NEG),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_sqrt,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_SQRT),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.sqrt());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_sqrt,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_SQRT),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.sqrt());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_ceil,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_CEIL),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.ceil());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_ceil,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_CEIL),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.ceil());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_floor,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_FLOOR),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.floor());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_floor,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_FLOOR),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.floor());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_trunc,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_TRUNC),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.trunc());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_trunc,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_TRUNC),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.trunc());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_nearest,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_NEAREST),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| lane.nearest());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_nearest,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_NEAREST),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F64; 2] = to_lanes(data);
        let result: [F64; 2] = lanes.map(|lane| lane.nearest());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_popcnt,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_POPCNT),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.count_ones() as u8);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// shape.vbinop  <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vbinop>
define_instruction_fn! {
    i8x16_add,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_ADD),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_add,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_ADD),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_add,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_ADD),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_add,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_ADD),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_sub,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SUB),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_sub,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_SUB),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_sub,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_SUB),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_sub,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_SUB),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_add,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_ADD),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_add,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_ADD),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_sub,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_SUB),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_sub,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_SUB),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_mul,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_MUL),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_mul,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_MUL),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].mul(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_div,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_DIV),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_div,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_DIV),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].div(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_min,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_MIN),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_min,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_MIN),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_max,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_MAX),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [F32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_max,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_MAX),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [F64; 2] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_pmin,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_PMIN),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_pmin,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_PMIN),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_pmax,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_PMAX),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_pmax,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_PMAX),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_min_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_MIN_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_min_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_MIN_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_min_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_MIN_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_min_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_MIN_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_min_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_MIN_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_min_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_MIN_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].min(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_max_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_MAX_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_max_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_MAX_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_max_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_MAX_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_max_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_MAX_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_max_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_MAX_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_max_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_MAX_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].max(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i8x16_add_sat_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_ADD_SAT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_add_sat_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_ADD_SAT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_add_sat_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_ADD_SAT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_add_sat_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_ADD_SAT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_add(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_sub_sat_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SUB_SAT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_sub_sat_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_SUB_SAT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_sub_sat_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SUB_SAT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_sub_sat_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_SUB_SAT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].saturating_sub(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_mul,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_MUL),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_mul,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_MUL),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [u32; 4] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_mul,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_MUL),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [u64; 2] = array::from_fn(|i| lanes1[i].wrapping_mul(lanes2[i]));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_avgr_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_AVGR_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [u8; 16] =
            array::from_fn(|i| (lanes1[i] as u16 + lanes2[i] as u16).div_ceil(2) as u8);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_avgr_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_AVGR_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [u16; 8] =
            array::from_fn(|i| (lanes1[i] as u32 + lanes2[i] as u32).div_ceil(2) as u16);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_q15mulrsat_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_Q15MULRSAT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| {
            (((lanes1[i] as i64).mul(lanes2[i] as i64) + 2i64.pow(14)) >> 15i64)
                .clamp(i16::MIN as i64, i16::MAX as i64) as i16
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// txN.vrelop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vrelop>
define_instruction_fn! {
    i8x16_eq,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_EQ),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_eq,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EQ),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_eq,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EQ),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_eq,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EQ),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_ne,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_NE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_ne,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_NE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_ne,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_NE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_ne,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_NE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u64; 2] = to_lanes(data2);
        let lanes1: [u64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_lt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_LT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_lt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_LT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_lt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_LT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_lt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_LT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_lt_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_LT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_lt_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_LT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_lt_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_LT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_gt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_GT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_gt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_GT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_gt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_GT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_gt_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_GT_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_gt_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_GT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_gt_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_GT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_gt_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_GT_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_le_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_LE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_le_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_LE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_le_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_LE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_le_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_LE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_le_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_LE_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_le_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_LE_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_le_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_LE_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i8x16_ge_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_GE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i8; 16] = to_lanes(data2);
        let lanes1: [i8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_ge_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_GE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_ge_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_GE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_ge_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_GE_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i64; 2] = to_lanes(data2);
        let lanes1: [i64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_ge_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_GE_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u8; 16] = to_lanes(data2);
        let lanes1: [u8; 16] = to_lanes(data1);
        let result: [i8; 16] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i8).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_ge_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_GE_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u16; 8] = to_lanes(data2);
        let lanes1: [u16; 8] = to_lanes(data1);
        let result: [i16; 8] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i16).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_ge_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_GE_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [u32; 4] = to_lanes(data2);
        let lanes1: [u32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
// vfrelop
define_instruction_fn! {
    f32x4_eq,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_EQ),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_eq,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_EQ),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] == lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_ne,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_NE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_ne,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_NE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] != lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_lt,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_LT),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_lt,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_LT),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] < lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_gt,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_GT),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_gt,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_GT),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] > lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_le,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_LE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_le,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_LE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] <= lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_ge,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_GE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F32; 4] = to_lanes(data2);
        let lanes1: [F32; 4] = to_lanes(data1);
        let result: [i32; 4] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i32).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_ge,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_GE),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [F64; 2] = to_lanes(data2);
        let lanes1: [F64; 2] = to_lanes(data1);
        let result: [i64; 2] = array::from_fn(|i| ((lanes1[i] >= lanes2[i]) as i64).neg());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// txN.vishiftop
define_instruction_fn! {
    i8x16_shl,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SHL),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shl(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_shl,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_SHL),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shl(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_shl,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_SHL),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shl(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_shl,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_SHL),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shl(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_shr_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SHR_S),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let result: [i8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_shr_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_SHR_U),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let result: [u8; 16] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_shr_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_SHR_S),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let result: [i16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_shr_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_SHR_U),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let result: [u16; 8] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_shr_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_SHR_S),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [i32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_shr_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_SHR_U),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [u32; 4] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_shr_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_SHR_S),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let result: [i64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_shr_u,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_SHR_U),
    |args: Args| {
        let shift: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let result: [u64; 2] = lanes.map(|lane| lane.wrapping_shr(shift));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// shape.vtestop <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vtestop>
define_instruction_fn! {
    i8x16_all_true,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_ALL_TRUE),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        args.resumable.stack.push_value(Value::I32(all_true as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_all_true,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_ALL_TRUE),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        args.resumable.stack.push_value(Value::I32(all_true as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_all_true,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_ALL_TRUE),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        args.resumable.stack.push_value(Value::I32(all_true as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_all_true,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_ALL_TRUE),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u64; 2] = to_lanes(data);
        let all_true = lanes.into_iter().all(|lane| lane != 0);
        args.resumable.stack.push_value(Value::I32(all_true as u32))?;
        Ok(ControlFlow::Continue(()))
    }
}

// ishape.bitmask
define_instruction_fn! {
    i8x16_bitmask,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_BITMASK),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        args.resumable.stack.push_value(Value::I32(bitmask))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_bitmask,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_BITMASK),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        args.resumable.stack.push_value(Value::I32(bitmask))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_bitmask,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_BITMASK),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        args.resumable.stack.push_value(Value::I32(bitmask))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_bitmask,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_BITMASK),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i64; 2] = to_lanes(data);
        let bits = lanes.map(|lane| lane < 0);
        let bitmask = bits
            .into_iter()
            .enumerate()
            .fold(0u32, |acc, (i, bit)| acc | ((bit as u32) << i));
        args.resumable.stack.push_value(Value::I32(bitmask))?;
        Ok(ControlFlow::Continue(()))
    }
}

// ishape.narrow_ishape_sx
define_instruction_fn! {
    i8x16_narrow_i16x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_NARROW_I16X8_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(i8::MIN as i16, i8::MAX as i16) as i8);
        let result: [i8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i8x16_narrow_i16x8_u,
    fuel_check = flat_fc(instructions::fd_extensions::I8X16_NARROW_I16X8_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i16; 8] = to_lanes(data2);
        let lanes1: [i16; 8] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(u8::MIN as i16, u8::MAX as i16) as u8);
        let result: [u8; 16] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_narrow_i32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_NARROW_I32X4_S),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(i16::MIN as i32, i16::MAX as i32) as i16);
        let result: [i16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_narrow_i32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_NARROW_I32X4_U),
    |args: Args| {
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes2: [i32; 4] = to_lanes(data2);
        let lanes1: [i32; 4] = to_lanes(data1);
        let mut concatenated_narrowed_lanes = lanes1
            .into_iter()
            .chain(lanes2)
            .map(|lane| lane.clamp(u16::MIN as i32, u16::MAX as i32) as u16);
        let result: [u16; 8] = array::from_fn(|_| concatenated_narrowed_lanes.next().unwrap());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// t_2xN.vcvtop_t_1xM_sx
define_instruction_fn! {
    i32x4_trunc_sat_f32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_TRUNC_SAT_F32X4_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_trunc_sat_f32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_TRUNC_SAT_F32X4_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_convert_i32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_CONVERT_I32X4_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_convert_i32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_CONVERT_I32X4_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let result: [F32; 4] = lanes.map(|lane| F32(lane as f32));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// t_2xN.vcvtop_half_t_1xM_sx? <https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-vcvtop>
define_instruction_fn! {
    i16x8_extend_high_i8x16_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTEND_HIGH_I8X16_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let high_lanes: [i8; 8] = lanes[8..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i16);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extend_high_i8x16_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTEND_HIGH_I8X16_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let high_lanes: [u8; 8] = lanes[8..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u16);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extend_low_i8x16_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTEND_LOW_I8X16_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let low_lanes: [i8; 8] = lanes[..8].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i16);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extend_low_i8x16_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTEND_LOW_I8X16_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let low_lanes: [u8; 8] = lanes[..8].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u16);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extend_high_i16x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTEND_HIGH_I16X8_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let high_lanes: [i16; 4] = lanes[4..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i32);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extend_high_i16x8_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTEND_HIGH_I16X8_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let high_lanes: [u16; 4] = lanes[4..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u32);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extend_low_i16x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTEND_LOW_I16X8_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let low_lanes: [i16; 4] = lanes[..4].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i32);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extend_low_i16x8_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTEND_LOW_I16X8_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let low_lanes: [u16; 4] = lanes[..4].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u32);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extend_high_i32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTEND_HIGH_I32X4_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let high_lanes: [i32; 2] = lanes[2..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as i64);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extend_high_i32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTEND_HIGH_I32X4_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let high_lanes: [u32; 2] = lanes[2..].try_into().unwrap();
        let result = high_lanes.map(|lane| lane as u64);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extend_low_i32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTEND_LOW_I32X4_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as i64);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extend_low_i32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTEND_LOW_I32X4_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| lane as u64);
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_convert_low_i32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_CONVERT_LOW_I32X4_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i32; 4] = to_lanes(data);
        let low_lanes: [i32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| F64(lane as f64));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_convert_low_i32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_CONVERT_LOW_I32X4_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u32; 4] = to_lanes(data);
        let low_lanes: [u32; 2] = lanes[..2].try_into().unwrap();
        let result = low_lanes.map(|lane| F64(lane as f64));
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f64x2_promote_low_f32x4,
    fuel_check = flat_fc(instructions::fd_extensions::F64X2_PROMOTE_LOW_F32X4),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [F32; 4] = to_lanes(data);
        let half_lanes: [F32; 2] = lanes[..2].try_into().unwrap();
        let result = half_lanes.map(|lane| lane.as_f64());
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// t_2xN.vcvtop_t_1xM_sx?_zero
define_instruction_fn! {
    i32x4_trunc_sat_f64x2_s_zero,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_TRUNC_SAT_F64X2_S_ZERO),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_trunc_sat_f64x2_u_zero,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_TRUNC_SAT_F64X2_U_ZERO),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes([result[0], result[1], 0, 0])))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    f32x4_demote_f64x2_zero,
    fuel_check = flat_fc(instructions::fd_extensions::F32X4_DEMOTE_F64X2_ZERO),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes = to_lanes::<8, 2, F64>(data);
        let half_lanes = lanes.map(|lane| lane.as_f32());
        let result = [half_lanes[0], half_lanes[1], F32(0.0), F32(0.0)];
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(result)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// i32x4.dot_i16x8_s
define_instruction_fn! {
    i32x4_dot_i16x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_DOT_I16X8_S),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
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
        args.resumable.stack.push_value(Value::V128(from_lanes(added)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// ishape.extmul_half_ishape_sx
define_instruction_fn! {
    i16x8_extmul_high_i8x16_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTMUL_HIGH_I8X16_S),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i8; 16] = to_lanes(data1);
        let lanes2: [i8; 16] = to_lanes(data2);
        let high_lanes1: [i8; 8] = lanes1[8..].try_into().unwrap();
        let high_lanes2: [i8; 8] = lanes2[8..].try_into().unwrap();
        let multiplied: [i16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i16;
            let v2 = high_lanes2[i] as i16;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extmul_high_i8x16_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTMUL_HIGH_I8X16_U),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u8; 16] = to_lanes(data1);
        let lanes2: [u8; 16] = to_lanes(data2);
        let high_lanes1: [u8; 8] = lanes1[8..].try_into().unwrap();
        let high_lanes2: [u8; 8] = lanes2[8..].try_into().unwrap();
        let multiplied: [u16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u16;
            let v2 = high_lanes2[i] as u16;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extmul_low_i8x16_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTMUL_LOW_I8X16_S),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i8; 16] = to_lanes(data1);
        let lanes2: [i8; 16] = to_lanes(data2);
        let high_lanes1: [i8; 8] = lanes1[..8].try_into().unwrap();
        let high_lanes2: [i8; 8] = lanes2[..8].try_into().unwrap();
        let multiplied: [i16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i16;
            let v2 = high_lanes2[i] as i16;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extmul_low_i8x16_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTMUL_LOW_I8X16_U),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u8; 16] = to_lanes(data1);
        let lanes2: [u8; 16] = to_lanes(data2);
        let high_lanes1: [u8; 8] = lanes1[..8].try_into().unwrap();
        let high_lanes2: [u8; 8] = lanes2[..8].try_into().unwrap();
        let multiplied: [u16; 8] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u16;
            let v2 = high_lanes2[i] as u16;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extmul_high_i16x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTMUL_HIGH_I16X8_S),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i16; 8] = to_lanes(data1);
        let lanes2: [i16; 8] = to_lanes(data2);
        let high_lanes1: [i16; 4] = lanes1[4..].try_into().unwrap();
        let high_lanes2: [i16; 4] = lanes2[4..].try_into().unwrap();
        let multiplied: [i32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i32;
            let v2 = high_lanes2[i] as i32;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extmul_high_i16x8_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTMUL_HIGH_I16X8_U),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u16; 8] = to_lanes(data1);
        let lanes2: [u16; 8] = to_lanes(data2);
        let high_lanes1: [u16; 4] = lanes1[4..].try_into().unwrap();
        let high_lanes2: [u16; 4] = lanes2[4..].try_into().unwrap();
        let multiplied: [u32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u32;
            let v2 = high_lanes2[i] as u32;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extmul_low_i16x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTMUL_LOW_I16X8_S),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i16; 8] = to_lanes(data1);
        let lanes2: [i16; 8] = to_lanes(data2);
        let high_lanes1: [i16; 4] = lanes1[..4].try_into().unwrap();
        let high_lanes2: [i16; 4] = lanes2[..4].try_into().unwrap();
        let multiplied: [i32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i32;
            let v2 = high_lanes2[i] as i32;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extmul_low_i16x8_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTMUL_LOW_I16X8_U),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u16; 8] = to_lanes(data1);
        let lanes2: [u16; 8] = to_lanes(data2);
        let high_lanes1: [u16; 4] = lanes1[..4].try_into().unwrap();
        let high_lanes2: [u16; 4] = lanes2[..4].try_into().unwrap();
        let multiplied: [u32; 4] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u32;
            let v2 = high_lanes2[i] as u32;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extmul_high_i32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTMUL_HIGH_I32X4_S),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i32; 4] = to_lanes(data1);
        let lanes2: [i32; 4] = to_lanes(data2);
        let high_lanes1: [i32; 2] = lanes1[2..].try_into().unwrap();
        let high_lanes2: [i32; 2] = lanes2[2..].try_into().unwrap();
        let multiplied: [i64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i64;
            let v2 = high_lanes2[i] as i64;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extmul_high_i32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTMUL_HIGH_I32X4_U),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u32; 4] = to_lanes(data1);
        let lanes2: [u32; 4] = to_lanes(data2);
        let high_lanes1: [u32; 2] = lanes1[2..].try_into().unwrap();
        let high_lanes2: [u32; 2] = lanes2[2..].try_into().unwrap();
        let multiplied: [u64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u64;
            let v2 = high_lanes2[i] as u64;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extmul_low_i32x4_s,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTMUL_LOW_I32X4_S),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [i32; 4] = to_lanes(data1);
        let lanes2: [i32; 4] = to_lanes(data2);
        let high_lanes1: [i32; 2] = lanes1[..2].try_into().unwrap();
        let high_lanes2: [i32; 2] = lanes2[..2].try_into().unwrap();
        let multiplied: [i64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as i64;
            let v2 = high_lanes2[i] as i64;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i64x2_extmul_low_i32x4_u,
    fuel_check = flat_fc(instructions::fd_extensions::I64X2_EXTMUL_LOW_I32X4_U),
    |args: Args| {
        let data1: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let data2: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes1: [u32; 4] = to_lanes(data1);
        let lanes2: [u32; 4] = to_lanes(data2);
        let high_lanes1: [u32; 2] = lanes1[..2].try_into().unwrap();
        let high_lanes2: [u32; 2] = lanes2[..2].try_into().unwrap();
        let multiplied: [u64; 2] = array::from_fn(|i| {
            let v1 = high_lanes1[i] as u64;
            let v2 = high_lanes2[i] as u64;
            v1.wrapping_mul(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(multiplied)))?;
        Ok(ControlFlow::Continue(()))
    }
}

// ishape.extadd_pairwise_ishape_sx
define_instruction_fn! {
    i16x8_extadd_pairwise_i8x16_s,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTADD_PAIRWISE_I8X16_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i8; 16] = to_lanes(data);
        let added_pairwise: [i16; 8] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as i16;
            let v2 = lanes[2 * i + 1] as i16;
            v1.wrapping_add(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(added_pairwise)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i16x8_extadd_pairwise_i8x16_u,
    fuel_check = flat_fc(instructions::fd_extensions::I16X8_EXTADD_PAIRWISE_I8X16_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u8; 16] = to_lanes(data);
        let added_pairwise: [u16; 8] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as u16;
            let v2 = lanes[2 * i + 1] as u16;
            v1.wrapping_add(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(added_pairwise)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extadd_pairwise_i16x8_s,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTADD_PAIRWISE_I16X8_S),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [i16; 8] = to_lanes(data);
        let added_pairwise: [i32; 4] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as i32;
            let v2 = lanes[2 * i + 1] as i32;
            v1.wrapping_add(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(added_pairwise)))?;
        Ok(ControlFlow::Continue(()))
    }
}
define_instruction_fn! {
    i32x4_extadd_pairwise_i16x8_u,
    fuel_check = flat_fc(instructions::fd_extensions::I32X4_EXTADD_PAIRWISE_I16X8_U),
    |args: Args| {
        let data: [u8; 16] = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let lanes: [u16; 8] = to_lanes(data);
        let added_pairwise: [u32; 4] = array::from_fn(|i| {
            let v1 = lanes[2 * i] as u32;
            let v2 = lanes[2 * i + 1] as u32;
            v1.wrapping_add(v2)
        });
        args.resumable
            .stack
            .push_value(Value::V128(from_lanes(added_pairwise)))?;
        Ok(ControlFlow::Continue(()))
    }
}
