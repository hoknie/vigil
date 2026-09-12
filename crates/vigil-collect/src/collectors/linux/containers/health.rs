use std::fs;

use super::ContainersCollector;
use crate::Health;

impl ContainersCollector {
    pub(super) fn health(&self) -> Health {
        let shown = self.proc_directory.display().to_string();
        if fs::read_dir(&self.proc_directory).is_err() {
            return Health::Unavailable(format!(
                "{shown} cannot be listed, so nothing on this host can be told from a container \
                 on it. This build reads a Linux /proc, and a pid namespace that shows a slice \
                 of one is a reading this agent cannot take rather than a host running nothing"
            ));
        }

        let complaints = self.unread().clone();
        match complaints.is_empty() {
            true => Health::Ok,
            false => Health::Degraded(complaints.join("; ")),
        }
    }
}
