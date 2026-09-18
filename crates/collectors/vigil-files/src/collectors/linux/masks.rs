use std::fs;
use std::os::unix::fs::MetadataExt;

use super::stat;
use super::walk::{Opened, Walker, joined};
use crate::helpers::{is_a_mask, matches};
use crate::parsers::WalkRow;

pub(super) const MASK: &str = "mask";

struct Candidate {
    path: String,
    device: u64,
    followed: bool,
}

impl Walker {
    pub(super) fn mask(&mut self, entry: &str, ceiling_bytes: u64) {
        let mut walk = self.walk_row(entry, MASK, ceiling_bytes);
        let first = self.files.len();
        if self.stopped.at.is_some() {
            self.stopped.unwalked.push(entry.to_string());
            walk.complete = false;
            self.close(walk, first);
            return;
        }

        for candidate in self.matched(entry, &mut walk) {
            if !self.take(&candidate.path, &mut walk) {
                break;
            }
            let Ok(metadata) = fs::symlink_metadata(&candidate.path) else {
                continue;
            };
            self.files.push(stat::walked(
                &candidate.path,
                &metadata,
                ceiling_bytes,
                entry,
            ));
            if !metadata.is_dir() {
                continue;
            }
            if self
                .filesystems
                .crossed(&candidate.path, metadata.dev(), candidate.device)
                && let Err(why) =
                    self.filesystems
                        .enters(&candidate.path, metadata.dev(), candidate.device)
            {
                walk.not_entered.push(why);
                continue;
            }
            self.walk_from(
                Opened {
                    path: candidate.path,
                    device: metadata.dev(),
                    depth: 0,
                },
                entry,
                ceiling_bytes,
                &mut walk,
            );
        }

        self.close(walk, first);
    }

    fn matched(&mut self, entry: &str, walk: &mut WalkRow) -> Vec<Candidate> {
        let components: Vec<&str> = entry.split('/').filter(|name| !name.is_empty()).collect();
        let Some(first_mask) = components.iter().position(|name| is_a_mask(name)) else {
            return Vec::new();
        };
        let prefix = format!("/{}", components[..first_mask].join("/"));
        let Ok(opened) = fs::metadata(&prefix) else {
            return Vec::new();
        };
        if let Err(why) = self.filesystems.enters(&prefix, opened.dev(), opened.dev()) {
            walk.not_entered.push(why);
            return Vec::new();
        }

        let mut candidates = vec![Candidate {
            path: prefix,
            device: opened.dev(),
            followed: true,
        }];
        for name in &components[first_mask..] {
            let mut next = Vec::new();
            for candidate in candidates {
                match is_a_mask(name) {
                    true => next.extend(self.matching(&candidate, name, walk)),
                    false => next.push(Candidate {
                        path: joined(&candidate.path, name),
                        followed: false,
                        ..candidate
                    }),
                }
            }
            candidates = next;
        }

        candidates.retain(|candidate| fs::symlink_metadata(&candidate.path).is_ok());
        candidates.sort_by(|left, right| left.path.cmp(&right.path));
        candidates.dedup_by(|left, right| left.path == right.path);
        candidates
    }

    fn matching(
        &mut self,
        candidate: &Candidate,
        mask: &str,
        walk: &mut WalkRow,
    ) -> Vec<Candidate> {
        let metadata = match candidate.followed {
            true => fs::metadata(&candidate.path),
            false => fs::symlink_metadata(&candidate.path),
        };
        let Ok(metadata) = metadata else {
            return Vec::new();
        };
        if !metadata.is_dir() {
            return Vec::new();
        }
        if self
            .filesystems
            .crossed(&candidate.path, metadata.dev(), candidate.device)
            && let Err(why) =
                self.filesystems
                    .enters(&candidate.path, metadata.dev(), candidate.device)
        {
            walk.not_entered.push(why);
            return Vec::new();
        }
        self.listed(&candidate.path, walk)
            .unwrap_or_default()
            .into_iter()
            .filter(|name| matches(mask, name))
            .map(|name| Candidate {
                path: joined(&candidate.path, &name),
                device: metadata.dev(),
                followed: false,
            })
            .collect()
    }
}
