use std::fs;

use super::FilesCollector;
use vigil_collect::Health;

impl FilesCollector {
    pub(super) fn health(&self) -> Health {
        if self.watched.is_empty() {
            return Health::Unavailable(
                "there is nothing to check: no path is named for this collector. What it watches \
                 is a list in the configuration file and never a walk of a tree, because a walk \
                 that hashes everything is the first thing an operator turns off"
                    .into(),
            );
        }

        let mut complaints: Vec<String> = Vec::new();
        for path in &self.watched {
            let shown = path.display();
            match fs::symlink_metadata(path) {
                Err(_) => {}
                Ok(metadata) if metadata.len() > self.ceiling_bytes => complaints.push(format!(
                    "{shown} is {} bytes, over the {} this collector hashes, so a change to it \
                     will be seen in its size and its mode and not in its content",
                    metadata.len(),
                    self.ceiling_bytes
                )),
                Ok(_) if fs::read(path).is_err() => complaints.push(format!(
                    "{shown} is there and cannot be read by this agent, so a change to its \
                     content will pass unseen"
                )),
                Ok(_) => {}
            }
        }

        match complaints.is_empty() {
            true => Health::Ok,
            false => Health::Degraded(complaints.join("; ")),
        }
    }
}
