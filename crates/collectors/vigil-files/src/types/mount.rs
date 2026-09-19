#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub device: (u32, u32),
    pub mount_point: String,
    pub filesystem: String,
    pub source: String,
}

const NEVER_WALKED: &[&str] = &[
    "proc",
    "sysfs",
    "devpts",
    "devtmpfs",
    "cgroup",
    "cgroup2",
    "securityfs",
    "debugfs",
    "tracefs",
    "bpf",
    "mqueue",
    "hugetlbfs",
    "pstore",
    "configfs",
    "fusectl",
    "autofs",
    "binfmt_misc",
    "rpc_pipefs",
    "nsfs",
    "efivarfs",
    "selinuxfs",
    "devfs",
];

impl Mount {
    pub fn pseudo(&self) -> bool {
        NEVER_WALKED.contains(&self.filesystem.as_str())
    }

    pub fn named_by(&self, entry: &str) -> bool {
        let entry = trimmed(entry);
        entry == self.source || entry == trimmed(&self.mount_point) || entry == self.filesystem
    }

    pub fn holds(&self, path: &str) -> bool {
        let point = trimmed(&self.mount_point);
        point == "/"
            || path == point
            || path
                .strip_prefix(point)
                .is_some_and(|rest| rest.starts_with('/'))
    }
}

fn trimmed(path: &str) -> &str {
    match path.len() > 1 {
        true => path.trim_end_matches('/'),
        false => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mounted(point: &str, filesystem: &str, source: &str) -> Mount {
        Mount {
            device: (0, 42),
            mount_point: point.to_string(),
            filesystem: filesystem.to_string(),
            source: source.to_string(),
        }
    }

    #[test]
    fn a_filesystem_that_holds_the_kernel_rather_than_files_is_never_walked() {
        for filesystem in [
            "proc", "sysfs", "cgroup2", "devpts", "nsfs", "bpf", "tracefs",
        ] {
            assert!(mounted("/x", filesystem, "none").pseudo(), "{filesystem}");
        }
        for filesystem in ["devfs", "autofs"] {
            assert!(
                mounted("/dev", filesystem, "devfs").pseudo(),
                "{filesystem} is the kernel of macOS or a promise to mount something later, and \
                 walking it reads devices or wakes a network share"
            );
        }
        for filesystem in ["apfs", "hfs", "nullfs"] {
            assert!(!mounted("/x", filesystem, "none").pseudo(), "{filesystem}");
        }
        for filesystem in ["ext4", "xfs", "tmpfs", "overlay", "nfs4", "btrfs"] {
            assert!(!mounted("/x", filesystem, "none").pseudo(), "{filesystem}");
        }
    }

    #[test]
    fn a_mount_is_named_by_its_device_its_mount_point_or_the_kind_of_its_filesystem() {
        let share = mounted("/mnt/nfs", "nfs4", "server:/export");

        for entry in ["server:/export", "/mnt/nfs", "/mnt/nfs/", "nfs4"] {
            assert!(share.named_by(entry), "{entry}");
        }
        for entry in ["nfs", "/mnt", "/dev/sda1", ""] {
            assert!(!share.named_by(entry), "{entry}");
        }
    }

    #[test]
    fn a_mount_holds_the_paths_under_its_mount_point_and_not_a_neighbour_sharing_its_prefix() {
        let data = mounted("/srv/data", "ext4", "/dev/sdb1");

        assert!(data.holds("/srv/data"));
        assert!(data.holds("/srv/data/a/b"));
        assert!(!data.holds("/srv/database"));
        assert!(mounted("/", "ext4", "/dev/sda1").holds("/anything"));
    }
}
