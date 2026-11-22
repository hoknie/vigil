use super::FilesCollector;
use super::walk::Stop;
use crate::parsers::WatchedFile;
use vigil_collect::{Health, Limit, Standing};

pub(super) const NAMED_AT_MOST: usize = 3;

#[derive(Debug, Clone, Default)]
pub(super) struct Notes {
    nothing: Option<String>,
    complaints: Vec<String>,
    hidden: Vec<String>,
}

pub(super) struct Walked<'a> {
    pub(super) files: &'a [WatchedFile],
    pub(super) stopped: &'a Stop,
    pub(super) max_files: usize,
    pub(super) blind: bool,
    pub(super) complaints: Vec<String>,
}

impl Notes {
    pub(super) fn nothing(why: String) -> Notes {
        Notes {
            nothing: Some(why),
            complaints: Vec::new(),
            hidden: Vec::new(),
        }
    }

    pub(super) fn of(walked: Walked<'_>) -> Notes {
        let mut complaints = walked.complaints;

        let over: Vec<String> = walked
            .files
            .iter()
            .filter(|file| file.present && file.over_the_ceiling)
            .map(|file| {
                format!(
                    "{} is {} bytes, over the {} this collector hashes, so a change to it will \
                     be seen in its size and its mode and not in its content",
                    file.path, file.size, file.ceiling_bytes
                )
            })
            .collect();
        complaints.extend(named_at_most(
            over,
            "file(s) over the size they are hashed to",
        ));

        let (refused, unreadable): (Vec<&WatchedFile>, Vec<&WatchedFile>) = walked
            .files
            .iter()
            .filter(|file| file.present && !file.readable && !file.over_the_ceiling)
            .partition(|file| refused_to_this_account(&file.path, false));
        let mut hidden = named_at_most(
            refused
                .iter()
                .map(|file| {
                    format!(
                        "{} is there and cannot be read by this account, so a change to its \
                         content will pass unseen",
                        file.path
                    )
                })
                .collect(),
            "file(s) this account cannot read",
        );
        complaints.extend(named_at_most(
            unreadable
                .iter()
                .map(|file| {
                    format!(
                        "{} is there and cannot be read by this agent, so a change to its \
                         content will pass unseen",
                        file.path
                    )
                })
                .collect(),
            "file(s) this agent cannot read",
        ));

        let (refused, unlisted): (Vec<&String>, Vec<&String>) = walked
            .stopped
            .unlisted
            .iter()
            .partition(|directory| refused_to_this_account(directory, true));
        hidden.extend(named_at_most(
            refused
                .iter()
                .map(|directory| {
                    format!(
                        "{directory} cannot be listed by this account, so what is put in it \
                         passes unseen"
                    )
                })
                .collect(),
            "directories this account cannot list",
        ));
        complaints.extend(named_at_most(
            unlisted
                .iter()
                .map(|directory| {
                    format!(
                        "{directory} cannot be listed by this agent, so what is put in it \
                         passes unseen"
                    )
                })
                .collect(),
            "directories this agent cannot list",
        ));

        if let Some(at) = &walked.stopped.at {
            complaints.push(stopped(at, walked.max_files, &walked.stopped.unwalked));
        }
        if walked.blind {
            complaints.push(
                "the mount table of this host cannot be read, so no walk steps into another \
                 filesystem and devices are not applied"
                    .to_string(),
            );
        }

        Notes {
            nothing: None,
            complaints,
            hidden,
        }
    }

    pub(super) fn health(&self) -> Health {
        if let Some(why) = &self.nothing {
            return Health::Unavailable(why.clone());
        }
        match self.complaints.is_empty() {
            true => Health::Ok,
            false => Health::Degraded(self.complaints.join("; ")),
        }
    }

    pub(super) fn standing(&self) -> Standing {
        let limit = match (&self.nothing, self.hidden.is_empty()) {
            (None, false) => Limit::of(vec![format!("{}; run as root", self.hidden.join("; "))]),
            _ => None,
        };
        Standing::limited(self.health(), limit)
    }
}

#[cfg(target_os = "macos")]
fn refused_to_this_account(path: &str, directory: bool) -> bool {
    let opened = match directory {
        true => std::fs::read_dir(path).map(|_| ()),
        false => std::fs::File::open(path).map(|_| ()),
    };
    opened.is_err_and(|error| vigil_collect::denied_to_this_account(&error))
}

#[cfg(not(target_os = "macos"))]
fn refused_to_this_account(_: &str, _: bool) -> bool {
    false
}

impl FilesCollector {
    pub(super) fn health(&self) -> Health {
        self.notes().health()
    }

    pub(super) fn standing(&self) -> Standing {
        self.notes().standing()
    }

    fn notes(&self) -> Notes {
        if let Some(notes) = self.remembered() {
            return notes;
        }
        let _ = self.reading();
        self.remembered().unwrap_or_default()
    }

    pub(super) fn remember(&self, notes: Notes) {
        match self.last.lock() {
            Ok(mut last) => *last = Some(notes),
            Err(poisoned) => *poisoned.into_inner() = Some(notes),
        }
    }

    fn remembered(&self) -> Option<Notes> {
        match self.last.lock() {
            Ok(last) => last.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

fn named_at_most(said: Vec<String>, more: &str) -> Vec<String> {
    let left = said.len().saturating_sub(NAMED_AT_MOST);
    let mut kept: Vec<String> = said.into_iter().take(NAMED_AT_MOST).collect();
    if left > 0 {
        kept.push(format!("and {left} more {more}"));
    }
    kept
}

fn stopped(at: &str, max_files: usize, unwalked: &[String]) -> String {
    let mut said = format!(
        "the walk stopped at {at}: max_files is {max_files} paths a reading, so what lies past \
         it is not watched and what appears or goes there is not reported"
    );
    if !unwalked.is_empty() {
        said.push_str(&format!(
            ", and {} entr{} of the watch list {} not walked at all: {}",
            unwalked.len(),
            match unwalked.len() {
                1 => "y",
                _ => "ies",
            },
            match unwalked.len() {
                1 => "was",
                _ => "were",
            },
            unwalked.join(", ")
        ));
    }
    said
}
