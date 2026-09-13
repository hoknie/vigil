use crate::helpers::unescaped;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountPoint {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub read_only: bool,
}

pub fn parse_mounts(text: &str) -> Vec<MountPoint> {
    let mut mounted = Vec::new();

    for line in text.lines() {
        let mut fields = line.split(' ');
        let (Some(source), Some(target), Some(kind), Some(options)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if target.is_empty() || kind.is_empty() {
            continue;
        }

        mounted.push(MountPoint {
            source: unescaped(source),
            target: unescaped(target),
            kind: kind.to_string(),
            read_only: options.split(',').any(|option| option == "ro"),
        });
    }

    mounted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::holds_files_of_this_host;

    const FROM_A_SMALL_HOST: &str = "sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0\n\
         proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0\n\
         /dev/sda1 / ext4 rw,relatime,errors=remount-ro 0 0\n\
         tmpfs /run tmpfs rw,nosuid,nodev,size=802016k,mode=755 0 0\n\
         /dev/sda2 /var ext4 rw,relatime 0 0\n\
         /dev/sda3 /boot/efi vfat ro,relatime,fmask=0022 0 0\n\
         nfs.example.net:/export /mnt/shared nfs4 rw,relatime,vers=4.2 0 0\n\
         cgroup2 /sys/fs/cgroup cgroup2 rw,nosuid,nodev,noexec,relatime 0 0\n";

    fn targets(text: &str) -> Vec<String> {
        parse_mounts(text)
            .into_iter()
            .filter(|mount| holds_files_of_this_host(&mount.kind))
            .map(|mount| mount.target)
            .collect()
    }

    #[test]
    fn a_filesystem_on_the_other_side_of_a_network_is_never_one_this_collector_asks_about() {
        assert_eq!(
            targets(FROM_A_SMALL_HOST),
            vec!["/", "/run", "/var", "/boot/efi"],
            "asking a network filesystem how full it is waits for a server that may never \
             answer, and a reading that waits forever stops the watch behind it"
        );

        for network in [
            "nfs",
            "nfs4",
            "cifs",
            "smb3",
            "fuse.sshfs",
            "ceph",
            "afs",
            "9p",
        ] {
            assert!(!holds_files_of_this_host(network), "{network}");
        }
    }

    #[test]
    fn what_the_kernel_mounts_for_itself_is_not_a_filesystem_anything_fills_up() {
        for pseudo in [
            "proc",
            "sysfs",
            "devtmpfs",
            "cgroup2",
            "securityfs",
            "debugfs",
        ] {
            assert!(!holds_files_of_this_host(pseudo), "{pseudo}");
        }
    }

    #[test]
    fn a_filesystem_mounted_read_only_is_read_and_marked_rather_than_left_out() {
        let mounted = parse_mounts(FROM_A_SMALL_HOST);
        let efi = mounted
            .iter()
            .find(|mount| mount.target == "/boot/efi")
            .expect("is mounted");

        assert!(efi.read_only);
        assert_eq!(efi.kind, "vfat");
        assert_eq!(efi.source, "/dev/sda3");
        assert!(!mounted[2].read_only);
    }

    #[test]
    fn a_path_with_a_space_in_it_is_the_path_the_kernel_escaped_and_not_the_escape() {
        let mounted = parse_mounts("/dev/sdb1 /mnt/my\\040disk ext4 rw,relatime 0 0\n");

        assert_eq!(mounted[0].target, "/mnt/my disk");
    }

    #[test]
    fn a_line_in_a_shape_we_do_not_know_is_skipped_rather_than_half_read() {
        assert!(parse_mounts("").is_empty());
        assert!(parse_mounts("/dev/sda1 /\n").is_empty());
        assert_eq!(parse_mounts("/dev/sda1 / ext4 rw 0 0\nrubbish\n").len(), 1);
    }
}
