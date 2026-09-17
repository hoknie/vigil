use std::fs;
use std::path::{Path, PathBuf};

use rustix::fs::{major, minor, stat};

use crate::ports::BlockTree;
use crate::types::BlockEntry;

pub(super) const SYS: &str = "/sys";

const BY_NUMBER: &str = "dev/block";

const BY_NAME: &str = "class/block";

const PARTITION: &str = "partition";

const MADE_OF: &str = "slaves";

const PARTS_CEILING: usize = 32;

pub(super) struct Sysfs {
    sys_directory: PathBuf,
}

impl Sysfs {
    pub(super) fn under(sys_directory: impl Into<PathBuf>) -> Sysfs {
        Sysfs {
            sys_directory: sys_directory.into(),
        }
    }

    pub(super) fn block_of(&self, mount: &str) -> Option<String> {
        let held = stat(mount).ok()?.st_dev;
        let (major, minor) = (major(held), minor(held));
        if major == 0 {
            return None;
        }

        let at = self
            .sys_directory
            .join(BY_NUMBER)
            .join(format!("{major}:{minor}"));

        named(&fs::canonicalize(at).ok()?)
    }
}

impl BlockTree for Sysfs {
    fn entry(&self, name: &str) -> BlockEntry {
        let at = self.sys_directory.join(BY_NAME).join(name);

        if at.join(PARTITION).exists()
            && let Ok(whole) = fs::canonicalize(&at)
            && let Some(disk) = whole.parent().and_then(named)
        {
            return BlockEntry::partition_of(disk);
        }

        let Ok(parts) = fs::read_dir(at.join(MADE_OF)) else {
            return BlockEntry::nothing();
        };
        let mut made_of: Vec<String> = parts
            .flatten()
            .take(PARTS_CEILING)
            .filter_map(|part| part.file_name().to_str().map(str::to_string))
            .collect();
        made_of.sort();
        made_of.dedup();

        BlockEntry {
            partition_of: None,
            made_of,
        }
    }
}

fn named(at: &Path) -> Option<String> {
    at.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}
