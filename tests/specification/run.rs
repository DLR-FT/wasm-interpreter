use std::any::Any;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::panic::UnwindSafe;

use bumpalo::Bump;
use itertools::enumerate;
use log::debug;
use wasm::addrs::ModuleAddr;
use wasm::function_ref::FunctionRef;
use wasm::ExternVal;
use wasm::RefType;
use wasm::RuntimeError;
use wasm::TrapError;
use wasm::Value;
use wasm::{validate, RuntimeInstance};
use wast::core::WastArgCore;
use wast::core::WastRetCore;
use wast::QuoteWat;
use wast::WastArg;
use wast::WastDirective;
use wast::WastRet;
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
    modules: &mut Vec<ModuleAddr>,
) -> Result<(), WastError> {
    let validation_info =
        catch_unwind_and_suppress_panic_handler(|| validate(bytes)).map_err(WastError::Panic)??;

    // TODO change hacky hidden name that uses interpreter internals
    let module_name = format!("module_{}", modules.len());
    let module = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
        interpreter.add_module(module_name.as_str(), &validation_info)
    }))
    .map_err(WastError::Panic)??;

    modules.push(module);

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
    let (mut interpreter, spectest_module) = catch_unwind_and_suppress_panic_handler(|| {
        RuntimeInstance::new_named((), "spectest", &spectest_validation_info)
    })
    .unwrap()
    .unwrap();

    // A list of all modules
    //
    // We keep this list because linking is currently still done automatically during instantiation.
    // However, the spectests expect linking to be a separate operation from instantiation.
    let mut modules = vec![spectest_module];

    for (i, directive) in enumerate(wast.directives) {
        debug!("at directive {:?}", i);

        let directive_result = run_directive(
            directive,
            &arena,
            &mut visible_modules,
            &mut interpreter,
            &contents,
            filepath,
            &mut modules,
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
    visible_modules: &mut HashMap<String, ModuleAddr>,
    interpreter: &mut RuntimeInstance<'a>,
    contents: &str,
    filepath: &str,
    modules: &mut Vec<ModuleAddr>,
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

            // re-allocate the wasm bytecode into an arena backed allocation, gifting it a
            // lifetime of the outermost scope in the current function
            let wasm_bytes = arena.alloc_slice_clone(&wasm_bytes) as &[u8];

            validate_instantiate(interpreter, wasm_bytes, modules).map_err(|err| {
                ScriptError::new(
                    filepath,
                    err,
                    "Module directive (WAT) failed in validation or instantiation.",
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
                })) => visible_modules.insert(id.name().to_owned(), *modules.last().unwrap()),
                _ => None,
            };

            Ok(None)
        }
        wast::WastDirective::AssertReturn {
            span,
            exec,
            results,
        } => {
            let err_or_panic =
                execute_assert_return(visible_modules, modules, interpreter, exec, results);

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
            let result = execute(arena, visible_modules, modules, interpreter, exec);
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
                validate_instantiate(interpreter, bytes, modules)
            });

            let maybe_assert_error = match result {
                Ok(()) => Some(WastError::AssertInvalidButValid),
                Err(panic_err @ WastError::Panic(_)) => {
                    return Err(ScriptError::new(
                        filepath,
                        panic_err,
                        "Module directive (WAT) failed in validation or instantiation.",
                        line_number,
                        cmd,
                    ))
                }
                Err(_other) => None,
            };

            Ok(Some(AssertOutcome {
                line_number,
                command: cmd.to_owned(),
                maybe_error: maybe_assert_error,
            }))
        }

        wast::WastDirective::Register {
            name,
            module: modulee,
            span,
        } => {
            // TODO this implementation is incorrect, but a correct implementation requires a refactor discussion

            // spec tests tells us to use the last defined module if module name is not specified
            let module = match modulee {
                None => modules.last().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(ScriptError::new(
                filepath,
                WastError::UnknownModuleReferenced,
                "Register directive (WAT) failed",
                get_linenum(contents, span),
                get_command(contents, span),
            ))?;

            interpreter
                .store
                .reregister_module_unchecked(module, name)
                .map_err(|runtime_error| {
                    ScriptError::new(
                        filepath,
                        WastError::WasmRuntimeError(runtime_error),
                        "Register directive (WAT) failed",
                        get_linenum(contents, span),
                        get_command(contents, span),
                    )
                })?;

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

            let result = match validate_instantiate(interpreter, bytes, modules) {
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
                modules,
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

            // spec tests tells us to use the last defined module if module name is not specified
            let module = match invoke.module {
                None => modules.last().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(ScriptError::new(
                filepath,
                WastError::UnknownModuleReferenced,
                "Invoke directive (WAT) failed",
                get_linenum(contents, invoke.span),
                get_command(contents, invoke.span),
            ))?;

            let function_ref = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let func_addr = interpreter
                    .store
                    .instance_export_unchecked(module, invoke.name)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        ExternVal::Func(func) => Ok(func),
                        _ => Err(WastError::UnknownFunctionReferenced),
                    })
                    .map_err(|err| {
                        ScriptError::new(
                            filepath,
                            err,
                            "Invoke Wast directive",
                            get_linenum(contents, invoke.span),
                            get_command(contents, invoke.span),
                        )
                    })?;

                Ok(FunctionRef { func_addr })
            }))
            .map_err(|panic_error| {
                ScriptError::new(
                    filepath,
                    WastError::Panic(panic_error),
                    "main module validation panicked",
                    get_linenum(contents, invoke.span),
                    get_command(contents, invoke.span),
                )
            })??;

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
    visible_modules: &HashMap<String, ModuleAddr>,
    modules: &[ModuleAddr],
    interpreter: &mut RuntimeInstance,
    exec: wast::WastExecute,
    results: Vec<wast::WastRet>,
) -> Result<(), WastError> {
    match exec {
        wast::WastExecute::Invoke(invoke_info) => {
            let args = invoke_info
                .args
                .into_iter()
                .map(arg_to_value)
                .collect::<Vec<_>>();

            let result_vals = results
                .into_iter()
                .map(|ret| match ret {
                    WastRet::Core(ret_core) => ret_core,
                    WastRet::Component(_) => {
                        unimplemented!("wasm components are not supported")
                    }
                })
                .collect::<Vec<WastRetCore>>();

            // spec tests tells us to use the last defined module if module name is not specified
            let module = match invoke_info.module {
                None => modules.last().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(WastError::UnknownModuleReferenced)?;

            let function_ref = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let func_addr = interpreter
                    .store
                    .instance_export_unchecked(module, invoke_info.name)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        ExternVal::Func(func) => Ok(func),
                        _ => Err(WastError::UnknownFunctionReferenced),
                    })?;

                Ok::<FunctionRef, WastError>(FunctionRef { func_addr })
            }))
            .map_err(WastError::Panic)??;

            let actual = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.invoke(&function_ref, args)
            }))
            .map_err(WastError::Panic)??;

            assert_eq(actual, result_vals).map_err(Into::into)
        }
        wast::WastExecute::Get {
            span: _,
            module,
            global,
        } => {
            let result_vals = results
                .into_iter()
                .map(|ret| match ret {
                    WastRet::Core(ret_core) => ret_core,
                    WastRet::Component(_) => {
                        unimplemented!("wasm components are not supported")
                    }
                })
                .collect::<Vec<WastRetCore>>();

            // spec tests tells us to use the last defined module if module name is not specified
            let module = match module {
                None => modules.last().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(WastError::UnknownModuleReferenced)?;

            let actual = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let global_addr = interpreter
                    .store
                    .instance_export_unchecked(module, global)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        ExternVal::Global(global) => Ok(global),
                        _ => Err(WastError::UnknownGlobalReferenced),
                    })?;

                Ok::<Value, WastError>(interpreter.store.global_read_unchecked(global_addr))
            }))
            .map_err(WastError::Panic)??;

            assert_eq(vec![actual], result_vals).map_err(Into::into)
        }
        wast::WastExecute::Wat(_) => todo!("`wat` directive inside `assert_return`"),
    }
}

