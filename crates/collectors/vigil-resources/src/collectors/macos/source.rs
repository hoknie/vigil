use vigil_collect::sysctl_named;

use crate::parsers::{MemoryFacts, parse_boot_session, parse_boot_time, parse_memory_size};

pub(super) const NAME: &str = "resources";

pub(super) const TOLERANCE_SECONDS: i64 = 2;

pub(super) const BOOT_SESSION: &str = "kern.bootsessionuuid";

pub(super) const BOOT_TIME: &str = "kern.boottime";

pub(super) const MEMORY_SIZE: &str = "hw.memsize";

pub(super) fn boot_session() -> Option<String> {
    parse_boot_session(&sysctl_named(BOOT_SESSION).ok()?)
}

pub(super) fn boot_time() -> Option<i64> {
    parse_boot_time(&sysctl_named(BOOT_TIME).ok()?)
}

pub(super) fn memory() -> Option<MemoryFacts> {
    parse_memory_size(&sysctl_named(MEMORY_SIZE).ok()?)
}
