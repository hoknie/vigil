use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::CollectError;

const MAX_BYTES_PER_READING: u64 = 2 * 1024 * 1024;

pub(super) fn read_chunk(file: &File, from: u64, path: &Path) -> Result<Vec<u8>, CollectError> {
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

pub(super) fn tail(path: &Path, length: u64, wanted: u64) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(length.saturating_sub(wanted)))?;
    let mut buffer = Vec::new();
    file.take(wanted).read_to_end(&mut buffer)?;
    Ok(buffer)
}
