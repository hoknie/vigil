use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use super::ContainersCollector;
use crate::parsers::RuntimeSocket;

const PERMISSION_BITS: u32 = 0o7777;

impl ContainersCollector {
    pub(super) fn runtime_sockets(&self) -> Vec<RuntimeSocket> {
        let mut found: Vec<RuntimeSocket> = self
            .socket_paths
            .iter()
            .filter_map(|path| {
                let metadata = fs::symlink_metadata(path).ok()?;
                Some(RuntimeSocket {
                    path: path.display().to_string(),
                    mode: format!("{:04o}", metadata.permissions().mode() & PERMISSION_BITS),
                    uid: metadata.uid(),
                    gid: metadata.gid(),
                })
            })
            .collect();

        found.sort_by(|one, other| one.path.cmp(&other.path));
        found
    }
}
