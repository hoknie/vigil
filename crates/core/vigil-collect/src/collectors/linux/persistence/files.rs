use std::fs;
use std::path::{Path, PathBuf};

use super::{DIRECTORY_LIMIT, FILE_LIMIT};

pub(super) fn sorted_files(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .take(DIRECTORY_LIMIT)
        .collect();
    files.sort();

    files
}

pub(super) fn read_capped(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > FILE_LIMIT {
        return None;
    }
    fs::read_to_string(path).ok()
}
