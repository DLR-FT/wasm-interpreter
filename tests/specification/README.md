# Testsuite Runner

## Using the Testsuite Runner

1. `git submodule update --init` to fetch the testsuite submodule
1. Edit the `mod.rs` file to your liking
   - This means you can include and exclude files and folders as you wish (only one of these two filters can be active at any one point)
   - Example:

```rs
#[test_log::test]
pub fn spec_tests() {
    let filters = Filter::Exclude(FnF {
        folders: Some(vec!["proposals".to_string()]), // exclude any folders you want
        files: Some(vec!["custom.wast".to_string()]) // files, as well
    });

    let filters = Filter::Include(FnF {
        folders: Some(vec!["proposals".to_string()]), // include only folders you want
    });

    // then get the paths of the files and you are good to go
    let paths = get_wast_files(Path::new("./tests/specification/testsuite/"), &filters)
        .expect("Failed to find testsuite");


    // or you can just do this for one test
    let pb: PathBuf = "./tests/specification/testsuite/table_grow.wast".into();
    let mut paths = Vec::new();
    paths.push(pb);

    let mut successful_reports = 0;
    let mut failed_reports = 0;
    let mut compile_error_reports = 0;

    for test_path in paths {
        println!("Report for {}:", test_path.display());
        let report = run::run_spec_test(test_path.to_str().unwrap());
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
```
