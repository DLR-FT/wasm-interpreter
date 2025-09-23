use std::any::Any;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::panic::UnwindSafe;

use bumpalo::Bump;
use itertools::enumerate;
use log::debug;
use wasm::function_ref::FunctionRef;
use wasm::RuntimeError;
use wasm::TrapError;
use wasm::Value;
use wasm::{validate, RuntimeInstance};
use wast::core::WastArgCore;
use wast::core::WastRetCore;
use wast::QuoteWat;
use wast::WastArg;
use wast::WastDirective;
use wast::Wat;

use crate::specification::reports::*;
use crate::specification::test_errors::*;

use super::ENV_CONFIG;

//Each script runs within an interpreter that has the following module with name "spectest" defined
//https://github.com/WebAssembly/spec/tree/main/interpreter#spectest-host-module
// TODO printing not possible since host functions are not implemented yet
const SPEC_TEST_WAT: &str = r#"
(module
  ;; Memory
  (memory (export "memory") 1 2)

  ;; Table
  (table (export "table") 10 20 funcref)

  ;; Globals
  (global (export "global_i32") i32 (i32.const 666))
  (global (export "global_i64") i64 (i64.const 666))
  (global (export "global_f32") f32 (f32.const 666.6))
  (global (export "global_f64") f64 (f64.const 666.6))

  ;; Dummy functions for printing
  (func (export "print")
    ;; No params, no results
  )

  (func (export "print_i32") (param i32)
    ;; No results
  )

  (func (export "print_i64") (param i64)
    ;; No results
  )

  (func (export "print_f32") (param f32)
    ;; No results
  )

  (func (export "print_f64") (param f64)
    ;; No results
  )

  (func (export "print_i32_f32") (param i32 f32)
    ;; No results
  )

  (func (export "print_f64_f64") (param f64 f64)
    ;; No results
  )
)
"#;

pub fn error_to_wasm_testsuite_string(runtime_error: &RuntimeError) -> Result<String, WastError> {
    match runtime_error {
        RuntimeError::Trap(TrapError::DivideBy0) => Ok("integer divide by zero"),
        RuntimeError::Trap(TrapError::UnrepresentableResult) => Ok("integer overflow"),
        RuntimeError::Trap(TrapError::BadConversionToInteger) => {
            Ok("invalid conversion to integer")
        }
        RuntimeError::Trap(TrapError::ReachedUnreachable) => Ok("unreachable"),
        RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds) => {
            Ok("out of bounds memory access")
        }
        RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds) => {
            Ok("out of bounds table access")
        }
        RuntimeError::Trap(TrapError::UninitializedElement) => Ok("uninitialized element"),
        RuntimeError::Trap(TrapError::SignatureMismatch) => Ok("indirect call type mismatch"),
        RuntimeError::Trap(TrapError::TableAccessOutOfBounds) => Ok("undefined element"),

        RuntimeError::StackExhaustion => Ok("call stack exhausted"),
        RuntimeError::ModuleNotFound => Ok("module not found"),
        RuntimeError::FunctionNotFound => Err(WastError::UnrepresentedRuntimeError),
        RuntimeError::HostFunctionSignatureMismatch => Ok("host function signature mismatch"),
        _ => Err(WastError::UnrepresentedRuntimeError),
    }
    .map(ToOwned::to_owned)
}

/// Clear the bytes and runtime instance before calling this function
fn encode(modulee: &mut wast::QuoteWat) -> Result<Vec<u8>, WastError> {
    match &modulee {
        QuoteWat::QuoteComponent(..) | QuoteWat::Wat(wast::Wat::Component(..)) => {
            unimplemented!("Component modules");
        }
        QuoteWat::Wat(..) | QuoteWat::QuoteModule(..) => (),
    };

    modulee.encode().map_err(Into::into)
}

fn validate_instantiate<'a, 'b: 'a>(
    interpreter: &'a mut RuntimeInstance<'b>,
    bytes: &'b [u8],
) -> Result<(), WastError> {
    let validation_info =
        catch_unwind_and_suppress_panic_handler(|| validate(bytes)).map_err(WastError::Panic)??;

    // TODO change hacky hidden name that uses interpreter internals
    let module_name = format!("module_{}", interpreter.store.modules.len());
    catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
        interpreter.add_module(module_name.as_str(), &validation_info)
    }))
    .map_err(WastError::Panic)??;

    Ok(())
}

