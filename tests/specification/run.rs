use std::error::Error;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

use wasm::function_ref::FunctionRef;
use wasm::RuntimeError;
use wasm::Value;
use wasm::DEFAULT_MODULE;
use wasm::{validate, RuntimeInstance};
use wast::core::WastArgCore;
use wast::core::WastRetCore;
use wast::QuoteWat;
use wast::WastArg;

use crate::specification::reports::*;
use crate::specification::test_errors::*;

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
fn encode(module: &mut wast::QuoteWat) -> Result<Vec<u8>, Box<dyn Error>> {
    match &module {
        QuoteWat::QuoteComponent(..) | QuoteWat::Wat(wast::Wat::Component(..)) => {
            return Err(GenericError::new_boxed(
                "Component modules are not supported",
            ))
        }
        QuoteWat::Wat(..) | QuoteWat::QuoteModule(..) => (),
    };

    let inner_bytes = module.encode().map_err(|err| Box::new(err))?;
    Ok(inner_bytes)
}

fn validate_instantiate<'a>(bytes: &'a [u8]) -> Result<RuntimeInstance<'a>, Box<dyn Error>> {
    let validation_info_attempt =
        catch_unwind(|| validate(bytes)).map_err(|panic| PanicError::from_panic_boxed(panic))?;

    let validation_info =
        validation_info_attempt.map_err(|err| WasmInterpreterError::new_boxed(err))?;

    let runtime_instance_attempt = catch_unwind(|| RuntimeInstance::new(&validation_info))
        .map_err(|panic| PanicError::from_panic_boxed(panic))?;

    let runtime_instance =
        runtime_instance_attempt.map_err(|err| WasmInterpreterError::new_boxed(err))?;

    Ok(runtime_instance)
}

