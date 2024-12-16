use std::{
    fs,
    path::{Path, PathBuf},
};

mod reports;
mod run;
mod test_errors;

enum Filter {
    Include(FnF),
    Exclude(FnF),
}

struct FnF {
    files: Option<Vec<String>>,
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

// #[ignore = "Globals cause a panic"]
#[test_log::test]
pub fn spec_tests() {
    let filters = Filter::Exclude(FnF {
        files: Some(vec!["binary-leb128.wast".to_string()]),
        folders: Some(vec!["proposals".to_string()]),
    });

    // let only_these_tests: Vec<String> = vec![];

    let paths = get_wast_files(Path::new("./tests/specification/testsuite/"), &filters)
        .expect("Failed to find testsuite");

    // let pb: PathBuf = "./tests/specification/testsuite/custom_conversions.wast".into();
    // let pb: PathBuf = "./tests/specification/testsuite/custom_f64.wast".into();
    // let paths = vec![pb];

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
