use crate::ports::BlockTree;
use crate::types::Backing;

pub const UNNAMED: &str = "?";

pub const DEEPEST: usize = 4;

const BETWEEN: &str = " + ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backed {
    pub name: String,
    pub from: Backing,
}

pub fn backed_by(source: &str, block: Option<&str>, tree: &dyn BlockTree) -> Backed {
    let Some(block) = block else {
        return mounted_by(source);
    };

    match disks_under(block, tree, DEEPEST) {
        disks if disks.is_empty() => Backed {
            name: block.to_string(),
            from: Backing::Device,
        },
        disks => Backed {
            name: disks.join(BETWEEN),
            from: Backing::Disk,
        },
    }
}

fn mounted_by(source: &str) -> Backed {
    match source.trim() {
        "" | UNNAMED => Backed {
            name: UNNAMED.to_string(),
            from: Backing::Unnamed,
        },
        said => Backed {
            name: said.to_string(),
            from: Backing::Source,
        },
    }
}

fn disks_under(name: &str, tree: &dyn BlockTree, depth: usize) -> Vec<String> {
    if depth == 0 {
        return Vec::new();
    }

    let entry = tree.entry(name);
    if entry.says_nothing() {
        return Vec::new();
    }
    if let Some(disk) = entry.partition_of {
        return vec![disk];
    }

    let mut disks: Vec<String> = Vec::new();
    for part in entry.made_of {
        match disks_under(&part, tree, depth - 1) {
            found if found.is_empty() => disks.push(part),
            found => disks.extend(found),
        }
    }
    disks.sort();
    disks.dedup();
    disks
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::types::BlockEntry;

    struct Sample {
        entries: Vec<(&'static str, BlockEntry)>,
        asked: RefCell<Vec<String>>,
    }

    impl Sample {
        fn of(entries: Vec<(&'static str, BlockEntry)>) -> Sample {
            Sample {
                entries,
                asked: RefCell::new(Vec::new()),
            }
        }

        fn asked(&self) -> usize {
            self.asked.borrow().len()
        }
    }

    impl BlockTree for Sample {
        fn entry(&self, name: &str) -> BlockEntry {
            self.asked.borrow_mut().push(name.to_string());
            self.entries
                .iter()
                .find(|(named, _)| *named == name)
                .map(|(_, entry)| entry.clone())
                .unwrap_or_default()
        }
    }

    fn partitioned() -> Sample {
        Sample::of(vec![
            ("sda1", BlockEntry::partition_of("sda")),
            ("sda2", BlockEntry::partition_of("sda")),
            ("sda3", BlockEntry::partition_of("sda")),
            ("sda", BlockEntry::nothing()),
        ])
    }

    #[test]
    fn two_filesystems_on_two_partitions_of_one_disk_are_backed_by_that_disk_and_not_by_two() {
        let sysfs = partitioned();

        let root = backed_by("/dev/sda1", Some("sda1"), &sysfs);
        let var = backed_by("/dev/sda2", Some("sda2"), &sysfs);

        assert_eq!(root.name, "sda");
        assert_eq!(var.name, "sda");
        assert_eq!(root.from, Backing::Disk);
        assert_eq!(
            root.name, var.name,
            "two mounts of the same size are a coincidence; two partitions of one disk are a \
             fact, and a disk that fills up takes both of them down together"
        );
    }

    #[test]
    fn a_volume_stacked_on_a_partition_is_backed_by_the_disk_under_all_of_it() {
        let sysfs = Sample::of(vec![
            ("dm-0", BlockEntry::made_of(&["sda3"])),
            ("dm-1", BlockEntry::made_of(&["sda3"])),
            ("sda3", BlockEntry::partition_of("sda")),
        ]);

        assert_eq!(
            backed_by("/dev/mapper/vg-root", Some("dm-0"), &sysfs).name,
            "sda"
        );
        assert_eq!(
            backed_by("/dev/mapper/vg-home", Some("dm-1"), &sysfs).name,
            "sda"
        );
    }

    #[test]
    fn a_volume_spread_over_two_disks_names_both_of_them_and_names_each_of_them_once() {
        let sysfs = Sample::of(vec![
            ("dm-0", BlockEntry::made_of(&["sdb1", "sdc1", "sdb1"])),
            ("sdb1", BlockEntry::partition_of("sdb")),
            ("sdc1", BlockEntry::partition_of("sdc")),
        ]);

        let backed = backed_by("/dev/mapper/pool-data", Some("dm-0"), &sysfs);

        assert_eq!(backed.name, "sdb + sdc");
        assert_eq!(backed.from, Backing::Disk);
    }

    #[test]
    fn a_whole_disk_mounted_as_it_is_is_named_by_the_device_and_not_left_unnamed() {
        let sysfs = Sample::of(vec![("vda", BlockEntry::nothing())]);

        let backed = backed_by("/dev/vda", Some("vda"), &sysfs);

        assert_eq!(backed.name, "vda");
        assert_eq!(backed.from, Backing::Device);
    }

    #[test]
    fn a_filesystem_on_no_block_device_is_gathered_under_whatever_mounted_it() {
        let sysfs = Sample::of(Vec::new());

        let run = backed_by("tmpfs", None, &sysfs);

        assert_eq!(run.name, "tmpfs");
        assert_eq!(run.from, Backing::Source);
        assert_eq!(
            sysfs.asked(),
            0,
            "a filesystem the kernel gives no block device for is never looked up in /sys, so \
             a host with no /sys at all costs nothing here"
        );
    }

    #[test]
    fn a_host_that_names_neither_a_device_nor_a_source_leaves_the_filesystem_on_its_own() {
        let sysfs = Sample::of(Vec::new());

        for said in ["", "   ", UNNAMED] {
            let backed = backed_by(said, None, &sysfs);
            assert_eq!(backed.name, UNNAMED, "{said:?}");
            assert_eq!(backed.from, Backing::Unnamed, "{said:?}");
        }
    }

    #[test]
    fn a_stack_that_points_at_itself_is_walked_to_a_fixed_depth_and_never_for_ever() {
        let sysfs = Sample::of(vec![
            ("dm-0", BlockEntry::made_of(&["dm-1"])),
            ("dm-1", BlockEntry::made_of(&["dm-0"])),
        ]);

        let backed = backed_by("/dev/mapper/loop", Some("dm-0"), &sysfs);

        assert!(!backed.name.is_empty(), "{backed:?}");
        assert!(
            sysfs.asked() <= DEEPEST,
            "a device tree that loops is a host this agent still has to finish reading: {} \
             lookups",
            sysfs.asked()
        );
    }

    #[test]
    fn what_one_filesystem_costs_to_place_is_one_lookup_for_a_partition_and_two_for_a_volume() {
        let sysfs = partitioned();
        backed_by("/dev/sda1", Some("sda1"), &sysfs);
        assert_eq!(
            sysfs.asked(),
            1,
            "the partition names its disk, so the walk stops at the first step"
        );

        let stacked = Sample::of(vec![
            ("dm-0", BlockEntry::made_of(&["sda3"])),
            ("sda3", BlockEntry::partition_of("sda")),
        ]);
        backed_by("/dev/mapper/vg-root", Some("dm-0"), &stacked);
        assert_eq!(
            stacked.asked(),
            2,
            "a logical volume costs one lookup per layer under it, and the layers are counted \
             here so a change to the walk cannot make the reading cost more without saying so"
        );
    }
}
