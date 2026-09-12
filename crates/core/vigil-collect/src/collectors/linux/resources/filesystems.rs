use rustix::fs::statvfs;

use super::ResourcesCollector;
use super::files::MOUNTS;
use crate::parsers::{Filesystem, MountPoint, free_percent_step};

impl ResourcesCollector {
    pub(super) fn filesystems(&self) -> (Vec<Filesystem>, Vec<String>) {
        let Ok(mounted) = self.files.mounted() else {
            return (
                Vec::new(),
                vec![format!(
                    "{} could not be read, so what this host has mounted is unknown and a \
                     filesystem filling up on it will pass unseen",
                    self.files.path(MOUNTS).display()
                )],
            );
        };

        let mut read = Vec::new();
        let mut refused = Vec::new();

        for mount in mounted {
            match measured(&mount) {
                Some(filesystem) => read.push(filesystem),
                None => refused.push(format!(
                    "{} is mounted and would not say how full it is, so a filesystem filling up \
                     there will pass unseen: a reading this agent could not take rather than \
                     room it has",
                    mount.target
                )),
            }
        }

        (read, refused)
    }
}

fn measured(mount: &MountPoint) -> Option<Filesystem> {
    let room = statvfs(mount.target.as_str()).ok()?;
    let block = match room.f_frsize {
        0 => room.f_bsize,
        size => size,
    };

    Some(Filesystem {
        mount: mount.target.clone(),
        device: mount.source.clone(),
        kind: mount.kind.clone(),
        read_only: mount.read_only,
        total_bytes: room.f_blocks.saturating_mul(block),
        free_percent_step: free_percent_step(room.f_bavail, room.f_blocks),
        free_inodes_percent_step: free_percent_step(room.f_ffree, room.f_files),
    })
}
