use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::panic::AssertUnwindSafe;
use std::panic::UnwindSafe;

use itertools::enumerate;
use log::debug;
use wasm::function_ref::FunctionRef;
use wasm::ExportInst;
use wasm::RuntimeError;
use wasm::Value;
use wasm::{validate, RuntimeInstance};
use wast::core::WastArgCore;
use wast::core::WastRetCore;
use wast::QuoteWat;
use wast::WastArg;

use crate::specification::reports::*;
use crate::specification::test_errors::*;

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

pub fn to_wasm_testsuite_string(runtime_error: RuntimeError) -> Result<String, Box<dyn Error>> {
    let not_represented = Err(GenericError::new_boxed(
        "Runtime error not represented in WAST",
    ));

    match runtime_error {
        RuntimeError::DivideBy0 => Ok("integer divide by zero"),
        RuntimeError::UnrepresentableResult => Ok("integer overflow"),
        RuntimeError::FunctionNotFound => not_represented,
        RuntimeError::StackSmash => not_represented,
        RuntimeError::BadConversionToInteger => Ok("invalid conversion to integer"),

        RuntimeError::MemoryAccessOutOfBounds => Ok("out of bounds memory access"),
        RuntimeError::TableAccessOutOfBounds => Ok("out of bounds table access"),
        RuntimeError::ElementAccessOutOfBounds => not_represented,

        RuntimeError::UninitializedElement => Ok("uninitialized element"),
        RuntimeError::SignatureMismatch => Ok("indirect call type mismatch"),
        RuntimeError::ExpectedAValueOnTheStack => not_represented,

        RuntimeError::UndefinedTableIndex => Ok("undefined element"),
        RuntimeError::ModuleNotFound => Ok("module not found"),
        RuntimeError::UnmetImport => Ok("unmet import"),
    }
    .map(|s| s.to_string())
}

/// Attempt to unwrap the result of an expression. If the expression is an `Err`, then `return` the
/// error.
///
/// # Motivation
/// The `Try` trait is not yet stable, so we define our own macro to simulate the `Result` type.
macro_rules! try_to {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => return err,
        }
    };
}

/// Clear the bytes and runtime instance before calling this function
fn encode(modulee: &mut wast::QuoteWat) -> Result<Vec<u8>, Box<dyn Error>> {
    match &modulee {
        QuoteWat::QuoteComponent(..) | QuoteWat::Wat(wast::Wat::Component(..)) => {
            return Err(GenericError::new_boxed(
                "Component modules are not supported",
            ))
        }
        QuoteWat::Wat(..) | QuoteWat::QuoteModule(..) => (),
    };

    let inner_bytes = modulee.encode().map_err(Box::new)?;
    Ok(inner_bytes)
}

fn validate_instantiate<'a, 'b: 'a>(
    interpreter: &'a mut RuntimeInstance<'b>,
    bytes: &'b [u8],
) -> Result<(), Box<dyn Error>> {
    let validation_info = catch_unwind_and_suppress_panic_handler(|| validate(bytes))
        .map_err(PanicError::from_panic_boxed)?
        .map_err(WasmInterpreterError::new_boxed)?;

    // TODO change hacky hidden name that uses interpreter internals
    let module_name = format!(
        "module_{}",
        interpreter.store.as_ref().unwrap().modules.len()
    );
    catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
        interpreter.add_module(module_name.as_str(), &validation_info)
    }))
    .map_err(PanicError::from_panic_boxed)?
    .map_err(WasmInterpreterError::new_boxed)?;

    Ok(())
}

