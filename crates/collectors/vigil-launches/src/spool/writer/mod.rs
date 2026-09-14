mod on_disk;
mod report;
mod writing;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::path::PathBuf;

pub use report::Report;

pub struct SpoolWriter {
    path: PathBuf,
    ceiling: u64,
    file: File,
    bytes: u64,
    pending: Vec<u8>,
    resyncing: bool,
    report: Report,
}
