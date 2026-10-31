use super::*;
use std::fs::File;
use std::path::PathBuf;

pub struct SpoolWriter {
    pub(super) path: PathBuf,
    pub(super) ceiling: u64,
    pub(super) file: File,
    pub(super) bytes: u64,
    pub(super) pending: Vec<u8>,
    pub(super) resyncing: bool,
    pub(super) report: Report,
}