pub fn run_spec_test(filepath: &str) -> WastTestReport {
    // -=-= Initialization =-=-
    let arena = bumpalo::Bump::new();
    debug!("{}", filepath);

    let mut visible_modules = HashMap::new();

    let contents =
        try_to!(
            std::fs::read_to_string(filepath).map_err(|err| ScriptError::new_lineless(
                filepath,
                Box::new(err),
                "failed to open wast file",
            )
            .compile_report())
        );

    let buf = try_to!(wast::parser::ParseBuffer::new(&contents).map_err(|err| {
        ScriptError::new_lineless(filepath, Box::new(err), "failed to create wast buffer")
            .compile_report()
    }));

    let wast =
        try_to!(
            wast::parser::parse::<wast::Wast>(&buf).map_err(|err| ScriptError::new_lineless(
                filepath,
                Box::new(err),
                "failed to parse wast file"
            )
            .compile_report())
        );

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
        RuntimeInstance::new_named("spectest", &spectest_validation_info)
    })
    .unwrap()
    .unwrap();

    for (i, directive) in enumerate(wast.directives) {
        debug!("at directive {:?}", i);
        match directive {
            wast::WastDirective::Wat(mut quoted) => {
                // If we fail to compile or to validate the main module, then we should treat this
                // as a fatal (compilation) error.
                let wasm_bytes = try_to!(encode(&mut quoted).map_err(|err| {
                    ScriptError::new(
                        filepath,
                        err,
                        "Module directive (WAT) failed in encoding step.",
                        get_linenum(&contents, quoted.span()),
                        get_command(&contents, quoted.span()),
                    )
                    .compile_report()
                }));

                // retain information of the id of the current wast
                match quoted {
                    QuoteWat::Wat(wast::Wat::Module(wast::core::Module {
                        id: _maybe_id @ Some(id),
                        ..
                    }))
                    | QuoteWat::Wat(wast::Wat::Component(wast::component::Component {
                        id: _maybe_id @ Some(id),
                        ..
                    })) => visible_modules.insert(
                        id.name().to_owned(),
                        interpreter.store.as_ref().unwrap().modules.len(),
                    ),
                    _ => None,
                };

                // re-allocate the wasm bytecode into an arena backed allocation, gifting it a
                // lifetime of the outermost scope in the current function
                let wasm_bytes = arena.alloc_slice_clone(&wasm_bytes) as &[u8];

                try_to!(
                    validate_instantiate(interpreter, wasm_bytes).map_err(|err| {
                        ScriptError::new(
                            filepath,
                            err,
                            "Module directive (WAT) failed in validation or instantiation.",
                            get_linenum(&contents, quoted.span()),
                            get_command(&contents, quoted.span()),
                        )
                        .compile_report()
                    })
                );
            }
            wast::WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                let err_or_panic =
                    execute_assert_return(&visible_modules, interpreter, exec, results);

                match err_or_panic {
                    Ok(()) => {
                        asserts.push_success(WastSuccess::new(
                            get_linenum(&contents, span),
                            get_command(&contents, span),
                        ));
                    }
                    Err(inner) => {
                        asserts.push_error(WastError::new(
                            inner,
                            get_linenum(&contents, span),
                            get_command(&contents, span),
                        ));
                    }
                }
            }
            wast::WastDirective::AssertTrap {
                span,
                exec,
                message,
            } => {
                let err_or_panic =
                    execute_assert_trap(&visible_modules, interpreter, exec, message);

                match err_or_panic {
                    Ok(_) => {
                        asserts.push_success(WastSuccess::new(
                            get_linenum(&contents, span),
                            get_command(&contents, span),
                        ));
                    }
                    Err(inner) => {
                        asserts.push_error(WastError::new(
                            inner,
                            get_linenum(&contents, span),
                            get_command(&contents, span),
                        ));
                    }
                }
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
                let line_number = get_linenum(&contents, span);
                let cmd = get_command(&contents, span);
                let error = GenericError::new_boxed(
                    "Module validated and instantiated successfully, when it shouldn't have",
                );

                match encode(&mut modulee).and_then(|bytes| {
                    let bytes = arena.alloc_slice_clone(&bytes);
                    validate_instantiate(interpreter, bytes)
                }) {
                    Err(_) => asserts.push_success(WastSuccess::new(line_number, cmd)),
                    Ok(_) => asserts.push_error(WastError::new(error, line_number, cmd)),
                };
            }

            wast::WastDirective::Register {
                name,
                module: modulee,
                ..
            } => {
                // TODO this implementation is incorrect, but a correct implementation requires a refactor discussion

                // spec tests tells us to use the last defined module if module name is not specified
                // TODO this ugly chunk might need to be refactored out
                let store = interpreter.store.as_mut().unwrap();
                let module_addr = match modulee {
                    None => store.modules.len() - 1,
                    Some(id) => {
                        log::error!("looking for {:?}\n{:?}", id.name(), store.module_names);
                        visible_modules[id.name()]
                    }
                };
                store.module_names.insert(String::from(name), module_addr);
            }
            wast::WastDirective::AssertUnlinkable { span, .. } => {
                asserts.push_error(WastError::new(
                    GenericError::new_boxed("Assert directive not yet implemented"),
                    get_linenum(&contents, span),
                    get_command(&contents, span),
                ));
            }
            wast::WastDirective::AssertExhaustion {
                span,
                call: _,
                message: _,
            }
            | wast::WastDirective::AssertException { span, exec: _ } => {
                asserts.push_error(WastError::new(
                    GenericError::new_boxed("Assert directive not yet implemented"),
                    get_linenum(&contents, span),
                    get_command(&contents, span),
                ));
            }
            wast::WastDirective::Wait { span, thread: _ } => {
                asserts.push_error(WastError::new(
                    GenericError::new_boxed("Wait directive not yet implemented"),
                    get_linenum(&contents, span),
                    get_command(&contents, span),
                ));
            }
            wast::WastDirective::Invoke(invoke) => {
                let args = invoke
                    .args
                    .into_iter()
                    .map(arg_to_value)
                    .collect::<Vec<_>>();

                let function_ref_attempt =
                    catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                        let store = interpreter.store.as_ref().unwrap();
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
                            .iter()
                            .find_map(|ExportInst { name, value }| {
                                if name != invoke.name {
                                    return None;
                                }
                                match value {
                                    wasm::ExternVal::Func(func_addr) => Some(FunctionRef {
                                        func_addr: *func_addr,
                                    }),
                                    _ => None,
                                }
                            })
                            .ok_or(RuntimeError::FunctionNotFound)
                    }))
                    .map(|result| {
                        result.map_err(|err| {
                            ScriptError::new(
                                filepath,
                                WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err)),
                                "Invoke directive failed to find function",
                                get_linenum(&contents, invoke.span),
                                get_command(&contents, invoke.span),
                            )
                            .compile_report()
                        })
                    });

                let function_ref = match function_ref_attempt {
                    Ok(original_result) => try_to!(original_result),
                    Err(panic) => {
                        return ScriptError::new(
                            filepath,
                            PanicError::from_panic_boxed(panic),
                            "main module validation panicked",
                            get_linenum(&contents, invoke.span),
                            get_command(&contents, invoke.span),
                        )
                        .compile_report();
                    }
                };

                let err_or_panic: Result<_, Box<dyn Error>> =
                    catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                        interpreter.invoke_dynamic_unchecked_return_ty(&function_ref, args)
                    }))
                    .map_err(PanicError::from_panic_boxed)
                    .and_then(|result| {
                        result.map_err(|err| {
                            WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err))
                        })
                    });

                try_to!(err_or_panic.map_err(|inner| ScriptError::new(
                    filepath,
                    inner,
                    "Invoke returned error or panicked",
                    get_linenum(&contents, invoke.span),
                    get_command(&contents, invoke.span)
                )
                .compile_report()));
            }
            wast::WastDirective::Thread(thread) => {
                asserts.push_error(WastError::new(
                    GenericError::new_boxed("Thread directive not yet implemented"),
                    get_linenum(&contents, thread.span),
                    get_command(&contents, thread.span),
                ));
            }
        }
    }

    asserts.compile_report()
}

