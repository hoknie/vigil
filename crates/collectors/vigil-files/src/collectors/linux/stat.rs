use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use vigil_collect::{hex, sha256};

use crate::parsers::{Found, Walked, WatchedDirectory, WatchedFile};

const PERMISSION_BITS: u32 = 0o7777;

pub(super) const FILE: &str = "file";

pub(super) const DIRECTORY: &str = "directory";

pub(super) const SYMLINK: &str = "symlink";

pub(super) const OTHER: &str = "other";

pub(super) fn named(path: &str, ceiling_bytes: u64) -> WatchedFile {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return absent(path, ceiling_bytes);
    };

    let over_the_ceiling = metadata.len() > ceiling_bytes;
    let regular = fs::metadata(path).is_ok_and(|followed| followed.is_file());
    let digest = match !over_the_ceiling && regular {
        true => hashed(path),
        false => None,
    };

    WatchedFile {
        readable: digest.is_some(),
        digest,
        size: metadata.len(),
        over_the_ceiling,
        ..owned(path, &metadata, ceiling_bytes, Found::default())
    }
}

pub(super) fn root(path: &str, metadata: &fs::Metadata, ceiling_bytes: u64) -> WatchedFile {
    owned(
        path,
        metadata,
        ceiling_bytes,
        Found {
            kind: Some(DIRECTORY),
            ..Found::default()
        },
    )
}

pub(super) fn walked(
    path: &str,
    metadata: &fs::Metadata,
    ceiling_bytes: u64,
    by: &str,
) -> WatchedFile {
    let kind = kind_of(metadata);
    let found = Found {
        kind: Some(kind),
        target: match kind {
            SYMLINK => fs::read_link(path)
                .ok()
                .map(|target| target.to_string_lossy().into_owned()),
            _ => None,
        },
        walked: Some(Walked {
            by: by.to_string(),
            complete: true,
        }),
    };
    let row = owned(path, metadata, ceiling_bytes, found);
    if kind != FILE {
        return WatchedFile {
            size: match kind {
                SYMLINK => metadata.len(),
                _ => 0,
            },
            ..row
        };
    }

    let over_the_ceiling = metadata.len() > ceiling_bytes;
    let digest = match over_the_ceiling {
        true => None,
        false => hashed(path),
    };
    WatchedFile {
        readable: digest.is_some(),
        digest,
        size: metadata.len(),
        over_the_ceiling,
        ..row
    }
}

fn owned(path: &str, metadata: &fs::Metadata, ceiling_bytes: u64, found: Found) -> WatchedFile {
    WatchedFile {
        path: path.to_string(),
        present: true,
        readable: true,
        digest: None,
        size: 0,
        mode: Some(mode_of(metadata)),
        uid: Some(metadata.uid()),
        gid: Some(metadata.gid()),
        over_the_ceiling: false,
        ceiling_bytes,
        found,
    }
}

pub(super) fn absent(path: &str, ceiling_bytes: u64) -> WatchedFile {
    WatchedFile {
        path: path.to_string(),
        present: false,
        readable: false,
        digest: None,
        size: 0,
        mode: None,
        uid: None,
        gid: None,
        over_the_ceiling: false,
        ceiling_bytes,
        found: Found::default(),
    }
}

pub(super) fn kind_of(metadata: &fs::Metadata) -> &'static str {
    let kind = metadata.file_type();
    if kind.is_file() {
        FILE
    } else if kind.is_dir() {
        DIRECTORY
    } else if kind.is_symlink() {
        SYMLINK
    } else {
        OTHER
    }
}

fn hashed(path: &str) -> Option<String> {
    fs::read(path).ok().map(|bytes| hex(&sha256(&bytes)))
}

pub(super) fn path_directory(path: &Path) -> WatchedDirectory {
    let shown = path.display().to_string();
    let Ok(metadata) = fs::metadata(path) else {
        return WatchedDirectory {
            path: shown,
            present: false,
            mode: None,
            uid: None,
            gid: None,
        };
    };

    WatchedDirectory {
        path: shown,
        present: true,
        mode: Some(mode_of(&metadata)),
        uid: Some(metadata.uid()),
        gid: Some(metadata.gid()),
    }
}

fn mode_of(metadata: &fs::Metadata) -> String {
    format!("{:04o}", metadata.permissions().mode() & PERMISSION_BITS)
}