/// If returns `Some`:
/// The script ran successfully, having directives run successfuly (though
/// not necessarily meaning all asserts pass!)
/// else when returning `None`:
/// The script could not run successfully, a non-assert directive failed in
/// such a way the script cannot continue running.
pub fn run_spec_test(filepath: &str) -> Result<AssertReport, ScriptError> {
    // -=-= Initialization =-=-
    let arena = bumpalo::Bump::new();
    debug!("{}", filepath);

    let mut visible_modules = HashMap::new();

    let contents = std::fs::read_to_string(filepath).map_err(|err| {
        ScriptError::new_lineless(filepath, err.into(), "failed to open wast file")
    })?;

    let buf = wast::parser::ParseBuffer::new(&contents).map_err(|err| {
        ScriptError::new_lineless(filepath, err.into(), "failed to create wast buffer")
    })?;

    let wast = wast::parser::parse::<wast::Wast>(&buf).map_err(|err| {
        ScriptError::new_lineless(filepath, err.into(), "failed to parse wast file")
    })?;

    // -=-= Testing & Compilation =-=-
    let mut asserts = AssertReport::new(filepath);

    // We need to keep the wasm_bytes in-scope for the lifetime of the interpreter.
    // As such, we hoist the bytes into an Option, and assign it once a module directive is found.
    #[allow(unused_assignments)]
    // let mut wasm_bytes: Option<Vec<u8>> = None;
    let spectest_wasm = wat::parse_str(SPEC_TEST_WAT).unwrap();
    let spectest_validation_info =
        catch_unwind_and_suppress_panic_handler(|| validate(&spectest_wasm))
            .unwrap()
            .unwrap();
    let interpreter = &mut catch_unwind_and_suppress_panic_handler(|| {
        RuntimeInstance::new_named((), "spectest", &spectest_validation_info)
    })
    .unwrap()
    .unwrap();

    for (i, directive) in enumerate(wast.directives) {
        debug!("at directive {:?}", i);

        let directive_result = run_directive(
            directive,
            &arena,
            &mut visible_modules,
            interpreter,
            &contents,
            filepath,
        )?;

        if let Some(assert_outcome) = directive_result {
            asserts.results.push(assert_outcome);
        }
    }

    Ok(asserts)
}

