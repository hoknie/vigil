use std::fs::{self, File};
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use vigil_model::Snapshot;

use super::LaunchesCollector;
use super::chunk::read_chunk;
use crate::parsers::{LaunchReading, launches_snapshot, parse_audit_log, parse_passwd};
use crate::spool::{Cursor, cursor_path, dropped_note};
use crate::{CollectError, Collector};

const FIRST_READING_TAIL: u64 = 2 * 1024 * 1024;

impl LaunchesCollector {
    pub(super) fn reading(&self) -> Result<Snapshot, CollectError> {
        let mut seen = self
            .seen
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let from_plugin = self.spool_bytes() > 0;
        let path = match from_plugin {
            true => self.spool_path.clone(),
            false => self.log_path.clone(),
        };

        let file = File::open(&path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => CollectError::Absent(path.display().to_string()),
            ErrorKind::PermissionDenied => CollectError::Denied(path.display().to_string()),
            _ => CollectError::Unreadable(format!("{}: {error}", path.display())),
        })?;
        let metadata = file
            .metadata()
            .map_err(|error| CollectError::Unreadable(format!("{}: {error}", path.display())))?;

        if seen.from.as_deref() != Some(path.as_path())
            || metadata.ino() != seen.inode
            || metadata.len() < seen.offset
        {
            seen.from = Some(path.clone());
            seen.inode = metadata.ino();
            seen.offset = 0;
        }
        let resumed_from = (seen.inode, seen.offset);

        let mut from = seen.offset;
        let mut mid_line = false;
        if !from_plugin && from == 0 && metadata.len() > FIRST_READING_TAIL {
            from = metadata.len() - FIRST_READING_TAIL;
            mid_line = true;
        }

        let buffer = read_chunk(&file, from, &path)?;

        let start = match mid_line {
            true => buffer
                .iter()
                .position(|byte| *byte == b'\n')
                .map(|at| at + 1)
                .unwrap_or(buffer.len()),
            false => 0,
        };
        let chunk = &buffer[start..];

        let dropped = from == 0
            && dropped_note(
                chunk
                    .split(|byte| *byte == b'\n')
                    .next()
                    .unwrap_or_default(),
            )
            .is_some();

        let reading = parse_audit_log(chunk, self.keep_arguments);
        seen.offset = from + start as u64 + reading.consumed as u64;

        let logins = fs::read_to_string("/etc/passwd")
            .map(|text| parse_passwd(&text))
            .unwrap_or_default();

        let snapshot = launches_snapshot(
            &(self.now)(),
            &seen.items,
            &LaunchReading {
                executions: &reading.executions,
                logins: &logins,
                any_unnamed: reading.unnamed > 0,
                keep_arguments: self.keep_arguments,
                on_disk: &|path: &str| Path::new(path).exists(),
                from_plugin,
                dropped,
            },
        );

        if from_plugin && (seen.inode, seen.offset) != resumed_from {
            let cursor = cursor_path(&path);
            let _ = Cursor {
                inode: seen.inode,
                offset: seen.offset,
            }
            .write(&cursor);

            if fs::metadata(&path).map(|at| at.ino()).ok() != Some(seen.inode) {
                let _ = fs::remove_file(&cursor);
            }
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
