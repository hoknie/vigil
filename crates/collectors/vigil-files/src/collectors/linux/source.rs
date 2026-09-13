use std::path::PathBuf;

use vigil_model::Rfc3339;

use super::FilesCollector;

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
        watched: &[String],
        ceiling_bytes: u64,
    ) -> Self {
        FilesCollector::with_directories(now, watched, ceiling_bytes, DIRECTORIES_OF_THE_PATH)
    }

    pub fn with_directories(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        watched: &[String],
        ceiling_bytes: u64,
        directories: &[&str],
    ) -> Self {
        FilesCollector {
            now: Box::new(now),
            watched: watched
                .iter()
                .take(PATH_CEILING)
                .map(PathBuf::from)
                .collect(),
            directories: directories.iter().map(PathBuf::from).collect(),
            ceiling_bytes,
        }
    }
}
