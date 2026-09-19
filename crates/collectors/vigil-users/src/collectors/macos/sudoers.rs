use std::fs;
use std::path::PathBuf;

use crate::parsers::{SudoGrant, parse_sudoers};

pub(super) const SUDOERS: &str = "/private/etc/sudoers";

pub(super) const SUDOERS_DIRECTORY: &str = "/private/etc/sudoers.d";

pub(super) fn read_sudoers() -> Vec<SudoGrant> {
    let mut grants = Vec::new();

    if let Ok(text) = fs::read_to_string(SUDOERS) {
        grants.extend(parse_sudoers(&text, SUDOERS).grants);
    }

    let mut files: Vec<PathBuf> = fs::read_dir(SUDOERS_DIRECTORY)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();

    for path in files {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.ends_with('~') || name.contains('.') {
            continue;
        }
        if let Ok(text) = fs::read_to_string(&path) {
            grants.extend(parse_sudoers(&text, &path.to_string_lossy()).grants);
        }
    }

    grants
}
