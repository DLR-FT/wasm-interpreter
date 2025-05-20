use std::error::Error;

pub struct WastSuccess {
    pub line_number: u32,
    pub command: String,
}

impl WastSuccess {
    pub fn new(line_number: u32, command: &str) -> Self {
        Self {
            line_number,
            command: command.to_string(),
        }
    }
}

pub struct WastError {
    pub inner: Box<dyn Error>,
    pub line_number: u32,
    pub command: String,
}

impl WastError {
    pub fn new(error: Box<dyn Error>, line_number: u32, command: &str) -> Self {
        Self {
            inner: error,
            line_number,
            command: command.to_string(),
        }
    }
}

/// Wast script executed successfuly. The results of asserts (pass/fail) are
/// stored here.
pub struct AssertReport {
    pub filename: String,
    pub results: Vec<Result<WastSuccess, WastError>>,
}

impl AssertReport {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            results: Vec::new(),
        }
    }

    pub fn push_success(&mut self, success: WastSuccess) {
        self.results.push(Ok(success));
    }

    pub fn push_error(&mut self, error: WastError) {
        self.results.push(Err(error));
    }

    pub fn compile_report(self) -> WastTestReport {
        return WastTestReport::Asserts(self);
    }

    pub fn has_errors(&self) -> bool {
        self.results.iter().any(|r| r.is_err())
    }

    pub fn total_asserts(&self) -> u32 {
        self.results.len() as u32
    }

    pub fn passed_asserts(&self) -> u32 {
        self.results.iter().filter(|el| el.is_ok()).count() as u32
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
            match elem {
                Ok(success) => {
                    writeln!(
                        f,
                        "✅ {}:{} -> {}",
                        self.filename,
                        if success.line_number == u32::MAX {
                            "?".to_string()
                        } else {
                            success.line_number.to_string()
                        },
                        success.command
                    )?;
                }
                Err(error) => {
                    writeln!(
                        f,
                        "❌ {}:{} -> {}",
                        self.filename, error.line_number, error.command
                    )?;
                    writeln!(f, "    Error: {}", error.inner)?;
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

    pub fn compile_report(self) -> WastTestReport {
        return WastTestReport::ScriptError(self);
    }
}

pub enum WastTestReport {
    /// The script ran successfully, having directives run successfuly (though
    /// not necessarily meaning all asserts pass!)
    Asserts(AssertReport),
    /// The script could not run successfully, a non-assert directive failed in
    /// such a way the script cannot continue running.
    ScriptError(ScriptError),
}

impl std::fmt::Display for WastTestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WastTestReport::ScriptError(error) => {
                writeln!(f, "------ {} ------", error.filename)?;
                writeln!(f, "⚠ Compilation Failed ⚠")?;
                writeln!(f, "Context: {}", error.context)?;
                writeln!(f, "Error: {}", error.error)?;
                writeln!(f, "~~~~~~~~~~~~~~~~")?;
                writeln!(f, "")?;
            }
            WastTestReport::Asserts(assert_report) => {
                writeln!(f, "------ {} ------", assert_report.filename)?;
                writeln!(f, "{}", assert_report)?;
                let passed_asserts = assert_report.results.iter().filter(|r| r.is_ok()).count();
                let failed_asserts = assert_report.results.iter().filter(|r| r.is_err()).count();
                let total_asserts = assert_report.results.len();

                writeln!(f, "")?;
                writeln!(
                    f,
                    "Execution finished. Passed: {}, Failed: {}, Total: {}",
                    passed_asserts, failed_asserts, total_asserts
                )?;
                writeln!(f, "~~~~~~~~~~~~~~~~")?;
                writeln!(f, "")?;
            }
        }

        Ok(())
    }
}