fn execute<'a>(
    arena: &'a bumpalo::Bump,
    visible_modules: &HashMap<String, ModuleAddr>,
    modules: &mut Vec<ModuleAddr>,
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
            let module = match invoke_info.module {
                None => modules.last().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(WastError::UnknownModuleReferenced)?;

            let function_ref = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let func_addr = interpreter
                    .store
                    .instance_export_unchecked(module, invoke_info.name)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        ExternVal::Func(func) => Ok(func),
                        _ => Err(WastError::UnknownFunctionReferenced),
                    })?;

                Ok::<FunctionRef, WastError>(FunctionRef { func_addr })
            }))
            .map_err(WastError::Panic)??;

            catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.invoke(&function_ref, args)
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

            let module_name = format!("module_{}", modules.len());
            let module_addr = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.add_module(module_name.as_str(), &validation_info)
            }))
            .map_err(WastError::Panic)??;
            modules.push(module_addr);

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
            WastArgCore::V128(val) => Value::V128(val.to_le_bytes()),
            WastArgCore::RefNull(rref) => match rref {
                wast::core::HeapType::Concrete(_) => {
                    unreachable!("Null refs don't point to any specific reference")
                }
                wast::core::HeapType::Abstract { shared: _, ty } => {
                    use wasm::value::*;
                    use wast::core::AbstractHeapType::*;
                    match ty {
                        Func => Value::Ref(Ref::Null(RefType::FuncRef)),
                        Extern => Value::Ref(Ref::Null(RefType::ExternRef)),
                        _ => todo!("`GC` proposal"),
                    }
                }
            },
            WastArgCore::RefExtern(index) => wasm::value::Value::Ref(wasm::value::Ref::Extern(
                wasm::value::ExternAddr(index as usize),
            )),
            WastArgCore::RefHost(_) => {
                todo!("`RefHost` value arguments")
            }
        },
        WastArg::Component(_) => todo!("`Component` value arguments"),
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
