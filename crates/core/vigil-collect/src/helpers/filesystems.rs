const LOCAL_FILESYSTEMS: &[&str] = &[
    "btrfs", "exfat", "ext2", "ext3", "ext4", "f2fs", "jfs", "msdos", "ntfs", "ntfs3", "overlay",
    "reiserfs", "tmpfs", "vfat", "xfs", "zfs",
];

pub fn holds_files_of_this_host(kind: &str) -> bool {
    LOCAL_FILESYSTEMS.contains(&kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filesystem_that_holds_nothing_of_this_host_is_not_one_of_its_own() {
        for network in ["nfs", "nfs4", "cifs", "smbfs", "fuse.sshfs"] {
            assert!(!holds_files_of_this_host(network), "{network}");
        }
        for pseudo in ["proc", "sysfs", "devtmpfs", "cgroup2"] {
            assert!(!holds_files_of_this_host(pseudo), "{pseudo}");
        }
        for kept in ["tmpfs", "overlay"] {
            assert!(
                holds_files_of_this_host(kept),
                "{kept} holds files a process can write and another can read, and a container \
                 that mounts one of them from this host is holding part of this host"
            );
        }
        for real in ["ext4", "xfs", "btrfs", "zfs"] {
            assert!(holds_files_of_this_host(real), "{real}");
        }
    }
}
