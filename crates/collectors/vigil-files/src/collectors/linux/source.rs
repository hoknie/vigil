use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::Rfc3339;

use super::FilesCollector;
use super::plan::Plan;
use crate::types::Listing;

pub(super) const NAME: &str = "files";

pub(super) const PATH_CEILING: usize = 256;

pub(super) const DIRECTORIES_OF_THE_PATH: &[&str] = &[
    "/usr/local/sbin",
    "/usr/local/bin",
    "/usr/sbin",
    "/usr/bin",
    "/sbin",
    "/bin",
];

impl FilesCollector {
    pub fn new(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        watched: &[(String, u64)],
    ) -> Self {
        FilesCollector::with_directories(now, watched, DIRECTORIES_OF_THE_PATH)
    }

    pub fn with_directories(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        watched: &[(String, u64)],
        directories: &[&str],
    ) -> Self {
        FilesCollector::of(
            now,
            Plan::Named(watched.iter().take(PATH_CEILING).cloned().collect()),
            directories,
        )
    }

    pub fn listed(now: impl Fn() -> Rfc3339 + Send + Sync + 'static, listing: Listing) -> Self {
        FilesCollector::listed_with_directories(now, listing, DIRECTORIES_OF_THE_PATH)
    }

    pub fn listed_with_directories(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        listing: Listing,
        directories: &[&str],
    ) -> Self {
        FilesCollector::of(
            now,
            Plan::Listed {
                listing,
                lists: Mutex::new(Default::default()),
            },
            directories,
        )
    }

    fn of(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        plan: Plan,
        directories: &[&str],
    ) -> Self {
        FilesCollector {
            now: Box::new(now),
            plan,
            directories: directories.iter().map(PathBuf::from).collect(),
            last: Mutex::new(None),
        }
    }
}