fn run_directive<'a>(
    wast_directive: WastDirective,
    arena: &'a Bump,
    visible_modules: &mut HashMap<String, usize>,
    interpreter: &mut RuntimeInstance<'a>,
    contents: &str,
    filepath: &str,
) -> Result<Option<AssertOutcome>, ScriptError> {
    match wast_directive {
        wast::WastDirective::Wat(mut quoted) => {
            // If we fail to compile or to validate the main module, then we should treat this
            // as a fatal (compilation) error.
            let wasm_bytes = encode(&mut quoted).map_err(|err| {
                ScriptError::new(
                    filepath,
                    err,
                    "Module directive (WAT) failed in encoding step.",
                    get_linenum(contents, quoted.span()),
                    get_command(contents, quoted.span()),
                )
            })?;

            // retain information of the id of the current wast
            match quoted {
                QuoteWat::Wat(wast::Wat::Module(wast::core::Module {
                    id: _maybe_id @ Some(id),
                    ..
                }))
                | QuoteWat::Wat(wast::Wat::Component(wast::component::Component {
                    id: _maybe_id @ Some(id),
                    ..
                })) => {
                    visible_modules.insert(id.name().to_owned(), interpreter.store.modules.len())
                }
                _ => None,
            };

            // re-allocate the wasm bytecode into an arena backed allocation, gifting it a
            // lifetime of the outermost scope in the current function
            let wasm_bytes = arena.alloc_slice_clone(&wasm_bytes) as &[u8];

            validate_instantiate(interpreter, wasm_bytes).map_err(|err| {
                ScriptError::new(
                    filepath,
                    err,
                    "Module directive (WAT) failed in validation or instantiation.",
                    get_linenum(contents, quoted.span()),
                    get_command(contents, quoted.span()),
                )
            })?;

            Ok(None)
        }
        wast::WastDirective::AssertReturn {
            span,
            exec,
            results,
        } => {
            let err_or_panic = execute_assert_return(visible_modules, interpreter, exec, results);

            Ok(Some(AssertOutcome {
                line_number: get_linenum(contents, span),
                command: get_command(contents, span).to_owned(),
                maybe_error: err_or_panic.err(),
            }))
        }
        wast::WastDirective::AssertTrap {
            span,
            exec,
            message,
        } => {
            let result = execute(arena, visible_modules, interpreter, exec);
            let result = match result {
                Err(WastError::WasmRuntimeError(wasm::RuntimeError::Trap(trap_error))) => {
                    let actual_matches_expected =
                        error_to_wasm_testsuite_string(&RuntimeError::Trap(trap_error.clone()))
                            .is_ok_and(|actual| {
                                actual.contains(message)
                                    || (message.contains("uninitialized element 2")
                                        && actual.contains("uninitialized element"))
                            });

                    actual_matches_expected.then_some(()).ok_or_else(|| {
                        WastError::AssertTrapButTrapWasIncorrect {
                            expected: message.to_owned(),
                            actual: Some(trap_error),
                        }
                    })
                }
                other => Err(other
                    .err()
                    .unwrap_or(WastError::AssertTrapButTrapWasIncorrect {
                        expected: message.to_owned(),
                        actual: None,
                    })),
            };

            Ok(Some(AssertOutcome {
                line_number: get_linenum(contents, span),
                command: get_command(contents, span).to_owned(),
                maybe_error: result.err(),
            }))
        }

        wast::WastDirective::AssertMalformed {
            span,
            module: mut modulee,
            message: _,
        }
        | wast::WastDirective::AssertInvalid {
            span,
            module: mut modulee,
            message: _,
        } => {
            let line_number = get_linenum(contents, span);
            let cmd = get_command(contents, span);
            let result = encode(&mut modulee).and_then(|bytes| {
                let bytes = arena.alloc_slice_clone(&bytes);
                validate_instantiate(interpreter, bytes)
            });

            Ok(Some(AssertOutcome {
                line_number,
                command: cmd.to_owned(),
                maybe_error: result.is_ok().then_some(WastError::AssertInvalidButValid),
            }))
        }

        wast::WastDirective::Register {
            name,
            module: modulee,
            ..
        } => {
            // TODO this implementation is incorrect, but a correct implementation requires a refactor discussion

            // spec tests tells us to use the last defined module if module name is not specified
            // TODO this ugly chunk might need to be refactored out
            let store = &mut interpreter.store;
            let module_addr = match modulee {
                None => store.modules.len() - 1,
                Some(id) => {
                    log::error!("looking for {:?} in \n{:?}", id.name(), visible_modules);
                    visible_modules[id.name()]
                }
            };
            store
                .registry
                .register_module(name.to_owned().into(), &store.modules[module_addr])
                .unwrap();

            Ok(None)
        }
        wast::WastDirective::AssertUnlinkable {
            span,
            mut module,
            message: _,
        } => {
            let line_number = get_linenum(contents, span);
            let cmd = get_command(contents, span);

            // if it can't be parsed, then the test itself must be written incorrectly, thus the unwrap
            let bytes: &[u8] = arena.alloc_slice_clone(&module.encode().unwrap());

            let result = match validate_instantiate(interpreter, bytes) {
                // module shouldn't have instantiated
                Err(WastError::WasmRuntimeError(
                    RuntimeError::ModuleNotFound
                    | RuntimeError::UnknownImport
                    | RuntimeError::InvalidImportType,
                )) => Ok(()),
                _ => Err(WastError::AssertUnlinkableButLinked),
            };

            Ok(Some(AssertOutcome {
                line_number,
                command: cmd.to_owned(),
                maybe_error: result.err(),
            }))
        }
        wast::WastDirective::AssertExhaustion {
            span,
            call,
            message,
        } => {
            let execution_result = execute(
                arena,
                visible_modules,
                interpreter,
                wast::WastExecute::Invoke(call),
            );

            let result = match execution_result {
                Err(WastError::WasmRuntimeError(runtime_error)) => {
                    match error_to_wasm_testsuite_string(&runtime_error) {
                        Ok(actual) if actual.contains(message) => Ok(()),
                        _other => Err(WastError::AssertExhaustionButDidNotExhaust {
                            expected: message.to_owned(),
                            actual: Some(runtime_error),
                        }),
                    }
                }
                Ok(()) => Err(WastError::AssertExhaustionButDidNotExhaust {
                    expected: message.to_owned(),
                    actual: None,
                }),
                Err(other_error) => Err(other_error),
            };

            Ok(Some(AssertOutcome {
                line_number: get_linenum(contents, span),
                command: get_command(contents, span).to_owned(),
                maybe_error: result.err(),
            }))
        }
        wast::WastDirective::Invoke(invoke) => {
            let args: Vec<Value> = invoke.args.into_iter().map(arg_to_value).collect();

            let function_ref = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let store = &mut interpreter.store;
                let module_inst = match invoke.module {
                    None => store.modules.last().unwrap(),
                    Some(id) => {
                        let module_addr = visible_modules
                            .get(id.name())
                            .ok_or(RuntimeError::ModuleNotFound)?;
                        store
                            .modules
                            .get(*module_addr)
                            .ok_or(RuntimeError::ModuleNotFound)?
                    }
                };

                module_inst
                    .exports
                    .get(invoke.name)
                    .and_then(|value| match value {
                        wasm::ExternVal::Func(func_addr) => Some(FunctionRef {
                            func_addr: *func_addr,
                        }),
                        _ => None,
                    })
                    .ok_or(RuntimeError::FunctionNotFound)
            }))
            .map_err(|panic_error| {
                ScriptError::new(
                    filepath,
                    WastError::Panic(panic_error),
                    "main module validation panicked",
                    get_linenum(contents, invoke.span),
                    get_command(contents, invoke.span),
                )
            })?
            .map_err(|runtime_error| {
                ScriptError::new(
                    filepath,
                    WastError::WasmRuntimeError(runtime_error),
                    "invoke directive failed to find function",
                    get_linenum(contents, invoke.span),
                    get_command(contents, invoke.span),
                )
            })?;

            catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.invoke(&function_ref, args)
            }))
            .map_err(|panic_error| {
                ScriptError::new(
                    filepath,
                    WastError::Panic(panic_error),
                    "invocation of function panicked",
                    get_linenum(contents, invoke.span),
                    get_command(contents, invoke.span),
                )
            })?
            .map_err(|runtime_error| {
                ScriptError::new(
                    filepath,
                    WastError::WasmRuntimeError(runtime_error),
                    "invoke returned error or panicked",
                    get_linenum(contents, invoke.span),
                    get_command(contents, invoke.span),
                )
            })?;

            Ok(None)
        }
        wast::WastDirective::Thread(_) => {
            unimplemented!("`thread` directive does not exist anymore");
        }
        wast::WastDirective::AssertException { .. } => {
            unimplemented!("`assert_exception` directive is required only by tests for certain proposals which are not yet supported")
        }
        wast::WastDirective::Wait { .. } => {
            unimplemented!("`wait` directive does not exist anymore");
        }
    }
}

