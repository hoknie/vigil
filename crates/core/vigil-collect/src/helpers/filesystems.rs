const LOCAL_FILESYSTEMS: &[&str] = &[
    "btrfs", "exfat", "ext2", "ext3", "ext4", "f2fs", "jfs", "msdos", "ntfs", "ntfs3", "overlay",
    "reiserfs", "tmpfs", "vfat", "xfs", "zfs",
];

const LOCAL_FILESYSTEMS_OF_MACOS: &[&str] =
    &["apfs", "exfat", "hfs", "msdos", "ntfs", "ufs", "zfs"];

pub fn holds_files_of_this_host(kind: &str) -> bool {
    LOCAL_FILESYSTEMS.contains(&kind)
}

pub fn holds_files_of_this_macos_host(kind: &str) -> bool {
    LOCAL_FILESYSTEMS_OF_MACOS.contains(&kind)
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

    #[test]
    fn a_mac_holds_its_files_on_its_own_kinds_of_filesystem_and_not_on_the_linux_ones() {
        for real in ["apfs", "hfs", "msdos", "exfat"] {
            assert!(holds_files_of_this_macos_host(real), "{real}");
        }
        for pseudo in [
            "devfs", "autofs", "nullfs", "smbfs", "nfs", "afpfs", "webdav",
        ] {
            assert!(
                !holds_files_of_this_macos_host(pseudo),
                "{pseudo} is either the kernel's own, another host's, or a second view of a                  filesystem already listed"
            );
        }
        assert!(
            !holds_files_of_this_host("apfs") && !holds_files_of_this_host("hfs"),
            "the list a Linux host reads stays the list it read before macOS was added"
        );
    }
}
