use std::{
    fs,
    path::{Path, PathBuf},
};

mod reports;
mod run;
mod test_errors;

#[test_log::test]
pub fn spec_dummy() {
    let path = "./tests/specification/dummy.wast";
    let report = run::run_spec_test(path);
    println!("Report for {}:\n{}", path, report);
}

#[test_log::test]
pub fn spec_tests() {
    let paths = get_wast_files(Path::new("./tests/specification/testsuite/"))
        .expect("Failed to find testsuite");
    for test_path in paths {
        let report = run::run_spec_test(test_path.to_str().unwrap());
        println!("Report for {}:\n{}", test_path.display(), report);
    }
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
