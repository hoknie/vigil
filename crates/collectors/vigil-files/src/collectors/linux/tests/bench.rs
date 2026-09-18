use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use vigil_collect::Health;

use crate::collectors::linux::FilesCollector;
use crate::types::{Devices, Listing};

pub(super) struct Bench {
    pub(super) directory: PathBuf,
}

impl Bench {
    pub(super) fn new(named: &str) -> Bench {
        let directory = std::env::temp_dir().join(format!(
            "vigil-files-{named}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a bench to read from");
        Bench { directory }
    }

    pub(super) fn at(&self, name: &str) -> String {
        self.directory.join(name).display().to_string()
    }

    pub(super) fn write(&self, name: &str, text: &str) -> String {
        let at = self.directory.join(name);
        if let Some(parent) = at.parent() {
            fs::create_dir_all(parent).expect("a directory to write in");
        }
        fs::write(&at, text).expect("writes");
        self.chmod(name, 0o644);
        at.display().to_string()
    }

    pub(super) fn chmod(&self, name: &str, mode: u32) {
        fs::set_permissions(self.directory.join(name), fs::Permissions::from_mode(mode))
            .expect("sets the mode");
    }

    pub(super) fn collector(&self, watched: &[String], ceiling: u64) -> FilesCollector {
        let shown = self.directory.display().to_string();
        let hashed: Vec<(String, u64)> =
            watched.iter().map(|path| (path.clone(), ceiling)).collect();

        FilesCollector::with_directories(
            || "2026-09-11T12:00:00.000Z".to_string(),
            &hashed,
            &[&shown],
        )
    }

    pub(super) fn listed(&self, list: &str, max_files: usize) -> FilesCollector {
        FilesCollector::listed_with_directories(
            || "2026-09-18T12:00:00.000Z".to_string(),
            Listing {
                watched_path: self.directory.join(list),
                max_file_size: 1024 * 1024,
                devices: Devices::default(),
                max_files,
            },
            &[],
        )
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

pub(super) fn said(health: &Health) -> String {
    match health {
        Health::Ok => String::new(),
        Health::Degraded(detail) | Health::Unavailable(detail) => detail.clone(),
    }
}
