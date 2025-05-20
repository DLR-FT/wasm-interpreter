/*
# This file incorporates code from Wasmtime, originally
# available at https://github.com/bytecodealliance/wasmtime.
#
# The original code is licensed under the Apache License, Version 2.0
# (the "License"); you may not use this file except in compliance
# with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
*/
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
use wast::WastArg;

use crate::specification::reports::*;
use crate::specification::test_errors::*;

pub fn to_wasm_testsuite_string(runtime_error: RuntimeError) -> std::string::String {
    match runtime_error {
        RuntimeError::DivideBy0 => "integer divide by zero",
        RuntimeError::UnrepresentableResult => "integer overflow",
        RuntimeError::FunctionNotFound => todo!(),
        RuntimeError::StackSmash => todo!(),
        RuntimeError::BadConversionToInteger => "invalid conversion to integer",

        RuntimeError::MemoryAccessOutOfBounds => "out of bounds memory access",
        RuntimeError::TableAccessOutOfBounds => "out of bounds table access",
        RuntimeError::ElementAccessOutOfBounds => todo!(),

        RuntimeError::UninitializedElement => "uninitialized element",
        RuntimeError::SignatureMismatch => "indirect call type mismatch",
        RuntimeError::ExpectedAValueOnTheStack => todo!(),

        RuntimeError::UndefinedTableIndex => "undefined element",
        RuntimeError::ModuleNotFound => "module not found",
        RuntimeError::UnmetImport => "unmet import",
    }
    .to_string()
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

#[derive(Debug)]
enum ErrEVI {
    Encode,
    Validate,
    Instantiate,
}

impl std::fmt::Display for ErrEVI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for ErrEVI {}

/// Clear the bytes and runtime instance before calling this function
fn encode_validate_instantiate<'a>(
    module: &mut wast::QuoteWat,
    bytes: &'a mut Option<Vec<u8>>,
    runtime_inst: &'a mut Option<RuntimeInstance<'a>>,
) -> Result<(), ErrEVI> {
    use wast::*;
    let is_module = match &module {
        QuoteWat::QuoteComponent(..) | QuoteWat::Wat(Wat::Component(..)) => false,
        QuoteWat::Wat(..) | QuoteWat::QuoteModule(..) => true,
    };

    if is_module {
        let inner_bytes = module.encode();

        match inner_bytes {
            Err(_) => Err(ErrEVI::Encode),
            Ok(inner_bytes) => {
                bytes.replace(inner_bytes);
                let validation_info_attempt = catch_unwind(|| validate(bytes.as_ref().unwrap()));

                match validation_info_attempt {
                    Err(_) => Err(ErrEVI::Validate),
                    Ok(validation_info) => match validation_info {
                        Err(_) => Err(ErrEVI::Validate),
                        Ok(validation_info) => {
                            let runtime_instance_result =
                                catch_unwind(|| RuntimeInstance::new(&validation_info));

                            match runtime_instance_result {
                                Err(_) => Err(ErrEVI::Instantiate),
                                Ok(runtime_instance) => match runtime_instance {
                                    Err(_) => Err(ErrEVI::Instantiate),
                                    Ok(runtime_instance) => {
                                        runtime_inst.replace(runtime_instance);
                                        Ok(())
                                    }
                                },
                            }
                        }
                    },
                }
            }
        }
    } else {
        Err(ErrEVI::Encode)
    }
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

    // We need to keep the wasm_bytes in-scope for the lifetime of the interpreter.
    // As such, we hoist the bytes into an Option, and assign it once a module directive is found.
    #[allow(unused_assignments)]
    let mut wasm_bytes: Option<Vec<u8>> = None;
    let mut interpreter = None;

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

                let runtime_instance = catch_unwind(|| RuntimeInstance::new(&validation_info));

                let instance = match runtime_instance {
                    Err(_) => {
                        return CompilationError::new(
                            Box::new(ErrEVI::Instantiate),
                            filepath,
                            "failed to create runtime instance",
                        )
                        .compile_report();
                    }
                    Ok(instance_result) => match instance_result {
                        Err(e) => {
                            return CompilationError::new(
                                Box::new(WasmInterpreterError(e)),
                                filepath,
                                "failed to create runtime instance",
                            )
                            .compile_report()
                        }
                        Ok(instance) => instance,
                    },
                };

                interpreter = Some(instance);
            }
            wast::WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                if interpreter.is_none() {
                    return CompilationError::new(
                        Box::new(GenericError::new(
                            "Attempted to assert before module directive",
                        )),
                        filepath,
                        "no module directive found",
                    )
                    .compile_report();
                }

                let interpreter = interpreter.as_mut().unwrap();

                let err_or_panic: Result<(), Box<dyn Error>> =
                    match catch_unwind(AssertUnwindSafe(|| {
                        execute_assert_return(interpreter, exec, results)
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
            wast::WastDirective::AssertTrap {
                span,
                exec,
                message,
            } => {
                if interpreter.is_none() {
                    return CompilationError::new(
                        Box::new(GenericError::new(
                            "Attempted to assert before module directive",
                        )),
                        filepath,
                        "no module directive found",
                    )
                    .compile_report();
                }

                let interpreter = interpreter.as_mut().unwrap();

                let err_or_panic: Result<(), Box<dyn Error>> =
                    match catch_unwind(AssertUnwindSafe(|| {
                        execute_assert_trap(interpreter, exec, message)
                    })) {
                        Err(inner) => {
                            // TODO: Do we want to exit on panic? State may be left in an inconsistent state, and cascading panics may occur.
                            if let Ok(msg) = inner.downcast::<&str>() {
                                Err(Box::new(PanicError::new(&msg)))
                            } else {
                                Err(Box::new(PanicError::new("Unknown panic")))
                            }
                        }
                        Ok(original_result) => original_result,
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
                mut module,
                message: _,
            } => {
                let line_number = span.linecol_in(&contents).0 as u32 + 1;
                let cmd = get_command(&contents, span);

                match encode_validate_instantiate(&mut module, &mut None, &mut None) {
                    Err(_) => asserts.push_success(WastSuccess::new(line_number, cmd)),
                    Ok(_) => asserts.push_error(WastError::new("Module validated and instantiated successfully, when it shouldn't have been".into(), line_number, cmd))
                };
            }

            wast::WastDirective::AssertInvalid {
                span,
                mut module,
                message: _,
            } => {
                let line_number = span.linecol_in(&contents).0 as u32 + 1;
                let cmd = get_command(&contents, span);

                match encode_validate_instantiate(&mut module, &mut None, &mut None) {
                    Err(_) => asserts.push_success(WastSuccess::new(line_number, cmd)),
                    Ok(_) => asserts.push_error(WastError::new("Module validated and instantiated successfully, when it shouldn't have been".into(), line_number, cmd))
                };
            }
            wast::WastDirective::Register {
                span,
                name: _,
                module: _,
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
                match interpreter {
                    None => asserts.push_error(WastError::new(
                        Box::new(GenericError::new(
                            "Couldn't invoke, interpreter not present",
                        )),
                        u32::MAX,
                        "invoke",
                    )),
                    Some(ref mut interpreter) => {
                        let args = invoke
                            .args
                            .into_iter()
                            .map(arg_to_value)
                            .collect::<Vec<_>>();

                        // TODO: more modules ¯\_(ツ)_/¯
                        match interpreter.get_function_by_name(DEFAULT_MODULE, invoke.name) {
                            Err(_) => asserts.push_error(WastError::new(
                                Box::new(GenericError::new(&format!(
                                    "Couldn't get the function '{}' from module '{}'",
                                    invoke.name, "DEFAULT_MODULE"
                                ))),
                                u32::MAX,
                                "invoke",
                            )),
                            Ok(funcref) => {
                                match interpreter.invoke_dynamic_unchecked_return_ty(&funcref, args)
                                {
                                    Err(e) => asserts.push_error(WastError::new(
                                        Box::new(GenericError::new(&format!(
                                            "failed to execute function '{}' from module '{}' - error: {:?}",
                                            invoke.name, "DEFAULT_MODULE", e
                                        ))),
                                        u32::MAX,
                                        "invoke",
                                    )),
                                    Ok(_) => {
                                        asserts.push_success(WastSuccess::new(u32::MAX, "invoke"))
                                    }
                                }
                            }
                        };
                    }
                };
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

            let result_vals = results.into_iter().map(result_to_value).collect::<Vec<_>>();
            let result_types = result_vals
                .iter()
                .map(|val| val.to_ty())
                .collect::<Vec<_>>();

            // TODO: more modules ¯\_(ツ)_/¯
            let func = interpreter
                .get_function_by_name(DEFAULT_MODULE, invoke_info.name)
                .map_err(|err| Box::new(WasmInterpreterError(wasm::Error::RuntimeError(err))))?;

            let actual = interpreter
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

fn execute_assert_trap(
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

            // TODO: more modules ¯\_(ツ)_/¯
            let func_res = interpreter
                .get_function_by_name(DEFAULT_MODULE, invoke_info.name)
                .map_err(|err| Box::new(WasmInterpreterError(wasm::Error::RuntimeError(err))));

            let func: FunctionRef;
            match func_res {
                Err(e) => {
                    return Err(e);
                }
                Ok(func_ref) => func = func_ref,
            };

            let actual = interpreter.invoke_dynamic_unchecked_return_ty(&func, args);

            match actual {
                Ok(_) => Err(Box::new(GenericError::new("assert_trap did NOT trap"))),
                Err(e) => {
                    let actual = to_wasm_testsuite_string(e);
                    let expected = message;

                    if actual.contains(expected)
                        || (expected.contains("uninitialized element 2")
                            && actual.contains("uninitialized element"))
                    {
                        Ok(())
                    } else {
                        Err(Box::new(GenericError::new(
                            format!("'assert_trap': Expected '{expected}' - Actual: '{actual}'")
                                .as_str(),
                        )))
                    }
                }
            }
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

pub fn get_command(contents: &str, span: wast::token::Span) -> &str {
    contents[span.offset()..]
        .lines()
        .next()
        .unwrap_or("<unknown>")
}
