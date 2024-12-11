use std::{
    fs,
    path::{Path, PathBuf},
};

mod reports;
mod run;
mod test_errors;

#[test_log::test]
pub fn spec_tests() {
    let paths = get_wast_files(Path::new("./tests/specification/testsuite/"))
        .expect("Failed to find testsuite");

    let mut successful_reports = 0;
    let mut failed_reports = 0;
    let mut compile_error_reports = 0;

    for test_path in paths {
        println!("Report for {}:", test_path.display());
        let report = run::run_spec_test(test_path.to_str().unwrap());
        println!("{}", report);

        match report {
            reports::WastTestReport::Asserts(assert_report) => {
                if assert_report.has_errors() {
                    failed_reports += 1;
                } else {
                    successful_reports += 1;
                }
            }
            reports::WastTestReport::CompilationError(_) => {
                compile_error_reports += 1;
            }
        }
    }

    println!(
        "Tests: {} Passed, {} Failed, {} Compilation Errors",
        successful_reports, failed_reports, compile_error_reports
    );
}

// See: https://stackoverflow.com/a/76820878
fn get_wast_files(base_path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut buf = vec![];
    let entries = fs::read_dir(base_path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut subdir = get_wast_files(&entry.path())?;
            buf.append(&mut subdir);
        }

        if meta.is_file() && entry.path().extension().unwrap_or_default() == "wast" {
            buf.push(entry.path());
        }
    }

    Ok(buf)
}