fn execute_assert_return(
    visible_modules: &HashMap<String, usize>,
    interpreter: &mut RuntimeInstance,
    exec: wast::WastExecute,
    results: Vec<wast::WastRet>,
) -> Result<(), Box<dyn Error>> {
    match exec {
        wast::WastExecute::Invoke(invoke_info) => {
            let args = invoke_info
                .args
                .into_iter()
                .map(arg_to_value)
                .collect::<Vec<_>>();

            let result_vals = results
                .into_iter()
                .map(result_to_value)
                .collect::<Result<Vec<_>, _>>()?;
            let result_types = result_vals
                .iter()
                .map(|val| val.to_ty())
                .collect::<Vec<_>>();

            // spec tests tells us to use the last defined module if module name is not specified
            // TODO this ugly chunk might need to be refactored out
            let func = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let store = interpreter.store.as_ref().unwrap();
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
                    .iter()
                    .find_map(|ExportInst { name, value }| {
                        if name != invoke_info.name {
                            return None;
                        }
                        match value {
                            wasm::ExternVal::Func(func_addr) => Some(FunctionRef {
                                func_addr: *func_addr,
                            }),
                            _ => None,
                        }
                    })
                    .ok_or(RuntimeError::FunctionNotFound)
            }))
            .map_err(PanicError::from_panic_boxed)?
            .map_err(|err| WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err)))?;

            let actual = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.invoke_dynamic(&func, args, &result_types)
            }))
            .map_err(PanicError::from_panic_boxed)?
            .map_err(|err| WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err)))?;

            AssertEqError::assert_eq(actual, result_vals)?;
            Ok(())
        }
        wast::WastExecute::Get {
            span: _,
            module: _,
            global: _,
        } => Err(GenericError::new_boxed(
            "`get` directive inside `assert_return` not yet implemented",
        )),
        wast::WastExecute::Wat(_) => Err(GenericError::new_boxed(
            "`wat` directive inside `assert_return` not yet implemented",
        )),
    }
}

