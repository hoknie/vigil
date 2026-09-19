use std::fs;
use std::io::{self, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::place::writing_path;
use super::private::create_owner_only;

pub const RUNNING: &str = "running";

pub const REFUSED: &str = "refused";

pub const STOPPED: &str = "stopped";

pub const ABSENT: &str = "absent";

pub const STATUS_CEILING_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpoolerStatus {
    pub state: String,
    pub since: String,
    pub program: String,
    pub arguments_recorded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

impl SpoolerStatus {
    pub fn read(path: &Path) -> Result<SpoolerStatus, io::Error> {
        let held = fs::metadata(path)?;
        if held.len() > STATUS_CEILING_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} bytes, over the {STATUS_CEILING_BYTES} a status is",
                    held.len()
                ),
            ));
        }
        let text = fs::read(path)?;
        serde_json::from_slice(&text)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        let temporary = writing_path(path);
        {
            let mut file = create_owner_only(&temporary, false)?;
            let text = serde_json::to_vec_pretty(self)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
            file.write_all(&text)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
        }
        fs::rename(&temporary, path)
    }

    pub fn is_running(&self) -> bool {
        self.state == RUNNING
    }
}
