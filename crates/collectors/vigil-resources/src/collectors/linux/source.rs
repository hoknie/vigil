use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use vigil_model::Rfc3339;

use super::ResourcesCollector;
use super::files::Files;
use super::storage::{SYS, Sysfs};

pub(super) const NAME: &str = "resources";

pub(super) const PROC: &str = "/proc";

pub(super) const TOLERANCE_SECONDS: i64 = 2;

impl ResourcesCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ResourcesCollector::with_sources(now, PROC, wall_clock_seconds)
    }

    pub fn with_sources(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        proc_directory: impl Into<PathBuf>,
        wall_clock: impl Fn() -> Option<i64> + Send + Sync + 'static,
    ) -> Self {
        ResourcesCollector::with_block_devices(now, proc_directory, wall_clock, SYS)
    }

    pub fn with_block_devices(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        proc_directory: impl Into<PathBuf>,
        wall_clock: impl Fn() -> Option<i64> + Send + Sync + 'static,
        sys_directory: impl Into<PathBuf>,
    ) -> Self {
        ResourcesCollector {
            now: Box::new(now),
            wall_clock: Box::new(wall_clock),
            files: Files::under(proc_directory),
            blocks: Sysfs::under(sys_directory),
            booted_at: Mutex::new(None),
        }
    }
}

fn wall_clock_seconds() -> Option<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|since| i64::try_from(since.as_secs()).ok())
}
