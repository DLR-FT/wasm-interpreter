use std::fs;
use std::path::{Path, PathBuf};

pub enum Filter {
    #[allow(dead_code)]
    Include(FnF),
    Exclude(FnF),
}

pub struct FnF {
    pub files: Option<Vec<String>>,
    pub folders: Option<Vec<String>>,
}

// See: https://stackoverflow.com/a/76820878
pub fn get_wast_files(base_path: &Path, filter: &Filter) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut buf = vec![];
    let entries = fs::read_dir(base_path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            if should_add_folder_to_buffer(&entry.path(), &filter) {
                let mut subdir = get_wast_files(&entry.path(), &filter)?;
                buf.append(&mut subdir);
            }
        }

        if meta.is_file() && entry.path().extension().unwrap_or_default() == "wast" {
            if should_add_file_to_buffer(&entry.path(), &filter) {
                buf.push(entry.path())
            }
        }
    }

    Ok(buf)
}

fn should_add_file_to_buffer(file_path: &PathBuf, filter: &Filter) -> bool {
    match filter {
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

fn should_add_folder_to_buffer(file_path: &PathBuf, filter: &Filter) -> bool {
    match filter {
        Filter::Exclude(fnf) => match &fnf.folders {
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
        Filter::Include(fnf) => match &fnf.folders {
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
