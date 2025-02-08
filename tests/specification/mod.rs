use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use reports::{CIReport, CIReportEntry, WastTestReport};

mod reports;
mod run;
mod test_errors;

enum Filter {
    #[allow(dead_code)]
    Include(FnF),
    #[allow(dead_code)]
    Exclude(FnF),
}

struct Report {
    #[allow(dead_code)]
    fp: PathBuf,
    report: WastTestReport,
}

struct FnF {
    #[allow(dead_code)]
    files: Option<Vec<String>>,
    #[allow(dead_code)]
    folders: Option<Vec<String>>,
}

impl Default for FnF {
    fn default() -> Self {
        Self {
            files: None,
            folders: None,
        }
    }
}

#[test_log::test]
pub fn spec_tests() {
    // so we don't see unnecessary stacktraces of catch_unwind (sadly this also means we don't see panics from outside catch_unwind either)
    std::panic::set_hook(Box::new(|_| {}));

    let filters = Filter::Exclude(FnF {
        folders: Some(vec!["proposals".to_string()]),
        ..Default::default()
    });

    let paths = get_wast_files(Path::new("./tests/specification/testsuite/"), &filters)
        .expect("Failed to find testsuite");

    // let pb: PathBuf = "./tests/specification/testsuite/table_get.wast".into();
    // let mut paths = Vec::new();
    // paths.push(pb);

    assert!(paths.len() > 0, "Submodules not instantiated");

    let mut successful_reports = 0;
    let mut failed_reports = 0;
    let mut compile_error_reports = 0;
    let mut reports: Vec<Report> = Vec::with_capacity(paths.len());

    let mut longest_string_len: usize = 0;

    for test_path in paths {
        let mut report = run::run_spec_test(test_path.to_str().unwrap());

        match &mut report {
            reports::WastTestReport::Asserts(ref mut assert_report) => {
                // compute auxiliary data
                assert_report.compute_data();
                if assert_report.filename.len() > longest_string_len {
                    longest_string_len = assert_report.filename.len();
                }
                if assert_report.has_errors() {
                    failed_reports += 1;
                } else {
                    successful_reports += 1;
                }
            }
            reports::WastTestReport::CompilationError(_) => {
                compile_error_reports += 1;
            }
        };

        let rep = Report {
            fp: test_path.clone(),
            report,
        };

        reports.push(rep);
    }

    let mut no_compile_errors_reports = reports
        .iter()
        .filter_map(|e| match &e.report {
            WastTestReport::Asserts(asserts) => Some(asserts),
            _ => None,
        })
        .collect::<Vec<&reports::AssertReport>>();
    no_compile_errors_reports.sort_by(|a, b| b.percentage.total_cmp(&a.percentage));

    let mut successful_mini_tests = 0;
    let mut total_mini_tests = 0;

    let mut final_status: String = String::new();
    // Printing success rate per file for those that did NOT error out when compiling
    for report in no_compile_errors_reports {
        final_status += format!(
            "Report for {:filename_width$}: Tests: {:passed_width$} Passed, {:failed_width$} Failed --- {:percentage_width$.2}%\n",
            report.filename,
            report.successful,
            report.failed,
            report.percentage,
            filename_width = longest_string_len + 1,
            passed_width = 7,
            failed_width = 7,
            percentage_width = 6
        ).as_str();

        successful_mini_tests += report.successful;
        total_mini_tests += report.total;

        if report.successful < report.total {
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
        let mut entries = vec![];
        for report in reports {
            let filename = report.fp.to_str().unwrap().to_string();
            match report.report {
                WastTestReport::CompilationError(_compilation_error) => {
                    entries.push(CIReportEntry {
                        filename,
                        compiled: false,
                        tests_total: 0,
                        tests_passed: 0,
                        tests_failed: 0,
                    });
                }
                WastTestReport::Asserts(assert_report) => {
                    entries.push(CIReportEntry {
                        filename,
                        compiled: true,
                        tests_total: assert_report.total,
                        tests_passed: assert_report.successful,
                        tests_failed: assert_report.failed,
                    });
                }
            }
        }

        let ci_report = CIReport { entries };
        let ci_report_json = serde_json::to_string_pretty(&ci_report).unwrap();

        std::fs::File::create("./testsuite_results.json")
            .unwrap()
            .write_all(ci_report_json.as_bytes())
            .unwrap();
    }
}

#[allow(dead_code)]
// See: https://stackoverflow.com/a/76820878
fn get_wast_files(
    base_path: &Path,
    // run_only_these_tests: &Vec<String>,
    filters: &Filter,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut buf = vec![];
    let entries = fs::read_dir(base_path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            if should_add_folder_to_buffer(&entry.path(), &filters) {
                let mut subdir = get_wast_files(&entry.path(), &filters)?;
                buf.append(&mut subdir);
            }
        }

        if meta.is_file() && entry.path().extension().unwrap_or_default() == "wast" {
            if should_add_file_to_buffer(&entry.path(), &filters) {
                buf.push(entry.path())
            }
        }
    }

    Ok(buf)
}

fn should_add_file_to_buffer(file_path: &PathBuf, filters: &Filter) -> bool {
    match filters {
        Filter::Exclude(ref fnf) => match &fnf.files {
            None => true,
            Some(files) => {
                if files.is_empty() {
                    return true;
                }

                if let Some(file_name) = file_path.file_name() {
                    if files.contains(&file_name.to_str().unwrap().to_owned()) {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
        },
        Filter::Include(ref fnf) => match &fnf.files {
            None => false,
            Some(files) => {
                if files.is_empty() {
                    return false;
                }

                if let Some(file_name) = file_path.file_name() {
                    if files.contains(&file_name.to_str().unwrap().to_owned()) {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        },
    }
}

fn should_add_folder_to_buffer(file_path: &PathBuf, filters: &Filter) -> bool {
    match filters {
        Filter::Exclude(ref fnf) => match &fnf.folders {
            None => true,
            Some(folders) => {
                if folders.is_empty() {
                    return true;
                }

                if let Some(file_name) = file_path.file_name() {
                    if folders.contains(&file_name.to_str().unwrap().to_owned()) {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
        },
        Filter::Include(ref fnf) => match &fnf.folders {
            None => false,
            Some(folders) => {
                if folders.is_empty() {
                    return false;
                }

                if let Some(file_name) = file_path.file_name() {
                    if folders.contains(&file_name.to_str().unwrap().to_owned()) {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        },
    }
}
