use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::parsers::{
    MemoryFacts, MountPoint, holds_files_of_this_host, parse_boot_id, parse_meminfo, parse_mounts,
    parse_uptime_seconds,
};

pub(super) const BOOT_ID: &str = "sys/kernel/random/boot_id";

pub(super) const UPTIME: &str = "uptime";

pub(super) const MEMINFO: &str = "meminfo";

pub(super) const MOUNTS: &str = "self/mounts";

const CEILING_BYTES: u64 = 128 * 1024;

const MOUNT_CEILING: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Refusal {
    Absent,
    Denied,
    Unreadable,
}

pub(super) struct Files {
    proc_directory: PathBuf,
}

impl Files {
    pub(super) fn under(proc_directory: impl Into<PathBuf>) -> Self {
        Files {
            proc_directory: proc_directory.into(),
        }
    }

    pub(super) fn path(&self, file: &str) -> PathBuf {
        self.proc_directory.join(file)
    }

    pub(super) fn boot_id(&self) -> Result<String, Refusal> {
        let text = read(&self.path(BOOT_ID))?;
        parse_boot_id(&text).ok_or(Refusal::Unreadable)
    }

    pub(super) fn uptime_seconds(&self) -> Result<u64, Refusal> {
        let text = read(&self.path(UPTIME))?;
        parse_uptime_seconds(&text).ok_or(Refusal::Unreadable)
    }

    pub(super) fn memory(&self) -> Result<MemoryFacts, Refusal> {
        let text = read(&self.path(MEMINFO))?;
        parse_meminfo(&text).ok_or(Refusal::Unreadable)
    }

    pub(super) fn mounted(&self) -> Result<Vec<MountPoint>, Refusal> {
        let text = read(&self.path(MOUNTS))?;
        Ok(parse_mounts(&text)
            .into_iter()
            .filter(|mount| holds_files_of_this_host(&mount.kind))
            .take(MOUNT_CEILING)
            .collect())
    }
}

fn read(path: &Path) -> Result<String, Refusal> {
    let mut text = String::new();

    match fs::File::open(path) {
        Ok(file) => {
            use std::io::Read;
            file.take(CEILING_BYTES)
                .read_to_string(&mut text)
                .map_err(|_| Refusal::Unreadable)?;
            Ok(text)
        }
        Err(error) => Err(match error.kind() {
            ErrorKind::NotFound => Refusal::Absent,
            ErrorKind::PermissionDenied => Refusal::Denied,
            _ => Refusal::Unreadable,
        }),
    }
}
