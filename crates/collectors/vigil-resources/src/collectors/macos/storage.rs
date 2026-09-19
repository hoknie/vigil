use crate::parsers::whole_disk_of;
use crate::ports::BlockTree;
use crate::types::BlockEntry;

pub(super) struct Disks;

impl BlockTree for Disks {
    fn entry(&self, name: &str) -> BlockEntry {
        match whole_disk_of(name) {
            Some(disk) if disk != name => BlockEntry::partition_of(disk),
            _ => BlockEntry::nothing(),
        }
    }
}

pub(super) fn block_of(source: &str) -> Option<&str> {
    let name = source.strip_prefix("/dev/")?;
    whole_disk_of(name).map(|_| name)
}
