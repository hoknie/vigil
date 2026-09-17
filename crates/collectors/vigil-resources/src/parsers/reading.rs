use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::filesystem::Filesystem;
use super::meminfo::MemoryFacts;

pub const SOURCE: &str = "resources";

pub const BOOT: &str = "boot|current";

pub const MEMORY: &str = "memory|summary";

pub const FILESYSTEM: &str = "fs";

pub struct ResourcesReading<'a> {
    pub boot_id: Option<&'a str>,
    pub booted_at: Option<i64>,
    pub memory: Option<MemoryFacts>,
    pub filesystems: &'a [Filesystem],
}

impl ResourcesReading<'_> {
    pub fn says_nothing(&self) -> bool {
        self.boot_id.is_none()
            && self.booted_at.is_none()
            && self.memory.is_none()
            && self.filesystems.is_empty()
    }
}

pub fn resources_snapshot(taken_at: &str, reading: &ResourcesReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    add_boot(&mut snapshot, reading);
    add_memory(&mut snapshot, reading);
    add_filesystems(&mut snapshot, reading);

    snapshot
}

fn add_boot(snapshot: &mut Snapshot, reading: &ResourcesReading<'_>) {
    snapshot.items.insert(
        BOOT.to_string(),
        json!({
            "boot_id": reading.boot_id,
            "booted_at": reading.booted_at,
            "readable": reading.boot_id.is_some() && reading.booted_at.is_some(),
        }),
    );
}

fn add_memory(snapshot: &mut Snapshot, reading: &ResourcesReading<'_>) {
    let Some(memory) = reading.memory else {
        return;
    };

    snapshot.items.insert(
        MEMORY.to_string(),
        json!({
            "total_bytes": memory.total_bytes,
            "swap_total_bytes": memory.swap_total_bytes,
            "readable": true,
        }),
    );
}

fn add_filesystems(snapshot: &mut Snapshot, reading: &ResourcesReading<'_>) {
    for filesystem in reading.filesystems {
        snapshot.items.insert(
            format!("{FILESYSTEM}|{}", filesystem.mount),
            json!({
                "mount": filesystem.mount,
                "device": filesystem.device,
                "type": filesystem.kind,
                "storage": filesystem.storage,
                "storage_from": filesystem.backing.name(),
                "read_only": filesystem.read_only,
                "total_bytes": filesystem.total_bytes,
                "free_percent_step": filesystem.free_percent_step,
                "free_inodes_percent_step": filesystem.free_inodes_percent_step,
            }),
        );
    }
}

pub fn booted_at_of(item: &Value) -> Option<i64> {
    item["booted_at"].as_i64()
}

#[cfg(test)]
mod tests {
    use vigil_model::class_of;

    use super::super::filesystem::Filesystem;
    use super::*;

    const AT: &str = "2026-09-11T12:00:00.000Z";

    const BOOT_ID: &str = "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31";

    fn host() -> MemoryFacts {
        MemoryFacts {
            total_bytes: 8_039_152 * 1024,
            swap_total_bytes: Some(1_048_572 * 1024),
        }
    }

    fn mounted() -> Vec<Filesystem> {
        vec![Filesystem {
            mount: "/var".into(),
            device: "/dev/sda2".into(),
            kind: "ext4".into(),
            storage: "sda".into(),
            backing: crate::types::Backing::Disk,
            read_only: false,
            total_bytes: 20_938_809_344,
            free_percent_step: Some(35),
            free_inodes_percent_step: Some(85),
        }]
    }

