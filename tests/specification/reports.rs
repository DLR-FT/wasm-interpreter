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
    pub filename: String,
    pub results: Vec<Result<WastSuccess, WastError>>,
    pub successful: usize,
    pub failed: usize,
    pub total: usize,
    pub percentage: f64,
}

impl Default for AssertReport {
    fn default() -> Self {
        Self {
            filename: String::from(""),
            results: Vec::new(),
            successful: 0,
            failed: 0,
            total: 0,
            percentage: 0.0,
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
                        self.filename,
                        match error.line_number {
                            None => u32::MAX.to_string(),
                            Some(line_number) =>
                                if line_number == u32::MAX {
                                    "?".to_string()
                                } else {
                                    line_number.to_string()
                                },
                        },
                        error.command
                    )?;
                    writeln!(f, "    Error: {}", error.inner)?;
                }
            }
        }

        Ok(())
    }
}

impl AssertReport {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            results: Vec::new(),
            ..Default::default()
        }
    }

    pub fn compute_data(&mut self) {
        self.total = self.results.len();
        self.successful = self.results.iter().filter(|el| el.is_ok()).count();
        self.failed = self.total - self.successful;
        self.percentage = if self.total == 0 {
            0.0
        } else {
            (self.successful as f64) * 100.0 / (self.total as f64)
        };
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
        return WastTestReport::CompilationError(self);
    }
}

pub enum WastTestReport {
    Asserts(AssertReport),
    CompilationError(CompilationError),
}

impl std::fmt::Display for WastTestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WastTestReport::CompilationError(error) => {
                writeln!(f, "------ {} ------", error.filename)?;
                writeln!(f, "⚠ Compilation Failed ⚠")?;
                writeln!(f, "Context: {}", error.context)?;
                writeln!(f, "Error: {}", error.inner)?;
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

#[derive(serde::Serialize)]
pub struct CIReport {
    pub entries: Vec<CIReportEntry>,
}

#[derive(serde::Serialize)]
pub struct CIReportEntry {
    pub filename: String,
    pub compiled: bool,
    pub tests_total: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
}
