#![expect(
    clippy::missing_safety_doc,
    reason = "see `instructions::State` for more information"
)]

use core::ops::ControlFlow;

use crate::{
    execution::{
        assert_validated::UnwrapValidatedExt,
        instructions::{define_instruction, InterpreterLoopOutcome, State},
    },
    trace, RuntimeError, TrapError, F32, F64,
};

// t.const
define_instruction!(
    super::i32_const,
    i32_const_mod,
    fuel_check = flat(I32_CONST)
);
#[inline(always)]
pub unsafe fn i32_const(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let constant = state.wasm.decode_var_i32().unwrap_validated();
    trace!("Instruction: i32.const [] -> [{constant}]");
    state.resumable.stack.push_value(constant.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_const,
    i64_const_mod,
    fuel_check = flat(I64_CONST)
);
#[inline(always)]
pub unsafe fn i64_const(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let constant = state.wasm.decode_var_i64().unwrap_validated();
    trace!("Instruction: i64.const [] -> [{constant}]");
    state.resumable.stack.push_value(constant.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_const,
    f32_const_mod,
    fuel_check = flat(F32_CONST)
);
#[inline(always)]
pub unsafe fn f32_const(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let constant = F32::from_bits(state.wasm.decode_f32().unwrap_validated());
    trace!("Instruction: f32.const [] -> [{constant:.7}]");
    state.resumable.stack.push_value(constant.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_const,
    f64_const_mod,
    fuel_check = flat(F64_CONST)
);
#[inline(always)]
pub unsafe fn f64_const(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let constant = F64::from_bits(state.wasm.decode_f64().unwrap_validated());
    trace!("Instruction: f64.const [] -> [{constant}]");
    state.resumable.stack.push_value(constant.into())?;
    Ok(ControlFlow::Continue(()))
}

// i32.unop
define_instruction!(super::i32_clz, i32_clz_mod, fuel_check = flat(I32_CLZ));
#[inline(always)]
pub unsafe fn i32_clz(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.leading_zeros() as i32;

    trace!("Instruction: i32.clz [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_ctz, i32_ctz_mod, fuel_check = flat(I32_CTZ));
#[inline(always)]
pub unsafe fn i32_ctz(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.trailing_zeros() as i32;

    trace!("Instruction: i32.ctz [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_popcnt,
    i32_popcnt_mod,
    fuel_check = flat(I32_POPCNT)
);
#[inline(always)]
pub unsafe fn i32_popcnt(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.count_ones() as i32;

    trace!("Instruction: i32.popcnt [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i64.unop
define_instruction!(super::i64_clz, i64_clz_mod, fuel_check = flat(I64_CLZ));
#[inline(always)]
pub unsafe fn i64_clz(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.leading_zeros() as i64;

    trace!("Instruction: i64.clz [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_ctz, i64_ctz_mod, fuel_check = flat(I64_CTZ));
#[inline(always)]
pub unsafe fn i64_ctz(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.trailing_zeros() as i64;

    trace!("Instruction: i64.ctz [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_popcnt,
    i64_popcnt_mod,
    fuel_check = flat(I64_POPCNT)
);
#[inline(always)]
pub unsafe fn i64_popcnt(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.count_ones() as i64;

    trace!("Instruction: i64.popcnt [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f32.unop
define_instruction!(super::f32_abs, f32_abs_mod, fuel_check = flat(F32_ABS));
#[inline(always)]
pub unsafe fn f32_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.abs();

    trace!("Instruction: f32.abs [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_neg, f32_neg_mod, fuel_check = flat(F32_NEG));
#[inline(always)]
pub unsafe fn f32_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.neg();

    trace!("Instruction: f32.neg [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_ceil, f32_ceil_mod, fuel_check = flat(F32_CEIL));
#[inline(always)]
pub unsafe fn f32_ceil(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.ceil();

    trace!("Instruction: f32.ceil [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_floor,
    f32_floor_mod,
    fuel_check = flat(F32_FLOOR)
);
#[inline(always)]
pub unsafe fn f32_floor(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.floor();

    trace!("Instruction: f32.floor [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_trunc,
    f32_trunc_mod,
    fuel_check = flat(F32_TRUNC)
);
#[inline(always)]
pub unsafe fn f32_trunc(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.trunc();

    trace!("Instruction: f32.trunc [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_nearest,
    f32_nearest_mod,
    fuel_check = flat(F32_NEAREST)
);
#[inline(always)]
pub unsafe fn f32_nearest(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.nearest();

    trace!("Instruction: f32.nearest [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_sqrt, f32_sqrt_mod, fuel_check = flat(F32_SQRT));
#[inline(always)]
pub unsafe fn f32_sqrt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.sqrt();

    trace!("Instruction: f32.sqrt [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f64.unop
define_instruction!(super::f64_abs, f64_abs_mod, fuel_check = flat(F64_ABS));
#[inline(always)]
pub unsafe fn f64_abs(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.abs();

    trace!("Instruction: f64.abs [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_neg, f64_neg_mod, fuel_check = flat(F64_NEG));
#[inline(always)]
pub unsafe fn f64_neg(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.neg();

    trace!("Instruction: f64.neg [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_ceil, f64_ceil_mod, fuel_check = flat(F64_CEIL));
#[inline(always)]
pub unsafe fn f64_ceil(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.ceil();

    trace!("Instruction: f64.ceil [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_floor,
    f64_floor_mod,
    fuel_check = flat(F64_FLOOR)
);
#[inline(always)]
pub unsafe fn f64_floor(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.floor();

    trace!("Instruction: f64.floor [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_trunc,
    f64_trunc_mod,
    fuel_check = flat(F64_TRUNC)
);
#[inline(always)]
pub unsafe fn f64_trunc(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.trunc();

    trace!("Instruction: f64.trunc [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_nearest,
    f64_nearest_mod,
    fuel_check = flat(F64_NEAREST)
);
#[inline(always)]
pub unsafe fn f64_nearest(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.nearest();

    trace!("Instruction: f64.nearest [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_sqrt, f64_sqrt_mod, fuel_check = flat(F64_SQRT));
#[inline(always)]
pub unsafe fn f64_sqrt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.sqrt();

    trace!("Instruction: f64.sqrt [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i32.binop
define_instruction!(super::i32_add, i32_add_mod, fuel_check = flat(I32_ADD));
#[inline(always)]
pub unsafe fn i32_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.wrapping_add(v2);

    trace!("Instruction: i32.add [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_sub, i32_sub_mod, fuel_check = flat(I32_SUB));
#[inline(always)]
pub unsafe fn i32_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.wrapping_sub(v2);

    trace!("Instruction: i32.sub [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_mul, i32_mul_mod, fuel_check = flat(I32_MUL));
#[inline(always)]
pub unsafe fn i32_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.wrapping_mul(v2);

    trace!("Instruction: i32.mul [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_div_s,
    i32_div_s_mod,
    fuel_check = flat(I32_DIV_S)
);
#[inline(always)]
pub unsafe fn i32_div_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }
    if divisor == i32::MIN && dividend == -1 {
        return Err(TrapError::UnrepresentableResult.into());
    }

    let res = divisor / dividend;

    trace!("Instruction: i32.div_s [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_div_u,
    i32_div_u_mod,
    fuel_check = flat(I32_DIV_U)
);
#[inline(always)]
pub unsafe fn i32_div_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let dividend = dividend as u32;
    let divisor = divisor as u32;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = (divisor / dividend) as i32;

    trace!("Instruction: i32.div_u [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_rem_s,
    i32_rem_s_mod,
    fuel_check = flat(I32_REM_S)
);
#[inline(always)]
pub unsafe fn i32_rem_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = divisor.checked_rem(dividend);
    let res = res.unwrap_or_default();

    trace!("Instruction: i32.rem_s [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_rem_u,
    i32_rem_u_mod,
    fuel_check = flat(I32_REM_U)
);
#[inline(always)]
pub unsafe fn i32_rem_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let dividend = dividend as u32;
    let divisor = divisor as u32;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = divisor.checked_rem(dividend);
    let res = res.unwrap_or_default() as i32;

    trace!("Instruction: i32.rem_u [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_and, i32_and_mod, fuel_check = flat(I32_AND));
#[inline(always)]
pub unsafe fn i32_and(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1 & v2;

    trace!("Instruction: i32.and [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_or, i32_or_mod, fuel_check = flat(I32_OR));
#[inline(always)]
pub unsafe fn i32_or(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1 | v2;

    trace!("Instruction: i32.or [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_xor, i32_xor_mod, fuel_check = flat(I32_XOR));
#[inline(always)]
pub unsafe fn i32_xor(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1 ^ v2;

    trace!("Instruction: i32.xor [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_shl, i32_shl_mod, fuel_check = flat(I32_SHL));
#[inline(always)]
pub unsafe fn i32_shl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v2.wrapping_shl(v1 as u32);

    trace!("Instruction: i32.shl [{v2} {v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_shr_s,
    i32_shr_s_mod,
    fuel_check = flat(I32_SHR_S)
);
#[inline(always)]
pub unsafe fn i32_shr_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v2.wrapping_shr(v1 as u32);

    trace!("Instruction: i32.shr_s [{v2} {v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_shr_u,
    i32_shr_u_mod,
    fuel_check = flat(I32_SHR_U)
);
#[inline(always)]
pub unsafe fn i32_shr_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = (v2 as u32).wrapping_shr(v1 as u32) as i32;

    trace!("Instruction: i32.shr_u [{v2} {v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_rotl, i32_rotl_mod, fuel_check = flat(I32_ROTL));
#[inline(always)]
pub unsafe fn i32_rotl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v2.rotate_left(v1 as u32);

    trace!("Instruction: i32.rotl [{v2} {v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_rotr, i32_rotr_mod, fuel_check = flat(I32_ROTR));
#[inline(always)]
pub unsafe fn i32_rotr(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v2.rotate_right(v1 as u32);

    trace!("Instruction: i32.rotr [{v2} {v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i64.binop
define_instruction!(super::i64_add, i64_add_mod, fuel_check = flat(I64_ADD));
#[inline(always)]
pub unsafe fn i64_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.wrapping_add(v2);

    trace!("Instruction: i64.add [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_sub, i64_sub_mod, fuel_check = flat(I64_SUB));
#[inline(always)]
pub unsafe fn i64_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.wrapping_sub(v2);

    trace!("Instruction: i64.sub [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_mul, i64_mul_mod, fuel_check = flat(I64_MUL));
#[inline(always)]
pub unsafe fn i64_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res = v1.wrapping_mul(v2);

    trace!("Instruction: i64.mul [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_div_s,
    i64_div_s_mod,
    fuel_check = flat(I64_DIV_S)
);
#[inline(always)]
pub unsafe fn i64_div_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }
    if divisor == i64::MIN && dividend == -1 {
        return Err(TrapError::UnrepresentableResult.into());
    }

    let res = divisor / dividend;

    trace!("Instruction: i64.div_s [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_div_u,
    i64_div_u_mod,
    fuel_check = flat(I64_DIV_U)
);
#[inline(always)]
pub unsafe fn i64_div_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let dividend = dividend as u64;
    let divisor = divisor as u64;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = (divisor / dividend) as i64;

    trace!("Instruction: i64.div_u [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_rem_s,
    i64_rem_s_mod,
    fuel_check = flat(I64_REM_S)
);
#[inline(always)]
pub unsafe fn i64_rem_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = divisor.checked_rem(dividend);
    let res = res.unwrap_or_default();

    trace!("Instruction: i64.rem_s [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_rem_u,
    i64_rem_u_mod,
    fuel_check = flat(I64_REM_U)
);
#[inline(always)]
pub unsafe fn i64_rem_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let dividend: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let divisor: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let dividend = dividend as u64;
    let divisor = divisor as u64;

    if dividend == 0 {
        return Err(TrapError::DivideBy0.into());
    }

    let res = (divisor % dividend) as i64;

    trace!("Instruction: i64.rem_u [{divisor} {dividend}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_and, i64_and_mod, fuel_check = flat(I64_AND));
#[inline(always)]
pub unsafe fn i64_and(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v1 & v2;

    trace!("Instruction: i64.and [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_or, i64_or_mod, fuel_check = flat(I64_OR));
#[inline(always)]
pub unsafe fn i64_or(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v1 | v2;

    trace!("Instruction: i64.or [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_xor, i64_xor_mod, fuel_check = flat(I64_XOR));
#[inline(always)]
pub unsafe fn i64_xor(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v1 ^ v2;

    trace!("Instruction: i64.xor [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_shl, i64_shl_mod, fuel_check = flat(I64_SHL));
#[inline(always)]
pub unsafe fn i64_shl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v1.wrapping_shl((v2 & 63) as u32);

    trace!("Instruction: i64.shl [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_shr_s,
    i64_shr_s_mod,
    fuel_check = flat(I64_SHR_S)
);
#[inline(always)]
pub unsafe fn i64_shr_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v1.wrapping_shr((v2 & 63) as u32);

    trace!("Instruction: i64.shr_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_shr_u,
    i64_shr_u_mod,
    fuel_check = flat(I64_SHR_U)
);
#[inline(always)]
pub unsafe fn i64_shr_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = (v1 as u64).wrapping_shr((v2 & 63) as u32);

    trace!("Instruction: i64.shr_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_rotl, i64_rotl_mod, fuel_check = flat(I64_ROTL));
#[inline(always)]
pub unsafe fn i64_rotl(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v1.rotate_left((v2 & 63) as u32);

    trace!("Instruction: i64.rotl [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_rotr, i64_rotr_mod, fuel_check = flat(I64_ROTR));
#[inline(always)]
pub unsafe fn i64_rotr(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = v1.rotate_right((v2 & 63) as u32);

    trace!("Instruction: i64.rotr [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f32.binop
define_instruction!(super::f32_add, f32_add_mod, fuel_check = flat(F32_ADD));
#[inline(always)]
pub unsafe fn f32_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1 + v2;

    trace!("Instruction: f32.add [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_sub, f32_sub_mod, fuel_check = flat(F32_SUB));
#[inline(always)]
pub unsafe fn f32_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1 - v2;

    trace!("Instruction: f32.sub [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_mul, f32_mul_mod, fuel_check = flat(F32_MUL));
#[inline(always)]
pub unsafe fn f32_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1 * v2;

    trace!("Instruction: f32.mul [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_div, f32_div_mod, fuel_check = flat(F32_DIV));
#[inline(always)]
pub unsafe fn f32_div(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1 / v2;

    trace!("Instruction: f32.div [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_min, f32_min_mod, fuel_check = flat(F32_MIN));
#[inline(always)]
pub unsafe fn f32_min(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.min(v2);

    trace!("Instruction: f32.min [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_max, f32_max_mod, fuel_check = flat(F32_MAX));
#[inline(always)]
pub unsafe fn f32_max(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.max(v2);

    trace!("Instruction: f32.max [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_copysign,
    f32_copysign_mod,
    fuel_check = flat(F32_COPYSIGN)
);
#[inline(always)]
pub unsafe fn f32_copysign(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v1.copysign(v2);

    trace!("Instruction: f32.copysign [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f64.binop
define_instruction!(super::f64_add, f64_add_mod, fuel_check = flat(F64_ADD));
#[inline(always)]
pub unsafe fn f64_add(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1 + v2;

    trace!("Instruction: f64.add [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_sub, f64_sub_mod, fuel_check = flat(F64_SUB));
#[inline(always)]
pub unsafe fn f64_sub(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1 - v2;

    trace!("Instruction: f64.sub [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_mul, f64_mul_mod, fuel_check = flat(F64_MUL));
#[inline(always)]
pub unsafe fn f64_mul(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1 * v2;

    trace!("Instruction: f64.mul [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_div, f64_div_mod, fuel_check = flat(F64_DIV));
#[inline(always)]
pub unsafe fn f64_div(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1 / v2;

    trace!("Instruction: f64.div [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_min, f64_min_mod, fuel_check = flat(F64_MIN));
#[inline(always)]
pub unsafe fn f64_min(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.min(v2);

    trace!("Instruction: f64.min [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_max, f64_max_mod, fuel_check = flat(F64_MAX));
#[inline(always)]
pub unsafe fn f64_max(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.max(v2);

    trace!("Instruction: f64.max [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_copysign,
    f64_copysign_mod,
    fuel_check = flat(F64_COPYSIGN)
);
#[inline(always)]
pub unsafe fn f64_copysign(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v1.copysign(v2);

    trace!("Instruction: f64.copysign [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i32.testop
define_instruction!(super::i32_eqz, i32_eqz_mod, fuel_check = flat(I32_EQZ));
#[inline(always)]
pub unsafe fn i32_eqz(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 == 0 { 1 } else { 0 };

    trace!("Instruction: i32.eqz [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i64.testop
define_instruction!(super::i64_eqz, i64_eqz_mod, fuel_check = flat(I64_EQZ));
#[inline(always)]
pub unsafe fn i64_eqz(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 == 0 { 1 } else { 0 };

    trace!("Instruction: i64.eqz [{v1}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i32.relop
define_instruction!(super::i32_eq, i32_eq_mod, fuel_check = flat(I32_EQ));
#[inline(always)]
pub unsafe fn i32_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: i32.eq [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_ne, i32_ne_mod, fuel_check = flat(I32_NE));
#[inline(always)]
pub unsafe fn i32_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: i32.ne [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_lt_s, i32_lt_s_mod, fuel_check = flat(I32_LT_S));
#[inline(always)]
pub unsafe fn i32_lt_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: i32.lt_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_lt_u, i32_lt_u_mod, fuel_check = flat(I32_LT_U));
#[inline(always)]
pub unsafe fn i32_lt_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u32) < (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.lt_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_gt_s, i32_gt_s_mod, fuel_check = flat(I32_GT_S));
#[inline(always)]
pub unsafe fn i32_gt_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: i32.gt_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_gt_u, i32_gt_u_mod, fuel_check = flat(I32_GT_U));
#[inline(always)]
pub unsafe fn i32_gt_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u32) > (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.gt_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_le_s, i32_le_s_mod, fuel_check = flat(I32_LE_S));
#[inline(always)]
pub unsafe fn i32_le_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: i32.le_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_le_u, i32_le_u_mod, fuel_check = flat(I32_LE_U));
#[inline(always)]
pub unsafe fn i32_le_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u32) <= (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.le_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_ge_s, i32_ge_s_mod, fuel_check = flat(I32_GE_S));
#[inline(always)]
pub unsafe fn i32_ge_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: i32.ge_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i32_ge_u, i32_ge_u_mod, fuel_check = flat(I32_GE_U));
#[inline(always)]
pub unsafe fn i32_ge_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u32) >= (v2 as u32) { 1 } else { 0 };

    trace!("Instruction: i32.ge_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i64.relop
define_instruction!(super::i64_eq, i64_eq_mod, fuel_check = flat(I64_EQ));
#[inline(always)]
pub unsafe fn i64_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: i64.eq [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_ne, i64_ne_mod, fuel_check = flat(I64_NE));
#[inline(always)]
pub unsafe fn i64_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: i64.ne [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_lt_s, i64_lt_s_mod, fuel_check = flat(I64_LT_S));
#[inline(always)]
pub unsafe fn i64_lt_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: i64.lt_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_lt_u, i64_lt_u_mod, fuel_check = flat(I64_LT_U));
#[inline(always)]
pub unsafe fn i64_lt_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u64) < (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.lt_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_gt_s, i64_gt_s_mod, fuel_check = flat(I64_GT_S));
#[inline(always)]
pub unsafe fn i64_gt_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: i64.gt_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_gt_u, i64_gt_u_mod, fuel_check = flat(I64_GT_U));
#[inline(always)]
pub unsafe fn i64_gt_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u64) > (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.gt_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_le_s, i64_le_s_mod, fuel_check = flat(I64_LE_S));
#[inline(always)]
pub unsafe fn i64_le_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: i64.le_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_le_u, i64_le_u_mod, fuel_check = flat(I64_LE_U));
#[inline(always)]
pub unsafe fn i64_le_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u64) <= (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.le_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_ge_s, i64_ge_s_mod, fuel_check = flat(I64_GE_S));
#[inline(always)]
pub unsafe fn i64_ge_s(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: i64.ge_s [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::i64_ge_u, i64_ge_u_mod, fuel_check = flat(I64_GE_U));
#[inline(always)]
pub unsafe fn i64_ge_u(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if (v1 as u64) >= (v2 as u64) { 1 } else { 0 };

    trace!("Instruction: i64.ge_u [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f32.relop
define_instruction!(super::f32_eq, f32_eq_mod, fuel_check = flat(F32_EQ));
#[inline(always)]
pub unsafe fn f32_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: f32.eq [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_ne, f32_ne_mod, fuel_check = flat(F32_NE));
#[inline(always)]
pub unsafe fn f32_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: f32.ne [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_lt, f32_lt_mod, fuel_check = flat(F32_LT));
#[inline(always)]
pub unsafe fn f32_lt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: f32.lt [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_gt, f32_gt_mod, fuel_check = flat(F32_GT));
#[inline(always)]
pub unsafe fn f32_gt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: f32.gt [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_le, f32_le_mod, fuel_check = flat(F32_LE));
#[inline(always)]
pub unsafe fn f32_le(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: f32.le [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f32_ge, f32_ge_mod, fuel_check = flat(F32_GE));
#[inline(always)]
pub unsafe fn f32_ge(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: f32.ge [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f64.relop
define_instruction!(super::f64_eq, f64_eq_mod, fuel_check = flat(F64_EQ));
#[inline(always)]
pub unsafe fn f64_eq(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 == v2 { 1 } else { 0 };

    trace!("Instruction: f64.eq [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_ne, f64_ne_mod, fuel_check = flat(F64_NE));
#[inline(always)]
pub unsafe fn f64_ne(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 != v2 { 1 } else { 0 };

    trace!("Instruction: f64.ne [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_lt, f64_lt_mod, fuel_check = flat(F64_LT));
#[inline(always)]
pub unsafe fn f64_lt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 < v2 { 1 } else { 0 };

    trace!("Instruction: f64.lt [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_gt, f64_gt_mod, fuel_check = flat(F64_GT));
#[inline(always)]
pub unsafe fn f64_gt(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 > v2 { 1 } else { 0 };

    trace!("Instruction: f64.gt [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_le, f64_le_mod, fuel_check = flat(F64_LE));
#[inline(always)]
pub unsafe fn f64_le(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 <= v2 { 1 } else { 0 };

    trace!("Instruction: f64.le [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(super::f64_ge, f64_ge_mod, fuel_check = flat(F64_GE));
#[inline(always)]
pub unsafe fn f64_ge(state: State) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v2: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res = if v1 >= v2 { 1 } else { 0 };

    trace!("Instruction: f64.ge [{v1} {v2}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i32.cvtop
define_instruction!(
    super::i32_wrap_i64,
    i32_wrap_i64_mod,
    fuel_check = flat(I32_WRAP_I64)
);
#[inline(always)]
pub unsafe fn i32_wrap_i64(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: i32 = v as i32;

    trace!("Instruction: i32.wrap_i64 [{v}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_f32_s,
    i32_trunc_f32_s_mod,
    fuel_check = flat(I32_TRUNC_F32_S)
);
#[inline(always)]
pub unsafe fn i32_trunc_f32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_f32_u,
    i32_trunc_f32_u_mod,
    fuel_check = flat(I32_TRUNC_F32_U)
);
#[inline(always)]
pub unsafe fn i32_trunc_f32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_f64_s,
    i32_trunc_f64_s_mod,
    fuel_check = flat(I32_TRUNC_F64_S)
);
#[inline(always)]
pub unsafe fn i32_trunc_f64_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_f64_u,
    i32_trunc_f64_u_mod,
    fuel_check = flat(I32_TRUNC_F64_U)
);
#[inline(always)]
pub unsafe fn i32_trunc_f64_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_reinterpret_f32,
    i32_reinterpret_f32_mod,
    fuel_check = flat(I32_REINTERPRET_F32)
);
#[inline(always)]
pub unsafe fn i32_reinterpret_f32(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: i32 = v.reinterpret_as_i32();

    trace!("Instruction: i32.reinterpret_f32 [{v:.7}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_extend8_s,
    i32_extend8_s_mod,
    fuel_check = flat(I32_EXTEND8_S)
);
#[inline(always)]
pub unsafe fn i32_extend8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let mut v: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if v | 0xFF != 0xFF {
        trace!("Number v ({}) not contained in 8 bits, truncating", v);
        v &= 0xFF;
    }

    let res = if v | 0x7F != 0x7F { v | 0xFFFFFF00 } else { v };

    state.resumable.stack.push_value(res.into())?;

    trace!("Instruction i32.extend8_s [{}] -> [{}]", v, res);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_extend16_s,
    i32_extend16_s_mod,
    fuel_check = flat(I32_EXTEND16_S)
);
#[inline(always)]
pub unsafe fn i32_extend16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let mut v: u32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if v | 0xFFFF != 0xFFFF {
        trace!("Number v ({}) not contained in 16 bits, truncating", v);
        v &= 0xFFFF;
    }

    let res = if v | 0x7FFF != 0x7FFF {
        v | 0xFFFF0000
    } else {
        v
    };

    state.resumable.stack.push_value(res.into())?;

    trace!("Instruction i32.extend16_s [{}] -> [{}]", v, res);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_sat_f32_s,
    i32_trunc_sat_f32_s_mod,
    fuel_check = flat_fc(I32_TRUNC_SAT_F32_S)
);
#[inline(always)]
pub unsafe fn i32_trunc_sat_f32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_sat_f32_u,
    i32_trunc_sat_f32_u_mod,
    fuel_check = flat_fc(I32_TRUNC_SAT_F32_U)
);
#[inline(always)]
pub unsafe fn i32_trunc_sat_f32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_sat_f64_s,
    i32_trunc_sat_f64_s_mod,
    fuel_check = flat_fc(I32_TRUNC_SAT_F64_S)
);
#[inline(always)]
pub unsafe fn i32_trunc_sat_f64_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i32_trunc_sat_f64_u,
    i32_trunc_sat_f64_u_mod,
    fuel_check = flat_fc(I32_TRUNC_SAT_F64_U)
);
#[inline(always)]
pub unsafe fn i32_trunc_sat_f64_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// i64.cvtop
define_instruction!(
    super::i64_extend_i32_s,
    i64_extend_i32_s_mod,
    fuel_check = flat(I64_EXTEND_I32_S)
);
#[inline(always)]
pub unsafe fn i64_extend_i32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res: i64 = v as i64;

    trace!("Instruction: i64.extend_i32_s [{v}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_extend_i32_u,
    i64_extend_i32_u_mod,
    fuel_check = flat(I64_EXTEND_I32_U)
);
#[inline(always)]
pub unsafe fn i64_extend_i32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    let res: i64 = v as u32 as i64;

    trace!("Instruction: i64.extend_i32_u [{v}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_f32_s,
    i64_trunc_f32_s_mod,
    fuel_check = flat(I64_TRUNC_F32_S)
);
#[inline(always)]
pub unsafe fn i64_trunc_f32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_f32_u,
    i64_trunc_f32_u_mod,
    fuel_check = flat(I64_TRUNC_F32_U)
);
#[inline(always)]
pub unsafe fn i64_trunc_f32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_f64_s,
    i64_trunc_f64_s_mod,
    fuel_check = flat(I64_TRUNC_F64_S)
);
#[inline(always)]
pub unsafe fn i64_trunc_f64_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_f64_u,
    i64_trunc_f64_u_mod,
    fuel_check = flat(I64_TRUNC_F64_U)
);
#[inline(always)]
pub unsafe fn i64_trunc_f64_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_reinterpret_f64,
    i64_reinterpret_f64_mod,
    fuel_check = flat(I64_REINTERPRET_F64)
);
#[inline(always)]
pub unsafe fn i64_reinterpret_f64(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: i64 = v.reinterpret_as_i64();

    trace!("Instruction: i64.reinterpret_f64 [{v:.17}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_extend8_s,
    i64_extend8_s_mod,
    fuel_check = flat(I64_EXTEND8_S)
);
#[inline(always)]
pub unsafe fn i64_extend8_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let mut v: u64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if v | 0xFF != 0xFF {
        trace!("Number v ({}) not contained in 8 bits, truncating", v);
        v &= 0xFF;
    }

    let res = if v | 0x7F != 0x7F {
        v | 0xFFFFFFFF_FFFFFF00
    } else {
        v
    };

    state.resumable.stack.push_value(res.into())?;

    trace!("Instruction i64.extend8_s [{}] -> [{}]", v, res);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_extend16_s,
    i64_extend16_s_mod,
    fuel_check = flat(I64_EXTEND16_S)
);
#[inline(always)]
pub unsafe fn i64_extend16_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let mut v: u64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if v | 0xFFFF != 0xFFFF {
        trace!("Number v ({}) not contained in 16 bits, truncating", v);
        v &= 0xFFFF;
    }

    let res = if v | 0x7FFF != 0x7FFF {
        v | 0xFFFFFFFF_FFFF0000
    } else {
        v
    };

    state.resumable.stack.push_value(res.into())?;

    trace!("Instruction i64.extend16_s [{}] -> [{}]", v, res);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_extend32_s,
    i64_extend32_s_mod,
    fuel_check = flat(I64_EXTEND32_S)
);
#[inline(always)]
pub unsafe fn i64_extend32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let mut v: u64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();

    if v | 0xFFFF_FFFF != 0xFFFF_FFFF {
        trace!("Number v ({}) not contained in 32 bits, truncating", v);
        v &= 0xFFFF_FFFF;
    }

    let res = if v | 0x7FFF_FFFF != 0x7FFF_FFFF {
        v | 0xFFFFFFFF_00000000
    } else {
        v
    };

    state.resumable.stack.push_value(res.into())?;

    trace!("Instruction i64.extend32_s [{}] -> [{}]", v, res);
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_sat_f32_s,
    i64_trunc_sat_f32_s_mod,
    fuel_check = flat_fc(I64_TRUNC_SAT_F32_S)
);
#[inline(always)]
pub unsafe fn i64_trunc_sat_f32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_sat_f32_u,
    i64_trunc_sat_f32_u_mod,
    fuel_check = flat_fc(I64_TRUNC_SAT_F32_U)
);
#[inline(always)]
pub unsafe fn i64_trunc_sat_f32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_sat_f64_s,
    i64_trunc_sat_f64_s_mod,
    fuel_check = flat_fc(I64_TRUNC_SAT_F64_S)
);
#[inline(always)]
pub unsafe fn i64_trunc_sat_f64_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::i64_trunc_sat_f64_u,
    i64_trunc_sat_f64_u_mod,
    fuel_check = flat_fc(I64_TRUNC_SAT_F64_U)
);
#[inline(always)]
pub unsafe fn i64_trunc_sat_f64_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
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
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f32.cvtop
define_instruction!(
    super::f32_convert_i32_s,
    f32_convert_i32_s_mod,
    fuel_check = flat(F32_CONVERT_I32_S)
);
#[inline(always)]
pub unsafe fn f32_convert_i32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = F32(v as f32);

    trace!("Instruction: f32.convert_i32_s [{v}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_convert_i32_u,
    f32_convert_i32_u_mod,
    fuel_check = flat(F32_CONVERT_I32_U)
);
#[inline(always)]
pub unsafe fn f32_convert_i32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = F32(v as u32 as f32);

    trace!("Instruction: f32.convert_i32_u [{v}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_convert_i64_s,
    f32_convert_i64_s_mod,
    fuel_check = flat(F32_CONVERT_I64_S)
);
#[inline(always)]
pub unsafe fn f32_convert_i64_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = F32(v as f32);

    trace!("Instruction: f32.convert_i64_s [{v}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_convert_i64_u,
    f32_convert_i64_u_mod,
    fuel_check = flat(F32_CONVERT_I64_U)
);
#[inline(always)]
pub unsafe fn f32_convert_i64_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = F32(v as u64 as f32);

    trace!("Instruction: f32.convert_i64_u [{v}] -> [{res}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_demote_f64,
    f32_demote_f64_mod,
    fuel_check = flat(F32_DEMOTE_F64)
);
#[inline(always)]
pub unsafe fn f32_demote_f64(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = v.as_f32();

    trace!("Instruction: f32.demote_f64 [{v:.17}] -> [{res:.7}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f32_reinterpret_i32,
    f32_reinterpret_i32_mod,
    fuel_check = flat(F32_REINTERPRET_I32)
);
#[inline(always)]
pub unsafe fn f32_reinterpret_i32(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F32 = F32::from_bits(v1 as u32);

    trace!("Instruction: f32.reinterpret_i32 [{v1}] -> [{res:.7}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

// f64.cvtop
define_instruction!(
    super::f64_convert_i32_s,
    f64_convert_i32_s_mod,
    fuel_check = flat(F64_CONVERT_I32_S)
);
#[inline(always)]
pub unsafe fn f64_convert_i32_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = F64(v as f64);

    trace!("Instruction: f64.convert_i32_s [{v}] -> [{res:.17}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_convert_i32_u,
    f64_convert_i32_u_mod,
    fuel_check = flat(F64_CONVERT_I32_U)
);
#[inline(always)]
pub unsafe fn f64_convert_i32_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = F64(v as u32 as f64);

    trace!("Instruction: f64.convert_i32_u [{v}] -> [{res:.17}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_convert_i64_s,
    f64_convert_i64_s_mod,
    fuel_check = flat(F64_CONVERT_I64_S)
);
#[inline(always)]
pub unsafe fn f64_convert_i64_s(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = F64(v as f64);

    trace!("Instruction: f64.convert_i64_s [{v}] -> [{res:.17}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_convert_i64_u,
    f64_convert_i64_u_mod,
    fuel_check = flat(F64_CONVERT_I64_U)
);
#[inline(always)]
pub unsafe fn f64_convert_i64_u(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = F64(v as u64 as f64);

    trace!("Instruction: f64.convert_i64_u [{v}] -> [{res:.17}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_promote_f32,
    f64_promote_f32_mod,
    fuel_check = flat(F64_PROMOTE_F32)
);
#[inline(always)]
pub unsafe fn f64_promote_f32(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v: F32 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = v.as_f64();

    trace!("Instruction: f64.promote_f32 [{v:.7}] -> [{res:.17}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}

define_instruction!(
    super::f64_reinterpret_i64,
    f64_reinterpret_i64_mod,
    fuel_check = flat(F64_REINTERPRET_I64)
);
#[inline(always)]
pub unsafe fn f64_reinterpret_i64(
    state: State,
) -> Result<ControlFlow<InterpreterLoopOutcome>, RuntimeError> {
    let v1: i64 = state
        .resumable
        .stack
        .pop_value()
        .try_into()
        .unwrap_validated();
    let res: F64 = F64::from_bits(v1 as u64);

    trace!("Instruction: f64.reinterpret_i64 [{v1}] -> [{res:.17}]");
    state.resumable.stack.push_value(res.into())?;
    Ok(ControlFlow::Continue(()))
}
