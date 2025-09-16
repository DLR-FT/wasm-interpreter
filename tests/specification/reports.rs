use std::{any::Any, error::Error};

use wasm::{RuntimeError, TrapError};

use super::test_errors::AssertEqError;

pub struct AssertOutcome {
    pub line_number: u32,
    pub command: String,
    pub maybe_error: Option<WastError>,
}

#[derive(thiserror::Error, Debug)]
pub enum WastError {
    #[error("Panic: {}", .0.downcast_ref::<&str>().unwrap_or(&"Unknown panic"))]
    Panic(Box<dyn Any + Send + 'static>),
    #[error("{0}")]
    WasmError(wasm::Error),
    #[error("{0}")]
    AssertEqualFailed(#[from] AssertEqError),
    #[error("Module validated and instantiated successfully, when it shouldn't have")]
    AssertInvalidButValid,
    #[error("Module linked successfully, when it shouldn't have")]
    AssertUnlinkableButLinked,
    #[error("'assert_exhaustion': Expected '{expected}' - Actual: '{}'", actual.as_ref()
        .map(|actual| format!("{actual}"))
        .unwrap_or_else(|| "---".to_owned())
    )]
    AssertExhaustionButDidNotExhaust {
        expected: String,
        actual: Option<RuntimeError>,
    },
    #[error("'assert_trap': Expected '{expected}' - Actual: '{}'", actual.as_ref()
        .map(|actual| format!("{actual}"))
        .unwrap_or_else(|| "---".to_owned())
    )]
    AssertTrapButTrapWasIncorrect {
        expected: String,
        actual: Option<TrapError>,
    },
    #[error("{0}")]
    Wast(#[from] wast::Error),
    #[error("Runtime error not represented in WAST")]
    UnrepresentedRuntimeError,
}

impl From<wasm::Error> for WastError {
    fn from(value: wasm::Error) -> Self {
        Self::WasmError(value)
    }
}

/// Wast script executed successfully. The outcomes of asserts (pass/fail) are
/// stored here.
pub struct AssertReport {
    pub filename: String,
    pub results: Vec<AssertOutcome>,
}

impl AssertReport {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            results: Vec::new(),
        }
    }

    pub fn push_success(&mut self, line_number: u32, command: String) {
        self.results.push(AssertOutcome {
            line_number,
            command,
            maybe_error: None,
        });
    }

    pub fn push_error(&mut self, line_number: u32, command: String, error: WastError) {
        self.results.push(AssertOutcome {
            line_number,
            command,
            maybe_error: Some(error),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.results.iter().any(|r| r.maybe_error.is_some())
    }

    pub fn total_asserts(&self) -> u32 {
        self.results.len() as u32
    }

    pub fn passed_asserts(&self) -> u32 {
        self.results
            .iter()
            .filter(|el| el.maybe_error.is_some())
            .count() as u32
    }

    pub fn failed_asserts(&self) -> u32 {
        self.total_asserts() - self.passed_asserts()
    }

    pub fn percentage_asserts_passed(&self) -> f32 {
        if self.total_asserts() == 0 {
            0.0
        } else {
            (self.passed_asserts() as f32) * 100.0 / (self.total_asserts() as f32)
        }
    }
}

impl std::fmt::Display for AssertReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for elem in &self.results {
            match &elem.maybe_error {
                None => {
                    writeln!(
                        f,
                        "✅ {}:{} -> {}",
                        self.filename,
                        if elem.line_number == u32::MAX {
                            "?".to_string()
                        } else {
                            elem.line_number.to_string()
                        },
                        elem.command
                    )?;
                }
                Some(error) => {
                    writeln!(
                        f,
                        "❌ {}:{} -> {}",
                        self.filename, elem.line_number, elem.command
                    )?;
                    writeln!(f, "    Error: {error}")?;
                }
            }
        }

        Ok(())
    }
}

/// An error originating from within the WAST Script. If a non-assert directive
/// fails, a ScriptError will be raised.
pub struct ScriptError {
    pub filename: String,
    pub error: Box<dyn Error>,
    pub context: String,
    #[allow(unused)]
    pub line_number: Option<u32>,
    #[allow(unused)]
    pub command: Option<String>,
}

impl ScriptError {
    pub fn new(
        filename: &str,
        error: Box<dyn Error>,
        context: &str,
        line_number: u32,
        command: &str,
    ) -> Self {
        Self {
            filename: filename.to_string(),
            error,
            context: context.to_string(),
            line_number: Some(line_number),
            command: Some(command.to_string()),
        }
    }

    pub fn new_lineless(filename: &str, error: Box<dyn Error>, context: &str) -> Self {
        Self {
            filename: filename.to_string(),
            error,
            context: context.to_string(),
            line_number: None,
            command: None,
        }
    }
}
