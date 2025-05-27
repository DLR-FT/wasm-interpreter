use std::{borrow::Cow, fmt::Display};

use crate::{
    ci_reports::{CIFullReport, CIReportData, CIReportHeader},
    sanitize_path, write_details_summary,
};

pub fn generate(report: &CIFullReport) -> ReportSummary {
    // No logic here
    // The analysis done for the summary is minimal and can be done in the Display trait to avoid code duplication
    ReportSummary(report)
}

pub struct ReportSummary<'report>(&'report CIFullReport);

impl Display for ReportSummary<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "## Per-test details")?;
        writeln!(f)?;
        write_details_summary(
            f,
            |f| write!(f, "Click here to open"),
            |f| {
                write!(
                    f,
                    "

| **File** | **Passed Asserts** | **Failed Asserts** | **% Passed** | **Notes** |
|:--------:|:------------------:|:------------------:|:------------:|-----------|
"
                )?;
                self.0
            .entries
            .iter()
            .try_for_each(|CIReportHeader { filepath, data }| {
                write!(f,
                    "| [{filename}](https://github.com/WebAssembly/testsuite/blob/main/{filename}) | ",
                    filename = sanitize_path(filepath)
                )?;

                match data {
                    CIReportData::Assert { results } => {
                        let num_total = results.len();
                        let num_success = results
                            .iter()
                            .filter(|assert| assert.error.is_none())
                            .count();
                        let num_failed = num_total - num_success;
                        let success_percentage =
                            (num_total != 0).then(|| 100.0 * num_success as f32 / num_total as f32);

                        write!(
                            f,
                            "{num_success} / {num_total} | {num_failed} / {num_total} | "
                        )?;
                        if let Some(success_percentage) = success_percentage {
                            write!(f, "{success_percentage}% | ")?;
                        } else {
                            write!(f, "- |")?;
                        }

                        write!(f, "- |")?;
                    }
                    CIReportData::ScriptError {
                        context,
                        line_number,
                        ..
                    } => {
                        write!(f, "- | - | - | ")?;
                        write!(f, "Context: {}<br>", sanitize_table_item(Some(context)))?; // # TODO when is this supposed to be None?
                        if let Some(line_number) = line_number {
                            write!(f, "Line: {line_number} |")?;
                        } else {
                            write!(f, "Line: - |")?;
                        }
                    }
                }
                writeln!(f)
            })
            },
        )
    }
}

fn sanitize_table_item(item: Option<&str>) -> Cow<'static, str> {
    let Some(item) = item else {
        return Cow::Borrowed("-");
    };

    let item = item
        .chars()
        .map(|c| match c {
            '`' => '\'',
            '|' => '/',
            '\n' => ' ',
            x => x,
        })
        .collect();

    Cow::Owned(item)
}
