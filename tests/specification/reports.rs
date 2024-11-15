use std::error::Error;

pub struct WastSuccess {
    line_number: u32,
    command: String,
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
    inner: Box<dyn Error>,
    line_number: Option<u32>,
    command: String,
}

impl WastError {
    pub fn new(error: Box<dyn Error>, line_number: u32, command: &str) -> Self {
        Self {
            inner: error,
            line_number: Some(line_number),
            command: command.to_string(),
        }
    }
}

pub struct AssertReport {
    filename: String,
    results: Vec<Result<WastSuccess, WastError>>,
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
        WastTestReport::Asserts(self)
    }

    pub fn has_errors(&self) -> bool {
        self.results.iter().any(|r| r.is_err())
    }
}

pub struct CompilationError {
    inner: Box<dyn Error>,
    filename: String,
    context: String,
}

impl CompilationError {
    pub fn new(error: Box<dyn Error>, filename: &str, context: &str) -> Self {
        Self {
            inner: error,
            filename: filename.to_string(),
            context: context.to_string(),
        }
    }

    pub fn compile_report(self) -> WastTestReport {
        WastTestReport::CompilationError(self)
    }
}

pub enum WastTestReport {
    Asserts(AssertReport),
    CompilationError(CompilationError),
}

// .------------------------.
// | Display Implementation |
// '------------------------'

impl std::fmt::Display for WastTestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WastTestReport::CompilationError(error) => {
                writeln!(f, "------ {} ------", error.filename)?;
                writeln!(f, "⚠ Compilation Failed ⚠")?;
                writeln!(f, "Context: {}", error.context)?;
                writeln!(f, "Error: {}", error.inner)?;
                writeln!(f, "~~~~~~~~~~~~~~~~")?;
                writeln!(f)?;
            }
            WastTestReport::Asserts(assert_report) => {
                writeln!(f, "------ {} ------", assert_report.filename)?;
                for result in &assert_report.results {
                    match result {
                        Ok(success) => {
                            writeln!(
                                f,
                                "✅ {}:{} -> {}",
                                assert_report.filename, success.line_number, success.command
                            )?;
                        }
                        Err(error) => {
                            writeln!(
                                f,
                                "❌ {}:{} -> {}",
                                assert_report.filename,
                                error.line_number.unwrap_or(0),
                                error.command
                            )?;
                            writeln!(f, "    Error: {}", error.inner)?;
                        }
                    }
                }
                let passed_asserts = assert_report.results.iter().filter(|r| r.is_ok()).count();
                let failed_asserts = assert_report.results.iter().filter(|r| r.is_err()).count();
                let total_asserts = assert_report.results.len();

                writeln!(f)?;
                writeln!(
                    f,
                    "Execution finished. Passed: {}, Failed: {}, Total: {}",
                    passed_asserts, failed_asserts, total_asserts
                )?;
                writeln!(f, "~~~~~~~~~~~~~~~~")?;
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