fn execute_assert_return(
    visible_modules: &HashMap<String, usize>,
    interpreter: &mut RuntimeInstance,
    exec: wast::WastExecute,
    results: Vec<wast::WastRet>,
) -> Result<(), WastError> {
    match exec {
        wast::WastExecute::Invoke(invoke_info) => {
            let args: Vec<Value> = invoke_info.args.into_iter().map(arg_to_value).collect();
            let result_vals: Vec<Value> = results.into_iter().map(result_to_value).collect();

            // spec tests tells us to use the last defined module if module name is not specified
            // TODO this ugly chunk might need to be refactored out
            let func = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let store = &mut interpreter.store;
                let module_inst = match invoke_info.module {
                    None => store.modules.last().unwrap(),
                    Some(id) => {
                        let module_addr = visible_modules
                            .get(id.name())
                            .ok_or(RuntimeError::ModuleNotFound)?;
                        store
                            .modules
                            .get(*module_addr)
                            .ok_or(RuntimeError::ModuleNotFound)?
                    }
                };

                module_inst
                    .exports
                    .get(invoke_info.name)
                    .and_then(|value| match value {
                        wasm::ExternVal::Func(func_addr) => Some(FunctionRef {
                            func_addr: *func_addr,
                        }),
                        _ => None,
                    })
                    .ok_or(RuntimeError::FunctionNotFound)
            }))
            .map_err(WastError::Panic)??;

            let actual = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.invoke(&func, args)
            }))
            .map_err(WastError::Panic)??;

            assert_eq(actual, result_vals).map_err(Into::into)
        }
        wast::WastExecute::Get {
            span: _,
            module,
            global,
        } => {
            let result_vals: Vec<Value> = results.into_iter().map(result_to_value).collect();
            let actual = catch_unwind_and_suppress_panic_handler::<Result<Value, RuntimeError>>(
                AssertUnwindSafe(|| {
                    let store = &mut interpreter.store;
                    let module_inst = match module {
                        None => store.modules.last().unwrap(),
                        Some(id) => {
                            let module_addr = visible_modules
                                .get(id.name())
                                .ok_or(RuntimeError::ModuleNotFound)?;
                            store
                                .modules
                                .get(*module_addr)
                                .ok_or(RuntimeError::ModuleNotFound)?
                        }
                    };
                    let global_addr = module_inst
                        .exports
                        .get(global)
                        .and_then(|value| match value {
                            wasm::ExternVal::Global(global_addr) => Some(*global_addr),
                            _ => None,
                        })
                        .ok_or(RuntimeError::FunctionNotFound)?; // TODO fix error
                    Ok(store.globals[global_addr].value)
                }),
            )
            .map_err(WastError::Panic)??;

            assert_eq(vec![actual], result_vals).map_err(Into::into)
        }
        wast::WastExecute::Wat(_) => todo!("`wat` directive inside `assert_return`"),
    }
}

