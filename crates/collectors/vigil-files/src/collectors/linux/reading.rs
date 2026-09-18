use std::fs;

use vigil_model::Snapshot;

use super::FilesCollector;
use super::filesystems::Filesystems;
use super::health::{Notes, Walked};
use super::plan::Plan;
use super::stat;
use super::walk::Walker;
use vigil_collect::CollectError;

use crate::helpers::holds_a_mask;
use crate::parsers::{FilesReading, WatchedDirectory, files_snapshot};
use crate::types::Devices;

struct Asked {
    entries: Vec<(String, u64)>,
    devices: Devices,
    max_files: usize,
    listed: bool,
    complaints: Vec<String>,
    nothing: Option<String>,
}

enum Entry {
    Plain(String, u64),
    Tree(String, fs::Metadata, u64),
    Mask(String, u64),
}

impl FilesCollector {
    pub(super) fn reading(&self) -> Result<Snapshot, CollectError> {
        let asked = self.asked();
        if let Some(why) = asked.nothing {
            self.remember(Notes::nothing(why));
            return Err(CollectError::Absent(
                "a path named for this collector to check".into(),
            ));
        }

        let entries: Vec<Entry> = asked
            .entries
            .into_iter()
            .map(|(path, ceiling_bytes)| entry_of(path, ceiling_bytes, asked.listed))
            .collect();
        let expanding = entries
            .iter()
            .any(|entry| !matches!(entry, Entry::Plain(..)));
        let filesystems = match expanding {
            true => Filesystems::read(asked.devices),
            false => Filesystems::of(None, Devices::default()),
        };
        let blind = expanding && !filesystems.known();

        let mut walker = Walker::new(asked.max_files, filesystems);
        for entry in &entries {
            if let Entry::Plain(path, ceiling_bytes) = entry {
                walker.named(path, *ceiling_bytes);
            }
        }
        for entry in &entries {
            match entry {
                Entry::Plain(..) => {}
                Entry::Tree(path, metadata, ceiling_bytes) => {
                    walker.tree(path, metadata, *ceiling_bytes)
                }
                Entry::Mask(path, ceiling_bytes) => walker.mask(path, *ceiling_bytes),
            }
        }

        let directories: Vec<WatchedDirectory> = self
            .directories
            .iter()
            .map(|path| stat::path_directory(path))
            .collect();
        let snapshot = files_snapshot(
            &(self.now)(),
            &FilesReading {
                files: &walker.files,
                directories: &directories,
                walks: &walker.walks,
            },
        );

        self.remember(Notes::of(Walked {
            files: &walker.files,
            stopped: &walker.stopped,
            max_files: asked.max_files,
            blind,
            complaints: asked.complaints,
        }));
        Ok(snapshot)
    }

    fn asked(&self) -> Asked {
        match &self.plan {
            Plan::Named(paths) => Asked {
                nothing: paths.is_empty().then(|| {
                    "there is nothing to check: no path is named for this collector. What it \
                     watches is a list in the configuration file and never a walk of a tree, \
                     because a walk that hashes everything is the first thing an operator turns \
                     off"
                    .to_string()
                }),
                entries: paths.clone(),
                devices: Devices::default(),
                max_files: usize::MAX,
                listed: false,
                complaints: Vec::new(),
            },
            Plan::Listed { listing, lists } => {
                let gathered = match lists.lock() {
                    Ok(mut lists) => lists.gather(&listing.watched_path),
                    Err(poisoned) => poisoned.into_inner().gather(&listing.watched_path),
                };
                Asked {
                    entries: gathered
                        .entries
                        .iter()
                        .map(|watched| {
                            (
                                watched.path().to_string(),
                                watched.hashed_to(listing.max_file_size),
                            )
                        })
                        .collect(),
                    devices: listing.devices.joined(&gathered.devices),
                    max_files: listing.max_files,
                    listed: true,
                    complaints: gathered.complaints,
                    nothing: gathered.nothing,
                }
            }
        }
    }
}

fn entry_of(path: String, ceiling_bytes: u64, listed: bool) -> Entry {
    if !listed {
        return Entry::Plain(path, ceiling_bytes);
    }
    if holds_a_mask(&path) {
        return Entry::Mask(path, ceiling_bytes);
    }
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_dir() => Entry::Tree(path, metadata, ceiling_bytes),
        _ => Entry::Plain(path, ceiling_bytes),
    }
}