    fn reading<'a>(
        boot_id: Option<&'a str>,
        booted_at: Option<i64>,
        filesystems: &'a [Filesystem],
    ) -> ResourcesReading<'a> {
        ResourcesReading {
            boot_id,
            booted_at,
            memory: Some(host()),
            filesystems,
        }
    }

    #[test]
    fn the_boot_this_host_is_running_and_the_size_of_it_are_a_row_each() {
        let taken =
            resources_snapshot(AT, &reading(Some(BOOT_ID), Some(1_757_419_200), &mounted()));

        assert_eq!(taken.source, SOURCE);
        assert_eq!(taken.items.len(), 3);
        assert_eq!(taken.items[BOOT]["boot_id"], BOOT_ID);
        assert_eq!(taken.items[BOOT]["booted_at"], 1_757_419_200);
        assert_eq!(taken.items[BOOT]["readable"], true);
        assert_eq!(taken.items[MEMORY]["total_bytes"], 8_039_152u64 * 1024);
        assert_eq!(taken.items[MEMORY]["swap_total_bytes"], 1_048_572u64 * 1024);
    }

    #[test]
    fn each_row_of_the_reading_is_a_class_of_its_own() {
        let taken =
            resources_snapshot(AT, &reading(Some(BOOT_ID), Some(1_757_419_200), &mounted()));

        let mut classes: Vec<&str> = taken.items.keys().map(|key| class_of(key)).collect();
        classes.sort_unstable();

        assert_eq!(classes, vec!["boot", "fs", "memory"]);
    }

    #[test]
    fn the_moment_this_host_booted_and_the_boot_it_is_running_share_one_row() {
        let taken =
            resources_snapshot(AT, &reading(Some(BOOT_ID), Some(1_757_419_200), &mounted()));

        assert!(
            taken.items[BOOT].get("boot_id").is_some()
                && taken.items[BOOT].get("booted_at").is_some(),
            "a reboot moves both of these at once. Two rows would make one reboot look like a \
             reboot and a clock that jumped years, and the second of those would be a lie"
        );
    }

    #[test]
    fn a_kernel_that_would_not_say_which_boot_this_is_leaves_a_row_that_says_so() {
        let taken = resources_snapshot(AT, &reading(None, Some(1_757_419_200), &mounted()));

        assert_eq!(taken.items[BOOT]["boot_id"], Value::Null);
        assert_eq!(
            taken.items[BOOT]["readable"], false,
            "a null that nothing marks reads as a host without a boot identifier, and there is \
             no such host"
        );
    }

    #[test]
    fn a_host_whose_memory_could_not_be_read_carries_no_row_saying_it_has_none() {
        let taken = resources_snapshot(
            AT,
            &ResourcesReading {
                boot_id: Some(BOOT_ID),
                booted_at: Some(1_757_419_200),
                memory: None,
                filesystems: &[],
            },
        );

        assert!(!taken.items.contains_key(MEMORY));
        assert_eq!(taken.items.len(), 1);
    }

    #[test]
    fn a_reading_of_a_host_that_did_not_move_is_the_reading_before_it_down_to_the_byte() {
        let first =
            resources_snapshot(AT, &reading(Some(BOOT_ID), Some(1_757_419_200), &mounted()));
        let later = resources_snapshot(
            "2026-09-11T12:01:00.000Z",
            &reading(Some(BOOT_ID), Some(1_757_419_200), &mounted()),
        );

        assert_eq!(
            first.items, later.items,
            "the load, the free bytes and the seconds since boot are nowhere in this reading, \
             and that is why a host that did not move costs nothing to talk about"
        );
    }

    #[test]
    fn a_mounted_filesystem_is_a_row_naming_the_step_it_is_full_to_and_not_the_bytes_left() {
        let taken =
            resources_snapshot(AT, &reading(Some(BOOT_ID), Some(1_757_419_200), &mounted()));

        assert_eq!(taken.items["fs|/var"]["mount"], "/var");
        assert_eq!(taken.items["fs|/var"]["device"], "/dev/sda2");
        assert_eq!(taken.items["fs|/var"]["type"], "ext4");
        assert_eq!(taken.items["fs|/var"]["read_only"], false);
        assert_eq!(taken.items["fs|/var"]["free_percent_step"], 35);
        assert_eq!(taken.items["fs|/var"]["free_inodes_percent_step"], 85);
        assert!(
            taken.items["fs|/var"].get("free_bytes").is_none(),
            "the bytes free on a filesystem move between any two readings, and a value that \
             moves on every tick is a change on every tick"
        );
    }

    #[test]
    fn a_filesystem_that_counts_no_inodes_says_so_rather_than_saying_it_has_none_left() {
        let mut without = mounted();
        without[0].free_inodes_percent_step = None;

        let taken = resources_snapshot(AT, &reading(Some(BOOT_ID), Some(1_757_419_200), &without));

        assert_eq!(
            taken.items["fs|/var"]["free_inodes_percent_step"],
            Value::Null
        );
    }

    #[test]
    fn a_reading_that_says_nothing_at_all_is_recognisable_before_it_is_written() {
        let nothing = ResourcesReading {
            boot_id: None,
            booted_at: None,
            memory: None,
            filesystems: &[],
        };

        assert!(nothing.says_nothing());
        assert!(!reading(None, Some(1), &[]).says_nothing());
    }
}
