/// Interfaces copied from the main project and annotated with `serde::Deserialize` and `Debug`.
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CIFullReport {
    pub entries: Vec<CIReportHeader>,
}

#[derive(Deserialize, Debug)]
pub struct CIReportHeader {
    pub filepath: String,
    pub data: CIReportData,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct CIAssert {
    pub error: Option<String>,
    pub line_number: u32,
    pub command: String,
}
