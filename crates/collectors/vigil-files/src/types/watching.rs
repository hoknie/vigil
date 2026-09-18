use std::path::PathBuf;

use serde::Deserialize;

use super::devices::Devices;
use super::layout::{Layout, Listing, Named};
use super::size::Size;
use super::watched::{LONGEST_PATH, MOST_HASHED_BYTES, Watched};

pub const CEILING_BYTES: u64 = 1024 * 1024;

pub const MAX_FILES: usize = 10_000;

pub const MOST_FILES: usize = 10_000;

pub const WATCHED_BY_DEFAULT: &[&str] = &[
    "/etc/ssh/sshd_config",
    "/etc/pam.d/sshd",
    "/etc/pam.d/su",
    "/etc/nsswitch.conf",
    "/etc/login.defs",
    "/etc/hosts",
];

const OLD_KEYS: &[&str] = &["paths", "ceiling_bytes"];

const NEW_KEYS: &[&str] = &["watched_path", "max_file_size", "devices", "max_files"];

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Watching {
    pub paths: Option<Vec<Watched>>,
    pub ceiling_bytes: Option<u64>,
    pub watched_path: Option<String>,
    pub max_file_size: Option<Size>,
    pub devices: Option<Devices>,
    pub max_files: Option<usize>,
}

impl Watching {
    pub fn layout(&self) -> Layout {
        let ceiling_bytes = self
            .ceiling_bytes
            .or(self.max_file_size.map(Size::bytes))
            .unwrap_or(CEILING_BYTES);

        match &self.watched_path {
            Some(watched_path) => Layout::Listed(Listing {
                watched_path: PathBuf::from(watched_path),
                max_file_size: ceiling_bytes,
                devices: self.devices.clone().unwrap_or_default(),
                max_files: self.max_files.unwrap_or(MAX_FILES),
            }),
            None => Layout::Named(Named {
                paths: self.paths.clone().unwrap_or_else(shipped),
                ceiling_bytes,
            }),
        }
    }

    pub fn check(&self) -> Result<(), String> {
        self.one_layout()?;
        if self.ceiling_bytes == Some(0) {
            return Err(
                "ceiling_bytes: 0 hashes nothing, and a file nobody hashes is a file nobody \
                 watches"
                    .to_string(),
            );
        }
        self.sizes()?;
        self.place()?;
        if let Some(devices) = &self.devices {
            devices.check()?;
        }

        let Some(paths) = &self.paths else {
            return Ok(());
        };
        for (place, watched) in paths.iter().enumerate() {
            watched
                .check()
                .map_err(|why| format!("paths #{}: {why}", place + 1))?;
            if paths[..place]
                .iter()
                .any(|before| before.path() == watched.path())
            {
                return Err(format!(
                    "paths #{}: {:?} is named twice, and the second entry watches nothing the \
                     first does not",
                    place + 1,
                    watched.path()
                ));
            }
        }

        Ok(())
    }

    pub fn hashed(&self) -> Vec<(String, u64)> {
        match self.layout() {
            Layout::Named(named) => named.hashed(),
            Layout::Listed(_) => Vec::new(),
        }
    }

    fn one_layout(&self) -> Result<(), String> {
        let old: Vec<&str> = OLD_KEYS
            .iter()
            .zip([self.paths.is_some(), self.ceiling_bytes.is_some()])
            .filter_map(|(key, said)| said.then_some(*key))
            .collect();
        let new: Vec<&str> = NEW_KEYS
            .iter()
            .zip([
                self.watched_path.is_some(),
                self.max_file_size.is_some(),
                self.devices.is_some(),
                self.max_files.is_some(),
            ])
            .filter_map(|(key, said)| said.then_some(*key))
            .collect();

        match old.is_empty() || new.is_empty() {
            true => Ok(()),
            false => Err(format!(
                "{} and {} are written in one block: {} are the keys of the list written \
                 inside the configuration, and {} of the list kept in a file of its own. Keep \
                 one of the two, or the agent would watch one list while the file reads as \
                 the other",
                old.join(", "),
                new.join(", "),
                OLD_KEYS.join(" and "),
                NEW_KEYS.join(", ")
            )),
        }
    }

    fn sizes(&self) -> Result<(), String> {
        match self.max_file_size.map(Size::bytes) {
            Some(0) => {
                return Err(
                    "max_file_size: 0 hashes nothing, and a file nobody hashes is a file \
                     nobody watches"
                        .to_string(),
                );
            }
            Some(bytes) if bytes > MOST_HASHED_BYTES => {
                return Err(format!(
                    "max_file_size: {} is over the {} this agent hashes of one file on one \
                     pass, and a reading that takes longer than the pass it belongs to is the \
                     agent an operator turns off first",
                    Size::shown(bytes),
                    Size::shown(MOST_HASHED_BYTES)
                ));
            }
            _ => {}
        }
        match self.max_files {
            Some(0) => Err(
                "max_files: 0 walks nothing, so every directory in the watch list would read \
                 as one nobody asked about"
                    .to_string(),
            ),
            Some(files) if files > MOST_FILES => Err(format!(
                "max_files: {files} is over the {MOST_FILES} paths this agent walks on one \
                 pass: that many rows, held twice to be compared, are already most of the \
                 memory this agent may take. Watch the directories that matter rather than the \
                 whole tree"
            )),
            _ => Ok(()),
        }
    }

    fn place(&self) -> Result<(), String> {
        let Some(watched_path) = &self.watched_path else {
            return Ok(());
        };
        if !watched_path.starts_with('/') {
            return Err(format!(
                "watched_path: {watched_path:?} is not an absolute path, and the agent that \
                 reads it has no working directory of its own"
            ));
        }
        if watched_path.chars().any(char::is_control) || watched_path.len() > LONGEST_PATH {
            return Err(format!(
                "watched_path: {watched_path:?} is not a path this kernel opens"
            ));
        }
        Ok(())
    }
}

fn shipped() -> Vec<Watched> {
    WATCHED_BY_DEFAULT
        .iter()
        .map(|path| Watched::Named((*path).to_string()))
        .collect()
}