fn execute<'a>(
    arena: &'a bumpalo::Bump,
    visible_modules: &HashMap<String, usize>,
    interpreter: &mut RuntimeInstance<'a>,
    exec: wast::WastExecute,
) -> Result<(), WastError> {
    match exec {
        wast::WastExecute::Invoke(invoke_info) => {
            let args = invoke_info
                .args
                .into_iter()
                .map(arg_to_value)
                .collect::<Vec<_>>();

            // spec tests tells us to use the last defined module if module name is not specified
            // TODO this ugly chunk might need to be refactored out
            let func = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let store = &mut interpreter.store;
                let module_inst = match invoke_info.module {
                    None => store.modules.last().unwrap(),
                    Some(id) => {
                        let module_addr = visible_modules
                            .get(id.name())
                            .ok_or(RuntimeError::ModuleNotFound)?;
                        store
                            .modules
                            .get(*module_addr)
                            .ok_or(RuntimeError::ModuleNotFound)?
                    }
                };

                module_inst
                    .exports
                    .get(invoke_info.name)
                    .and_then(|value| match value {
                        wasm::ExternVal::Func(func_addr) => Some(FunctionRef {
                            func_addr: *func_addr,
                        }),
                        _ => None,
                    })
                    .ok_or(RuntimeError::FunctionNotFound)
            }))
            .map_err(WastError::Panic)??;

            catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.invoke(&func, args)
            }))
            .map_err(WastError::Panic)??;

            Ok(())
        }
        wast::WastExecute::Get {
            span: _,
            module: _,
            global: _,
        } => todo!("`get` directive inside `assert_trap`"),
        wast::WastExecute::Wat(Wat::Module(mut module)) => {
            let bytecode: &[u8] = arena.alloc_slice_clone(&module.encode()?);
            let validation_info = catch_unwind_and_suppress_panic_handler(|| validate(bytecode))
                .map_err(WastError::Panic)??;

            // TODO change hacky hidden name that uses interpreter internals
            let module_name = format!("module_{}", interpreter.store.modules.len());
            catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.add_module(module_name.as_str(), &validation_info)
            }))
            .map_err(WastError::Panic)??;

            Ok(())
        }
        wast::WastExecute::Wat(Wat::Component(_)) => todo!("components inside `assert_trap`"),
    }
}

pub fn arg_to_value(arg: WastArg) -> Value {
    match arg {
        WastArg::Core(core_arg) => match core_arg {
            WastArgCore::I32(val) => Value::I32(val as u32),
            WastArgCore::I64(val) => Value::I64(val as u64),
            WastArgCore::F32(val) => Value::F32(wasm::value::F32(f32::from_bits(val.bits))),
            WastArgCore::F64(val) => Value::F64(wasm::value::F64(f64::from_bits(val.bits))),
            WastArgCore::V128(_) => todo!("`V128` value arguments"),
            WastArgCore::RefNull(rref) => match rref {
                wast::core::HeapType::Concrete(_) => {
                    unreachable!("Null refs don't point to any specific reference")
                }
                wast::core::HeapType::Abstract { shared: _, ty } => {
                    use wasm::value::*;
                    use wast::core::AbstractHeapType::*;
                    match ty {
                        Func => Value::Ref(Ref::Func(FuncAddr::null())),
                        Extern => Value::Ref(Ref::Extern(ExternAddr::null())),
                        _ => todo!("`GC` proposal"),
                    }
                }
            },
            WastArgCore::RefExtern(index) => wasm::value::Value::Ref(wasm::value::Ref::Extern(
                wasm::value::ExternAddr::new(Some(index as usize)),
            )),
            WastArgCore::RefHost(_) => {
                todo!("`RefHost` value arguments")
            }
        },
        WastArg::Component(_) => todo!("`Component` value arguments"),
    }
}

