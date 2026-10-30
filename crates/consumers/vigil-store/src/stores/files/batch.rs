use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::private::create_owner_only;
use crate::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchRecord {
    pub batch_id: String,
    pub findings: usize,
    pub first_event_id: String,
    pub last_event_id: String,
}

pub struct BatchFile {
    path: PathBuf,
}

impl BatchFile {
    pub fn beside(buffer: &Path) -> Self {
        BatchFile {
            path: buffer.with_extension("batch"),
        }
    }

    pub fn read(&self) -> Option<BatchRecord> {
        let text = fs::read_to_string(&self.path).ok()?;
        let value: Value = serde_json::from_str(text.trim()).ok()?;
        let text_of = |field: &str| value.get(field)?.as_str().map(str::to_string);

        let record = BatchRecord {
            batch_id: text_of("batch_id")?,
            findings: usize::try_from(value.get("findings")?.as_u64()?).ok()?,
            first_event_id: text_of("first_event_id")?,
            last_event_id: text_of("last_event_id")?,
        };
        match record.findings {
            0 => None,
            _ => Some(record),
        }
    }

    pub fn write(&self, record: &BatchRecord) -> Result<(), StoreError> {
        let temporary = self.path.with_extension("batch.writing");
        let mut line = json!({
            "batch_id": record.batch_id,
            "findings": record.findings,
            "first_event_id": record.first_event_id,
            "last_event_id": record.last_event_id,
        })
        .to_string();
        line.push('\n');

        let mut file = create_owner_only(&temporary, false)?;
        file.write_all(line.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|error| StoreError::Io(format!("{}: {error}", temporary.display())))?;
        drop(file);

        fs::rename(&temporary, &self.path)
            .map_err(|error| StoreError::Io(format!("{}: {error}", self.path.display())))
    }

    pub fn forget(&self) {
        let _ = fs::remove_file(&self.path);
    }
}
