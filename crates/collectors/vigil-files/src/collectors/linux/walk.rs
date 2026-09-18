use std::fs;
use std::os::unix::fs::MetadataExt;

use super::filesystems::Filesystems;
use super::stat::{self, DIRECTORY};
use crate::parsers::{WalkRow, WatchedFile};

pub(super) const DEEPEST: usize = 64;

pub(super) const MOST_NOT_ENTERED: usize = 32;

pub(super) const TREE: &str = "tree";

pub(super) struct Walker {
    pub(super) left: usize,
    pub(super) filesystems: Filesystems,
    pub(super) files: Vec<WatchedFile>,
    pub(super) walks: Vec<WalkRow>,
    pub(super) stopped: Stop,
}

#[derive(Default)]
pub(super) struct Stop {
    pub(super) at: Option<String>,
    pub(super) unwalked: Vec<String>,
    pub(super) unlisted: Vec<String>,
}

pub(super) struct Opened {
    pub(super) path: String,
    pub(super) device: u64,
    pub(super) depth: usize,
}

impl Walker {
    pub(super) fn new(left: usize, filesystems: Filesystems) -> Walker {
        Walker {
            left,
            filesystems,
            files: Vec::new(),
            walks: Vec::new(),
            stopped: Stop::default(),
        }
    }

    pub(super) fn named(&mut self, path: &str, ceiling_bytes: u64) {
        self.left = self.left.saturating_sub(1);
        self.files.push(stat::named(path, ceiling_bytes));
    }

    pub(super) fn tree(&mut self, entry: &str, metadata: &fs::Metadata, ceiling_bytes: u64) {
        self.files.push(stat::root(entry, metadata, ceiling_bytes));
        let mut walk = self.walk_row(entry, TREE, ceiling_bytes);
        let first = self.files.len();

        if self.stopped.at.is_some() {
            self.stopped.unwalked.push(entry.to_string());
            walk.complete = false;
        } else {
            match self
                .filesystems
                .enters(entry, metadata.dev(), metadata.dev())
            {
                Err(why) => walk.not_entered.push(why),
                Ok(()) => self.walk_from(
                    Opened {
                        path: entry.to_string(),
                        device: metadata.dev(),
                        depth: 0,
                    },
                    entry,
                    ceiling_bytes,
                    &mut walk,
                ),
            }
        }

        self.close(walk, first);
    }

    pub(super) fn walk_row(&self, entry: &str, kind: &'static str, ceiling_bytes: u64) -> WalkRow {
        WalkRow {
            entry: entry.to_string(),
            kind,
            matched: 0,
            complete: true,
            not_entered: Vec::new(),
            max_file_size: ceiling_bytes,
        }
    }

    pub(super) fn close(&mut self, mut walk: WalkRow, first: usize) {
        walk.matched = self.files.len() - first;
        walk.not_entered.truncate(MOST_NOT_ENTERED);
        for row in &mut self.files[first..] {
            if let Some(walked) = row.found.walked.as_mut() {
                walked.complete = walk.complete;
            }
        }
        self.walks.push(walk);
    }

    pub(super) fn take(&mut self, path: &str, walk: &mut WalkRow) -> bool {
        if self.stopped.at.is_some() {
            walk.complete = false;
            return false;
        }
        if self.left == 0 {
            self.stopped.at = Some(path.to_string());
            walk.complete = false;
            return false;
        }
        self.left -= 1;
        true
    }

    pub(super) fn walk_from(
        &mut self,
        start: Opened,
        entry: &str,
        ceiling_bytes: u64,
        walk: &mut WalkRow,
    ) {
        let mut waiting = vec![start];

        while let Some(opened) = waiting.pop() {
            let Some(names) = self.listed(&opened.path, walk) else {
                continue;
            };
            let mut deeper: Vec<Opened> = Vec::new();

            for name in names {
                let path = joined(&opened.path, &name);
                if !self.take(&path, walk) {
                    return;
                }
                let Ok(metadata) = fs::symlink_metadata(&path) else {
                    continue;
                };
                self.files
                    .push(stat::walked(&path, &metadata, ceiling_bytes, entry));
                if stat::kind_of(&metadata) != DIRECTORY {
                    continue;
                }
                if opened.depth + 1 >= DEEPEST {
                    walk.not_entered
                        .push(format!("{path} (deeper than {DEEPEST})"));
                    continue;
                }
                if self
                    .filesystems
                    .crossed(&path, metadata.dev(), opened.device)
                    && let Err(why) = self
                        .filesystems
                        .enters(&path, metadata.dev(), opened.device)
                {
                    walk.not_entered.push(why);
                    continue;
                }
                deeper.push(Opened {
                    path,
                    device: metadata.dev(),
                    depth: opened.depth + 1,
                });
            }
            waiting.extend(deeper.into_iter().rev());
        }
    }

    pub(super) fn listed(&mut self, directory: &str, walk: &mut WalkRow) -> Option<Vec<String>> {
        let read = match fs::read_dir(directory) {
            Ok(read) => read,
            Err(_) => {
                self.stopped.unlisted.push(directory.to_string());
                return None;
            }
        };
        let mut names: Vec<String> = Vec::new();
        for entry in read.filter_map(Result::ok) {
            match entry.file_name().into_string() {
                Ok(name) => names.push(name),
                Err(name) => walk.not_entered.push(format!(
                    "{} (a name that is not text)",
                    joined(directory, &name.to_string_lossy())
                )),
            }
        }
        names.sort_unstable();
        Some(names)
    }
}

pub(super) fn joined(directory: &str, name: &str) -> String {
    match directory.ends_with('/') {
        true => format!("{directory}{name}"),
        false => format!("{directory}/{name}"),
    }
}
