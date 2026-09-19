use std::collections::BTreeSet;

use vigil_collect::{Mounted, mounted};

use crate::helpers::macos_device_numbers;
use crate::types::{Devices, Mount};

pub(super) struct Filesystems {
    mounts: Option<Vec<Mount>>,
    points: BTreeSet<String>,
    devices: Devices,
}

impl Filesystems {
    pub(super) fn read(devices: Devices) -> Filesystems {
        Filesystems::of(
            mounted()
                .ok()
                .map(|table| table.iter().map(mount_of).collect()),
            devices,
        )
    }

    pub(super) fn of(mounts: Option<Vec<Mount>>, devices: Devices) -> Filesystems {
        let mounts = mounts.filter(|mounts| !mounts.is_empty());
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
        let Some(mount) = holding(mounts, path, device) else {
            return Ok(());
        };
        match self.devices.allows(mount) {
            true => Ok(()),
            false => Err(format!("{path} ({})", mount.filesystem)),
        }
    }
}

fn mount_of(mounted: &Mounted) -> Mount {
    Mount {
        device: macos_device_numbers(mounted.device),
        mount_point: mounted.target.clone(),
        filesystem: mounted.kind.clone(),
        source: mounted.source.clone(),
    }
}

fn holding<'a>(mounts: &'a [Mount], path: &str, device: u64) -> Option<&'a Mount> {
    if let Some(mounted) = mounts.iter().rev().find(|mount| mount.mount_point == path) {
        return Some(mounted);
    }
    let numbers = macos_device_numbers(device);

    mounts
        .iter()
        .filter(|mount| mount.device == numbers && mount.holds(path))
        .max_by_key(|mount| mount.mount_point.len())
        .or_else(|| mounts.iter().find(|mount| mount.device == numbers))
        .or_else(|| {
            mounts
                .iter()
                .filter(|mount| mount.holds(path))
                .max_by_key(|mount| mount.mount_point.len())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYSTEM: u64 = 0x0100_0010;

    const DATA: u64 = 0x0100_0012;

    const KERNEL: u64 = 0x1300_0005;

    fn mounted(device: u64, point: &str, filesystem: &str, source: &str) -> Mount {
        Mount {
            device: macos_device_numbers(device),
            mount_point: point.to_string(),
            filesystem: filesystem.to_string(),
            source: source.to_string(),
        }
    }

    fn table(exclude: &[&str]) -> Filesystems {
        Filesystems::of(
            Some(vec![
                mounted(SYSTEM, "/", "apfs", "/dev/disk3s1s1"),
                mounted(KERNEL, "/dev", "devfs", "devfs"),
                mounted(DATA, "/System/Volumes/Data", "apfs", "/dev/disk3s5"),
                mounted(
                    0x2d00_0001,
                    "/System/Volumes/Data/home",
                    "autofs",
                    "map auto_home",
                ),
            ]),
            Devices {
                include: Vec::new(),
                exclude: exclude.iter().map(|entry| entry.to_string()).collect(),
            },
        )
    }

    #[test]
    fn the_devices_of_the_kernel_are_never_walked_and_say_what_they_are() {
        assert_eq!(
            table(&[]).enters("/dev", KERNEL, SYSTEM),
            Err("/dev (devfs)".to_string())
        );
    }

    #[test]
    fn a_firmlink_into_the_data_volume_is_a_step_onto_that_volume_and_is_walked() {
        let filesystems = table(&[]);

        assert!(
            filesystems.crossed("/usr/local", DATA, SYSTEM),
            "/usr/local sits on the data volume under a path of the system volume, and only its \
             device number says the walk changed filesystems"
        );
        assert_eq!(filesystems.enters("/usr/local", DATA, SYSTEM), Ok(()));
    }

    #[test]
    fn the_data_volume_excluded_by_its_device_is_not_entered_through_a_firmlink_either() {
        assert_eq!(
            table(&["/dev/disk3s5"]).enters("/usr/local", DATA, SYSTEM),
            Err("/usr/local (apfs)".to_string())
        );
    }

    #[test]
    fn with_no_mount_table_a_walk_stays_on_the_filesystem_it_started_on() {
        let blind = Filesystems::of(None, Devices::default());

        assert!(!blind.known());
        assert!(blind.enters("/etc", SYSTEM, SYSTEM).is_ok());
        assert!(blind.enters("/usr/local", DATA, SYSTEM).is_err());
    }

    #[test]
    fn the_mount_table_of_this_mac_is_read_with_its_root_and_its_kernel_devices() {
        let filesystems = Filesystems::read(Devices::default());

        assert!(filesystems.known());
        assert!(filesystems.points.contains("/"));
        assert!(filesystems.points.contains("/dev"));
    }
}
