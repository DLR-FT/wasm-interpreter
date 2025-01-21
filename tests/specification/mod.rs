use std::{
    fs,
    path::{Path, PathBuf},
};

use reports::{CompilationError, WastTestReport};

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
    // std::panic::set_hook(Box::new(|_| {}));

    let filters = Filter::Exclude(FnF {
        folders: Some(vec!["proposals".to_string()]),
        ..Default::default()
    });

    // let filters = Filter::Include(FnF {
    //     folders: None,
    //     files: Some(
    //         vec![
    //             // "memory_copy.wast",
    //             // "memory_fill.wast",
    //             "memory_grow.wast",
    //             // "memory_grow_manual.wast",
    //             // "memory_init.wast",
    //             // "memory_redundancy.wast",
    //             // "memory_size.wast",
    //             // "memory_trap.wast",
    //             // "memory.wast",
    //         ]
    //         .iter()
    //         .map(|el| (*el).to_owned())
    //         .collect::<Vec<String>>(),
    //     ),
    // });

    // let filters = Filter::Include(FnF {
    //     folders: None,
    //     files: Some(
    //         vec![
    //             "table_copy.wast",
    //             "table_fill.wast",
    //             "table_get.wast",
    //             "table_grow.wast",
    //             "table_init.wast",
    //             "table_set.wast",
    //             "table_size.wast",
    //             "table-sub.wast",
    //             "table.wast",
    //         ]
    //         .iter()
    //         .map(|el| (*el).to_owned())
    //         .collect::<Vec<String>>(),
    //     ),
    // });

    let filters = Filter::Exclude(FnF {
        folders: Some(vec!["proposals".to_string()]),
        files: Some(vec![
            "simd_address.wast".to_string(),
            "simd_align.wast".to_string(),
            "simd_bit_shift.wast".to_string(),
            "simd_bitwise.wast".to_string(),
            "simd_boolean.wast".to_string(),
            "simd_const.wast".to_string(),
            "simd_conversions.wast".to_string(),
            "simd_f32x4.wast".to_string(),
            "simd_f32x4_arith.wast".to_string(),
            "simd_f32x4_cmp.wast".to_string(),
            "simd_f32x4_pmin_pmax.wast".to_string(),
            "simd_f32x4_rounding.wast".to_string(),
            "simd_f64x2.wast".to_string(),
            "simd_f64x2_arith.wast".to_string(),
            "simd_f64x2_cmp.wast".to_string(),
            "simd_f64x2_pmin_pmax.wast".to_string(),
            "simd_f64x2_rounding.wast".to_string(),
            "simd_i16x8_arith.wast".to_string(),
            "simd_i16x8_arith2.wast".to_string(),
            "simd_i16x8_cmp.wast".to_string(),
            "simd_i16x8_extadd_pairwise_i8x16.wast".to_string(),
            "simd_i16x8_extmul_i8x16.wast".to_string(),
            "simd_i16x8_q15mulr_sat_s.wast".to_string(),
            "simd_i16x8_sat_arith.wast".to_string(),
            "simd_i32x4_arith.wast".to_string(),
            "simd_i32x4_arith2.wast".to_string(),
            "simd_i32x4_cmp.wast".to_string(),
            "simd_i32x4_dot_i16x8.wast".to_string(),
            "simd_i32x4_extadd_pairwise_i16x8.wast".to_string(),
            "simd_i32x4_extmul_i16x8.wast".to_string(),
            "simd_i32x4_trunc_sat_f32x4.wast".to_string(),
            "simd_i32x4_trunc_sat_f64x2.wast".to_string(),
            "simd_i64x2_arith.wast".to_string(),
            "simd_i64x2_arith2.wast".to_string(),
            "simd_i64x2_cmp.wast".to_string(),
            "simd_i64x2_extmul_i32x4.wast".to_string(),
            "simd_i8x16_arith.wast".to_string(),
            "simd_i8x16_arith2.wast".to_string(),
            "simd_i8x16_cmp.wast".to_string(),
            "simd_i8x16_sat_arith.wast".to_string(),
            "simd_int_to_int_extend.wast".to_string(),
            "simd_lane.wast".to_string(),
            "simd_linking.wast".to_string(),
            "simd_load.wast".to_string(),
            "simd_load16_lane.wast".to_string(),
            "simd_load32_lane.wast".to_string(),
            "simd_load64_lane.wast".to_string(),
            "simd_load8_lane.wast".to_string(),
            "simd_load_extend.wast".to_string(),
            "simd_load_splat.wast".to_string(),
            "simd_load_zero.wast".to_string(),
            "simd_splat.wast".to_string(),
            "simd_store.wast".to_string(),
            "simd_store16_lane.wast".to_string(),
            "simd_store32_lane.wast".to_string(),
            "simd_store64_lane.wast".to_string(),
            "simd_store8_lane.wast".to_string(),
        ]),
    });

    // let filters = Filter::Include(FnF {
    //     folders: None,
    //     files: Some(
    //         // vec!["linking.wast"]
    //         vec!["linking.wast"]
    //             .iter()
    //             .map(|el| (*el).to_owned())
    //             .collect::<Vec<String>>(),
    //     ),
    // });

    let paths = get_wast_files(Path::new("./tests/specification/testsuite/"), &filters)
        .expect("Failed to find testsuite");

    // let pb: PathBuf = "./tests/specification/testsuite/memory_manual.wast".into();
    // let mut paths = Vec::new();
    // paths.push(pb);

    assert!(paths.len() > 0, "Submodules not instantiated");

    let mut successful_reports = 0;
    let mut failed_reports = 0;
    let mut compile_error_reports = 0;
    let mut reports: Vec<Report> = Vec::with_capacity(paths.len());

    // let mut compile_errors: Vec<CompilationError> = Vec::new();
    let mut compile_error_paths: Vec<String> = Vec::new();

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
            reports::WastTestReport::CompilationError(_compilation_error) => {
                println!("{:#?}", _compilation_error);
                compile_error_paths.push(test_path.to_str().unwrap().to_string());
                // compile_errors.push(_compilation_error);
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

    for compile_report_path in &compile_error_paths {
        println!("Compile error: {compile_report_path}");
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