pub fn run_spec_test(filepath: &str) -> WastTestReport {
    // -=-= Initialization =-=-
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
                wasm_bytes = Some(try_to!(encode(&mut quoted).map_err(|err| {
                    ScriptError::new(
                        filepath,
                        err,
                        "Module directive (WAT) failed in encoding step.",
                        get_linenum(&contents, quoted.span()),
                        get_command(&contents, quoted.span()),
                    )
                    .compile_report()
                })));

                interpeter = Some(try_to!(validate_instantiate(wasm_bytes.as_ref().unwrap())
                    .map_err(|err| {
                        ScriptError::new(
                            filepath,
                            err,
                            "Module directive (WAT) failed in validation or instantiation.",
                            get_linenum(&contents, quoted.span()),
                            get_command(&contents, quoted.span()),
                        )
                        .compile_report()
                    })));
            }
            wast::WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                if interpeter.is_none() {
                    return ScriptError::new(
                        filepath,
                        GenericError::new_boxed("Attempted to assert before module directive"),
                        "Assert Return",
                        get_linenum(&contents, span),
                        get_command(&contents, span),
                    )
                    .compile_report();
                }

                let interpeter = interpeter.as_mut().unwrap();

                let err_or_panic = catch_unwind(AssertUnwindSafe(|| {
                    execute_assert_return(interpeter, exec, results)
                }))
                .map_err(PanicError::from_panic_boxed)
                .and_then(|result| result);

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
                if interpeter.is_none() {
                    return ScriptError::new(
                        filepath,
                        GenericError::new_boxed("Attempted to assert before module directive"),
                        "Assert Trap",
                        get_linenum(&contents, span),
                        get_command(&contents, span),
                    )
                    .compile_report();
                }

                let interpeter = interpeter.as_mut().unwrap();

                let err_or_panic: Result<(), Box<dyn Error>> =
                    catch_unwind(AssertUnwindSafe(|| {
                        execute_assert_trap(interpeter, exec, message)
                    }))
                    .map_err(PanicError::from_panic_boxed)
                    .and_then(|result| result);

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
                mut module,
                message: _,
            } => {
                let line_number = get_linenum(&contents, span);
                let cmd = get_command(&contents, span);
                let error = GenericError::new_boxed(
                    "Module validated and instantiated successfully, when it shouldn't have",
                );

                match encode(&mut module).and_then(|bytes| validate_instantiate(&bytes).map(|_| ()))
                {
                    Err(_) => asserts.push_success(WastSuccess::new(line_number, cmd)),
                    Ok(_) => asserts.push_error(WastError::new(error, line_number, cmd)),
                };
            }

            wast::WastDirective::AssertInvalid {
                span,
                mut module,
                message: _,
            } => {
                let line_number = get_linenum(&contents, span);
                let cmd = get_command(&contents, span);
                let error = GenericError::new_boxed(
                    "Module validated and instantiated successfully, when it shouldn't have",
                );

                match encode(&mut module).and_then(|bytes| validate_instantiate(&bytes).map(|_| ()))
                {
                    Err(_) => asserts.push_success(WastSuccess::new(line_number, cmd)),
                    Ok(_) => asserts.push_error(WastError::new(error, line_number, cmd)),
                };
            }
            wast::WastDirective::Register {
                span,
                name: _,
                module: _,
            } => {
                asserts.push_error(WastError::new(
                    GenericError::new_boxed("Register directive not yet implemented"),
                    get_linenum(&contents, span),
                    get_command(&contents, span),
                ));
            }
            wast::WastDirective::AssertExhaustion {
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
                if interpeter.is_none() {
                    return ScriptError::new(
                        filepath,
                        GenericError::new_boxed(
                            "Attempted to run invoke directive before interpreter instantiation.",
                        ),
                        "Invoke",
                        get_linenum(&contents, invoke.span),
                        get_command(&contents, invoke.span),
                    )
                    .compile_report();
                }

                let interpeter = interpeter.as_mut().unwrap();

                let args = invoke
                    .args
                    .into_iter()
                    .map(arg_to_value)
                    .collect::<Vec<_>>();

                let function_ref_attempt = catch_unwind(AssertUnwindSafe(|| {
                    interpeter
                        .get_function_by_name(DEFAULT_MODULE, invoke.name)
                        .map_err(|err| {
                            ScriptError::new(
                                filepath,
                                WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err)),
                                "Invoke directive failed to find function",
                                get_linenum(&contents, invoke.span),
                                get_command(&contents, invoke.span),
                            )
                            .compile_report()
                        })
                }));

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
                    catch_unwind(AssertUnwindSafe(|| {
                        interpeter.invoke_dynamic_unchecked_return_ty(&function_ref, args)
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
                .map_err(|err| WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err)))?;

            let actual = interpeter
                .invoke_dynamic(&func, args, &result_types)
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
    interpeter: &mut RuntimeInstance,
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

            // TODO: more modules ¯\_(ツ)_/¯
            let func_res = interpeter
                .get_function_by_name(DEFAULT_MODULE, invoke_info.name)
                .map_err(|err| WasmInterpreterError::new_boxed(wasm::Error::RuntimeError(err)));

            let func: FunctionRef;
            match func_res {
                Err(e) => {
                    return Err(e);
                }
                Ok(func_ref) => func = func_ref,
            };

            let actual = interpeter.invoke_dynamic_unchecked_return_ty(&func, args);

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
            WastRetCore::V128(_) => todo!("`V128` result not yet implemented"),
            WastRetCore::RefNull(rref) => match rref {
                None => todo!("RefNull with no type not yet implemented"),
                Some(rref) => match rref {
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
            },
            WastRetCore::RefExtern(_) => {
                todo!("`RefExtern` result not yet implemented")
            }
            WastRetCore::RefHost(_) => todo!("`RefHost` result not yet implemented"),
            WastRetCore::RefFunc(index) => match index {
                None => unreachable!("Expected a non-null function reference"),
                Some(_index) => {
                    // use wasm::value::*;
                    // Value::Ref(Ref::Func(FuncAddr::new(Some(index))))
                    todo!("RefFuncs not yet implemented")
                }
            },
            //todo!("`RefFunc` result not yet implemented"),
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

pub fn get_linenum(contents: &str, span: wast::token::Span) -> u32 {
    span.linecol_in(&contents).0 as u32 + 1
}

pub fn get_command(contents: &str, span: wast::token::Span) -> &str {
    contents[span.offset()..]
        .lines()
        .next()
        .unwrap_or("<unknown>")
}
