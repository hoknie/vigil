use vigil_collect::{Mounted, holds_files_of_this_macos_host};

const ROOT: &str = "/";

const SYSTEM_VOLUMES: &str = "/System/Volumes/";

const DATA_VOLUME: &str = "/System/Volumes/Data";

pub fn measured_on_macos(mount: &Mounted) -> bool {
    if !mount.local() || !holds_files_of_this_macos_host(&mount.kind) {
        return false;
    }
    if mount.snapshot() && mount.target != ROOT {
        return false;
    }

    !mount.target.starts_with(SYSTEM_VOLUMES) || mount.target == DATA_VOLUME
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL: u32 = 0x0000_1000;

    const READ_ONLY: u32 = 0x0000_0001;

    const SNAPSHOT: u32 = 0x4000_0000;

    fn mounted(target: &str, kind: &str, flags: u32) -> Mounted {
        Mounted {
            source: "/dev/disk3s5".into(),
            target: target.into(),
            kind: kind.into(),
            flags,
            device: 0x0100_0010,
            block_bytes: 4096,
            blocks: 100,
            blocks_available: 40,
            files: 1000,
            files_free: 900,
        }
    }

    #[test]
    fn the_sealed_system_and_the_volume_the_host_writes_to_are_measured() {
        assert!(measured_on_macos(&mounted(
            "/",
            "apfs",
            LOCAL | READ_ONLY | SNAPSHOT
        )));
        assert!(measured_on_macos(&mounted(DATA_VOLUME, "apfs", LOCAL)));
        assert!(measured_on_macos(&mounted("/Volumes/Backup", "hfs", LOCAL)));
    }

    #[test]
    fn the_volumes_the_system_keeps_to_itself_are_not_measured_a_second_time() {
        for volume in [
            "/System/Volumes/VM",
            "/System/Volumes/Preboot",
            "/System/Volumes/Update",
            "/System/Volumes/xarts",
            "/System/Volumes/iSCPreboot",
            "/System/Volumes/Hardware",
        ] {
            assert!(
                !measured_on_macos(&mounted(volume, "apfs", LOCAL)),
                "{volume} shares its container with the data volume, or lives in the one only \
                 the firmware writes to, and one container filling up is one finding and not \
                 five"
            );
        }
    }

    #[test]
    fn a_snapshot_mounted_for_a_while_is_not_a_filesystem_the_host_fills() {
        assert!(!measured_on_macos(&mounted(
            "/Volumes/com.apple.TimeMachine.localsnapshots/Backups.backupdb/x",
            "apfs",
            LOCAL | READ_ONLY | SNAPSHOT
        )));
    }

    #[test]
    fn what_holds_no_file_of_this_host_is_not_measured() {
        assert!(!measured_on_macos(&mounted("/dev", "devfs", LOCAL)));
        assert!(!measured_on_macos(&mounted(
            "/System/Volumes/Data/home",
            "autofs",
            0
        )));
        assert!(!measured_on_macos(&mounted("/Volumes/share", "smbfs", 0)));
        assert!(
            !measured_on_macos(&mounted("/Volumes/stick", "apfs", 0)),
            "a filesystem the kernel does not call local is another host's"
        );
    }
}
