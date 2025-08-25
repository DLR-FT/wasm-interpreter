use ci_reports::CIFullReport;
use envconfig::Envconfig;
use regex::Regex;
use reports::WastTestReport;
use std::fmt::Write as _;
use std::path::Path;
use std::process::ExitCode;

mod ci_reports;
mod files;
mod reports;
mod run;
mod test_errors;

#[derive(Envconfig)]
pub struct GlobalConfig {
    /// A regex that acts as an allowlist filter for tests.
    /// By default all tests are allowed.
    #[envconfig(default = ".*")]
    pub allow_test_pattern: Regex,

    /// A regex that acts as a blocklist filter for tests.
    /// By default all `simd_*` and `proposals` tests are blocked.
    /// To not block anything use: `^$`
    #[envconfig(default = r"^(simd_.*|proposals|names\.wast)$")]
    pub block_test_pattern: Regex,

    /// This makes the testsuite runner re-enable the panic hook during all interpreter calls, resulting in the printing of panic info on every interpreter panic.
    #[envconfig(default = "false")]
    pub reenable_panic_hook: bool,

    /// This makes the testsuite runner exit with FAILURE with any failing test
    #[envconfig(default = "true")]
    pub fail_if_any_test_fails: bool,
}

lazy_static::lazy_static! {
    pub static ref ENV_CONFIG: GlobalConfig = GlobalConfig::init_from_env().expect("valid environment variables");
}

#[test_log::test]
pub fn spec_tests() -> ExitCode {
    // Load environment variables
    let _ = *ENV_CONFIG;

    // Edit this to ignore or only run specific tests
    let file_name_filter = |file_name: &str| {
        ENV_CONFIG.allow_test_pattern.is_match(file_name)
            && !ENV_CONFIG.block_test_pattern.is_match(file_name)
    };

    let paths = files::find_wast_files(
        Path::new("./tests/specification/testsuite/"),
        file_name_filter,
    )
    .expect("Failed to open testsuite directory");

    assert!(
        !paths.is_empty(),
        "No paths were found, please check if the git submodules are correctly fetched"
    );

    // Some statistics about the reports
    let mut num_failures = 0;
    let mut num_script_errors = 0;

    // Used for padding of filenames with spaces later
    let mut longest_filename_len: usize = 0;

    let reports: Vec<WastTestReport> = paths
        .into_iter()
        .map(|path| run::run_spec_test(path.to_str().unwrap()))
        .inspect(|report| {
            match report {
                reports::WastTestReport::Asserts(assert_report) => {
                    longest_filename_len = longest_filename_len.max(assert_report.filename.len());

                    if assert_report.has_errors() {
                        num_failures += 1;
                    }
                }
                reports::WastTestReport::ScriptError(_) => {
                    num_script_errors += 1;
                }
            };
        })
        .collect();

    // Calculate another required statistic
    let num_successes = reports.len() - num_script_errors - num_failures;

    // Collect all reports without errors along with some statistic
    let mut successful_mini_tests = 0;
    let mut total_mini_tests = 0;
    let mut assert_reports: Vec<&reports::AssertReport> = reports
        .iter()
        .filter_map(|r| match r {
            WastTestReport::Asserts(asserts) => Some(asserts),
            WastTestReport::ScriptError(_) => None,
        })
        .inspect(|assert_report| {
            successful_mini_tests += assert_report.passed_asserts();
            total_mini_tests += assert_report.total_asserts();
        })
        .collect();

    // Sort all reports without errors for displaying it to the user later
    assert_reports.sort_by(|a, b| {
        b.percentage_asserts_passed()
            .total_cmp(&a.percentage_asserts_passed())
    });

    let mut final_status: String = String::new();
    // Printing success rate per file for those that did NOT error out when compiling
    for report in assert_reports {
        writeln!(final_status,
            "Report for {:filename_width$}: Tests: {:passed_width$} Passed, {:failed_width$} Failed --- {:percentage_width$.2}%",
            report.filename,
            report.passed_asserts(),
            report.failed_asserts(),
            report.percentage_asserts_passed(),
            filename_width = longest_filename_len + 1,
            passed_width = 7,
            failed_width = 7,
            percentage_width = 6
        ).expect("writing into strings to never fail");

        if report.passed_asserts() < report.total_asserts() {
            println!("{report}");
        }
    }

    println!("{final_status}");

    println!(
        "\nReport for {:filename_width$}: Tests: {:passed_width$} Passed, {:failed_width$} Failed --- {:percentage_width$.2}%\n\n",
        "all of the above",
        successful_mini_tests,
        total_mini_tests - successful_mini_tests,
        if total_mini_tests == 0 { 0.0 } else {(successful_mini_tests as f64) * 100.0 / (total_mini_tests as f64)},
        filename_width = longest_filename_len + 1,
        passed_width = 7,
        failed_width = 7,
        percentage_width = 6
    );

    println!(
        "Tests: {num_successes} Passed, {num_failures} Failed, {num_script_errors} Compilation Errors"
    );

    // Optional: We need to save the result to a file for CI Regression Analysis
    if std::option_env!("TESTSUITE_SAVE").is_some() {
        let ci_report = CIFullReport::new(&reports);
        let output_file = std::fs::File::create("./testsuite_results.json").unwrap();

        serde_json::to_writer_pretty(output_file, &ci_report).unwrap();
    }

    if ENV_CONFIG.fail_if_any_test_fails && (num_failures != 0 || num_script_errors != 0) {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
