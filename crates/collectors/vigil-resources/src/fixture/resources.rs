use vigil_collect::holds_files_of_this_host;
use vigil_model::Snapshot;

use crate::parsers::{
    Filesystem, MountPoint, ResourcesReading, backed_by, free_percent_step, parse_boot_id,
    parse_meminfo, parse_mounts, parse_uptime_seconds, resources_snapshot,
};
use crate::ports::BlockTree;
use crate::types::BlockEntry;

const BOOT_ID: &str = "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31\n";

const UPTIME: &str = "128142.56 1005819.02\n";

const MEMINFO: &str = "MemTotal:        8039152 kB\n\
     MemFree:          311104 kB\n\
     MemAvailable:    5120884 kB\n\
     Buffers:          182300 kB\n\
     Cached:          4413028 kB\n\
     SwapTotal:       1048572 kB\n\
     SwapFree:        1048572 kB\n";

const MOUNTS: &str = "sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0\n\
     /dev/sda1 / ext4 rw,relatime,errors=remount-ro 0 0\n\
     tmpfs /run tmpfs rw,nosuid,nodev,size=802016k,mode=755 0 0\n\
     /dev/sda2 /var ext4 rw,relatime 0 0\n\
     /dev/mapper/vg-home /home ext4 rw,relatime 0 0\n\
     /dev/sdb1 /srv xfs rw,relatime 0 0\n\
     cgroup2 /sys/fs/cgroup cgroup2 rw,nosuid,nodev,noexec,relatime 0 0\n";

const BLOCK_OF: &[(&str, &str)] = &[
    ("/", "sda1"),
    ("/var", "sda2"),
    ("/home", "dm-0"),
    ("/srv", "sdb1"),
];

const BLOCK_TREE: &[(&str, &[&str])] = &[
    ("sda1", &["sda"]),
    ("sda2", &["sda"]),
    ("sda3", &["sda"]),
    ("sdb1", &["sdb"]),
];

const BLOCK_BYTES: u64 = 4096;

const ROOM: &[(&str, u64, u64, u64, u64)] = &[
    ("/", 12_812_286, 8_119_004, 3_276_800, 2_901_411),
    ("/run", 200_504, 200_102, 981_342, 981_328),
    ("/var", 5_112_014, 2_111_209, 1_310_720, 1_144_902),
    ("/home", 7_340_032, 4_404_019, 1_835_008, 1_744_902),
    ("/srv", 26_214_400, 18_350_080, 0, 0),
];

struct Sample;

impl BlockTree for Sample {
    fn entry(&self, name: &str) -> BlockEntry {
        match BLOCK_TREE.iter().find(|(named, _)| *named == name) {
            Some((_, [disk])) => BlockEntry::partition_of(*disk),
            Some((_, parts)) => BlockEntry::made_of(parts),
            None if name == "dm-0" => BlockEntry::made_of(&["sda3"]),
            None => BlockEntry::nothing(),
        }
    }
}

const READ_AT: i64 = 1_757_547_342;

pub fn resources() -> Snapshot {
    let uptime = parse_uptime_seconds(UPTIME).expect("the sample uptime reads");
    let memory = parse_meminfo(MEMINFO).expect("the sample memory reads");
    let filesystems: Vec<Filesystem> = parse_mounts(MOUNTS)
        .into_iter()
        .filter(|mount| holds_files_of_this_host(&mount.kind))
        .filter_map(|mount| measured(&mount))
        .collect();

    resources_snapshot(
        "2026-09-09T09:00:00.000Z",
        &ResourcesReading {
            boot_id: parse_boot_id(BOOT_ID).as_deref(),
            booted_at: Some(READ_AT - i64::try_from(uptime).expect("seconds")),
            memory: Some(memory),
            filesystems: &filesystems,
        },
    )
}

fn measured(mount: &MountPoint) -> Option<Filesystem> {
    let (_, blocks, available, inodes, free_inodes) =
        *ROOM.iter().find(|(at, ..)| *at == mount.target)?;
    let block = BLOCK_OF
        .iter()
        .find(|(at, _)| *at == mount.target)
        .map(|(_, block)| *block);
    let backed = backed_by(&mount.source, block, &Sample);

    Some(Filesystem {
        mount: mount.target.clone(),
        device: mount.source.clone(),
        kind: mount.kind.clone(),
        storage: backed.name,
        backing: backed.from,
        read_only: mount.read_only,
        total_bytes: blocks * BLOCK_BYTES,
        free_percent_step: free_percent_step(available, blocks),
        free_inodes_percent_step: free_percent_step(free_inodes, inodes),
    })
}
