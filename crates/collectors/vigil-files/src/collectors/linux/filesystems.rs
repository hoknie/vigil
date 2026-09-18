use std::collections::BTreeSet;
use std::fs;

use crate::helpers::device_numbers;
use crate::parsers::mounts_in;
use crate::types::{Devices, Mount};

const MOUNT_TABLE: &str = "/proc/self/mountinfo";

pub(super) struct Filesystems {
    mounts: Option<Vec<Mount>>,
    points: BTreeSet<String>,
    devices: Devices,
}

impl Filesystems {
    pub(super) fn read(devices: Devices) -> Filesystems {
        Filesystems::of(fs::read_to_string(MOUNT_TABLE).ok().as_deref(), devices)
    }

    pub(super) fn of(table: Option<&str>, devices: Devices) -> Filesystems {
        let mounts = table.map(mounts_in).filter(|mounts| !mounts.is_empty());
        let points = mounts
            .iter()
            .flatten()
            .map(|mount| mount.mount_point.clone())
            .collect();
        Filesystems {
            mounts,
            points,
            devices,
        }
    }

    pub(super) fn known(&self) -> bool {
        self.mounts.is_some()
    }

    pub(super) fn crossed(&self, path: &str, device: u64, from: u64) -> bool {
        device != from || self.points.contains(path)
    }

    pub(super) fn enters(&self, path: &str, device: u64, from: u64) -> Result<(), String> {
        let Some(mounts) = &self.mounts else {
            return match device == from {
                true => Ok(()),
                false => Err(format!("{path} (another filesystem)")),
            };
        };
        let Some(mount) = mount_of(mounts, path, device) else {
            return Ok(());
        };
        match self.devices.allows(mount) {
            true => Ok(()),
            false => Err(format!("{path} ({})", mount.filesystem)),
        }
    }
}

fn mount_of<'a>(mounts: &'a [Mount], path: &str, device: u64) -> Option<&'a Mount> {
    if let Some(mounted) = mounts.iter().rev().find(|mount| mount.mount_point == path) {
        return Some(mounted);
    }
    let numbers = device_numbers(device);
    let deepest = |mounts: &mut dyn Iterator<Item = &'a Mount>| {
        mounts
            .filter(|mount| mount.holds(path))
            .max_by_key(|mount| mount.mount_point.len())
    };

    deepest(&mut mounts.iter().filter(|mount| mount.device == numbers))
        .or_else(|| mounts.iter().find(|mount| mount.device == numbers))
        .or_else(|| deepest(&mut mounts.iter()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &str = "\
22 1 8:1 / / rw - ext4 /dev/sda1 rw
23 22 0:21 / /proc rw - proc proc rw
40 22 0:45 / /mnt/nfs rw - nfs4 server:/export rw
41 22 8:1 /srv/backup /mnt/backup rw - ext4 /dev/sda1 rw
";

    fn device(major: u64, minor: u64) -> u64 {
        (major << 8) | minor
    }

    fn with(exclude: &[&str]) -> Filesystems {
        Filesystems::of(
            Some(TABLE),
            Devices {
                include: Vec::new(),
                exclude: exclude.iter().map(|entry| entry.to_string()).collect(),
            },
        )
    }

    #[test]
    fn the_filesystem_of_the_kernel_is_never_entered_and_says_what_it_is() {
        let refusal = with(&[])
            .enters("/proc", device(0, 21), device(8, 1))
            .expect_err("never walked");

        assert_eq!(refusal, "/proc (proc)");
    }

    #[test]
    fn an_excluded_mount_is_not_entered_and_the_disk_around_it_still_is() {
        let filesystems = with(&["nfs4"]);

        assert!(
            filesystems
                .enters("/mnt/nfs", device(0, 45), device(8, 1))
                .is_err()
        );
        assert!(
            filesystems
                .enters("/etc", device(8, 1), device(8, 1))
                .is_ok()
        );
    }

    #[test]
    fn a_directory_bound_onto_another_place_of_the_same_disk_is_still_a_mount_to_decide_about() {
        let filesystems = with(&["/mnt/backup"]);

        assert!(
            filesystems.crossed("/mnt/backup", device(8, 1), device(8, 1)),
            "a bind mount keeps the device of the disk it came from, so only its place in the \
             mount table says a walk is stepping into it"
        );
        assert!(
            filesystems
                .enters("/mnt/backup", device(8, 1), device(8, 1))
                .is_err()
        );
        assert!(!filesystems.crossed("/mnt/elsewhere", device(8, 1), device(8, 1)));
    }

    #[test]
    fn with_no_mount_table_a_walk_stays_on_the_filesystem_it_started_on() {
        let blind = Filesystems::of(None, Devices::default());

        assert!(!blind.known());
        assert!(blind.enters("/etc", device(8, 1), device(8, 1)).is_ok());
        assert!(blind.enters("/proc", device(0, 21), device(8, 1)).is_err());
    }
}
