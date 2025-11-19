use std::any::Any;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::panic::UnwindSafe;

use bumpalo::Bump;
use itertools::enumerate;
use log::debug;
use wasm::addrs::ModuleAddr;
use wasm::checked::Stored;
use wasm::checked::StoredExternVal;
use wasm::checked::StoredRef;
use wasm::checked::StoredValue;
use wasm::linker::Linker;
use wasm::value::F32;
use wasm::value::F64;
use wasm::GlobalType;
use wasm::Limits;
use wasm::MemType;
use wasm::NumType;
use wasm::RefType;
use wasm::RuntimeError;
use wasm::TableType;
use wasm::TrapError;
use wasm::ValType;
use wasm::{validate, Store};
use wast::core::WastArgCore;
use wast::core::WastRetCore;
use wast::{QuoteWat, WastArg, WastDirective, WastRet, Wat};

use crate::specification::reports::*;
use crate::specification::test_errors::*;

use super::ENV_CONFIG;

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
    store: &'a mut Store<'b, ()>,
    bytes: &'b [u8],
    linker: &mut Linker,
    last_instantiated_module: &mut Option<Stored<ModuleAddr>>,
) -> Result<Stored<ModuleAddr>, WastError> {
    let validation_info =
        catch_unwind_and_suppress_panic_handler(|| validate(bytes)).map_err(WastError::Panic)??;

    let module = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
        linker.module_instantiate(store, &validation_info, None)
    }))
    .map_err(WastError::Panic)??
    .module_addr;

    *last_instantiated_module = Some(module);

    Ok(module)
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

    // We need to keep the wasm_bytes in-scope for the lifetime of the store.
    // As such, we hoist the bytes into an Option, and assign it once a module directive is found.
    let mut store = catch_unwind_and_suppress_panic_handler(|| Store::new(())).unwrap();

    // The last instantiated module is used implicitly whenever a module id
    // parameter is missing.
    let mut last_instantiated_module: Option<Stored<ModuleAddr>> = None;

    // The linker is used to perform automatic linking prior to every module
    // instantiation.
    let mut linker = Linker::new();

    // Initialize a few extern values inside the store and make them available
    // to be imported by all future module instantiations.
    init_spectest(&mut store, &mut linker)
        .expect("spectest environment initialization to always succeed on an empty Store/Linker");

    // Because the linker only links imports and exports and does not keep track
    // of modules, we need to store modules by their names separately.
    let mut visible_modules = HashMap::new();

    for (i, directive) in enumerate(wast.directives) {
        debug!("at directive {:?}", i);

        let directive_result = run_directive(
            directive,
            &arena,
            &mut store,
            &contents,
            filepath,
            &mut visible_modules,
            &mut last_instantiated_module,
            &mut linker,
        )?;

        if let Some(assert_outcome) = directive_result {
            asserts.results.push(assert_outcome);
        }
    }

    Ok(asserts)
}

#[allow(clippy::too_many_arguments)] // reason = "this testsuite runner module needs a redesign anyway"
fn run_directive<'a>(
    wast_directive: WastDirective,
    arena: &'a Bump,
    store: &mut Store<'a, ()>,
    contents: &str,
    filepath: &str,
    visible_modules: &mut HashMap<String, Stored<ModuleAddr>>,
    last_instantiated_module: &mut Option<Stored<ModuleAddr>>,
    linker: &mut Linker,
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

            let module = validate_instantiate(store, wasm_bytes, linker, last_instantiated_module)
                .map_err(|err| {
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
                })) => visible_modules.insert(id.name().to_owned(), module),
                _ => None,
            };

            Ok(None)
        }
        wast::WastDirective::AssertReturn {
            span,
            exec,
            results,
        } => {
            let err_or_panic = execute_assert_return(
                visible_modules,
                store,
                exec,
                results,
                last_instantiated_module,
            );

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
            let result = execute(
                arena,
                visible_modules,
                store,
                exec,
                last_instantiated_module,
                linker,
            );
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
                validate_instantiate(store, bytes, linker, last_instantiated_module)
            });

            let maybe_assert_error = match result {
                Ok(_module) => Some(WastError::AssertInvalidButValid),
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
            // spec tests tells us to use the last defined module if module name is not specified
            let module = match modulee {
                None => last_instantiated_module.as_ref().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(ScriptError::new(
                filepath,
                WastError::UnknownModuleReferenced,
                "Register directive (WAT) failed",
                get_linenum(contents, span),
                get_command(contents, span),
            ))?;

            linker
                .define_module_instance(store, name.to_owned(), module)
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

            let result = match validate_instantiate(store, bytes, linker, last_instantiated_module)
            {
                // module shouldn't have instantiated
                Err(WastError::WasmRuntimeError(
                    RuntimeError::ModuleNotFound
                    | RuntimeError::UnknownImport
                    | RuntimeError::InvalidImportType
                    | RuntimeError::UnableToResolveExternLookup,
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
                store,
                wast::WastExecute::Invoke(call),
                last_instantiated_module,
                linker,
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
            let args: Vec<StoredValue> = invoke.args.into_iter().map(arg_to_value).collect();

            // spec tests tells us to use the last defined module if module name is not specified
            let module = match invoke.module {
                None => last_instantiated_module.as_ref().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(ScriptError::new(
                filepath,
                WastError::UnknownModuleReferenced,
                "Invoke directive (WAT) failed",
                get_linenum(contents, invoke.span),
                get_command(contents, invoke.span),
            ))?;

            let func_addr = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                store
                    .instance_export(module, invoke.name)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        StoredExternVal::Func(func) => Ok(func),
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
                    })
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
                store.invoke_without_fuel(func_addr, args)
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
    visible_modules: &HashMap<String, Stored<ModuleAddr>>,
    store: &mut Store<()>,
    exec: wast::WastExecute,
    results: Vec<wast::WastRet>,
    last_instantiated_module: &mut Option<Stored<ModuleAddr>>,
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
                None => last_instantiated_module.as_ref().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(WastError::UnknownModuleReferenced)?;

            let func_addr = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                store
                    .instance_export(module, invoke_info.name)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        StoredExternVal::Func(func) => Ok(func),
                        _ => Err(WastError::UnknownFunctionReferenced),
                    })
            }))
            .map_err(WastError::Panic)??;

            let actual = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                store.invoke_without_fuel(func_addr, args)
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
                None => last_instantiated_module.as_ref().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(WastError::UnknownModuleReferenced)?;

            let actual = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                let global_addr = store
                    .instance_export(module, global)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        StoredExternVal::Global(global) => Ok(global),
                        _ => Err(WastError::UnknownGlobalReferenced),
                    })?;

                Ok::<StoredValue, WastError>(
                    store
                        .global_read(global_addr)
                        .expect("store ids to be correct"),
                )
            }))
            .map_err(WastError::Panic)??;

            assert_eq(vec![actual], result_vals).map_err(Into::into)
        }
        wast::WastExecute::Wat(_) => todo!("`wat` directive inside `assert_return`"),
    }
}

