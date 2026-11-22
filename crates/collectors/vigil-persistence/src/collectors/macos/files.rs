use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use vigil_collect::{denied_to, this_account};

use super::places::{DIRECTORY_LIMIT, FILE_LIMIT};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Unread {
    Refused(String),
    Failed(String),
}

impl Unread {
    pub(super) fn of(said: String, error: &io::Error) -> Unread {
        Unread::as_account(said, error.kind(), this_account())
    }

    pub(super) fn as_account(said: String, kind: ErrorKind, uid: u32) -> Unread {
        match denied_to(kind, uid) {
            true => Unread::Refused(said),
            false => Unread::Failed(said),
        }
    }

    pub(super) fn said(&self) -> &str {
        match self {
            Unread::Refused(said) | Unread::Failed(said) => said,
        }
    }
}

pub(super) enum Listed {
    Files(Vec<PathBuf>),
    Absent,
    Unreadable(Unread),
}

pub(super) fn listed(directory: &Path) -> Listed {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Listed::Absent,
        Err(error) => {
            return Listed::Unreadable(Unread::of(
                format!("{}: {error}", directory.display()),
                &error,
            ));
        }
    };

    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .take(DIRECTORY_LIMIT)
        .collect();
    files.sort();
    Listed::Files(files)
}

pub(super) fn files_in(directory: &Path) -> Vec<PathBuf> {
    match listed(directory) {
        Listed::Files(files) => files,
        Listed::Absent | Listed::Unreadable(_) => Vec::new(),
    }
}

pub(super) fn read_capped(path: &Path) -> io::Result<Vec<u8>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > FILE_LIMIT {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{} bytes, over the {FILE_LIMIT} a file of this kind is read to",
                metadata.len()
            ),
        ));
    }
    fs::read(path)
}
