use vigil_collect::{Mounted, mounted};

use super::storage::{Disks, block_of};
use crate::helpers::measured_on_macos;
use crate::parsers::{Filesystem, backed_by, free_percent_step};

const MOUNT_CEILING: usize = 64;

pub(super) fn filesystems() -> (Vec<Filesystem>, Vec<String>) {
    let table = match mounted() {
        Ok(table) => table,
        Err(error) => {
            return (
                Vec::new(),
                vec![format!(
                    "the mount table of this host could not be read ({error}), so what it has \
                     mounted is unknown and a filesystem filling up on it will pass unseen"
                )],
            );
        }
    };

    let mut read = Vec::new();
    let mut refused = Vec::new();
    for mount in table
        .iter()
        .filter(|mount| measured_on_macos(mount))
        .take(MOUNT_CEILING)
    {
        match measured(mount) {
            Some(filesystem) => read.push(filesystem),
            None => refused.push(format!(
                "{} is mounted and would not say how full it is, so a filesystem filling up \
                 there will pass unseen: a reading this agent could not take rather than room it \
                 has",
                mount.target
            )),
        }
    }

    (read, refused)
}

fn measured(mount: &Mounted) -> Option<Filesystem> {
    if mount.blocks == 0 || mount.block_bytes == 0 {
        return None;
    }
    let backed = backed_by(&mount.source, block_of(&mount.source), &Disks);

    Some(Filesystem {
        mount: mount.target.clone(),
        device: mount.source.clone(),
        kind: mount.kind.clone(),
        storage: backed.name,
        backing: backed.from,
        read_only: mount.read_only(),
        total_bytes: mount.blocks.saturating_mul(mount.block_bytes),
        free_percent_step: free_percent_step(mount.blocks_available, mount.blocks),
        free_inodes_percent_step: free_percent_step(mount.files_free, mount.files),
    })
}