fn execute<'a>(
    arena: &'a bumpalo::Bump,
    visible_modules: &HashMap<String, Stored<ModuleAddr>>,
    store: &mut Store<'a, ()>,
    exec: wast::WastExecute,
    last_instantiated_module: &mut Option<Stored<ModuleAddr>>,
    linker: &mut Linker,
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
                None => last_instantiated_module.as_ref().copied(),
                Some(id) => visible_modules.get(id.name()).copied(),
            }
            .ok_or(WastError::UnknownModuleReferenced)?;

            let func_addr = catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                store
                    .instance_export(module, invoke_info.name)
                    .map_err(WastError::WasmRuntimeError)
                    .and_then(|extern_val| match extern_val {
                        StoredExternVal::Func(func) => Ok(func),
                        _ => Err(WastError::UnknownFunctionReferenced),
                    })
            }))
            .map_err(WastError::Panic)??;

            catch_unwind_and_suppress_panic_handler(AssertUnwindSafe(|| {
                store.invoke_without_fuel(func_addr, args)
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

            let _module = validate_instantiate(store, bytecode, linker, last_instantiated_module)?;

            Ok(())
        }
        wast::WastExecute::Wat(Wat::Component(_)) => todo!("components inside `assert_trap`"),
    }
}

pub fn arg_to_value(arg: WastArg) -> StoredValue {
    match arg {
        WastArg::Core(core_arg) => match core_arg {
            WastArgCore::I32(val) => StoredValue::I32(val as u32),
            WastArgCore::I64(val) => StoredValue::I64(val as u64),
            WastArgCore::F32(val) => StoredValue::F32(wasm::value::F32(f32::from_bits(val.bits))),
            WastArgCore::F64(val) => StoredValue::F64(wasm::value::F64(f64::from_bits(val.bits))),
            WastArgCore::V128(val) => StoredValue::V128(val.to_le_bytes()),
            WastArgCore::RefNull(rref) => match rref {
                wast::core::HeapType::Concrete(_) => {
                    unreachable!("Null refs don't point to any specific reference")
                }
                wast::core::HeapType::Abstract { shared: _, ty } => {
                    use wast::core::AbstractHeapType::*;
                    match ty {
                        Func => StoredValue::Ref(StoredRef::Null(RefType::FuncRef)),
                        Extern => StoredValue::Ref(StoredRef::Null(RefType::ExternRef)),
                        _ => todo!("`GC` proposal"),
                    }
                }
            },
            WastArgCore::RefExtern(index) => {
                StoredValue::Ref(StoredRef::Extern(wasm::value::ExternAddr(index as usize)))
            }
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

// All *.wast test run within an interpreter context where some extern values
// are always available for imports. These extern values are automatically
// allocated and linked by this function.
//
// See: <https://github.com/WebAssembly/spec/tree/main/interpreter#spectest-host-module>
fn init_spectest(store: &mut Store<()>, linker: &mut Linker) -> Result<(), RuntimeError> {
    let memory = store.mem_alloc(MemType {
        limits: Limits {
            min: 1,
            max: Some(2),
        },
    });

    let table = store.table_alloc(
        TableType {
            lim: Limits {
                min: 10,
                max: Some(20),
            },
            et: RefType::FuncRef,
        },
        StoredRef::Null(RefType::FuncRef),
    )?;

    let global_i32 = store.global_alloc(
        GlobalType {
            ty: ValType::NumType(NumType::I32),
            is_mut: false,
        },
        StoredValue::I32(666),
    )?;

    let global_i64 = store.global_alloc(
        GlobalType {
            ty: ValType::NumType(NumType::I64),
            is_mut: false,
        },
        StoredValue::I64(666),
    )?;

    let global_f32 = store.global_alloc(
        GlobalType {
            ty: ValType::NumType(NumType::F32),
            is_mut: false,
        },
        StoredValue::F32(F32(666.6)),
    )?;

    let global_f64 = store.global_alloc(
        GlobalType {
            ty: ValType::NumType(NumType::F64),
            is_mut: false,
        },
        StoredValue::F64(F64(666.6)),
    )?;

    let print = store.func_alloc_typed::<(), ()>(spectec_functions::print);
    let print_i32 = store.func_alloc_typed::<i32, ()>(spectec_functions::print_i32);
    let print_i64 = store.func_alloc_typed::<i64, ()>(spectec_functions::print_i64);
    let print_f32 = store.func_alloc_typed::<f32, ()>(spectec_functions::print_f32);
    let print_f64 = store.func_alloc_typed::<f64, ()>(spectec_functions::print_f64);
    let print_i32_f32 = store.func_alloc_typed::<(i32, f32), ()>(spectec_functions::print_i32_f32);
    let print_f64_f64 = store.func_alloc_typed::<(f64, f64), ()>(spectec_functions::print_f64_f64);

    linker.define(
        "spectest".to_owned(),
        "memory".to_owned(),
        StoredExternVal::Mem(memory),
    )?;
    linker.define(
        "spectest".to_owned(),
        "table".to_owned(),
        StoredExternVal::Table(table),
    )?;
    linker.define(
        "spectest".to_owned(),
        "global_i32".to_owned(),
        StoredExternVal::Global(global_i32),
    )?;
    linker.define(
        "spectest".to_owned(),
        "global_i64".to_owned(),
        StoredExternVal::Global(global_i64),
    )?;
    linker.define(
        "spectest".to_owned(),
        "global_f32".to_owned(),
        StoredExternVal::Global(global_f32),
    )?;
    linker.define(
        "spectest".to_owned(),
        "global_f64".to_owned(),
        StoredExternVal::Global(global_f64),
    )?;
    linker.define(
        "spectest".to_owned(),
        "print".to_owned(),
        StoredExternVal::Func(print),
    )?;
    linker.define(
        "spectest".to_owned(),
        "print_i32".to_owned(),
        StoredExternVal::Func(print_i32),
    )?;
    linker.define(
        "spectest".to_owned(),
        "print_i64".to_owned(),
        StoredExternVal::Func(print_i64),
    )?;
    linker.define(
        "spectest".to_owned(),
        "print_f32".to_owned(),
        StoredExternVal::Func(print_f32),
    )?;
    linker.define(
        "spectest".to_owned(),
        "print_f64".to_owned(),
        StoredExternVal::Func(print_f64),
    )?;
    linker.define(
        "spectest".to_owned(),
        "print_i32_f32".to_owned(),
        StoredExternVal::Func(print_i32_f32),
    )?;
    linker.define(
        "spectest".to_owned(),
        "print_f64_f64".to_owned(),
        StoredExternVal::Func(print_f64_f64),
    )?;

    Ok(())
}

mod spectec_functions {
    use wasm::{host_function_wrapper, HaltExecutionError, Value};

    pub fn print(
        _user_data: &mut (),
        params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper::<(), ()>(params, |()| {
            // TODO print something here?
            Ok(())
        })
    }

    pub fn print_i32(
        _user_data: &mut (),
        params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper::<i32, ()>(params, |_x| {
            // TODO print parameters here?
            Ok(())
        })
    }

    pub fn print_i64(
        _user_data: &mut (),
        params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper::<i64, ()>(params, |_x| {
            // TODO print parameters here?
            Ok(())
        })
    }

    pub fn print_f32(
        _user_data: &mut (),
        params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper::<f32, ()>(params, |_x| {
            // TODO print parameters here?
            Ok(())
        })
    }

    pub fn print_f64(
        _user_data: &mut (),
        params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper::<f64, ()>(params, |_x| {
            // TODO print parameters here?
            Ok(())
        })
    }

    pub fn print_i32_f32(
        _user_data: &mut (),
        params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper::<(i32, f32), ()>(params, |(_a, _b)| {
            // TODO print parameters here?
            Ok(())
        })
    }

    pub fn print_f64_f64(
        _user_data: &mut (),
        params: Vec<Value>,
    ) -> Result<Vec<Value>, HaltExecutionError> {
        host_function_wrapper::<(f64, f64), ()>(params, |(_a, _b)| {
            // TODO print parameters here?
            Ok(())
        })
    }
}
