use core::ops::ControlFlow;

use crate::{
    core::structure::instructions,
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction_fn, Args},
    },
    trace, TrapError, F32, F64,
};

// t.const
define_instruction_fn! {
    i32_const,
    fuel_check = flat(instructions::I32_CONST),
    |args: Args| {
        let constant = args.wasm.decode_var_i32().unwrap_validated();
        trace!("Instruction: i32.const [] -> [{constant}]");
        args.resumable.stack.push_value(constant.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_const,
    fuel_check = flat(instructions::I64_CONST),
    |args: Args| {
        let constant = args.wasm.decode_var_i64().unwrap_validated();
        trace!("Instruction: i64.const [] -> [{constant}]");
        args.resumable.stack.push_value(constant.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_const,
    fuel_check = flat(instructions::F32_CONST),
    |args: Args| {
        let constant = F32::from_bits(args.wasm.decode_f32().unwrap_validated());
        trace!("Instruction: f32.const [] -> [{constant:.7}]");
        args.resumable.stack.push_value(constant.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_const,
    fuel_check = flat(instructions::F64_CONST),
    |args: Args| {
        let constant = F64::from_bits(args.wasm.decode_f64().unwrap_validated());
        trace!("Instruction: f64.const [] -> [{constant}]");
        args.resumable.stack.push_value(constant.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i32.unop
define_instruction_fn! {
    i32_clz,
    fuel_check = flat(instructions::I32_CLZ),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.leading_zeros() as i32;

        trace!("Instruction: i32.clz [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_ctz,
    fuel_check = flat(instructions::I32_CTZ),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.trailing_zeros() as i32;

        trace!("Instruction: i32.ctz [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_popcnt,
    fuel_check = flat(instructions::I32_POPCNT),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.count_ones() as i32;

        trace!("Instruction: i32.popcnt [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i64.unop
define_instruction_fn! {
    i64_clz,
    fuel_check = flat(instructions::I64_CLZ),
    |args: Args| {
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.leading_zeros() as i64;

        trace!("Instruction: i64.clz [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_ctz,
    fuel_check = flat(instructions::I64_CTZ),
    |args: Args| {
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.trailing_zeros() as i64;

        trace!("Instruction: i64.ctz [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_popcnt,
    fuel_check = flat(instructions::I64_POPCNT),
    |args: Args| {
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.count_ones() as i64;

        trace!("Instruction: i64.popcnt [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f32.unop
define_instruction_fn! {
    f32_abs,
    fuel_check = flat(instructions::F32_ABS),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.abs();

        trace!("Instruction: f32.abs [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_neg,
    fuel_check = flat(instructions::F32_NEG),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.neg();

        trace!("Instruction: f32.neg [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_ceil,
    fuel_check = flat(instructions::F32_CEIL),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.ceil();

        trace!("Instruction: f32.ceil [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_floor,
    fuel_check = flat(instructions::F32_FLOOR),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.floor();

        trace!("Instruction: f32.floor [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_trunc,
    fuel_check = flat(instructions::F32_TRUNC),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.trunc();

        trace!("Instruction: f32.trunc [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_nearest,
    fuel_check = flat(instructions::F32_NEAREST),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.nearest();

        trace!("Instruction: f32.nearest [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_sqrt,
    fuel_check = flat(instructions::F32_SQRT),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.sqrt();

        trace!("Instruction: f32.sqrt [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f64.unop
define_instruction_fn! {
    f64_abs,
    fuel_check = flat(instructions::F64_ABS),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.abs();

        trace!("Instruction: f64.abs [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_neg,
    fuel_check = flat(instructions::F64_NEG),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.neg();

        trace!("Instruction: f64.neg [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_ceil,
    fuel_check = flat(instructions::F64_CEIL),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.ceil();

        trace!("Instruction: f64.ceil [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_floor,
    fuel_check = flat(instructions::F64_FLOOR),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.floor();

        trace!("Instruction: f64.floor [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_trunc,
    fuel_check = flat(instructions::F64_TRUNC),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.trunc();

        trace!("Instruction: f64.trunc [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_nearest,
    fuel_check = flat(instructions::F64_NEAREST),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.nearest();

        trace!("Instruction: f64.nearest [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_sqrt,
    fuel_check = flat(instructions::F64_SQRT),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.sqrt();

        trace!("Instruction: f64.sqrt [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i32.binop
define_instruction_fn! {
    i32_add,
    fuel_check = flat(instructions::I32_ADD),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.wrapping_add(v2);

        trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_sub,
    fuel_check = flat(instructions::I32_SUB),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.wrapping_sub(v2);

        trace!("Instruction: i32.sub [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_mul,
    fuel_check = flat(instructions::I32_MUL),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.wrapping_mul(v2);

        trace!("Instruction: i32.mul [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_div_s,
    fuel_check = flat(instructions::I32_DIV_S),
    |args: Args| {
        let dividend: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }
        if divisor == i32::MIN && dividend == -1 {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res = divisor / dividend;

        trace!("Instruction: i32.div_s [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_div_u,
    fuel_check = flat(instructions::I32_DIV_U),
    |args: Args| {
        let dividend: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let dividend = dividend as u32;
        let divisor = divisor as u32;

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }

        let res = (divisor / dividend) as i32;

        trace!("Instruction: i32.div_u [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_rem_s,
    fuel_check = flat(instructions::I32_REM_S),
    |args: Args| {
        let dividend: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }

        let res = divisor.checked_rem(dividend);
        let res = res.unwrap_or_default();

        trace!("Instruction: i32.rem_s [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_rem_u,
    fuel_check = flat(instructions::I32_REM_U),
    |args: Args| {
        let dividend: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let dividend = dividend as u32;
        let divisor = divisor as u32;

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }

        let res = divisor.checked_rem(dividend);
        let res = res.unwrap_or_default() as i32;

        trace!("Instruction: i32.rem_u [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_and,
    fuel_check = flat(instructions::I32_AND),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1 & v2;

        trace!("Instruction: i32.and [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_or,
    fuel_check = flat(instructions::I32_OR),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1 | v2;

        trace!("Instruction: i32.or [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_xor,
    fuel_check = flat(instructions::I32_XOR),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1 ^ v2;

        trace!("Instruction: i32.xor [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_shl,
    fuel_check = flat(instructions::I32_SHL),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v2.wrapping_shl(v1 as u32);

        trace!("Instruction: i32.shl [{v2} {v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_shr_s,
    fuel_check = flat(instructions::I32_SHR_S),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v2.wrapping_shr(v1 as u32);

        trace!("Instruction: i32.shr_s [{v2} {v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_shr_u,
    fuel_check = flat(instructions::I32_SHR_U),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = (v2 as u32).wrapping_shr(v1 as u32) as i32;

        trace!("Instruction: i32.shr_u [{v2} {v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_rotl,
    fuel_check = flat(instructions::I32_ROTL),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v2.rotate_left(v1 as u32);

        trace!("Instruction: i32.rotl [{v2} {v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_rotr,
    fuel_check = flat(instructions::I32_ROTR),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v2.rotate_right(v1 as u32);

        trace!("Instruction: i32.rotr [{v2} {v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i64.binop
define_instruction_fn! {
    i64_add,
    fuel_check = flat(instructions::I64_ADD),
    |args: Args| {
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.wrapping_add(v2);

        trace!("Instruction: i64.add [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_sub,
    fuel_check = flat(instructions::I64_SUB),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.wrapping_sub(v2);

        trace!("Instruction: i64.sub [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_mul,
    fuel_check = flat(instructions::I64_MUL),
    |args: Args| {
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = v1.wrapping_mul(v2);

        trace!("Instruction: i64.mul [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_div_s,
    fuel_check = flat(instructions::I64_DIV_S),
    |args: Args| {
        let dividend: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }
        if divisor == i64::MIN && dividend == -1 {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res = divisor / dividend;

        trace!("Instruction: i64.div_s [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_div_u,
    fuel_check = flat(instructions::I64_DIV_U),
    |args: Args| {
        let dividend: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let dividend = dividend as u64;
        let divisor = divisor as u64;

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }

        let res = (divisor / dividend) as i64;

        trace!("Instruction: i64.div_u [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_rem_s,
    fuel_check = flat(instructions::I64_REM_S),
    |args: Args| {
        let dividend: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }

        let res = divisor.checked_rem(dividend);
        let res = res.unwrap_or_default();

        trace!("Instruction: i64.rem_s [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_rem_u,
    fuel_check = flat(instructions::I64_REM_U),
    |args: Args| {
        let dividend: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let divisor: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let dividend = dividend as u64;
        let divisor = divisor as u64;

        if dividend == 0 {
            return Err(TrapError::DivideBy0.into());
        }

        let res = (divisor % dividend) as i64;

        trace!("Instruction: i64.rem_u [{divisor} {dividend}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_and,
    fuel_check = flat(instructions::I64_AND),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v1 & v2;

        trace!("Instruction: i64.and [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_or,
    fuel_check = flat(instructions::I64_OR),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v1 | v2;

        trace!("Instruction: i64.or [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_xor,
    fuel_check = flat(instructions::I64_XOR),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v1 ^ v2;

        trace!("Instruction: i64.xor [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_shl,
    fuel_check = flat(instructions::I64_SHL),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v1.wrapping_shl((v2 & 63) as u32);

        trace!("Instruction: i64.shl [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_shr_s,
    fuel_check = flat(instructions::I64_SHR_S),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v1.wrapping_shr((v2 & 63) as u32);

        trace!("Instruction: i64.shr_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_shr_u,
    fuel_check = flat(instructions::I64_SHR_U),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = (v1 as u64).wrapping_shr((v2 & 63) as u32);

        trace!("Instruction: i64.shr_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_rotl,
    fuel_check = flat(instructions::I64_ROTL),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v1.rotate_left((v2 & 63) as u32);

        trace!("Instruction: i64.rotl [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_rotr,
    fuel_check = flat(instructions::I64_ROTR),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = v1.rotate_right((v2 & 63) as u32);

        trace!("Instruction: i64.rotr [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f32.binop
define_instruction_fn! {
    f32_add,
    fuel_check = flat(instructions::F32_ADD),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1 + v2;

        trace!("Instruction: f32.add [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_sub,
    fuel_check = flat(instructions::F32_SUB),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1 - v2;

        trace!("Instruction: f32.sub [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_mul,
    fuel_check = flat(instructions::F32_MUL),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1 * v2;

        trace!("Instruction: f32.mul [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_div,
    fuel_check = flat(instructions::F32_DIV),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1 / v2;

        trace!("Instruction: f32.div [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_min,
    fuel_check = flat(instructions::F32_MIN),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.min(v2);

        trace!("Instruction: f32.min [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_max,
    fuel_check = flat(instructions::F32_MAX),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.max(v2);

        trace!("Instruction: f32.max [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_copysign,
    fuel_check = flat(instructions::F32_COPYSIGN),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v1.copysign(v2);

        trace!("Instruction: f32.copysign [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f64.binop
define_instruction_fn! {
    f64_add,
    fuel_check = flat(instructions::F64_ADD),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1 + v2;

        trace!("Instruction: f64.add [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_sub,
    fuel_check = flat(instructions::F64_SUB),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1 - v2;

        trace!("Instruction: f64.sub [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_mul,
    fuel_check = flat(instructions::F64_MUL),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1 * v2;

        trace!("Instruction: f64.mul [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_div,
    fuel_check = flat(instructions::F64_DIV),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1 / v2;

        trace!("Instruction: f64.div [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_min,
    fuel_check = flat(instructions::F64_MIN),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.min(v2);

        trace!("Instruction: f64.min [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_max,
    fuel_check = flat(instructions::F64_MAX),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.max(v2);

        trace!("Instruction: f64.max [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_copysign,
    fuel_check = flat(instructions::F64_COPYSIGN),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v1.copysign(v2);

        trace!("Instruction: f64.copysign [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i32.testop
define_instruction_fn! {
    i32_eqz,
    fuel_check = flat(instructions::I32_EQZ),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 == 0 { 1 } else { 0 };

        trace!("Instruction: i32.eqz [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i64.testop
define_instruction_fn! {
    i64_eqz,
    fuel_check = flat(instructions::I64_EQZ),
    |args: Args| {
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 == 0 { 1 } else { 0 };

        trace!("Instruction: i64.eqz [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i32.relop
define_instruction_fn! {
    i32_eq,
    fuel_check = flat(instructions::I32_EQ),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 == v2 { 1 } else { 0 };

        trace!("Instruction: i32.eq [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_ne,
    fuel_check = flat(instructions::I32_NE),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 != v2 { 1 } else { 0 };

        trace!("Instruction: i32.ne [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_lt_s,
    fuel_check = flat(instructions::I32_LT_S),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 < v2 { 1 } else { 0 };

        trace!("Instruction: i32.lt_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_lt_u,
    fuel_check = flat(instructions::I32_LT_U),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u32) < (v2 as u32) { 1 } else { 0 };

        trace!("Instruction: i32.lt_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_gt_s,
    fuel_check = flat(instructions::I32_GT_S),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 > v2 { 1 } else { 0 };

        trace!("Instruction: i32.gt_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_gt_u,
    fuel_check = flat(instructions::I32_GT_U),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u32) > (v2 as u32) { 1 } else { 0 };

        trace!("Instruction: i32.gt_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_le_s,
    fuel_check = flat(instructions::I32_LE_S),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 <= v2 { 1 } else { 0 };

        trace!("Instruction: i32.le_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_le_u,
    fuel_check = flat(instructions::I32_LE_U),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u32) <= (v2 as u32) { 1 } else { 0 };

        trace!("Instruction: i32.le_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_ge_s,
    fuel_check = flat(instructions::I32_GE_S),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 >= v2 { 1 } else { 0 };

        trace!("Instruction: i32.ge_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_ge_u,
    fuel_check = flat(instructions::I32_GE_U),
    |args: Args| {
        let v2: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u32) >= (v2 as u32) { 1 } else { 0 };

        trace!("Instruction: i32.ge_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i64.relop
define_instruction_fn! {
    i64_eq,
    fuel_check = flat(instructions::I64_EQ),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 == v2 { 1 } else { 0 };

        trace!("Instruction: i64.eq [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_ne,
    fuel_check = flat(instructions::I64_NE),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 != v2 { 1 } else { 0 };

        trace!("Instruction: i64.ne [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_lt_s,
    fuel_check = flat(instructions::I64_LT_S),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 < v2 { 1 } else { 0 };

        trace!("Instruction: i64.lt_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_lt_u,
    fuel_check = flat(instructions::I64_LT_U),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u64) < (v2 as u64) { 1 } else { 0 };

        trace!("Instruction: i64.lt_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_gt_s,
    fuel_check = flat(instructions::I64_GT_S),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 > v2 { 1 } else { 0 };

        trace!("Instruction: i64.gt_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_gt_u,
    fuel_check = flat(instructions::I64_GT_U),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u64) > (v2 as u64) { 1 } else { 0 };

        trace!("Instruction: i64.gt_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_le_s,
    fuel_check = flat(instructions::I64_LE_S),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 <= v2 { 1 } else { 0 };

        trace!("Instruction: i64.le_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_le_u,
    fuel_check = flat(instructions::I64_LE_U),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u64) <= (v2 as u64) { 1 } else { 0 };

        trace!("Instruction: i64.le_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_ge_s,
    fuel_check = flat(instructions::I64_GE_S),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 >= v2 { 1 } else { 0 };

        trace!("Instruction: i64.ge_s [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_ge_u,
    fuel_check = flat(instructions::I64_GE_U),
    |args: Args| {
        let v2: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if (v1 as u64) >= (v2 as u64) { 1 } else { 0 };

        trace!("Instruction: i64.ge_u [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f32.relop
define_instruction_fn! {
    f32_eq,
    fuel_check = flat(instructions::F32_EQ),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 == v2 { 1 } else { 0 };

        trace!("Instruction: f32.eq [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_ne,
    fuel_check = flat(instructions::F32_NE),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 != v2 { 1 } else { 0 };

        trace!("Instruction: f32.ne [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_lt,
    fuel_check = flat(instructions::F32_LT),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 < v2 { 1 } else { 0 };

        trace!("Instruction: f32.lt [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_gt,
    fuel_check = flat(instructions::F32_GT),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 > v2 { 1 } else { 0 };

        trace!("Instruction: f32.gt [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_le,
    fuel_check = flat(instructions::F32_LE),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 <= v2 { 1 } else { 0 };

        trace!("Instruction: f32.le [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_ge,
    fuel_check = flat(instructions::F32_GE),
    |args: Args| {
        let v2: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 >= v2 { 1 } else { 0 };

        trace!("Instruction: f32.ge [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f64.relop
define_instruction_fn! {
    f64_eq,
    fuel_check = flat(instructions::F64_EQ),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 == v2 { 1 } else { 0 };

        trace!("Instruction: f64.eq [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_ne,
    fuel_check = flat(instructions::F64_NE),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 != v2 { 1 } else { 0 };

        trace!("Instruction: f64.ne [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_lt,
    fuel_check = flat(instructions::F64_LT),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 < v2 { 1 } else { 0 };

        trace!("Instruction: f64.lt [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_gt,
    fuel_check = flat(instructions::F64_GT),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 > v2 { 1 } else { 0 };

        trace!("Instruction: f64.gt [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_le,
    fuel_check = flat(instructions::F64_LE),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 <= v2 { 1 } else { 0 };

        trace!("Instruction: f64.le [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_ge,
    fuel_check = flat(instructions::F64_GE),
    |args: Args| {
        let v2: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res = if v1 >= v2 { 1 } else { 0 };

        trace!("Instruction: f64.ge [{v1} {v2}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i32.cvtop
define_instruction_fn! {
    i32_wrap_i64,
    fuel_check = flat(instructions::I32_WRAP_I64),
    |args: Args| {
        let v: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: i32 = v as i32;

        trace!("Instruction: i32.wrap_i64 [{v}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_f32_s,
    fuel_check = flat(instructions::I32_TRUNC_F32_S),
    |args: Args| {
        let v: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F32(2147483648.0) || v <= F32(-2147483904.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_i32();

        trace!("Instruction: i32.trunc_f32_s [{v:.7}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_f32_u,
    fuel_check = flat(instructions::I32_TRUNC_F32_U),
    |args: Args| {
        let v: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F32(4294967296.0) || v <= F32(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_u32() as i32;

        trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_f64_s,
    fuel_check = flat(instructions::I32_TRUNC_F64_S),
    |args: Args| {
        let v: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F64(2147483648.0) || v <= F64(-2147483649.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_i32();

        trace!("Instruction: i32.trunc_f64_s [{v:.7}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_f64_u,
    fuel_check = flat(instructions::I32_TRUNC_F64_U),
    |args: Args| {
        let v: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F64(4294967296.0) || v <= F64(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i32 = v.as_u32() as i32;

        trace!("Instruction: i32.trunc_f32_u [{v:.7}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_reinterpret_f32,
    fuel_check = flat(instructions::I32_REINTERPRET_F32),
    |args: Args| {
        let v: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: i32 = v.reinterpret_as_i32();

        trace!("Instruction: i32.reinterpret_f32 [{v:.7}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_extend8_s,
    fuel_check = flat(instructions::I32_EXTEND8_S),
    |args: Args| {
        let mut v: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if v | 0xFF != 0xFF {
            trace!("Number v ({}) not contained in 8 bits, truncating", v);
            v &= 0xFF;
        }

        let res = if v | 0x7F != 0x7F { v | 0xFFFFFF00 } else { v };

        args.resumable.stack.push_value(res.into())?;

        trace!("Instruction i32.extend8_s [{}] -> [{}]", v, res);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_extend16_s,
    fuel_check = flat(instructions::I32_EXTEND16_S),
    |args: Args| {
        let mut v: u32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if v | 0xFFFF != 0xFFFF {
            trace!("Number v ({}) not contained in 16 bits, truncating", v);
            v &= 0xFFFF;
        }

        let res = if v | 0x7FFF != 0x7FFF {
            v | 0xFFFF0000
        } else {
            v
        };

        args.resumable.stack.push_value(res.into())?;

        trace!("Instruction i32.extend16_s [{}] -> [{}]", v, res);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_sat_f32_s,
    fuel_check = flat_fc(instructions::fc_extensions::I32_TRUNC_SAT_F32_S),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i32::MIN
            } else if v1.is_infinity() {
                i32::MAX
            } else {
                v1.as_i32()
            }
        };

        trace!("Instruction: i32.trunc_sat_f32_s [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_sat_f32_u,
    fuel_check = flat_fc(instructions::fc_extensions::I32_TRUNC_SAT_F32_U),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u32::MAX as i32
            } else {
                v1.as_u32() as i32
            }
        };

        trace!("Instruction: i32.trunc_sat_f32_u [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_sat_f64_s,
    fuel_check = flat_fc(instructions::fc_extensions::I32_TRUNC_SAT_F64_S),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i32::MIN
            } else if v1.is_infinity() {
                i32::MAX
            } else {
                v1.as_i32()
            }
        };

        trace!("Instruction: i32.trunc_sat_f64_s [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i32_trunc_sat_f64_u,
    fuel_check = flat_fc(instructions::fc_extensions::I32_TRUNC_SAT_F64_U),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u32::MAX as i32
            } else {
                v1.as_u32() as i32
            }
        };

        trace!("Instruction: i32.trunc_sat_f64_u [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// i64.cvtop
define_instruction_fn! {
    i64_extend_i32_s,
    fuel_check = flat(instructions::I64_EXTEND_I32_S),
    |args: Args| {
        let v: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res: i64 = v as i64;

        trace!("Instruction: i64.extend_i32_s [{v}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_extend_i32_u,
    fuel_check = flat(instructions::I64_EXTEND_I32_U),
    |args: Args| {
        let v: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        let res: i64 = v as u32 as i64;

        trace!("Instruction: i64.extend_i32_u [{v}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_f32_s,
    fuel_check = flat(instructions::I64_TRUNC_F32_S),
    |args: Args| {
        let v: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F32(9223372036854775808.0) || v <= F32(-9223373136366403584.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_i64();

        trace!("Instruction: i64.trunc_f32_s [{v:.7}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_f32_u,
    fuel_check = flat(instructions::I64_TRUNC_F32_U),
    |args: Args| {
        let v: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F32(18446744073709551616.0) || v <= F32(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_u64() as i64;

        trace!("Instruction: i64.trunc_f32_u [{v:.7}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_f64_s,
    fuel_check = flat(instructions::I64_TRUNC_F64_S),
    |args: Args| {
        let v: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F64(9223372036854775808.0) || v <= F64(-9223372036854777856.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_i64();

        trace!("Instruction: i64.trunc_f64_s [{v:.17}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_f64_u,
    fuel_check = flat(instructions::I64_TRUNC_F64_U),
    |args: Args| {
        let v: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        if v.is_infinity() {
            return Err(TrapError::UnrepresentableResult.into());
        }
        if v.is_nan() {
            return Err(TrapError::BadConversionToInteger.into());
        }
        if v >= F64(18446744073709551616.0) || v <= F64(-1.0) {
            return Err(TrapError::UnrepresentableResult.into());
        }

        let res: i64 = v.as_u64() as i64;

        trace!("Instruction: i64.trunc_f64_u [{v:.17}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_reinterpret_f64,
    fuel_check = flat(instructions::I64_REINTERPRET_F64),
    |args: Args| {
        let v: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: i64 = v.reinterpret_as_i64();

        trace!("Instruction: i64.reinterpret_f64 [{v:.17}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_extend8_s,
    fuel_check = flat(instructions::I64_EXTEND8_S),
    |args: Args| {
        let mut v: u64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if v | 0xFF != 0xFF {
            trace!("Number v ({}) not contained in 8 bits, truncating", v);
            v &= 0xFF;
        }

        let res = if v | 0x7F != 0x7F {
            v | 0xFFFFFFFF_FFFFFF00
        } else {
            v
        };

        args.resumable.stack.push_value(res.into())?;

        trace!("Instruction i64.extend8_s [{}] -> [{}]", v, res);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_extend16_s,
    fuel_check = flat(instructions::I64_EXTEND16_S),
    |args: Args| {
        let mut v: u64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if v | 0xFFFF != 0xFFFF {
            trace!("Number v ({}) not contained in 16 bits, truncating", v);
            v &= 0xFFFF;
        }

        let res = if v | 0x7FFF != 0x7FFF {
            v | 0xFFFFFFFF_FFFF0000
        } else {
            v
        };

        args.resumable.stack.push_value(res.into())?;

        trace!("Instruction i64.extend16_s [{}] -> [{}]", v, res);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_extend32_s,
    fuel_check = flat(instructions::I64_EXTEND32_S),
    |args: Args| {
        let mut v: u64 = args.resumable.stack.pop_value().try_into().unwrap_validated();

        if v | 0xFFFF_FFFF != 0xFFFF_FFFF {
            trace!("Number v ({}) not contained in 32 bits, truncating", v);
            v &= 0xFFFF_FFFF;
        }

        let res = if v | 0x7FFF_FFFF != 0x7FFF_FFFF {
            v | 0xFFFFFFFF_00000000
        } else {
            v
        };

        args.resumable.stack.push_value(res.into())?;

        trace!("Instruction i64.extend32_s [{}] -> [{}]", v, res);
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_sat_f32_s,
    fuel_check = flat_fc(instructions::fc_extensions::I64_TRUNC_SAT_F32_S),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i64::MIN
            } else if v1.is_infinity() {
                i64::MAX
            } else {
                v1.as_i64()
            }
        };

        trace!("Instruction: i64.trunc_sat_f32_s [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_sat_f32_u,
    fuel_check = flat_fc(instructions::fc_extensions::I64_TRUNC_SAT_F32_U),
    |args: Args| {
        let v1: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u64::MAX as i64
            } else {
                v1.as_u64() as i64
            }
        };

        trace!("Instruction: i64.trunc_sat_f32_u [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_sat_f64_s,
    fuel_check = flat_fc(instructions::fc_extensions::I64_TRUNC_SAT_F64_S),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() {
                0
            } else if v1.is_negative_infinity() {
                i64::MIN
            } else if v1.is_infinity() {
                i64::MAX
            } else {
                v1.as_i64()
            }
        };

        trace!("Instruction: i64.trunc_sat_f64_s [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    i64_trunc_sat_f64_u,
    fuel_check = flat_fc(instructions::fc_extensions::I64_TRUNC_SAT_F64_U),
    |args: Args| {
        let v1: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res = {
            if v1.is_nan() || v1.is_negative_infinity() {
                0
            } else if v1.is_infinity() {
                u64::MAX as i64
            } else {
                v1.as_u64() as i64
            }
        };

        trace!("Instruction: i64.trunc_sat_f64_u [{v1}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f32.cvtop
define_instruction_fn! {
    f32_convert_i32_s,
    fuel_check = flat(instructions::F32_CONVERT_I32_S),
    |args: Args| {
        let v: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = F32(v as f32);

        trace!("Instruction: f32.convert_i32_s [{v}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_convert_i32_u,
    fuel_check = flat(instructions::F32_CONVERT_I32_U),
    |args: Args| {
        let v: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = F32(v as u32 as f32);

        trace!("Instruction: f32.convert_i32_u [{v}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_convert_i64_s,
    fuel_check = flat(instructions::F32_CONVERT_I64_S),
    |args: Args| {
        let v: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = F32(v as f32);

        trace!("Instruction: f32.convert_i64_s [{v}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_convert_i64_u,
    fuel_check = flat(instructions::F32_CONVERT_I64_U),
    |args: Args| {
        let v: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = F32(v as u64 as f32);

        trace!("Instruction: f32.convert_i64_u [{v}] -> [{res}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_demote_f64,
    fuel_check = flat(instructions::F32_DEMOTE_F64),
    |args: Args| {
        let v: F64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = v.as_f32();

        trace!("Instruction: f32.demote_f64 [{v:.17}] -> [{res:.7}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f32_reinterpret_i32,
    fuel_check = flat(instructions::F32_REINTERPRET_I32),
    |args: Args| {
        let v1: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F32 = F32::from_bits(v1 as u32);

        trace!("Instruction: f32.reinterpret_i32 [{v1}] -> [{res:.7}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

// f64.cvtop
define_instruction_fn! {
    f64_convert_i32_s,
    fuel_check = flat(instructions::F64_CONVERT_I32_S),
    |args: Args| {
        let v: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = F64(v as f64);

        trace!("Instruction: f64.convert_i32_s [{v}] -> [{res:.17}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_convert_i32_u,
    fuel_check = flat(instructions::F64_CONVERT_I32_U),
    |args: Args| {
        let v: i32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = F64(v as u32 as f64);

        trace!("Instruction: f64.convert_i32_u [{v}] -> [{res:.17}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_convert_i64_s,
    fuel_check = flat(instructions::F64_CONVERT_I64_S),
    |args: Args| {
        let v: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = F64(v as f64);

        trace!("Instruction: f64.convert_i64_s [{v}] -> [{res:.17}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_convert_i64_u,
    fuel_check = flat(instructions::F64_CONVERT_I64_U),
    |args: Args| {
        let v: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = F64(v as u64 as f64);

        trace!("Instruction: f64.convert_i64_u [{v}] -> [{res:.17}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_promote_f32,
    fuel_check = flat(instructions::F64_PROMOTE_F32),
    |args: Args| {
        let v: F32 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = v.as_f64();

        trace!("Instruction: f64.promote_f32 [{v:.7}] -> [{res:.17}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}

define_instruction_fn! {
    f64_reinterpret_i64,
    fuel_check = flat(instructions::F64_REINTERPRET_I64),
    |args: Args| {
        let v1: i64 = args.resumable.stack.pop_value().try_into().unwrap_validated();
        let res: F64 = F64::from_bits(v1 as u64);

        trace!("Instruction: f64.reinterpret_i64 [{v1}] -> [{res:.17}]");
        args.resumable.stack.push_value(res.into())?;
        Ok(ControlFlow::Continue(()))
    }
}
