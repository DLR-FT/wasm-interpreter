use super::global::GlobalType;
use super::RefType;
use crate::core::indices::FuncIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::TableType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::read_constant_expression::read_constant_expression;
use crate::validation_stack::ValidationStack;
use crate::{Error, NumType, Result, ValType};

use alloc::collections::btree_set::BTreeSet;
use alloc::vec::Vec;
use core::fmt::Debug;

#[derive(Clone)]
pub struct ElemType {
    pub init: ElemItems,
    pub mode: ElemMode,
}

impl Debug for ElemType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ElemType {{\n\tinit: {:?},\n\tmode: {:?},\n\t#ty: {:?}\n}}",
            self.init,
            self.mode,
            self.init.ty()
        )
    }
}

impl ElemType {
    pub fn ty(&self) -> RefType {
        self.init.ty()
    }

    pub fn to_ref_type(&self) -> RefType {
        match self.init {
            ElemItems::Exprs(rref, _) => rref,
            ElemItems::RefFuncs(_) => RefType::FuncRef,
        }
    }

    // TODO: @nerodesu017 maybe split up the validation from the parsing?
    /// Here we can't implement WasmReadable because we also want a mutable
    /// reference to a [`BTreeSet<u32>`] (`referenced_functions`)
    ///
    /// This comes in handy later on when we are validating the actual code of
    /// the functions so that we can make sure we are not referencing invalid
    /// functions
    pub fn read_from_wasm(
        wasm: &mut WasmReader,
        functions: &[usize],
        validation_context_refs: &mut BTreeSet<FuncIdx>,
        tables: &[TableType],
        imported_global_types: &[GlobalType],
    ) -> Result<Vec<Self>> {
        wasm.read_vec(|wasm| {
            let prop = wasm.read_var_u32()?;

            // hack to assert that C.funcs[...] exists
            let num_funcs = functions.len();

            let elem = match prop {
                0 => {
                    // binary format is: 0:u32 e:expr y*:vec(funcidx)
                    // should parse to spec struct {type funcref, init ((ref.func y) end)*, mode active {table 0, offset e}}
                    // which is equivalent to ElemType{init: ElemItems::RefFuncs(y*), mode: ElemMode::Active{0, e}} here
                    let e = parse_validate_active_segment_offset_expr(
                        wasm,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let init = parse_validate_shortened_initializer_list(
                        wasm,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Active(ActiveElem {
                        table_idx: 0,
                        init_expr: e,
                    });
                    ElemType { init, mode }
                }
                1 => {
                    // binary format is: 1:u32 et:elemkind y*:vec(funcidx)
                    // should parse to spec struct {type et, init ((ref.func y) end)*, mode passive}
                    // which is equivalent to ElemType{init: ElemItems::RefFuncs(y*), mode: ElemMode::Passive} here
                    let _et = parse_elemkind(wasm)?;
                    let init = parse_validate_shortened_initializer_list(
                        wasm,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Passive;
                    ElemType { init, mode }
                }
                2 => {
                    // binary format is: 2:u32 x:tableidx e:expr et:elemkind y*:vec(funcidx)
                    // should parse to spec struct {type et, init ((ref.func y) end)*, mode active {table x, offset e}}
                    // which reflects to ElemType{init: ElemItems::RefFuncs(y*), mode: ElemMode::Active{x, e}} here
                    let x = wasm.read_var_u32()?;
                    let e = parse_validate_active_segment_offset_expr(
                        wasm,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let _et = parse_elemkind(wasm)?;
                    let init = parse_validate_shortened_initializer_list(
                        wasm,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Active(ActiveElem {
                        table_idx: x,
                        init_expr: e,
                    });
                    ElemType { init, mode }
                }
                3 => {
                    // binary format is: 3:u32 et:elemkind y*:vec(funcidx)
                    // should parse to spec struct {type et, init ((ref.func y) end)*, mode declarative}
                    // which is equivalent to ElemType{init: ElemItems::RefFuncs(y*), mode: ElemMode::Declarative} here
                    let _et = parse_elemkind(wasm)?;
                    let init = parse_validate_shortened_initializer_list(
                        wasm,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Declarative;
                    ElemType { init, mode }
                }
                4 => {
                    // binary format is: 4:u32 e:expr el*:vec(expr)
                    // should parse to spec struct {type funcref, init el*, mode active { table 0, offset e}}
                    // which is equivalent to ElemType{init: ElemItems::Exprs(funcref, el*), mode: ElemMode::Active{0, e}}
                    let e = parse_validate_active_segment_offset_expr(
                        wasm,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let init = parse_validate_generic_initializer_list(
                        wasm,
                        RefType::FuncRef,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Active(ActiveElem {
                        table_idx: 0,
                        init_expr: e,
                    });
                    ElemType { init, mode }
                }
                5 => {
                    // binary format is 5:u32 et: reftype el*:vec(expr)
                    // should parse to spec struct {type et, init el*, mode passive}
                    // which is equivalent to ElemType{init: ElemItems::Exprs(et, el*), mode: ElemMode::Passive} here
                    let et = RefType::read(wasm)?;
                    let init = parse_validate_generic_initializer_list(
                        wasm,
                        et,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Passive;
                    ElemType { init, mode }
                }
                6 => {
                    // binary format is 6:u32 x:table_idx e:expr et:reftype el*:vec(expr)
                    // should parse to spec struct {type et, init el*, mode passive}
                    // which is equivalent to ElemType{init: Exprs(et, el*), mode: ElemMode::Active{table x, offset e}} here
                    let x = wasm.read_var_u32()?;
                    let e = parse_validate_active_segment_offset_expr(
                        wasm,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let et = RefType::read(wasm)?;
                    let init = parse_validate_generic_initializer_list(
                        wasm,
                        et,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Active(ActiveElem {
                        table_idx: x,
                        init_expr: e,
                    });
                    ElemType { init, mode }
                }
                7 => {
                    // binary format is 7:u32 et:reftype el*:vec(expr)
                    // should parse to spec struct {type et, init el*, mode declarative}
                    // which is equivalent to ElemType{init: Exprs(et, el*), mode: ElemMode::Declarative} here
                    let et = RefType::read(wasm)?;
                    let init = parse_validate_generic_initializer_list(
                        wasm,
                        et,
                        imported_global_types,
                        num_funcs,
                        validation_context_refs,
                    )?;
                    let mode = ElemMode::Declarative;
                    ElemType { init, mode }
                }
                8.. => {
                    // TODO fix error
                    return Err(Error::InvalidVersion);
                }
            };

            // assume the element segment is well formed in terms of abstract syntax from now on.
            // start validating element segment of form {type t, init e*, mode elemmode}: https://webassembly.github.io/spec/core/valid/modules.html#element-segments
            let t = elem.ty();
            // 1. Each e_i must be valid with type t and be const: this is already checked during the parse of initializer expressions above.
            // 2. elemmode must be valid with type t
            // -- start validating elemmode for type t:
            match elem.mode {
                ElemMode::Active(ActiveElem {
                    table_idx: x,
                    init_expr: _expr,
                }) => {
                    // start validating elemmode of form active {table x, offset expr}
                    // 1-2. C.tables[x] must be defined with type: limits t
                    let table_type = tables.get(x as usize).ok_or(Error::UnknownTable)?.et;
                    if table_type != t {
                        return Err(Error::UnknownTable);
                    }
                    // 3-4. _expr must be valid with type I32 and be const: already checked during the parse of initializer expressions above.
                    // Then elemmode is valid with type t.
                }
                ElemMode::Declarative | ElemMode::Passive => (), // these are valid for any type t.
            }
            // -- Then elemmmode is valid with type t.
            // Then the element segment is valid with type t.
            Ok(elem)
        })
    }
}

#[derive(Debug, Clone)]
pub enum ElemItems {
    RefFuncs(Vec<u32>),
    Exprs(RefType, Vec<Span>),
}

impl ElemItems {
    pub fn ty(&self) -> RefType {
        match self {
            Self::RefFuncs(_) => RefType::FuncRef,
            // the mapping for shortened lists above is always true, as the binary format
            // either parses an elemkind or assumes funcref, and the current spec always maps a well-formed elemkind to a funcref
            // https://webassembly.github.io/spec/core/binary/modules.html#element-section
            Self::Exprs(rty, _) => *rty,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::RefFuncs(ref_funcs) => ref_funcs.len(),
            Self::Exprs(_, exprs) => exprs.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElemMode {
    Passive,
    Active(ActiveElem),
    Declarative,
}

#[derive(Debug, Clone)]
pub struct ActiveElem {
    pub table_idx: u32,
    pub init_expr: Span,
}

// helpers for avoiding code duplication during parsing/validation of element segment
// https://webassembly.github.io/spec/core/binary/modules.html#element-section

/// Parse and validate an active segment offset.
/// An active segment offset is valid if its offset expr is of type I32 and is a const expr.
/// Additionally inserts new items to the set validation_context_refs.
/// Validation_context_refs corresponds to C.refs in <https://webassembly.github.io/spec/core/valid/conventions.html#context>
///
/// # Returns
/// - `Ok(Span)` of the expr validated to be of type I32 if parsing & validating succeeds, `Err(_)` otherwise.
fn parse_validate_active_segment_offset_expr(
    wasm: &mut WasmReader,
    imported_global_types: &[GlobalType],
    num_funcs: usize,
    validation_context_refs: &mut BTreeSet<FuncIdx>,
) -> Result<Span> {
    let mut valid_stack = ValidationStack::new();
    let (span, seen_func_refs) =
        read_constant_expression(wasm, &mut valid_stack, imported_global_types, num_funcs)?;
    validation_context_refs.extend(seen_func_refs);
    valid_stack.assert_val_types(&[ValType::NumType(NumType::I32)], true)?;
    Ok(span)
}

/// Parse and validate a vector of func_idx's that reflect as the initializer list of an element segment in the form of ((ref.func func_idx) end) of in the abstract syntax.
/// An expression of such form is valid and const with type funcref if `C.funcs[func_idx]` exists (see link for the definition of validation context)
/// This codebase holds these as `ElemItems::RefFuncs(Vec<u32>)` to sidestep the need to show `Span` for each expression.
/// Additionally inserts new items to the set validation_context_refs.
/// validation_context_refs corresponds to C.refs in <https://webassembly.github.io/spec/core/valid/conventions.html#context>
///
/// # Returns
/// - `Ok(ElemItems::RefFuncs(_))` corresponding to the parsed list if parsing & validating succeeds, `Err(_)` otherwise.
fn parse_validate_shortened_initializer_list(
    wasm: &mut WasmReader,
    num_funcs: usize,
    validation_context_refs: &mut BTreeSet<FuncIdx>,
) -> Result<ElemItems> {
    wasm.read_vec(|w| {
        let func_idx = w.read_var_u32()?;
        if num_funcs <= func_idx as usize {
            // TODO fix error
            return Err(Error::InvalidLocalIdx);
        }
        validation_context_refs.insert(func_idx as FuncIdx);
        Ok(func_idx)
    })
    .map(ElemItems::RefFuncs)
}

/// Parse and validate the initializer list of an element segment for the supplied type `expected_type`.
/// An initializer list is valid with type `expected_type` if all of the expressions within is const and is of type `expected_type`.
/// This codebase holds these as `ElemItems::Exprs(RefType, Vec<Span>)`.
/// Additionally inserts new items to the set validation_context_refs.
/// validation_context_refs corresponds to C.refs in <https://webassembly.github.io/spec/core/valid/conventions.html#context>
///
/// # Returns
/// - `Ok(ElemItems::Exprs(expected_type, _))` corresponding to the parsed list if parsing & validating succeeds, `Err(_)` otherwise.
fn parse_validate_generic_initializer_list(
    wasm: &mut WasmReader,
    expected_type: RefType,
    imported_global_types: &[GlobalType],
    num_funcs: usize,
    validation_context_refs: &mut BTreeSet<FuncIdx>,
) -> Result<ElemItems> {
    wasm.read_vec(|w| {
        let mut valid_stack = ValidationStack::new();
        let (span, seen_func_refs) =
            read_constant_expression(w, &mut valid_stack, imported_global_types, num_funcs)?;
        validation_context_refs.extend(seen_func_refs);
        valid_stack.assert_val_types(&[ValType::RefType(expected_type)], true)?;
        Ok(span)
    })
    .map(|v| ElemItems::Exprs(expected_type, v))
}

/// Parse an elemkind: <https://webassembly.github.io/spec/core/binary/modules.html#element-section>
/// # Returns
/// - `Ok(elemkind)` if parsing is successful, Err(_) otherwise
fn parse_elemkind(wasm: &mut WasmReader) -> Result<u8> {
    let et = wasm.read_u8()?;
    if et != 0x00 {
        Err(Error::OnlyFuncRefIsAllowed)
    } else {
        Ok(et)
    }
}
