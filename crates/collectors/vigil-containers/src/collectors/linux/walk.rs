use std::fs;

use super::ContainersCollector;
use super::source::{CONTAINER_CEILING, PROCESS_CEILING};
use crate::parsers::{ContainerReference, parse_container_reference};
use vigil_collect::CollectError;

pub(super) struct Found {
    pub(super) reference: ContainerReference,
    pub(super) pid: u32,
}

impl ContainersCollector {
    pub(super) fn containers_of_this_host(&self) -> Result<Vec<Found>, CollectError> {
        let mut pids = self.pids()?;
        pids.sort_unstable();

        let mut found: Vec<Found> = Vec::new();
        for pid in pids.into_iter().take(PROCESS_CEILING) {
            if found.len() == CONTAINER_CEILING {
                break;
            }
            let Ok(text) = fs::read_to_string(self.path(&format!("{pid}/cgroup"))) else {
                continue;
            };
            let Some(reference) = parse_container_reference(&text) else {
                continue;
            };
            if found.iter().any(|seen| seen.reference.id == reference.id) {
                continue;
            }
            found.push(Found { reference, pid });
        }

        Ok(found)
    }

    fn pids(&self) -> Result<Vec<u32>, CollectError> {
        let shown = self.proc_directory.display().to_string();
        let entries = fs::read_dir(&self.proc_directory).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => CollectError::Absent(shown.clone()),
            std::io::ErrorKind::PermissionDenied => CollectError::Denied(shown.clone()),
            _ => CollectError::Unreadable(format!("{shown}: {error}")),
        })?;

        Ok(entries
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
            .collect())
    }
}
