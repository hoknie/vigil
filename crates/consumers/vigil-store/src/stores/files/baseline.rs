use std::fs;
use std::path::{Path, PathBuf};

use vigil_model::Snapshot;

use super::private::create_owner_only;
use crate::StoreError;

pub struct Baselines {
    directory: PathBuf,
}

impl Baselines {
    pub fn open(directory: PathBuf) -> Result<Self, StoreError> {
        fs::create_dir_all(&directory).map_err(|error| StoreError::Io(error.to_string()))?;
        Ok(Baselines { directory })
    }

    pub fn read(&self, source: &str) -> Result<Option<Snapshot>, StoreError> {
        let path = self.path_for(source)?;
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|error| StoreError::Corrupt(format!("{}: {error}", path.display()))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(StoreError::Io(format!("{}: {error}", path.display()))),
        }
    }

    pub fn write(&self, snapshot: &Snapshot) -> Result<(), StoreError> {
        let path = self.path_for(&snapshot.source)?;
        let temporary = path.with_extension("json.writing");

        let text = serde_json::to_string(snapshot)
            .map_err(|error| StoreError::Corrupt(error.to_string()))?;
        {
            use std::io::Write;
            let mut file = create_owner_only(&temporary, false)?;
            file.write_all(text.as_bytes())
                .map_err(|error| StoreError::Io(format!("{}: {error}", temporary.display())))?;
        }

        fs::File::open(&temporary)
            .and_then(|file| file.sync_all())
            .map_err(|error| StoreError::Io(format!("{}: {error}", temporary.display())))?;

        fs::rename(&temporary, &path)
            .map_err(|error| StoreError::Io(format!("{}: {error}", path.display())))
    }

    fn path_for(&self, source: &str) -> Result<PathBuf, StoreError> {
        if source.is_empty()
            || !source
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(StoreError::Corrupt(format!(
                "{source:?} is not a usable collector name"
            )));
        }
        Ok(self.directory.join(source).with_extension("json"))
    }

    pub fn forget(&self, source: &str) -> Result<bool, StoreError> {
        let path = self.path_for(source)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(StoreError::Io(format!("{}: {error}", path.display()))),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(name: &str) -> PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vigil-baselines-{}-{name}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos()
                    + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn a_collector_name_that_would_escape_the_directory_is_refused() {
        let baselines = Baselines::open(temporary_directory("escape")).expect("opens");

        let error = baselines
            .read("../../etc/shadow")
            .expect_err("must not be treated as a file name");

        assert!(matches!(error, StoreError::Corrupt(_)), "{error}");
    }

    #[test]
    fn a_written_baseline_is_read_back_whole() {
        let directory = temporary_directory("round-trip");
        let baselines = Baselines::open(directory.clone()).expect("opens");
        let snapshot = crate::conformance::snapshot("network", "2026-09-09T10:00:00.000Z", 443);

        baselines.write(&snapshot).expect("writes");
        let read = baselines
            .read("network")
            .expect("readable")
            .expect("present");

        assert_eq!(read, snapshot);
        let _ = fs::remove_dir_all(&directory);
    }
}
