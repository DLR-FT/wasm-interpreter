use std::error::Error;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

use wasm::Value;
use wasm::DEFAULT_MODULE;
use wasm::{validate, RuntimeInstance};
use wast::core::WastArgCore;
use wast::core::WastRetCore;
use wast::WastArg;

use crate::specification::reports::*;
use crate::specification::test_errors::*;

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

pub fn run_spec_test(filepath: &str) -> WastTestReport {
    // -=-= Initialization =-=-
    let contents = try_to!(
        std::fs::read_to_string(filepath).map_err(|err| CompilationError::new(
            Box::new(err),
            filepath,
            "failed to open wast file"
        )
        .compile_report())
    );

    let buf =
        try_to!(
            wast::parser::ParseBuffer::new(&contents).map_err(|err| CompilationError::new(
                Box::new(err),
                filepath,
                "failed to create wast buffer"
            )
            .compile_report())
        );

    let wast =
        try_to!(
            wast::parser::parse::<wast::Wast>(&buf).map_err(|err| CompilationError::new(
                Box::new(err),
                filepath,
                "failed to parse wast file"
            )
            .compile_report())
        );

    // -=-= Testing & Compilation =-=-
    let mut asserts = AssertReport::new(filepath);

    // We need to keep the wasm_bytes in-scope for the lifetime of the interpeter.
    // As such, we hoist the bytes into an Option, and assign it once a module directive is found.
    #[allow(unused_assignments)]
    let mut wasm_bytes: Option<Vec<u8>> = None;
    let mut interpeter = None;

    for directive in wast.directives {
        match directive {
            wast::WastDirective::Wat(mut quoted) => {
                // If we fail to compile or to validate the main module, then we should treat this
                // as a fatal (compilation) error.
                let encoded = try_to!(quoted.encode().map_err(|err| CompilationError::new(
                    Box::new(err),
                    filepath,
                    "failed to encode main module's wat"
                )
                .compile_report()));

                wasm_bytes = Some(encoded);

                let validation_attempt = catch_unwind(|| {
                    validate(wasm_bytes.as_ref().unwrap()).map_err(|err| {
                        CompilationError::new(
                            Box::new(WasmInterpreterError(err)),
                            filepath,
                            "main module validation failed",
                        )
                        .compile_report()
                    })
                });

                let validation_info = match validation_attempt {
                    Ok(original_result) => try_to!(original_result),
                    Err(panic) => {
                        // TODO: Do we want to exit on panic? State may be left in an inconsistent state, and cascading panics may occur.
                        let err = if let Ok(msg) = panic.downcast::<&str>() {
                            Box::new(PanicError::new(&msg))
                        } else {
                            Box::new(PanicError::new("Unknown panic"))
                        };

                        return CompilationError::new(
                            err,
                            filepath,
                            "main module validation panicked",
                        )
                        .compile_report();
                    }
                };

                let instance = try_to!(RuntimeInstance::new(&validation_info).map_err(|err| {
                    CompilationError::new(
                        Box::new(WasmInterpreterError(wasm::Error::RuntimeError(err))),
                        filepath,
                        "failed to create runtime instance",
                    )
                    .compile_report()
                }));

                interpeter = Some(instance);
            }
            wast::WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                if interpeter.is_none() {
                    return CompilationError::new(
                        Box::new(GenericError::new(
                            "Attempted to assert before module directive",
                        )),
                        filepath,
                        "no module directive found",
                    )
                    .compile_report();
                }

                let interpeter = interpeter.as_mut().unwrap();

                let err_or_panic: Result<(), Box<dyn Error>> =
                    match catch_unwind(AssertUnwindSafe(|| {
                        execute_assert_return(interpeter, exec, results)
                    })) {
                        Ok(original_result) => original_result,
                        Err(inner) => {
                            // TODO: Do we want to exit on panic? State may be left in an inconsistent state, and cascading panics may occur.
                            if let Ok(msg) = inner.downcast::<&str>() {
                                Err(Box::new(PanicError::new(&msg)))
                            } else {
                                Err(Box::new(PanicError::new("Unknown panic")))
                            }
                        }
                    };

                match err_or_panic {
                    Ok(_) => {
                        asserts.push_success(WastSuccess::new(
                            span.linecol_in(&contents).0 as u32 + 1,
                            get_command(&contents, span),
                        ));
                    }
                    Err(inner) => {
                        asserts.push_error(WastError::new(
                            inner,
                            span.linecol_in(&contents).0 as u32 + 1,
                            get_command(&contents, span),
                        ));
                    }
                }
            }
            wast::WastDirective::AssertMalformed {
                span,
                module: _,
                message: _,
            }
            | wast::WastDirective::AssertInvalid {
                span,
                module: _,
                message: _,
            }
            | wast::WastDirective::Register {
                span,
                name: _,
                module: _,
            }
            | wast::WastDirective::AssertTrap {
                span,
                exec: _,
                message: _,
            }
            | wast::WastDirective::AssertExhaustion {
                span,
                call: _,
                message: _,
            }
            | wast::WastDirective::AssertUnlinkable {
                span,
                module: _,
                message: _,
            }
            | wast::WastDirective::AssertException { span, exec: _ } => {
                asserts.push_error(WastError::new(
                    Box::new(GenericError::new("Assert directive not yet implemented")),
                    span.linecol_in(&contents).0 as u32 + 1,
                    get_command(&contents, span),
                ));
            }
            wast::WastDirective::Wait { span, thread: _ } => {
                asserts.push_error(WastError::new(
                    Box::new(GenericError::new("Wait directive not yet implemented")),
                    span.linecol_in(&contents).0 as u32 + 1,
                    get_command(&contents, span),
                ));
            }
            wast::WastDirective::Invoke(invoke) => {
                asserts.push_error(WastError::new(
                    Box::new(GenericError::new("Invoke directive not yet implemented")),
                    invoke.span.linecol_in(&contents).0 as u32 + 1,
                    get_command(&contents, invoke.span),
                ));
            }
            wast::WastDirective::Thread(thread) => {
                asserts.push_error(WastError::new(
                    Box::new(GenericError::new("Thread directive not yet implemented")),
                    thread.span.linecol_in(&contents).0 as u32 + 1,
                    get_command(&contents, thread.span),
                ));
            }
        }
    }

    asserts.compile_report()
}

