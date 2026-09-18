use std::path::PathBuf;

use super::devices::Devices;
use super::watched::Watched;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    Named(Named),
    Listed(Listing),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    pub paths: Vec<Watched>,
    pub ceiling_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub watched_path: PathBuf,
    pub max_file_size: u64,
    pub devices: Devices,
    pub max_files: usize,
}

impl Named {
    pub fn hashed(&self) -> Vec<(String, u64)> {
        self.paths
            .iter()
            .map(|watched| {
                (
                    watched.path().to_string(),
                    watched.hashed_to(self.ceiling_bytes),
                )
            })
            .collect()
    }
}
