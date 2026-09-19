const DISK: &str = "disk";

pub fn whole_disk_of(device: &str) -> Option<&str> {
    let name = device.strip_prefix("/dev/").unwrap_or(device);
    let number = name.strip_prefix(DISK)?;
    let digits = number
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(number.len());
    if digits == 0 {
        return None;
    }

    Some(&name[..DISK.len() + digits])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_volume_of_an_apfs_container_is_backed_by_the_container_it_shares_its_room_with() {
        assert_eq!(whole_disk_of("/dev/disk3s1s1"), Some("disk3"));
        assert_eq!(whole_disk_of("/dev/disk3s5"), Some("disk3"));
        assert_eq!(whole_disk_of("disk12s2"), Some("disk12"));
        assert_eq!(whole_disk_of("/dev/disk4"), Some("disk4"));
    }

    #[test]
    fn a_source_that_names_no_disk_backs_nothing_by_a_disk() {
        for source in [
            "map auto_home",
            "devfs",
            "//user@server/share",
            "/dev/diskless",
            "",
        ] {
            assert_eq!(whole_disk_of(source), None, "{source}");
        }
    }
}
