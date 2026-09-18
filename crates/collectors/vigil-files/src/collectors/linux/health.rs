use super::FilesCollector;
use super::walk::Stop;
use crate::parsers::WatchedFile;
use vigil_collect::Health;

const NAMED_AT_MOST: usize = 3;

#[derive(Debug, Clone, Default)]
pub(super) struct Notes {
    nothing: Option<String>,
    complaints: Vec<String>,
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

        let unreadable: Vec<String> = walked
            .files
            .iter()
            .filter(|file| file.present && !file.readable && !file.over_the_ceiling)
            .map(|file| {
                format!(
                    "{} is there and cannot be read by this agent, so a change to its content \
                     will pass unseen",
                    file.path
                )
            })
            .collect();
        complaints.extend(named_at_most(unreadable, "file(s) this agent cannot read"));

        let unlisted: Vec<String> = walked
            .stopped
            .unlisted
            .iter()
            .map(|directory| {
                format!(
                    "{directory} cannot be listed by this agent, so what is put in it passes \
                     unseen"
                )
            })
            .collect();
        complaints.extend(named_at_most(
            unlisted,
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
}

impl FilesCollector {
    pub(super) fn health(&self) -> Health {
        if let Some(notes) = self.remembered() {
            return notes.health();
        }
        let _ = self.reading();
        self.remembered().unwrap_or_default().health()
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
