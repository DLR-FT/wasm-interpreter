use std::error::Error;

pub struct WastSuccess {
    filename: String,
    line_number: u32,
    command: String,
}

pub struct WastError {
    inner: Box<dyn Error>,
    filename: String,
    line_number: u32,
    command: String,
}

impl WastError {
    pub fn new(error: Box<dyn Error>, filename: String, line_number: u32, command: &str) -> Self {
        Self {
            inner: error,
            filename,
            line_number,
            command: command.to_string(),
        }
    }

    pub fn from_outside(error: Box<dyn Error>, reason: &str) -> Self {
        Self {
            inner: error,
            filename: "".to_string(),
            line_number: 0,
            command: reason.to_string(),
        }
    }
}

pub struct AssertReport {
    results: Vec<Result<WastSuccess, WastError>>,
}

impl AssertReport {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn push_success(&mut self, filename: String, line_number: u32, command: String) {
        self.results.push(Ok(WastSuccess {
            filename,
            line_number,
            command,
        }));
    }

    pub fn push_error(
        &mut self,
        filename: String,
        line_number: u32,
        command: String,
        error: Box<dyn Error>,
    ) {
        self.results.push(Err(WastError {
            inner: error,
            filename,
            line_number,
            command,
        }));
    }
}

pub enum WastTestReport {
    Asserts(AssertReport),
    CompilationError(WastError),
}

impl From<WastError> for WastTestReport {
    fn from(error: WastError) -> Self {
        WastTestReport::CompilationError(error)
    }
}

impl From<AssertReport> for WastTestReport {
    fn from(assert_report: AssertReport) -> Self {
        WastTestReport::Asserts(assert_report)
    }
}

// .------------------------.
// | Display Implementation |
// '------------------------'

impl std::fmt::Display for WastSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[SUCCESS] {} ({}:{})",
            self.command, self.filename, self.line_number
        )
    }
}

impl std::fmt::Display for WastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[ERROR] {} ({}:{})",
            self.command, self.filename, self.line_number
        )?;
        write!(f, "\t{}", self.inner)?;

        Ok(())
    }
}

impl std::fmt::Display for AssertReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for result in &self.results {
            match result {
                Ok(success) => writeln!(f, "{}", success)?,
                Err(error) => writeln!(f, "{}", error)?,
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for WastTestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WastTestReport::Asserts(assert_report) => write!(f, "{}", assert_report),
            WastTestReport::CompilationError(error) => write!(f, "{}", error),
        }
    }
}