fn result_to_value(result: wast::WastRet) -> Value {
    match result {
        wast::WastRet::Core(core_arg) => match core_arg {
            WastRetCore::I32(val) => Value::I32(val as u32),
            WastRetCore::I64(val) => Value::I64(val as u64),
            WastRetCore::F32(val) => match val {
                wast::core::NanPattern::CanonicalNan => {
                    Value::F32(wasm::value::F32(f32::from_bits(0x7fc0_0000)))
                }
                wast::core::NanPattern::ArithmeticNan => {
                    // First ArithmeticNan and Inf overlap, have a distinction (because we will revert this operation)
                    Value::F32(wasm::value::F32(f32::from_bits(0x7f80_0001)))
                }
                wast::core::NanPattern::Value(val) => {
                    Value::F32(wasm::value::F32(f32::from_bits(val.bits)))
                }
            },
            WastRetCore::F64(val) => match val {
                wast::core::NanPattern::CanonicalNan => {
                    Value::F64(wasm::value::F64(f64::from_bits(0x7ff8_0000_0000_0000)))
                }
                wast::core::NanPattern::ArithmeticNan => {
                    // First ArithmeticNan and Inf overlap, have a distinction (because we will revert this operation)
                    Value::F64(wasm::value::F64(f64::from_bits(0x7ff0_0000_0000_0001)))
                }
                wast::core::NanPattern::Value(val) => {
                    Value::F64(wasm::value::F64(f64::from_bits(val.bits)))
                }
            },
            WastRetCore::RefNull(Some(rref)) => match rref {
                wast::core::HeapType::Concrete(_) => {
                    unreachable!("Null refs don't point to any specific reference")
                }
                wast::core::HeapType::Abstract { shared: _, ty } => {
                    use wasm::value::*;
                    use wast::core::AbstractHeapType::*;
                    match ty {
                        Func => Value::Ref(Ref::Func(FuncAddr::null())),
                        Extern => Value::Ref(Ref::Extern(ExternAddr::null())),
                        _ => todo!("`GC` proposal"),
                    }
                }
            },
            WastRetCore::RefFunc(index) => match index {
                None => unreachable!("Expected a non-null function reference"),
                Some(_index) => {
                    // use wasm::value::*;
                    // Value::Ref(Ref::Func(FuncAddr::new(Some(index))))

                    todo!("RefFunc return type")
                }
            },
            WastRetCore::RefExtern(None) => unreachable!("Expected a non-null extern reference"),
            WastRetCore::RefExtern(Some(index)) => {
                Value::Ref(wasm::value::Ref::Extern(wasm::value::ExternAddr {
                    addr: Some(index as usize),
                }))
            }
            other => todo!("handling of wast ret type {other:?}"),
        },
        wast::WastRet::Component(_) => todo!("`Component` result"),
    }
}

pub fn get_linenum(contents: &str, span: wast::token::Span) -> u32 {
    span.linecol_in(contents).0 as u32 + 1
}

pub fn get_command(contents: &str, span: wast::token::Span) -> &str {
    contents[span.offset()..]
        .lines()
        .next()
        .unwrap_or("<unknown>")
}

pub fn catch_unwind_and_suppress_panic_handler<R>(
    f: impl FnOnce() -> R + UnwindSafe,
) -> Result<R, Box<dyn Any + Send + 'static>> {
    if !ENV_CONFIG.reenable_panic_hook {
        std::panic::set_hook(Box::new(|_| {}));
    }

    let result = std::panic::catch_unwind(f);

    if !ENV_CONFIG.reenable_panic_hook {
        let _ = std::panic::take_hook();
    }

    result
}
