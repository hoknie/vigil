use vigil_collect::unescaped;

use crate::types::Mount;

const SEPARATOR: &str = "-";

const MOUNT_POINT: usize = 4;

const DEVICE: usize = 2;

const FIRST_OPTIONAL: usize = 6;

pub fn mounts_in(text: &str) -> Vec<Mount> {
    text.lines().filter_map(mount_of).collect()
}

fn mount_of(line: &str) -> Option<Mount> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let separator = fields
        .iter()
        .skip(FIRST_OPTIONAL)
        .position(|field| *field == SEPARATOR)?
        + FIRST_OPTIONAL;
    let (major, minor) = fields.get(DEVICE)?.split_once(':')?;

    Some(Mount {
        device: (major.parse().ok()?, minor.parse().ok()?),
        mount_point: unescaped(fields.get(MOUNT_POINT)?),
        filesystem: (*fields.get(separator + 1)?).to_string(),
        source: unescaped(fields.get(separator + 2)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
22 1 8:1 / / rw,relatime shared:1 - ext4 /dev/sda1 rw,errors=remount-ro
23 22 0:21 / /proc rw,nosuid,nodev,noexec,relatime shared:12 - proc proc rw
24 22 0:22 / /sys rw,nosuid,nodev,noexec,relatime shared:7 - sysfs sysfs rw
25 24 0:6 / /sys/kernel/security rw,nosuid,nodev,noexec,relatime shared:8 - securityfs securityfs rw
40 22 0:45 / /mnt/my\\040share rw,relatime - nfs4 server:/export/a\\040b rw,vers=4.2
41 22 259:3 / /srv rw,relatime master:3 propagate_from:2 - xfs /dev/nvme0n1p3 rw
";

    #[test]
    fn every_mount_is_read_with_its_device_its_mount_point_its_kind_and_its_source() {
        let mounts = mounts_in(SAMPLE);

        assert_eq!(mounts.len(), 6);
        assert_eq!(
            mounts[0],
            Mount {
                device: (8, 1),
                mount_point: "/".into(),
                filesystem: "ext4".into(),
                source: "/dev/sda1".into(),
            }
        );
        assert_eq!(mounts[1].filesystem, "proc");
        assert_eq!(mounts[5].device, (259, 3));
        assert_eq!(mounts[5].source, "/dev/nvme0n1p3");
    }

    #[test]
    fn a_space_the_kernel_wrote_as_an_escape_is_read_back_as_the_space() {
        let share = &mounts_in(SAMPLE)[4];

        assert_eq!(share.mount_point, "/mnt/my share");
        assert_eq!(share.source, "server:/export/a b");
    }

    #[test]
    fn a_mount_with_no_optional_fields_and_one_with_several_are_both_read() {
        let mounts = mounts_in(SAMPLE);

        assert_eq!(
            mounts[4].filesystem, "nfs4",
            "no optional field before the dash"
        );
        assert_eq!(
            mounts[5].filesystem, "xfs",
            "two optional fields before the dash"
        );
    }

    #[test]
    fn a_line_the_kernel_would_never_write_is_left_out_rather_than_guessed_at() {
        for broken in [
            "",
            "22 1 8:1 / / rw,relatime shared:1 ext4 /dev/sda1 rw",
            "22 1 eight / / rw - ext4 /dev/sda1 rw",
            "22 1 8:1 / / rw -",
        ] {
            assert!(mounts_in(broken).is_empty(), "{broken:?}");
        }
    }
}
