use ci_reports::CIFullReport;
use files::{Filter, FnF};
use reports::WastTestReport;
use std::io::Write;
use std::path::Path;

mod ci_reports;
mod files;
mod reports;
mod run;
mod test_errors;

#[test_log::test]
pub fn spec_tests() {
    // so we don't see unnecessary stacktraces of catch_unwind (sadly this also means we don't see panics from outside catch_unwind either)
    std::panic::set_hook(Box::new(|_| {}));

    let filters = Filter::Exclude(FnF {
        folders: Some(vec!["proposals".to_string()]),
        files: None,
    });

    let paths = files::get_wast_files(Path::new("./tests/specification/testsuite/"), &filters)
        .expect("Failed to find testsuite");

    assert!(paths.len() > 0, "Submodules not instantiated");

    let mut successful_reports = 0;
    let mut failed_reports = 0;
    let mut compile_error_reports = 0;
    let mut reports: Vec<WastTestReport> = Vec::with_capacity(paths.len());

    let mut longest_string_len: usize = 0;

    for test_path in paths {
        let mut report = run::run_spec_test(test_path.to_str().unwrap());

        match &mut report {
            reports::WastTestReport::Asserts(assert_report) => {
                // compute auxiliary data
                if assert_report.filename.len() > longest_string_len {
                    longest_string_len = assert_report.filename.len();
                }
                if assert_report.has_errors() {
                    failed_reports += 1;
                } else {
                    successful_reports += 1;
                }
            }
            reports::WastTestReport::ScriptError(_) => {
                compile_error_reports += 1;
            }
        };

        reports.push(report);
    }

    let mut no_compile_errors_reports = reports
        .iter()
        .filter_map(|r| match r {
            WastTestReport::Asserts(asserts) => Some(asserts),
            _ => None,
        })
        .collect::<Vec<&reports::AssertReport>>();
    no_compile_errors_reports.sort_by(|a, b| {
        b.percentage_asserts_passed()
            .total_cmp(&a.percentage_asserts_passed())
    });

    let mut successful_mini_tests = 0;
    let mut total_mini_tests = 0;

    let mut final_status: String = String::new();
    // Printing success rate per file for those that did NOT error out when compiling
    for report in no_compile_errors_reports {
        final_status += format!(
            "Report for {:filename_width$}: Tests: {:passed_width$} Passed, {:failed_width$} Failed --- {:percentage_width$.2}%\n",
            report.filename,
            report.passed_asserts(),
            report.failed_asserts(),
            report.percentage_asserts_passed(),
            filename_width = longest_string_len + 1,
            passed_width = 7,
            failed_width = 7,
            percentage_width = 6
        ).as_str();

        successful_mini_tests += report.passed_asserts();
        total_mini_tests += report.total_asserts();

        if report.passed_asserts() < report.total_asserts() {
            println!("{}", report);
        }
    }

    println!("{}", final_status);

    println!(
        "\nReport for {:filename_width$}: Tests: {:passed_width$} Passed, {:failed_width$} Failed --- {:percentage_width$.2}%\n\n",
        "all of the above",
        successful_mini_tests,
        total_mini_tests - successful_mini_tests,
        if total_mini_tests == 0 { 0.0 } else {(successful_mini_tests as f64) * 100.0 / (total_mini_tests as f64)},
        filename_width = longest_string_len + 1,
        passed_width = 7,
        failed_width = 7,
        percentage_width = 6
    );

    println!(
        "Tests: {} Passed, {} Failed, {} Compilation Errors",
        successful_reports, failed_reports, compile_error_reports
    );

    // Optional: We need to save the result to a file for CI Regression Analysis
    if std::option_env!("TESTSUITE_SAVE").is_some() {
        let ci_report = CIFullReport::new(&reports);
        let ci_report_json = serde_json::to_string_pretty(&ci_report).unwrap();

        std::fs::File::create("./testsuite_results.json")
            .unwrap()
            .write_all(ci_report_json.as_bytes())
            .unwrap();
    }
}
