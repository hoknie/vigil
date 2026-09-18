use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::helpers::lists_in;
use crate::parsers::watch_list_in;
use crate::types::{Devices, WatchList, Watched};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stamp {
    device: u64,
    inode: u64,
    size: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

struct Held {
    stamp: Stamp,
    good: Option<WatchList>,
    broken: Option<String>,
}

#[derive(Default)]
pub struct Lists {
    held: BTreeMap<PathBuf, Held>,
    #[cfg(test)]
    parsed: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Gathered {
    pub entries: Vec<Watched>,
    pub devices: Devices,
    pub complaints: Vec<String>,
    pub nothing: Option<String>,
}

impl Lists {
    #[cfg(test)]
    pub fn parsed(&self) -> usize {
        self.parsed
    }

    pub fn gather(&mut self, path: &Path) -> Gathered {
        let files = match lists_in(path) {
            Ok(Some(files)) => files,
            Ok(None) => {
                self.held.clear();
                return Gathered::nothing(format!(
                    "the watch list {} is not there, so no path is watched: write it, or name \
                     the list this host keeps in watched_path",
                    path.display()
                ));
            }
            Err(why) => {
                let mut gathered = self.combined(&self.held.keys().cloned().collect::<Vec<_>>());
                gathered.complaints.insert(
                    0,
                    format!(
                        "{why}; the lists read from it before are watched until it reads again"
                    ),
                );
                return gathered;
            }
        };

        self.held.retain(|file, _| files.contains(file));
        for file in &files {
            self.refresh(file);
        }
        if files.is_empty() {
            return Gathered::nothing(format!(
                "{} holds no watch list, so no path is watched: a list there is a .yaml or .yml \
                 file whose name does not begin with a dot",
                path.display()
            ));
        }

        let gathered = self.combined(&files);
        match (gathered.entries.is_empty(), gathered.complaints.is_empty()) {
            (false, _) => gathered,
            (true, true) => Gathered::nothing(format!(
                "no path is named in the watch list {}, so there is nothing to check",
                path.display()
            )),
            (true, false) => Gathered::nothing(gathered.complaints.join("; ")),
        }
    }

    fn refresh(&mut self, file: &Path) {
        let stamp = match fs::metadata(file) {
            Ok(metadata) => Stamp {
                device: metadata.dev(),
                inode: metadata.ino(),
                size: metadata.len(),
                modified: (metadata.mtime(), metadata.mtime_nsec()),
                changed: (metadata.ctime(), metadata.ctime_nsec()),
            },
            Err(_) => return,
        };
        if self.held.get(file).is_some_and(|held| held.stamp == stamp) {
            return;
        }

        #[cfg(test)]
        {
            self.parsed += 1;
        }
        let read = fs::read_to_string(file)
            .map_err(|error| error.to_string())
            .and_then(|text| watch_list_in(&text));
        let good = self.held.remove(file).and_then(|held| held.good);
        let held = match read {
            Ok(list) => Held {
                stamp,
                good: Some(list),
                broken: None,
            },
            Err(why) => Held {
                stamp,
                good,
                broken: Some(why),
            },
        };
        self.held.insert(file.to_path_buf(), held);
    }

    fn combined(&self, files: &[PathBuf]) -> Gathered {
        let mut gathered = Gathered::default();
        for file in files {
            let Some(held) = self.held.get(file) else {
                continue;
            };
            if let Some(why) = &held.broken {
                gathered.complaints.push(match held.good {
                    Some(_) => format!(
                        "the watch list {} does not read ({why}); what it named before is \
                         watched until it reads again",
                        file.display()
                    ),
                    None => format!(
                        "the watch list {} does not read ({why}); nothing it names is watched \
                         until it does",
                        file.display()
                    ),
                });
            }
            let Some(list) = &held.good else {
                continue;
            };
            for watched in &list.files {
                if !gathered
                    .entries
                    .iter()
                    .any(|before| before.path() == watched.path())
                {
                    gathered.entries.push(watched.clone());
                }
            }
            gathered.devices = gathered.devices.joined(&list.devices);
        }
        gathered
    }
}

impl Gathered {
    fn nothing(why: String) -> Gathered {
        Gathered {
            nothing: Some(why),
            ..Gathered::default()
        }
    }
}