fn execute_assert_return(
    interpeter: &mut RuntimeInstance,
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

            let result_vals = results.into_iter().map(result_to_value).collect::<Vec<_>>();
            let result_types = result_vals
                .iter()
                .map(|val| val.to_ty())
                .collect::<Vec<_>>();

            // TODO: more modules ¯\_(ツ)_/¯
            let func = interpeter
                .get_function_by_name(DEFAULT_MODULE, invoke_info.name)
                .map_err(|err| Box::new(WasmInterpreterError(wasm::Error::RuntimeError(err))))?;

            let actual = interpeter
                .invoke_dynamic(&func, args, &result_types)
                .map_err(|err| Box::new(WasmInterpreterError(wasm::Error::RuntimeError(err))))?;

            AssertEqError::assert_eq(actual, result_vals)?;
        }
        wast::WastExecute::Get {
            span: _,
            module: _,
            global: _,
        } => todo!("`get` directive inside `assert_return` not yet implemented"),
        wast::WastExecute::Wat(_) => {
            todo!("`wat` directive inside `assert_return` not yet implemented")
        }
    }

    Ok(())
}

pub fn arg_to_value(arg: WastArg) -> Value {
    match arg {
        WastArg::Core(core_arg) => match core_arg {
            WastArgCore::I32(val) => Value::I32(val as u32),
            WastArgCore::I64(val) => Value::I64(val as u64),
            WastArgCore::F32(val) => Value::F32(wasm::value::F32(val.bits as f32)),
            WastArgCore::F64(val) => Value::F64(wasm::value::F64(val.bits as f64)),
            WastArgCore::V128(_) => todo!("`V128` value arguments not yet implemented"),
            WastArgCore::RefNull(_) => {
                todo!("`RefNull` value arguments not yet implemented")
            }
            WastArgCore::RefExtern(_) => {
                todo!("`RefExtern` value arguments not yet implemented")
            }
            WastArgCore::RefHost(_) => {
                todo!("`RefHost` value arguments not yet implemented")
            }
        },
        WastArg::Component(_) => todo!("`Component` value arguments not yet implemented"),
    }
}

fn result_to_value(result: wast::WastRet) -> Value {
    match result {
        wast::WastRet::Core(core_arg) => match core_arg {
            WastRetCore::I32(val) => Value::I32(val as u32),
            WastRetCore::I64(val) => Value::I64(val as u64),
            WastRetCore::F32(val) => match val {
                wast::core::NanPattern::CanonicalNan => {
                    todo!("`F32::CanonicalNan` result not yet implemented")
                }
                wast::core::NanPattern::ArithmeticNan => {
                    todo!("`F32::ArithmeticNan` result not yet implemented")
                }
                wast::core::NanPattern::Value(val) => Value::F32(wasm::value::F32(val.bits as f32)),
            },
            WastRetCore::F64(val) => match val {
                wast::core::NanPattern::CanonicalNan => {
                    todo!("`F64::CanonicalNan` result not yet implemented")
                }
                wast::core::NanPattern::ArithmeticNan => {
                    todo!("`F64::ArithmeticNan` result not yet implemented")
                }
                wast::core::NanPattern::Value(val) => Value::F64(wasm::value::F64(val.bits as f64)),
            },
            WastRetCore::V128(_) => todo!("`V128` result not yet implemented"),
            WastRetCore::RefNull(_) => todo!("`RefNull` result not yet implemented"),
            WastRetCore::RefExtern(_) => {
                todo!("`RefExtern` result not yet implemented")
            }
            WastRetCore::RefHost(_) => todo!("`RefHost` result not yet implemented"),
            WastRetCore::RefFunc(_) => todo!("`RefFunc` result not yet implemented"),
            WastRetCore::RefAny => todo!("`RefAny` result not yet implemented"),
            WastRetCore::RefEq => todo!("`RefEq` result not yet implemented"),
            WastRetCore::RefArray => todo!("`RefArray` result not yet implemented"),
            WastRetCore::RefStruct => todo!("`RefStruct` result not yet implemented"),
            WastRetCore::RefI31 => todo!("`RefI31` result not yet implemented"),
            WastRetCore::Either(_) => todo!("`Either` result not yet implemented"),
        },
        wast::WastRet::Component(_) => todo!("`Component` result not yet implemented"),
    }
}

pub fn get_command(contents: &str, span: wast::token::Span) -> &str {
    contents[span.offset()..]
        .lines()
        .next()
        .unwrap_or("<unknown>")
}
