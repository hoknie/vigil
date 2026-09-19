use std::fs;
use std::path::{Path, PathBuf};

use super::{SUDOERS, SUDOERS_DIRECTORY, VENDOR_SUDOERS};
use crate::parsers::{SudoGrant, parse_sudoers};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Included {
    pub(super) files: Vec<PathBuf>,
    pub(super) directories: Vec<PathBuf>,
}

pub(super) fn main_sudoers() -> &'static str {
    [SUDOERS, VENDOR_SUDOERS]
        .into_iter()
        .find(|path| fs::symlink_metadata(path).is_ok())
        .unwrap_or(SUDOERS)
}

pub(super) fn read_sudoers() -> Vec<SudoGrant> {
    let main = main_sudoers();
    let mut grants = Vec::new();
    let mut included = Included::default();

    if let Ok(text) = fs::read_to_string(main) {
        grants.extend(parse_sudoers(&text, main).grants);
        included = included_by(&text, Path::new(main).parent().unwrap_or(Path::new("/")));
    }

    for file in &included.files {
        if let Ok(text) = fs::read_to_string(file) {
            grants.extend(parse_sudoers(&text, &file.to_string_lossy()).grants);
        }
    }
    for directory in directories_to_read(&included) {
        for file in read_by_sudo(&directory) {
            if let Ok(text) = fs::read_to_string(&file) {
                grants.extend(parse_sudoers(&text, &file.to_string_lossy()).grants);
            }
        }
    }

    grants
}

pub(super) fn included_by(text: &str, beside: &Path) -> Included {
    let mut included = Included::default();

    for line in text.lines() {
        let mut words = line.split_whitespace();
        let (Some(directive), Some(target)) = (words.next(), words.next()) else {
            continue;
        };
        let target = target.trim_matches('"');
        if target.is_empty() || target.contains('%') {
            continue;
        }
        let path = beside.join(target);
        let list = match directive {
            "@includedir" | "#includedir" => &mut included.directories,
            "@include" | "#include" => &mut included.files,
            _ => continue,
        };
        if !list.contains(&path) {
            list.push(path);
        }
    }

    included
}

pub(super) fn directories_to_read(included: &Included) -> Vec<PathBuf> {
    let mut directories = included.directories.clone();
    let always = PathBuf::from(SUDOERS_DIRECTORY);
    if !directories.contains(&always) {
        directories.push(always);
    }
    directories
}

fn read_by_sudo(directory: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            !name.ends_with('~') && !name.contains('.')
        })
        .collect();
    files.sort();
    files
}
