use std::error::Error;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

use wasm::Value;
use wasm::{validate, RuntimeInstance};

use crate::specification::reports::*;
use crate::specification::test_errors::*;

macro_rules! try_to {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => return err.into(),
        }
    };
}

pub fn run_spec_test(filepath: &str) -> WastTestReport {
    // -=-= Initialization =-=-
    let contents = try_to!(std::fs::read_to_string(filepath)
        .map_err(|err| WastError::from_outside(Box::new(err), "failed to open wast file")));

    let buf = try_to!(wast::parser::ParseBuffer::new(&contents)
        .map_err(|err| WastError::from_outside(Box::new(err), "failed to create wast buffer")));

    let wast = try_to!(wast::parser::parse::<wast::Wast>(&buf)
        .map_err(|err| WastError::from_outside(Box::new(err), "failed to parse wast file")));

    // -=-= Testing & Compilation =-=-
    let mut asserts = AssertReport::new();

    // We need to keep the wasm_bytes in-scope for the lifetime of the interpeter.
    // As such, we hoist the bytes into an Option, and assign it once a module directive is found.
    #[allow(unused_assignments)]
    let mut wasm_bytes: Option<Vec<u8>> = None;
    let mut interpeter = None;

    for directive in wast.directives {
        match directive {
            wast::WastDirective::Wat(mut quoted) => {
                let encoded = try_to!(quoted
                    .encode()
                    .map_err(|err| WastError::from_outside(Box::new(err), "failed to encode wat")));

                wasm_bytes = Some(encoded);

                let validation_attempt = catch_unwind(|| {
                    validate(wasm_bytes.as_ref().unwrap()).map_err(|err| {
                        WastError::new(
                            Box::<WasmInterpreterError>::new(err.into()),
                            filepath.to_string(),
                            0,
                            "Module validation failed",
                        )
                    })
                });

                let validation_info = match validation_attempt {
                    Ok(original_result) => try_to!(original_result),
                    Err(inner) => {
                        // TODO: Do we want to exit on panic? State may be left in an inconsistent state, and cascading panics may occur.
                        let err = if let Ok(msg) = inner.downcast::<&str>() {
                            Box::new(PanicError::new(msg.to_string()))
                        } else {
                            Box::new(PanicError::new("Unknown panic".into()))
                        };

                        return WastError::new(
                            err,
                            filepath.to_string(),
                            0,
                            "Module validation panicked",
                        )
                        .into();
                    }
                };

                let instance = try_to!(RuntimeInstance::new(&validation_info).map_err(|err| {
                    let err: wasm::Error = err.into();
                    let err: WasmInterpreterError = err.into();
                    WastError::from_outside(Box::new(err), "failed to create runtime instance")
                }));

                interpeter = Some(instance);
            }
            wast::WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                if interpeter.is_none() {
                    asserts.push_error(
                        filepath.to_string(),
                        span.linecol_in(&contents).0 as u32 + 1,
                        get_command(&contents, span),
                        Box::new(GenericError::new(
                            "Attempted to run assert_return before a module was compiled",
                        )),
                    );
                    continue;
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
                                Err(Box::new(PanicError::new(msg.to_string())))
                            } else {
                                Err(Box::new(PanicError::new("Unknown panic".into())))
                            }
                        }
                    };

                match err_or_panic {
                    Ok(_) => {
                        asserts.push_success(
                            filepath.to_string(),
                            span.linecol_in(&contents).0 as u32 + 1,
                            get_command(&contents, span),
                        );
                    }
                    Err(inner) => {
                        asserts.push_error(
                            filepath.to_string(),
                            span.linecol_in(&contents).0 as u32 + 1,
                            get_command(&contents, span),
                            inner,
                        );
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
                asserts.push_error(
                    filepath.to_string(),
                    span.linecol_in(&contents).0 as u32 + 1,
                    get_command(&contents, span),
                    Box::new(GenericError::new("Assert directive not yet implemented")),
                );
            }
            wast::WastDirective::Wait { span: _, thread: _ } => todo!(),
            wast::WastDirective::Invoke(_) => todo!(),
            wast::WastDirective::Thread(_) => todo!(),
        }
    }

    asserts.into()
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

            let actual = interpeter
                .invoke_named_dynamic(invoke_info.name, args, &result_types)
                .map_err(|err| {
                    let err: wasm::Error = err.into();
                    let err: WasmInterpreterError = err.into();
                    Box::new(err)
                })?;

            AssertEqError::assert_eq(actual, result_vals)?;
        }
        wast::WastExecute::Get {
            span: _,
            module: _,
            global: _,
        } => todo!(),
        wast::WastExecute::Wat(_) => todo!(),
    }

    Ok(())
}

pub fn arg_to_value(arg: wast::WastArg) -> Value {
    match arg {
        wast::WastArg::Core(core_arg) => match core_arg {
            wast::core::WastArgCore::I32(val) => Value::I32(val as u32),
            wast::core::WastArgCore::I64(val) => Value::I64(val as u64),
            wast::core::WastArgCore::F32(val) => Value::F32(wasm::value::F32(val.bits as f32)),
            wast::core::WastArgCore::F64(val) => Value::F64(wasm::value::F64(val.bits as f64)),
            wast::core::WastArgCore::V128(_) => todo!(),
            wast::core::WastArgCore::RefNull(_) => todo!(),
            wast::core::WastArgCore::RefExtern(_) => todo!(),
            wast::core::WastArgCore::RefHost(_) => todo!(),
        },
        wast::WastArg::Component(_) => todo!(),
    }
}

fn result_to_value(result: wast::WastRet) -> Value {
    match result {
        wast::WastRet::Core(core_arg) => match core_arg {
            wast::core::WastRetCore::I32(val) => Value::I32(val as u32),
            wast::core::WastRetCore::I64(val) => Value::I64(val as u64),
            wast::core::WastRetCore::F32(val) => match val {
                wast::core::NanPattern::CanonicalNan => todo!(),
                wast::core::NanPattern::ArithmeticNan => todo!(),
                wast::core::NanPattern::Value(val) => Value::F32(wasm::value::F32(val.bits as f32)),
            },
            wast::core::WastRetCore::F64(val) => match val {
                wast::core::NanPattern::CanonicalNan => todo!(),
                wast::core::NanPattern::ArithmeticNan => todo!(),
                wast::core::NanPattern::Value(val) => Value::F64(wasm::value::F64(val.bits as f64)),
            },
            wast::core::WastRetCore::V128(_) => todo!(),
            wast::core::WastRetCore::RefNull(_) => todo!(),
            wast::core::WastRetCore::RefExtern(_) => todo!(),
            wast::core::WastRetCore::RefHost(_) => todo!(),
            wast::core::WastRetCore::RefFunc(_) => todo!(),
            wast::core::WastRetCore::RefAny => todo!(),
            wast::core::WastRetCore::RefEq => todo!(),
            wast::core::WastRetCore::RefArray => todo!(),
            wast::core::WastRetCore::RefStruct => todo!(),
            wast::core::WastRetCore::RefI31 => todo!(),
            wast::core::WastRetCore::Either(_) => todo!(),
        },
        wast::WastRet::Component(_) => todo!(),
    }
}

pub fn get_command(contents: &str, span: wast::token::Span) -> String {
    contents[span.offset() as usize..]
        .lines()
        .next()
        .unwrap_or("<unknown>")
        .to_string()
}
