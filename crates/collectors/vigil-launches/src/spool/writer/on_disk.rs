use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

const COPY_CHUNK: usize = 64 * 1024;

pub(super) fn next_record_at_or_after(path: &Path, wanted: u64) -> io::Result<u64> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(wanted))?;

    let mut at = wanted;
    let mut buffer = [0u8; COPY_CHUNK];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            return Ok(at);
        }
        if let Some(newline) = buffer[..read].iter().position(|byte| *byte == b'\n') {
            return Ok(at + newline as u64 + 1);
        }
        at += read as u64;
    }
}

pub(super) fn copy(source: &mut File, target: &mut File) -> io::Result<u64> {
    let mut buffer = vec![0u8; COPY_CHUNK];
    let mut moved = 0u64;
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            return Ok(moved);
        }
        target.write_all(&buffer[..read])?;
        moved += read as u64;
    }
}

#[cfg(unix)]
pub(super) fn inode_of(file: &File) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    Ok(file.metadata()?.ino())
}

#[cfg(not(unix))]
pub(super) fn inode_of(_file: &File) -> io::Result<u64> {
    Ok(0)
}
