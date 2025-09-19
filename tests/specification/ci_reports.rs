use super::reports::{AssertOutcome, AssertReport, ScriptError};

#[derive(serde::Serialize)]
pub struct CIFullReport {
    pub entries: Vec<CIReportHeader>,
}

impl CIFullReport {
    pub fn new(report: Vec<Result<AssertReport, ScriptError>>) -> Self {
        Self {
            entries: report.into_iter().map(CIReportHeader::new).collect(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct CIReportHeader {
    pub filepath: String,
    pub data: CIReportData,
}
impl CIReportHeader {
    fn new(report: Result<AssertReport, ScriptError>) -> Self {
        let filepath = match &report {
            Ok(assert_report) => assert_report.filename.clone(),
            Err(script_error) => script_error.filename.clone(),
        };

        Self {
            filepath,
            data: CIReportData::new(report),
        }
    }
}

#[derive(serde::Serialize)]
pub enum CIReportData {
    Assert {
        results: Vec<CIAssert>,
    },
    ScriptError {
        error: String,
        context: String,
        line_number: Option<u32>,
        command: Option<String>,
    },
}
impl CIReportData {
    fn new(report: Result<AssertReport, ScriptError>) -> Self {
        match report {
            Ok(assert_report) => Self::Assert {
                results: assert_report
                    .results
                    .into_iter()
                    .map(CIAssert::new)
                    .collect(),
            },
            Err(script_error) => Self::ScriptError {
                error: script_error.error.to_string(),
                context: script_error.context.clone(),
                line_number: script_error.line_number,
                command: script_error.command.clone(),
            },
        }
    }
}

#[derive(serde::Serialize)]
pub struct CIAssert {
    pub error: Option<String>,
    pub line_number: u32,
    pub command: String,
}
impl CIAssert {
    fn new(assert_outcome: AssertOutcome) -> Self {
        Self {
            line_number: assert_outcome.line_number,
            command: assert_outcome.command,
            error: assert_outcome.maybe_error.map(|err| err.to_string()),
        }
    }
}
