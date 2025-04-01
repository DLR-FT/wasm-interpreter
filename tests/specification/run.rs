use std::any::Any;
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

use wasm::validate;
use wasm::RuntimeError;
use wasm::Store;
use wasm::Value;
use wast::core::WastArgCore;
use wast::core::WastRetCore;
use wast::WastArg;
use wast::WastInvoke;

use crate::specification::reports::*;
use crate::specification::test_errors::*;

const DEFAULT_MODULE: &'static str = "__interpreter_default__";

const SPEC_TEST_WAT: &'static str = r#"
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

pub fn runtime_err_to_wasm_testsuite_string(runtime_error: RuntimeError) -> std::string::String {
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
        RuntimeError::StoreNotFound => "store not found",
    }
    .to_string()
}

pub fn linker_err_to_wasm_testsuite_string(linker_err: wasm::Error) -> Option<std::string::String> {
    use wasm::Error::*;
    match linker_err {
        InvalidImportType => Some("incompatible import type".to_string()),
        UnknownFunction | UnknownMemory | UnknownGlobal | UnknownTable => {
            Some("unknown import".to_string())
        }
        RuntimeError(rt_err) => Some(runtime_err_to_wasm_testsuite_string(rt_err)),
        _ => None,
    }
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
    // TODO: first: patch all panics in the lib
    //        next: uncomment the store arg and make it Option<Store<'a>>, as you can pass to this function without having to use catch_unwind
    //        this is needed for AssertTrap, see file from testsuite: linking.wast:266 where assert_trap can still fail
    //        also read the message above that assert directive, as it says about v2 that we still keep broken state in store even
    //        if module fails to be added to the store
    // store: &'a mut Option<Store<'a>>,
) -> Result<(), ErrEVI> {
    let store = &mut Some(Store::default());
    use wast::*;
    let is_module = match &module {
        QuoteWat::QuoteComponent(..) | QuoteWat::Wat(Wat::Component(..)) => false,
        QuoteWat::Wat(..) | QuoteWat::QuoteModule(..) => true,
    };
    let module_name = match &module {
        QuoteWat::QuoteComponent(..) | QuoteWat::Wat(Wat::Component(..)) => "".to_owned(),
        QuoteWat::Wat(wat) => match wat {
            wast::Wat::Module(wat_module) => match wat_module.id {
                None => DEFAULT_MODULE.to_owned(),
                Some(name) => name.name().to_owned(),
            },
            _ => unreachable!(),
        },
        QuoteWat::QuoteModule(_span, _vec_of_smth) => {
            // todo!()
            DEFAULT_MODULE.to_owned()
        }
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
                            let store_result = catch_unwind(|| {
                                let mut temp_store = Store::default();
                                let spectest_wat_parsed = wat::parse_str(SPEC_TEST_WAT).unwrap();
                                let spectest = validate(&spectest_wat_parsed).unwrap();

                                temp_store
                                    .add_module("spectest".to_owned(), spectest)
                                    .unwrap();

                                match temp_store
                                    .add_module(DEFAULT_MODULE.to_owned(), validation_info.clone())
                                {
                                    Err(_) => Err(ErrEVI::Instantiate),
                                    Ok(_) => Ok(()),
                                }
                            });

                            match store_result {
                                Err(_) => Err(ErrEVI::Instantiate),
                                Ok(_) => {
                                    let actual_name = if module_name == DEFAULT_MODULE {
                                        // if we already have a default module, change name
                                        if store.as_mut().unwrap().modules.len() > 0
                                            && store.as_mut().unwrap().modules[0].name
                                                == DEFAULT_MODULE
                                        {
                                            store.as_mut().unwrap().modules.len().to_string()
                                        } else {
                                            DEFAULT_MODULE.to_owned()
                                        }
                                    } else {
                                        module_name
                                    };
                                    let spectest_wat_parsed =
                                        wat::parse_str(SPEC_TEST_WAT).unwrap();
                                    let spectest = validate(&spectest_wat_parsed).unwrap();

                                    store
                                        .as_mut()
                                        .unwrap()
                                        .add_module("spectest".to_owned(), spectest)
                                        .unwrap();

                                    store
                                        .as_mut()
                                        .unwrap()
                                        .add_module(actual_name, validation_info)
                                        .map_err(|_| ErrEVI::Instantiate)?;

                                    Ok(())
                                }
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

    // We need to keep the wasm_bytes in-scope for the lifetime of the interpeter.
    // As such, we hoist the bytes into an Option, and assign it once a module directive is found.
    #[allow(unused_assignments)]
    let mut store = Some(Store::default());

    let spectest_wat_parsed = wat::parse_str(SPEC_TEST_WAT).unwrap();
    let spectest = validate(&spectest_wat_parsed).unwrap();

    store
        .as_mut()
        .unwrap()
        .add_module("spectest".to_owned(), spectest)
        .unwrap();

    for directive in wast.directives {
        // println!("Directive: {:?}", directive);
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

                // wasm_bytes_vec.push(encoded);
                let encoded = Box::leak(Box::new(encoded));
                // wasm_bytes = Some(encoded);

                let validation_attempt: Result<
                    Result<wasm::ValidationInfo<'_>, WastTestReport>,
                    Box<dyn Any + Send>,
                > = catch_unwind(|| {
                    validate(encoded).map_err(|err| {
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

                use wast::{QuoteWat, Wat};
                let mut module_name = match &quoted {
                    QuoteWat::QuoteComponent(..) | QuoteWat::Wat(Wat::Component(..)) => {
                        todo!()
                    }
                    QuoteWat::Wat(wat) => match wat {
                        wast::Wat::Module(wat_module) => match wat_module.id {
                            None => DEFAULT_MODULE.to_owned(),
                            Some(name) => name.name().to_owned(),
                        },
                        _ => unreachable!(),
                    },
                    QuoteWat::QuoteModule(_span, _vec_of_smth) => DEFAULT_MODULE.to_owned(),
                };
                if store.as_ref().unwrap().modules.len() > 1 && module_name == DEFAULT_MODULE {
                    module_name = store.as_ref().unwrap().modules.len().to_string();
                }

                // let store = catch_unwind(|| RuntimeInstance::new(&validation_info));
                // let spectest_wat_parsed = wat::parse_str(SPEC_TEST_WAT).unwrap();
                // let spectest = validate(&spectest_wat_parsed).unwrap();
                // let temp_store = catch_unwind(|| {
                //     let mut temp_store = Store::default();
                //     temp_store
                //         .add_module("spectest".to_owned(), spectest)
                //         .unwrap();
                //     temp_store.add_module(module_name.to_string(), validation_info.clone())?;
                //     Ok(temp_store)
                // });

                let temp_store: Result<Result<Store<'_>, wasm::Error>, _> =
                    Ok::<std::result::Result<Store<'_>, wasm::Error>, wasm::Error>(Ok(
                        Store::default(),
                    ));

                match temp_store {
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
                        Ok(_) => store
                            .as_mut()
                            .unwrap()
                            .add_module(module_name, validation_info)
                            .unwrap(),
                    },
                };
            }
            wast::WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                if store.is_none() {
                    return CompilationError::new(
                        Box::new(GenericError::new(
                            "Attempted to assert before module directive",
                        )),
                        filepath,
                        "no module directive found",
                    )
                    .compile_report();
                }

                let store = store.as_mut().unwrap();

                let err_or_panic: Result<(), Box<dyn Error>> =
                    match catch_unwind(AssertUnwindSafe(|| {
                        execute_assert_return(store, exec, results)
                    })) {
                        Ok(original_result) => original_result,
                        Err(inner) => {
                            // TODO: Do we want to exit on panic? State may be left in an inconsistent state, and cascading panics may occur.
                            println!("{:#?}", inner);
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
                if store.is_none() {
                    return CompilationError::new(
                        Box::new(GenericError::new(
                            "Attempted to assert before module directive",
                        )),
                        filepath,
                        "no module directive found",
                    )
                    .compile_report();
                }

                let store = store.as_mut().unwrap();

                let err_or_panic: Result<(), Box<dyn Error>> =
                    match catch_unwind(AssertUnwindSafe(|| {
                        execute_assert_trap(store, exec, message)
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

                match encode_validate_instantiate(&mut module, &mut None) {
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

                match encode_validate_instantiate(&mut module, &mut None) {
                    Err(_) => {
                        asserts.push_success(WastSuccess::new(line_number, cmd))
                    },
                    Ok(_) => asserts.push_error(WastError::new("Module validated and instantiated successfully, when it shouldn't have been".into(), line_number, cmd))
                };
            }

            wast::WastDirective::Register { span, name, module } => {
                match module {
                    None => {
                        // haven't tested, but maybe we just register the last module?
                        let module_idx = store.as_ref().unwrap().modules.len() - 1;
                        store
                            .as_mut()
                            .unwrap()
                            .register_alias(name.to_string(), module_idx);
                    }
                    Some(module) => {
                        match store
                            .as_ref()
                            .unwrap()
                            .get_module_idx_from_name(module.name())
                        {
                            Err(e) => {
                                asserts.push_error(WastError::new(
                                    Box::new(GenericError::new(&e.to_string())),
                                    span.linecol_in(&contents).0 as u32 + 1,
                                    get_command(&contents, span),
                                ));
                                // maybe just crash, cause register is vital?
                            }
                            Ok(module_idx) => {
                                asserts.push_success(WastSuccess::new(
                                    span.linecol_in(&contents).0 as u32 + 1,
                                    get_command(&contents, span),
                                ));
                                store
                                    .as_mut()
                                    .unwrap()
                                    .register_alias(name.to_string(), module_idx);
                            }
                        };
                    }
                }
            }
            wast::WastDirective::AssertUnlinkable {
                span,
                module,
                message,
            } => {
                // this directive is not used with other messages in the official testsuite in except memory64 proposal, which is not implemented for this interpreter
                assert!(message == "incompatible import type" || message == "unknown import");
                match store {
                    None => {
                        asserts.push_error(WastError::new(
                            Box::new(GenericError::new(
                                "AssertUnlinkable: No store present to try and link",
                            )),
                            span.linecol_in(&contents).0 as u32 + 1,
                            get_command(&contents, span),
                        ));
                    }
                    Some(ref mut store) => match module {
                        wast::Wat::Component(_) => {
                            asserts.push_error(WastError::new(
                                Box::new(GenericError::new(
                                    "AssertUnlinkable: Components not supported yet",
                                )),
                                span.linecol_in(&contents).0 as u32 + 1,
                                get_command(&contents, span),
                            ));
                        }
                        wast::Wat::Module(mut module) => {
                            // TODO: maybe remove the unwrap? but we are testing ONLY on official testsuite files
                            //        which SHOULD encode 100% of the time
                            let encoded = module.encode().unwrap();

                            let encoded = Box::leak(Box::new(encoded));

                            let validation_info_attempt = catch_unwind(|| validate(encoded));

                            match validation_info_attempt {
                                Err(_) => {
                                    asserts.push_error(WastError::new(
                                        Box::new(GenericError::new(
                                            "AssertUnlinkable: Couldn't validate",
                                        )),
                                        span.linecol_in(&contents).0 as u32 + 1,
                                        get_command(&contents, span),
                                    ));
                                }
                                Ok(validation_info) => match validation_info {
                                    Err(_) => {
                                        asserts.push_error(WastError::new(
                                            Box::new(GenericError::new(
                                                "AssertUnlinkable: Couldn't validate",
                                            )),
                                            span.linecol_in(&contents).0 as u32 + 1,
                                            get_command(&contents, span),
                                        ));
                                    }
                                    Ok(validation_info) => {
                                        let mut module_name = match module.name {
                                            None => DEFAULT_MODULE.to_owned(),
                                            Some(name) => name.name.to_owned(),
                                        };
                                        if store.modules.len() > 1 && module_name == DEFAULT_MODULE
                                        {
                                            module_name = store.modules.len().to_string();
                                        }

                                        let res = store.add_module(module_name, validation_info);

                                        match res {
                                            Ok(_) => {
                                                asserts.push_error(WastError::new(
                                                    Box::new(GenericError::new(
                                                        "Module linked successfully when it shouldn't have been",
                                                    )),
                                                    span.linecol_in(&contents).0 as u32 + 1,
                                                    get_command(&contents, span),
                                                ));
                                            }
                                            Err(e) => {
                                                let actual_linker_err =
                                                    linker_err_to_wasm_testsuite_string(e.clone());

                                                println!(
                                                    "Actual: {}; Expected: {}",
                                                    e.clone(),
                                                    message
                                                );

                                                match actual_linker_err {
                                                    None => {
                                                        asserts.push_error(WastError::new(
                                                        Box::new(GenericError::new(
                                                            &format!("Module failed to link, but with an error of {} instead of a linking (Runtime) error", e.to_string()),
                                                        )),
                                                        span.linecol_in(&contents).0 as u32 + 1,
                                                        get_command(&contents, span),
                                                    ));
                                                    }
                                                    Some(actual) => {
                                                        if actual != message {
                                                            asserts.push_error(WastError::new(
                                                            Box::new(GenericError::new(
                                                                &format!("Module failed to link, but with an error of {} instead of a linking (Runtime) error", e.to_string()),
                                                            )),
                                                            span.linecol_in(&contents).0 as u32 + 1,
                                                            get_command(&contents, span),
                                                        ));
                                                        } else {
                                                            asserts.push_success(WastSuccess::new(
                                                                span.linecol_in(&contents).0 as u32
                                                                    + 1,
                                                                get_command(&contents, span),
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                            }
                        }
                    },
                }
            }
            wast::WastDirective::AssertExhaustion {
                span,
                call: _,
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
                match store {
                    None => asserts.push_error(WastError::new(
                        Box::new(GenericError::new(
                            "Couldn't invoke, interpreter not present",
                        )),
                        u32::MAX,
                        "invoke",
                    )),
                    Some(ref mut interpreter) => 'WASTDIRECTIVEINVOKESOME: {
                        let module_name = get_module_name_from_wast_invoke(&interpreter, &invoke);
                        let args = invoke
                            .args
                            .into_iter()
                            .map(arg_to_value)
                            .collect::<Vec<_>>();

                        let module_idx = match interpreter.get_module_idx_from_name(&module_name) {
                            Err(_e) => {
                                asserts.push_error(WastError::new(
                                    Box::new(GenericError::new("Unknown module")),
                                    u32::MAX,
                                    "invoke",
                                ));
                                break 'WASTDIRECTIVEINVOKESOME;
                            }
                            Ok(module_idx) => module_idx,
                        };

                        match interpreter.get_global_function_idx_by_name(module_idx, invoke.name) {
                            None => asserts.push_error(WastError::new(
                                Box::new(GenericError::new(&format!(
                                    "Couldn't get the function '{}' from module '{}'",
                                    invoke.name, module_name
                                ))),
                                u32::MAX,
                                "invoke",
                            )),
                            Some(func_idx) => {
                                match interpreter.invoke_dynamic_unchecked_return_ty(func_idx, args)
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
    // interpeter: &mut RuntimeInstance,
    store: &mut Store,
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

            let module_name = if let Some(module_name) = invoke_info.module {
                module_name.name()
            } else {
                &store.modules.last().unwrap().name
            };

            let func_idx = store
                .lookup_function(module_name, invoke_info.name)
                .ok_or(Box::new(WasmInterpreterError(wasm::Error::RuntimeError(
                    RuntimeError::FunctionNotFound,
                ))))?;
            // let func = store
            //     .get_function_by_name(DEFAULT_MODULE, invoke_info.name)
            //     .map_err(|err| Box::new(WasmInterpreterError(wasm::Error::RuntimeError(err))))?;

            let actual = store
                .invoke_dynamic(func_idx, args, &result_types)
                .map_err(|err| Box::new(WasmInterpreterError(wasm::Error::RuntimeError(err))))?;

            AssertEqError::assert_eq(actual, result_vals)?;
        }
        wast::WastExecute::Get {
            span: _,
            module,
            global,
        } => {
            let module_idx = match module {
                None => store.modules.len() - 1,
                Some(module_name) => store
                    .modules
                    .iter()
                    .enumerate()
                    .find_map(|(idx, modul)| {
                        // if modul.names.contains(&module_name.name().to_string()) {
                        if modul.name == module_name.name() {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .ok_or(Box::new(PanicError::new(&format!(
                        "Couldn't find module '{}'",
                        module_name.name()
                    ))))?,
            };

            let global_export = store.modules[module_idx]
                .exports
                .iter()
                .find_map(|export| {
                    if export.name == global {
                        Some(export)
                    } else {
                        None
                    }
                })
                .ok_or(Box::new(PanicError::new(&format!(
                    "No export found with this name '{}'",
                    global
                ))))?;

            match global_export.desc {
                wasm::ExportDesc::GlobalIdx(idx) => {
                    let actual_global = &store.globals[store.modules[module_idx].globals[idx]];
                    let value = actual_global.value;
                    let result_vals = results.into_iter().map(result_to_value).collect::<Vec<_>>();
                    AssertEqError::assert_eq(result_vals, vec![value])?;
                }
                _ => {
                    return Err(Box::new(PanicError::new(&format!(
                        "'{}' not a global when it should be",
                        global
                    ))))
                }
            };
        }
        wast::WastExecute::Wat(_) => {
            todo!("`wat` directive inside `assert_return` not yet implemented")
        }
    }

    Ok(())
}

fn get_module_name_from_wast_invoke(store: &Store, invoke_info: &WastInvoke<'_>) -> String {
    match invoke_info.module {
        None => store.modules.last().unwrap().name.clone(),
        Some(module) => String::from(module.name()),
    }
}

fn execute_assert_trap(
    store: &mut Store,
    exec: wast::WastExecute,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    match exec {
        wast::WastExecute::Invoke(invoke_info) => {
            let module_name = get_module_name_from_wast_invoke(&store, &invoke_info);

            let args = invoke_info
                .args
                .into_iter()
                .map(arg_to_value)
                .collect::<Vec<_>>();

            let module_idx = match store.get_module_idx_from_name(&module_name) {
                Err(_e) => {
                    return Err(Box::new(GenericError::new("Unknown module")));
                }
                Ok(module_idx) => module_idx,
            };

            let func_idx = store
                .get_global_function_idx_by_name(module_idx, invoke_info.name)
                .ok_or(Box::new(WasmInterpreterError(wasm::Error::RuntimeError(
                    RuntimeError::FunctionNotFound,
                ))))?;

            let actual = store.invoke_dynamic_unchecked_return_ty(func_idx, args);

            match actual {
                Ok(_) => Err(Box::new(GenericError::new("assert_trap did NOT trap"))),
                Err(e) => {
                    let actual = runtime_err_to_wasm_testsuite_string(e);
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
        wast::WastExecute::Wat(wast::Wat::Module(module)) => {
            let mut quote_wat = wast::QuoteWat::Wat(wast::Wat::Module(module));
            match encode_validate_instantiate(&mut quote_wat, &mut None) {
                Err(_) => Ok(()),
                Ok(_) => Err(Box::new(PanicError::new(
                    "Module validated and instantiated successfully, when it shouldn't have been",
                ))),
            }
        }
        _ => Err(Box::new(PanicError::new(&format!(
            "Unsupported action: {:?}",
            exec
        )))),
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
            WastRetCore::RefExtern(index) => match index {
                None => unreachable!("Expected a non-null extern reference"),
                Some(index) => {
                    use wasm::value::*;
                    Value::Ref(Ref::Extern(ExternAddr::new(Some(index as usize))))
                }
            },
            WastRetCore::RefHost(_) => todo!("`RefHost` result not yet implemented"),
            WastRetCore::RefFunc(index) => match index {
                None => unreachable!("Expected a non-null function reference"),
                Some(_index) => {
                    // use wasm::value::*;
                    // Value::Ref(Ref::Func(FuncAddr::new(Some(index as usize))))
                    todo!("`RefFunc` result not yet implemented")
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
