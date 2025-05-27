use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::{Path, PathBuf};

pub fn find_wast_files(
    base_path: &Path,
    mut file_name_filter: impl FnMut(&OsStr) -> bool,
) -> std::io::Result<Vec<PathBuf>> {
    find_files_filtered(base_path, |path, meta| {
        let Some(file_name) = path.file_name() else {
            return false;
        };

        let if_file_then_wast_extension =
            meta.is_file() && path.extension() == Some(OsStr::new("wast"));

        file_name_filter(file_name) && if_file_then_wast_extension
    })
}

/// Simple non-recursive depth-first directory traversal
fn find_files_filtered(
    base_path: &Path,
    mut filter: impl FnMut(&Path, &Metadata) -> bool,
) -> std::io::Result<Vec<PathBuf>> {
    let base_path = base_path
        .to_str()
        .expect("path to contain valid unicode, which is required by glob");

    let mut paths = vec![];

    // The stack containing all directories we still have to traverse. At first contains only the base directory.
    let mut read_dirs = vec![std::fs::read_dir(base_path)?];

    while let Some(last_read_dir) = read_dirs.last_mut() {
        let Some(entry) = last_read_dir.next() else {
            read_dirs.pop();
            continue;
        };

        let entry = entry?;
        let meta = entry.metadata()?;
        let path = entry.path();

        if filter(&path, &meta) {
            if meta.is_file() {
                paths.push(entry.path())
            }

            if meta.is_dir() {
                read_dirs.push(std::fs::read_dir(path)?);
            }
        }
    }

    Ok(paths)
}
