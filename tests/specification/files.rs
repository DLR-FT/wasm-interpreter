use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Filter {
    pub mode: FilterMode,
    pub files: Vec<OsString>,
}

#[allow(dead_code)] // Currently, we can always use only one variant at a time
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FilterMode {
    Include,
    Exclude,
}

impl Filter {
    fn allows(&self, file_path: &Path) -> bool {
        // If the file does not have a file name, we simply disallow it
        let Some(file_name) = file_path.file_name() else {
            return false;
        };

        // Assume filter includes for now
        let result = self
            .files
            .iter()
            .any(|os_string| os_string.as_os_str() == file_name);

        // Now if the filter excludes, invert the result
        result ^ (self.mode == FilterMode::Exclude)
    }
}

/// Simple depth-first directory traversal with a filter for `wast` files
pub fn find_wast_files(base_path: &Path, filter: &Filter) -> std::io::Result<Vec<PathBuf>> {
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

        if filter.allows(&path) {
            if meta.is_file() && entry.path().extension() == Some(OsStr::new("wast")) {
                paths.push(entry.path())
            }

            if meta.is_dir() {
                read_dirs.push(std::fs::read_dir(path)?);
            }
        }
    }

    Ok(paths)
}
