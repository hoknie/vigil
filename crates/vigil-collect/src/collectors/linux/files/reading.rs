use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use vigil_model::Snapshot;

use super::FilesCollector;
use crate::CollectError;
use crate::helpers::{hex, sha256};
use crate::parsers::{FilesReading, WatchedDirectory, WatchedFile, files_snapshot};

const PERMISSION_BITS: u32 = 0o7777;

impl FilesCollector {
    pub(super) fn reading(&self) -> Result<Snapshot, CollectError> {
        if self.watched.is_empty() {
            return Err(CollectError::Absent(
                "no path is named for this collector to check".into(),
            ));
        }

        let files: Vec<WatchedFile> = self.watched.iter().map(|path| self.file(path)).collect();
        let directories: Vec<WatchedDirectory> = self
            .directories
            .iter()
            .map(|path| directory(path))
            .collect();

        Ok(files_snapshot(
            &(self.now)(),
            &FilesReading {
                files: &files,
                directories: &directories,
            },
        ))
    }

    fn file(&self, path: &Path) -> WatchedFile {
        let shown = path.display().to_string();
        let Ok(metadata) = fs::symlink_metadata(path) else {
            return WatchedFile {
                path: shown,
                present: false,
                readable: false,
                digest: None,
                size: 0,
                mode: None,
                uid: None,
                gid: None,
                over_the_ceiling: false,
            };
        };

        let over_the_ceiling = metadata.len() > self.ceiling_bytes;
        let digest = match over_the_ceiling {
            true => None,
            false => fs::read(path).ok().map(|bytes| hex(&sha256(&bytes))),
        };

        WatchedFile {
            path: shown,
            present: true,
            readable: digest.is_some(),
            digest,
            size: metadata.len(),
            mode: Some(mode_of(&metadata)),
            uid: Some(metadata.uid()),
            gid: Some(metadata.gid()),
            over_the_ceiling,
        }
    }
}

fn directory(path: &Path) -> WatchedDirectory {
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
