use std::os::unix::fs::MetadataExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    device: u64,
    inode: u64,
    bytes: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

impl Stamp {
    pub fn of(path: &str) -> Result<Stamp, String> {
        let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;

        Ok(Stamp {
            device: metadata.dev(),
            inode: metadata.ino(),
            bytes: metadata.len(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        })
    }
}
