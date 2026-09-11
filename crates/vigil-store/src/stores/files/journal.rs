use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use vigil_model::Finding;

use super::private::create_owner_only;
use crate::StoreError;
pub struct Replay {
    pub records: Vec<Finding>,
    pub damaged: usize,
}

pub struct Journal {
    path: PathBuf,
    file: File,
    bytes: u64,
}

impl Journal {
    pub fn open(path: PathBuf) -> Result<(Self, Replay), StoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| StoreError::Io(error.to_string()))?;
        }

        let replay = Self::replay(&path)?;
        let file = create_owner_only(&path, true)?;
        let bytes = file
            .metadata()
            .map(|meta| meta.len())
            .map_err(|error| StoreError::Io(error.to_string()))?;

        Ok((Journal { path, file, bytes }, replay))
    }

    fn replay(path: &Path) -> Result<Replay, StoreError> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(StoreError::Io(format!("{}: {error}", path.display()))),
        };

        let mut replay = Replay {
            records: Vec::new(),
            damaged: 0,
        };
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Finding>(line) {
                Ok(record) => replay.records.push(record),
                Err(_) => replay.damaged += 1,
            }
        }
        Ok(replay)
    }

    pub fn append(&mut self, record: &Finding) -> Result<(), StoreError> {
        let mut line = serde_json::to_string(record)
            .map_err(|error| StoreError::Corrupt(error.to_string()))?;
        line.push('\n');

        self.file
            .write_all(line.as_bytes())
            .map_err(|error| StoreError::Io(format!("{}: {error}", self.path.display())))?;
        self.file
            .sync_data()
            .map_err(|error| StoreError::Io(format!("{}: {error}", self.path.display())))?;
        self.bytes += line.len() as u64;
        Ok(())
    }

    pub fn rewrite(&mut self, records: &[Finding]) -> Result<(), StoreError> {
        let temporary = self.path.with_extension("ndjson.writing");
        {
            let file = create_owner_only(&temporary, false)?;
            let mut writer = BufWriter::new(file);
            for record in records {
                let line = serde_json::to_string(record)
                    .map_err(|error| StoreError::Corrupt(error.to_string()))?;
                writeln!(writer, "{line}")
                    .map_err(|error| StoreError::Io(format!("{}: {error}", temporary.display())))?;
            }
            let file = writer
                .into_inner()
                .map_err(|error| StoreError::Io(error.to_string()))?;
            file.sync_all()
                .map_err(|error| StoreError::Io(format!("{}: {error}", temporary.display())))?;
        }

        fs::rename(&temporary, &self.path)
            .map_err(|error| StoreError::Io(format!("{}: {error}", self.path.display())))?;

        let (reopened, _) = Journal::open(self.path.clone())?;
        *self = reopened;
        Ok(())
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::finding;

    fn temporary_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "vigil-journal-{}-{name}-{}.ndjson",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn what_was_appended_comes_back_in_order() {
        let path = temporary_path("order");
        let (mut journal, replay) = Journal::open(path.clone()).expect("opens");
        assert!(replay.records.is_empty(), "a new journal replays as empty");

        journal
            .append(&finding("a", "2026-09-09T10:00:00.000Z"))
            .expect("appends");
        journal
            .append(&finding("b", "2026-09-09T10:01:00.000Z"))
            .expect("appends");

        let (_, replay) = Journal::open(path.clone()).expect("reopens");
        assert_eq!(replay.records.len(), 2);
        assert_eq!(replay.records[0].finding_key, "a");
        assert_eq!(replay.records[1].finding_key, "b");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_torn_last_line_costs_one_record_and_not_the_whole_store() {
        let path = temporary_path("torn");
        let (mut journal, _) = Journal::open(path.clone()).expect("opens");
        journal
            .append(&finding("intact", "2026-09-09T10:00:00.000Z"))
            .expect("appends");
        drop(journal);

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("reopen");
        file.write_all(b"{\"event_id\":\"half-written\",\"find")
            .expect("tear");
        drop(file);

        let (_, replay) = Journal::open(path.clone()).expect("still opens");

        assert_eq!(replay.records.len(), 1, "the intact record survives");
        assert_eq!(replay.damaged, 1, "and the loss is counted, not hidden");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn rewriting_leaves_only_what_it_was_given_and_shrinks_the_file() {
        let path = temporary_path("rewrite");
        let (mut journal, _) = Journal::open(path.clone()).expect("opens");
        for index in 0..20 {
            journal
                .append(&finding(
                    &format!("key-{index}"),
                    "2026-09-09T10:00:00.000Z",
                ))
                .expect("appends");
        }
        let before = journal.bytes();

        let keep = vec![finding("key-7", "2026-09-09T10:00:00.000Z")];
        journal.rewrite(&keep).expect("rewrites");

        assert!(
            journal.bytes() < before,
            "compaction has to reclaim the space"
        );
        let (_, replay) = Journal::open(path.clone()).expect("reopens");
        assert_eq!(replay.records.len(), 1);
        assert_eq!(replay.records[0].finding_key, "key-7");
        let _ = fs::remove_file(&path);
    }
}
