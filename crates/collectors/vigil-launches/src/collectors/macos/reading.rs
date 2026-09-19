use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use vigil_collect::{CollectError, Collector, Presence, names_of_users, shown_to_the_agent};
use vigil_model::Snapshot;

use super::LaunchesCollector;
use crate::parsers::{ESLOGGER, LaunchReading, launches_snapshot_from, parse_audit_log};
use crate::spool::{AUID_UNSET, Cursor, cursor_path, dropped_note};

const MAX_BYTES_PER_READING: u64 = 2 * 1024 * 1024;

impl LaunchesCollector {
    pub(super) fn reading(&self) -> Result<Snapshot, CollectError> {
        let mut seen = self
            .seen
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let path = &self.spool_path;

        let file = File::open(path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => CollectError::Absent(path.display().to_string()),
            ErrorKind::PermissionDenied => CollectError::Denied(path.display().to_string()),
            _ => CollectError::Unreadable(format!("{}: {error}", path.display())),
        })?;
        let metadata = file
            .metadata()
            .map_err(|error| CollectError::Unreadable(format!("{}: {error}", path.display())))?;

        if metadata.ino() != seen.inode || metadata.len() < seen.offset {
            seen.inode = metadata.ino();
            seen.offset = Cursor::read(&cursor_path(path))
                .filter(|cursor| cursor.inode == metadata.ino() && cursor.offset <= metadata.len())
                .map_or(0, |cursor| cursor.offset);
        }
        let resumed_from = (seen.inode, seen.offset);

        let chunk = read_chunk(&file, seen.offset, path)?;
        let dropped = seen.offset == 0
            && dropped_note(
                chunk
                    .split(|byte| *byte == b'\n')
                    .next()
                    .unwrap_or_default(),
            )
            .is_some();

        let reading = parse_audit_log(&chunk, self.keep_arguments);
        seen.offset += reading.consumed as u64;

        let logins = names_of_users(
            reading
                .executions
                .iter()
                .filter_map(|execution| execution.auid)
                .filter(|auid| *auid != AUID_UNSET),
        );

        let snapshot = launches_snapshot_from(
            &(self.now)(),
            &seen.items,
            &LaunchReading {
                executions: &reading.executions,
                logins: &logins,
                any_unnamed: reading.unnamed > 0,
                keep_arguments: self.keep_arguments,
                on_disk: &look_on_disk,
                from_plugin: true,
                dropped,
            },
            &ESLOGGER,
        );

        if (seen.inode, seen.offset) != resumed_from {
            let _ = Cursor {
                inode: seen.inode,
                offset: seen.offset,
            }
            .write(&cursor_path(path));
        }

        seen.items = snapshot.items.clone();
        Ok(snapshot)
    }

    pub(super) fn remember(&self, previous: &Snapshot) {
        if previous.source != self.name() {
            return;
        }
        let mut seen = self
            .seen
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for (key, value) in &previous.items {
            seen.items
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
    }
}

fn read_chunk(file: &File, from: u64, path: &Path) -> Result<Vec<u8>, CollectError> {
    let mut handle = file;
    handle
        .seek(SeekFrom::Start(from))
        .and_then(|_| {
            let mut buffer = Vec::new();
            handle
                .take(MAX_BYTES_PER_READING)
                .read_to_end(&mut buffer)?;
            Ok(buffer)
        })
        .map_err(|error| CollectError::Unreadable(format!("{}: {error}", path.display())))
}

fn look_on_disk(path: &str) -> Presence {
    if !shown_to_the_agent(path) {
        return Presence::NotShown;
    }
    match Path::new(path).exists() {
        true => Presence::OnDisk,
        false => Presence::Gone,
    }
}