fn execute_assert_trap(
    visible_modules: &HashMap<String, usize>,
    interpreter: &mut RuntimeInstance,
    exec: wast::WastExecute,
    message: &str,
) -> Result<(), Box<dyn Error>> {
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
                let store = interpreter.store.as_ref().unwrap();
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
                    .iter()
                    .find_map(|ExportInst { name, value }| {
                        if name != invoke_info.name {
                            return None;
                        }
                        match value {
                            wasm::ExternVal::Func(func_addr) => Some(FunctionRef {
                                func_addr: *func_addr,
                            }),
                            _ => None,
                        }
                    })
                    .ok_or(RuntimeError::FunctionNotFound)
            }))
            .map_err(PanicError::from_panic_boxed)?
            .map_err(|err| WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err)))?;

            let actual = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                interpreter.invoke_dynamic_unchecked_return_ty(&func, args)
            }))
            .map_err(PanicError::from_panic_boxed)?;

            match actual {
                Ok(_) => Err(GenericError::new_boxed("assert_trap did NOT trap")),
                Err(e) => {
                    let actual = to_wasm_testsuite_string(e)?;
                    let expected = message;

                    if actual.contains(expected)
                        || (expected.contains("uninitialized element 2")
                            && actual.contains("uninitialized element"))
                    {
                        Ok(())
                    } else {
                        Err(GenericError::new_boxed(
                            format!("'assert_trap': Expected '{expected}' - Actual: '{actual}'")
                                .as_str(),
                        ))
                    }
                }
            }
        }
        wast::WastExecute::Get {
            span: _,
            module: _,
            global: _,
        } => Err(GenericError::new_boxed(
            "`get` directive inside `assert_return` not yet implemented",
        )),
        wast::WastExecute::Wat(_) => Err(GenericError::new_boxed(
            "`wat` directive inside `assert_return` not yet implemented",
        )),
    }
}

pub fn arg_to_value(arg: WastArg) -> Value {
    match arg {
        WastArg::Core(core_arg) => match core_arg {
            WastArgCore::I32(val) => Value::I32(val as u32),
            WastArgCore::I64(val) => Value::I64(val as u64),
            WastArgCore::F32(val) => Value::F32(wasm::value::F32(f32::from_bits(val.bits))),
            WastArgCore::F64(val) => Value::F64(wasm::value::F64(f64::from_bits(val.bits))),
            WastArgCore::V128(_) => todo!("`V128` value arguments not yet implemented"),
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
                        _ => todo!("`GC` proposal not yet implemented"),
                    }
                }
            },
            WastArgCore::RefExtern(index) => wasm::value::Value::Ref(wasm::value::Ref::Extern(
                wasm::value::ExternAddr::new(Some(index as usize)),
            )),
            WastArgCore::RefHost(_) => {
                todo!("`RefHost` value arguments not yet implemented")
            }
        },
        WastArg::Component(_) => todo!("`Component` value arguments not yet implemented"),
    }
}

fn result_to_value(result: wast::WastRet) -> Result<Value, Box<dyn Error>> {
    let value = match result {
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
                        _ => todo!("`GC` proposal not yet implemented"),
                    }
                }
            },
            WastRetCore::RefFunc(index) => match index {
                None => unreachable!("Expected a non-null function reference"),
                Some(_index) => {
                    // use wasm::value::*;
                    // Value::Ref(Ref::Func(FuncAddr::new(Some(index))))

                    return Err(GenericError::new_boxed("RefFuncs not yet implemented"));
                }
            },
            other => {
                return Err(Box::new(GenericError::new(&format!(
                    "handling of wast ret type {other:?} not yet implemented"
                ))));
            }
        },
        wast::WastRet::Component(_) => todo!("`Component` result not yet implemented"),
    };

    Ok(value)
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
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(f);
    let _ = std::panic::take_hook();
    result
}
